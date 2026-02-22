mod client;
mod tools;
mod types;

use anyhow::Result;
use clap::Parser;
use client::HevyClient;
use rmcp::service::ServiceExt;
use std::sync::Arc;
use tools::HevyTools;
use tracing::info;



/// CLI arguments — supports stdio and streamable-http transports
#[derive(Parser)]
#[command(name = "hevy-mcp", about = "Hevy MCP Server")]
struct Cli {
    /// Transport mode: "stdio" or "streamable-http"
    #[arg(long, default_value = "stdio", env = "MCP_TRANSPORT")]
    transport: String,

    /// Port for streamable-http mode
    #[arg(long, default_value = "3000", env = "MCP_PORT")]
    port: u16,

    /// Hevy API key
    #[arg(long, env = "HEVY_API_KEY")]
    hevy_api_key: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if present (silently ignored if missing)
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    // Initialize structured logging to stderr (stdout is used for MCP stdio transport)
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting Hevy MCP server (transport={})", cli.transport);

    let client = Arc::new(HevyClient::new(cli.hevy_api_key)?);
    let tools = HevyTools::new(client);

    match cli.transport.as_str() {
        "stdio" => {
            let (stdin, stdout) = rmcp::transport::io::stdio();
            info!("Starting MCP stdio transport loop");
            let server = tools
                .serve((stdin, stdout))
                .await
                .map_err(|e| anyhow::anyhow!("Failed to initialize server: {e}"))?;
            server
                .waiting()
                .await
                .map_err(|e| anyhow::anyhow!("Server error: {e}"))?;
        }
        "streamable-http" => {
            use rmcp::transport::streamable_http_server::{
                session::local::LocalSessionManager, StreamableHttpService,
                StreamableHttpServerConfig,
            };
            use tower_http::cors::CorsLayer;

            let mut config = StreamableHttpServerConfig::default();
            config.sse_keep_alive = None;
            config.sse_retry = None; // Disable priming events (data: \n) that crash strict clients
            let session_manager = Arc::new(LocalSessionManager::default());
            let service =
                StreamableHttpService::new(move || Ok(tools.clone()), session_manager, config);

            let app = axum::Router::new()
                .route("/", axum::routing::get(|| async { "Hevy MCP Server is running. Use /mcp as the streamable-http endpoint." }))
                .nest_service("/mcp", service)
                .layer(CorsLayer::permissive());
            
            let addr = format!("0.0.0.0:{}", cli.port);
            let listener = tokio::net::TcpListener::bind(&addr).await?;
            info!(port = cli.port, "Starting streamable HTTP transport with CORS enabled");
            axum::serve(listener, app).await?;
        }
        other => {
            anyhow::bail!(
                "Unknown transport: {other}. Use 'stdio' or 'streamable-http'."
            );
        }
    }

    Ok(())
}
