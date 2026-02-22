use rmcp::{
    model::ClientInfo, transport::child_process::TokioChildProcess, ClientHandler, ServiceExt,
};
use tokio::process::Command;

#[derive(Default, Clone)]
struct DummyClientHandler {}

impl ClientHandler for DummyClientHandler {
    fn get_info(&self) -> ClientInfo {
        ClientInfo::default()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--bin", "hevy-mcp"]);
    cmd.env("HEVY_API_KEY", "test");
    cmd.env("RUST_BACKTRACE", "1");
    //cmd.env("RUST_LOG", "debug");

    let transport = TokioChildProcess::new(cmd)?;

    let client_handler = DummyClientHandler::default();
    let client = client_handler.serve(transport).await?;

    let tools = client.list_tools(None).await?;
    println!("Tools returned from client: {}", tools.tools.len());

    client.cancel().await?;
    Ok(())
}
