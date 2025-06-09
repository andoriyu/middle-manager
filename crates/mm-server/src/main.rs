#[tokio::main]
async fn main() -> rust_mcp_sdk::error::SdkResult<()> {
    mm_server::run_server().await
}
