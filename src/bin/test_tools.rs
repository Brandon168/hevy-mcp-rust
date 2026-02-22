use rmcp::{handler::server::tool::ToolRouter, tool, tool_handler, tool_router, ServerHandler};

#[derive(Clone)]
pub struct TestServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl TestServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(name = "test-tool")]
    async fn test_tool(&self) -> Result<String, String> {
        Ok("hello".to_string())
    }
}

impl Default for TestServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for TestServer {}

fn main() {
    let ts = TestServer::new();
    let tools = ts.tool_router.list_all();
    println!("Tools: {}", tools.len());
}
