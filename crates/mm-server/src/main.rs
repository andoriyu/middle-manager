#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Default config paths
    let config_paths = vec![
        "config.toml",
        "config.local.toml",
    ];
    
    mm_server::run_server(&config_paths).await
}
