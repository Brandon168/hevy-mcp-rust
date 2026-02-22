use std::time::Duration;
use rmcp::{ClientHandler, ServiceExt, model::ClientInfo, transport::child_process::TokioChildProcess};
use tokio::process::Command;

#[derive(Default, Clone)]
struct DummyClientHandler {}

impl ClientHandler for DummyClientHandler {
    fn get_info(&self) -> ClientInfo {
        ClientInfo::default()
    }
}

#[tokio::test]
async fn test_full_mcp_client() {
    let _ = tracing_subscriber::fmt().with_env_filter("debug").try_init();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_hevy-mcp"));
    cmd.env("HEVY_API_KEY", "test");
    cmd.env("RUST_BACKTRACE", "1");

    let transport = TokioChildProcess::builder(cmd)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .unwrap()
        .0;

    let client_handler = DummyClientHandler::default();
    let client = client_handler.serve(transport).await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(500)).await;

    match tokio::time::timeout(Duration::from_secs(5), client.list_tools(None)).await {
        Ok(Ok(tools)) => {
            println!("Tools returned: {}", tools.tools.len());
            assert_eq!(tools.tools.len(), 20);
        }
        Ok(Err(e)) => panic!("Error listing tools: {:?}", e),
        Err(_) => panic!("Timeout listing tools"),
    }

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_streamable_http_startup() {
    use std::net::TcpListener as StdTcpListener;

    let port = {
        let listener = StdTcpListener::bind("127.0.0.1:0").expect("bind for free port");
        listener.local_addr().unwrap().port()
    };

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_hevy-mcp"));
    cmd.env("HEVY_API_KEY", "test")
       .env("MCP_TRANSPORT", "streamable-http")
       .env("MCP_PORT", port.to_string())
       .kill_on_drop(true)
       .stdout(std::process::Stdio::null())
       .stderr(std::process::Stdio::inherit());

    let mut child = cmd.spawn().expect("failed to spawn hevy-mcp in streamable-http mode");

    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    let reachable = loop {
        if tokio::time::Instant::now() > deadline {
            break false;
        }
        match tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port)).await {
            Ok(_) => break true,
            Err(_) => tokio::time::sleep(Duration::from_millis(50)).await,
        }
    };

    if !reachable {
        let _ = child.kill().await;
        panic!("streamable-http server did not become reachable within 5 s on port {}", port);
    }

    let url = format!("http://127.0.0.1:{}/mcp", port);
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();
    let resp = http
        .post(&url)
        .header("content-type", "application/json")
        .body("{}")
        .send()
        .await
        .expect("HTTP request to /mcp failed");

    assert!(resp.status().as_u16() < 500);
    let _ = child.kill().await;
}

#[tokio::test]
async fn test_streamable_mcp_full_listing() {
    use futures::StreamExt;
    use bytes::Bytes;

    let port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind for free port");
        listener.local_addr().unwrap().port()
    };

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_hevy-mcp"));
    cmd.env("HEVY_API_KEY", "test")
       .env("MCP_TRANSPORT", "streamable-http")
       .env("MCP_PORT", port.to_string())
       .env("RUST_LOG", "debug")
       .kill_on_drop(true)
       .stdout(std::process::Stdio::null())
       .stderr(std::process::Stdio::inherit());

    let mut child = cmd.spawn().expect("failed to spawn hevy-mcp");

    // Wait for server
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while tokio::time::Instant::now() < deadline {
        if tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port)).await.is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    let manifest_url = format!("http://127.0.0.1:{}/mcp/manifest", port);
    let http = reqwest::Client::new();
    
    // 1. Initialize
    let init_payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test-client", "version": "1.0.0"}
        }
    });

    let resp = http.post(&manifest_url)
        .header("Accept", "application/json, text/event-stream")
        .json(&init_payload)
        .send()
        .await
        .expect("Initialize failed");

    assert!(resp.status().is_success());
    let session_id = resp.headers().get("mcp-session-id").expect("missing session id").to_str().unwrap().to_string();

    let mut stream = resp.bytes_stream();
    println!("--- Reading initial SSE lines ---");
    if let Some(chunk_res) = stream.next().await {
        let chunk = chunk_res.unwrap();
        println!("SSE INITIAL CHUNK: {:?}", String::from_utf8_lossy(&chunk));
    }

    // 2. Initialized Notification
    let notify_payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    let notify_resp = http.post(&manifest_url)
        .header("mcp-session-id", &session_id)
        .header("Accept", "application/json, text/event-stream")
        .json(&notify_payload)
        .send()
        .await
        .expect("Notification failed");
    
    if !notify_resp.status().is_success() {
        let status = notify_resp.status();
        let body = notify_resp.text().await.unwrap_or_default();
        panic!("Notification failed with status {}: {}", status, body);
    }

    // 3. Tools List
    let list_payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });
    let list_resp = http.post(&manifest_url)
        .header("mcp-session-id", &session_id)
        .header("Accept", "application/json, text/event-stream")
        .json(&list_payload)
        .send()
        .await
        .expect("List tools failed");
    
    let list_body = list_resp.text().await.unwrap_or_default();
    println!("List tools body: {}", list_body);
    
    if list_body.contains("\"id\":2") && list_body.contains("get-workouts") {
        println!("SUCCESS: Tool list found in POST body!");
        let _ = child.kill().await;
        return;
    }

    // 4. Parse SSE stream for tool list (should not be needed based on spec but let's check)
    let mut buffer = String::new();
    while let Some(chunk_result) = stream.next().await {
        let chunk: Bytes = chunk_result.expect("stream error");
        let chunk_str = String::from_utf8_lossy(&chunk);
        buffer.push_str(&chunk_str);

        while let Some(pos) = buffer.find("\n") {
            let line = buffer.drain(..pos + 1).collect::<String>();
            if line.starts_with("data: ") {
                let data = line["data: ".len()..].trim();
                if !data.is_empty() {
                    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(data) {
                        if json_val.get("id") == Some(&serde_json::json!(2)) {
                            println!("FOUND TOOL LIST RESPONSE IN SSE!");
                            let _ = child.kill().await;
                            return;
                        }
                    }
                }
            }
        }
    }

    panic!("Tool list response not found anywhere");
}
