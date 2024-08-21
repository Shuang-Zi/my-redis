pub use tokio::net::{TcpListener, TcpStream};
pub use mini_redis::{Connection, Frame};

#[tokio::main]
async fn main () {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            process(socket).await;
        });
    }
}


async fn process(socket: TcpStream) {
    use std::collections::HashMap;
    use mini_redis::Command::{self, Get, Set};
    let mut connection = Connection::new(socket);
    let mut db = HashMap::new();
    // 从socket 中读取数据帧
    while let Some(frame) = connection.read_frame().await.unwrap() {
        println!("GOT: {:?}", frame);
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                db.insert(cmd.key().to_string(), cmd.value().to_vec());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
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
