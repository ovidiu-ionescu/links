mod links;
mod router;
mod utils;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt::init();
    lib_hyper_organizator::server::start_servers(router::request_handler, None).await?;
    Ok(())
}
