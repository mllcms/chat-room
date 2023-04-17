mod middleware;
mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    server::run("0.0.0.0:80").await?;
    Ok(())
}
