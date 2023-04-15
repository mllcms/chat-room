mod middleware;
mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    server::run("127.0.0.1:80").await?;
    Ok(())
}
