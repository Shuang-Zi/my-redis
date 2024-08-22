pub use tokio::net::{TcpListener, TcpStream};
pub use mini_redis::{Connection, Frame};
pub use bytes::Bytes;
pub use std::sync::{Arc, Mutex};
pub use std::collections::HashMap;

type Db = Arc<Mutex<HashMap<String, Bytes>>>;

#[tokio::main]
async fn main () {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let db = Arc::new(Mutex::new(HashMap::new()));
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let db1 = db.clone();
        tokio::spawn(async move {
            process(socket, db1).await;
        });
    }
}


async fn process(socket: TcpStream, db: Db) {
    use mini_redis::Command::{self, Get, Set};
    let mut connection = Connection::new(socket);
    // 从socket 中读取数据帧
    while let Some(frame) = connection.read_frame().await.unwrap() {
        println!("GOT: {:?}", frame);
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                let mut db = db.lock().unwrap();
                db.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                let db = db.lock().unwrap();
                if let Some(value) = db.get(cmd.key()) {
                    Frame::Bulk(value.clone().into())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented {:?}", cmd),
        };
        connection.write_frame(&response).await.unwrap();
    }
}
