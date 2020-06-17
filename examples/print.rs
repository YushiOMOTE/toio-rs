use toio::Cube;

#[tokio::main]
async fn main() {
    env_logger::init();

    // Search for the nearest cube.
    let mut cube = Cube::search().nearest().await.unwrap();

    // Connect.
    cube.connect().await.unwrap();

    // Print status.
    println!("version   : {}", cube.version().await.unwrap());
    println!("battery   : {}%", cube.battery().await.unwrap());
    println!("slope     : {}", cube.slope().await.unwrap());
    println!("collision : {}", cube.collision().await.unwrap());
    println!("button    : {}", cube.button().await.unwrap());
}
