use mini_redis::{Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

async fn process(socket: TcpStream) {
    use mini_redis::Command::{self, Get, Set};
    use std::collections::HashMap;

    //データを蓄えるためのHashMap
    let mut db = HashMap::new();

    //mini-redisが提供するコネクションによってソケットからのフレームの読み書きを行う
    let mut connection = Connection::new(socket);

    //コネクションからコマンドを受け取るためread_frameを呼び出す
    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                //Vec<u8>として保存
                db.insert(cmd.key().to_string(), cmd.value().to_vec());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                if let Some(value) = db.get(cmd.key()) {
                    // Frame::BulkはデータがByte型であることを期待する
                    // into()を使ってVec<u8>をBytesに変換する
                    Frame::Bulk(value.clone().into())
                } else {
                    Frame::Null
                }
            }

            cmd => panic!("unimplemented command: {:?}", cmd),
        };
        //クライアントにレスポンスを返す
        connection.write_frame(&response).await.unwrap();
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            process(socket).await;
        });
    }
}
