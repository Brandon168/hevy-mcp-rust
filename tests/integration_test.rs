#[cfg(test)]
mod tests {
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

    /// Phase 4 — streamable-http startup test.
    ///
    /// Spawns the server binary in `streamable-http` mode on an OS-assigned free
    /// port, waits up to 2 s for the TCP port to become reachable, then sends an
    /// HTTP POST to `/mcp` and asserts the server responds (any non-connection-
    /// error reply proves it is alive, even a 4xx is fine since we're not sending
    /// a valid MCP payload here).
    #[tokio::test]
    async fn test_streamable_http_startup() {
        use std::net::TcpListener as StdTcpListener;

        // Grab a free port by binding then releasing it.
        let port = {
            let listener = StdTcpListener::bind("127.0.0.1:0").expect("bind for free port");
            listener.local_addr().unwrap().port()
        };

        let mut cmd = Command::new(env!("CARGO_BIN_EXE_hevy-mcp"));
        cmd.env("HEVY_API_KEY", "test")
           .env("MCP_TRANSPORT", "streamable-http")
           .env("MCP_PORT", port.to_string())
           .kill_on_drop(true)   // ensure child is killed when this test finishes
           .stdout(std::process::Stdio::null())
           .stderr(std::process::Stdio::inherit());

        let mut child = cmd.spawn().expect("failed to spawn hevy-mcp in streamable-http mode");

        // Poll until the port is reachable or we time out.
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

        // Send an HTTP request — any response (including 4xx) proves the server is alive.
        let url = format!("http://127.0.0.1:{}/mcp", port);
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .unwrap();
        let resp = http
            .post(&url)
            .header("content-type", "application/json")
            .body("{}")  // intentionally malformed MCP payload
            .send()
            .await
            .expect("HTTP request to /mcp failed");

        println!("streamable-http /mcp status: {}", resp.status());
        assert!(
            resp.status().as_u16() < 500,
            "Expected non-5xx from /mcp, got {}",
            resp.status()
        );

        let _ = child.kill().await;
    }
}
