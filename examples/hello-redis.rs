use mini_redis::{client, Result};


#[tokio::main]
pub async fn main() -> Result<()>{
    // connect to the mini redis server
    let mut client = client::connect ("127.0.0.1:6379").await?;

    client.set("hello", "world".into()).await?;
    let result = client.get("hello").await?;
    println!("got: {:?}", result);
    Ok(())
}
