use rmcp::{ServiceExt, transport::stdio};

mod server;
use server::IoTServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[mcp-iot] v1.0.0");
    let service = IoTServer::new().serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
