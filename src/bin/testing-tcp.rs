use std::io;

use memory_db::{
  public_api::dataquery::{PutQuery, ReadQuery},
  tcp::protocol::{RawRequest, RawResponse},
};
use tokio::net::TcpStream;

async fn send(cmd: u8, body: Vec<u8>) -> io::Result<RawResponse> {
  let address = "127.0.0.1:8000"; // Replace with your server's address and port
  let mut stream = TcpStream::connect(address).await?;

  let req = RawRequest::new(cmd, body);
  req.write_to_tcp_stream(&mut stream).await.unwrap();

  let response = RawResponse::from_tcp_stream(&mut stream).await.unwrap();
  Ok(response)
}

#[tokio::main]
async fn main() -> io::Result<()> {
  let response = send(
    2,
    bincode::serialize(&PutQuery { key: "test2".to_string(), value: b"hello".to_vec() }).unwrap(),
  )
  .await?;

  println!("Raw Response: {response:?}");
  println!("Response body String: {}", String::from_utf8(response.body).unwrap());

  let response = send(
    2,
    bincode::serialize(&PutQuery { key: "test".to_string(), value: b"hello".to_vec() }).unwrap(),
  )
  .await?;

  println!("Raw Response: {response:?}");
  println!("Response body String: {}", String::from_utf8(response.body).unwrap());

  let response =
    send(1, bincode::serialize(&ReadQuery { key: "test".to_string() }).unwrap()).await?;

  println!("Raw Response: {response:?}");
  println!("Response body String: {}", String::from_utf8(response.body).unwrap());

  Ok(())
}
