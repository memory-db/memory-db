use std::collections::HashMap;

use tokio::{
    io::{AsyncBufReadExt as _, AsyncReadExt as _, BufReader},
    net::{TcpListener, TcpStream},
};

use crate::{
    node_state::NodeState,
    public_api::protocol::{Request, Response},
};

pub struct TcpServer {
    address: String,
    state: NodeState,
}

async fn read_payload(stream: &mut TcpStream, payload: &mut Vec<u8>) {
    let mut buf_reader = BufReader::new(stream);
    buf_reader.read_until(b'\0', payload).await.unwrap();
    payload.pop();
}

impl TcpServer {
    pub fn new(address: &str, state: NodeState) -> Self {
        TcpServer {
            address: address.to_string(),
            state,
        }
    }

    pub async fn handle_conn(&mut self, mut stream: TcpStream) {
        let mut payload = Vec::new();
        read_payload(&mut stream, &mut payload).await;

        let parts: Vec<&[u8]> = payload.split(|x| *x == b'\r').collect();
        let header_bytes = parts[0];
        let raw_body = parts[1];

        let headers_vec = header_bytes
            .split(|x| *x == b'\n')
            .fold(Vec::new(), |mut acc, test| {
                let test: Vec<&[u8]> = test.split(|x| *x == b'=').collect();

                let key = test[0];
                let value = test[1];

                acc.push((
                    String::from_utf8(key.to_vec()).unwrap(),
                    String::from_utf8(value.to_vec()).unwrap(),
                ));
                acc
            });
        let headers_map: HashMap<String, String> = HashMap::from_iter(headers_vec);

        println!("REQUEST:");
        println!("Headers:\n{:#?}", headers_map);
        println!("Body:\n{:?}", String::from_utf8(raw_body.to_vec()).unwrap());
    }

    pub async fn run(&mut self) {
        let listener = TcpListener::bind(&self.address)
            .await
            .unwrap_or_else(|_| panic!("Could not bind to address {}", self.address));

        println!("Server running on {}", self.address);

        loop {
            match listener.accept().await {
                Ok((stream, _)) => self.handle_conn(stream).await,
                Err(e) => {
                    eprintln!("Connection failed: {}", e);
                }
            };
        }
    }
}
