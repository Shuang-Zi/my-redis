// use tokio::sync::mpsc;
use tokio::sync::oneshot;
use bytes::Bytes;
use mini_redis::client;

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        responder: Responder<Option<Bytes>>
    },
    Set {
        key: String,
        value: Bytes,
        responder: Responder<()>
    },
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(32);
    let manager = tokio::spawn(async move {
        let mut connection = client::connect("127.0.0.1:6379").await.unwrap();
        while let Some(cmd) = rx.recv().await {
            use Command::*;
            match cmd {
                Get { key, responder} => {
                    let res = connection.get(&key).await;
                    responder.send(res).unwrap();
                }
                Set { key, value, responder } => {
                    let res = connection.set(&key, value).await;
                    responder.send(res).unwrap();
                }
            }
        }
    });
    let tx2 = tx.clone();
    let t1 = tokio::spawn(async move {
        let (res_tx, res_rx) = oneshot::channel();

        let cmd = Command::Get { key: "hello".to_string(), responder: res_tx };
        tx.send(cmd).await.unwrap();
        let res = res_rx.await.unwrap();
        println!("get hello: {:?}", res);
    });
    let t2= tokio::spawn(async move {
        let (res_tx, res_rx) = oneshot::channel();

        let cmd = Command::Set { key: "hello".to_string(), value: Bytes::from("world"), responder: res_tx};
        tx2.send(cmd).await.unwrap();
        let res = res_rx.await.unwrap();
        println!("set hello: {:?}", res);
    });
    t2.await.unwrap();
    t1.await.unwrap();
    manager.await.unwrap();
}