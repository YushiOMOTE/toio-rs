use toio::Cube;

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut searcher = Cube::search();
    let mut cube = searcher.nearest().await.unwrap();
    cube.connect().await.unwrap();
    tokio::time::delay_for(std::time::Duration::from_secs(30)).await;
}
