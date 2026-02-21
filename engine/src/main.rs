#[tokio::main]
async fn main() {
    tracing_subscriber::init();
    tracing::info!("Oddex engine starting...");
}
