use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let root_route = warp::fs::dir("static");

    warp::serve(root_route).run(([127, 0, 0, 1], 3333)).await;

    Ok(())
}
