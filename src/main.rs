mod circular_string;
mod links;
mod router;
mod static_files;
mod utils;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt::init();
    lazy_static::initialize(&links::CLICK_LOG);
    lib_hyper_organizator::server::start_servers(router::request_handler, None).await?;
    Ok(())
}
