use std::io;

use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::TcpStream;

/// This is the raw request that is received from tcp. This doesn't change based on request
/// versions, since it operates with raw bytes. This acts as the first formating.
#[derive(Debug)]
pub struct RawResponse {
  pub version: u8,
  pub r#type: u8,
  pub body: Vec<u8>,
}

impl RawResponse {
  pub fn new(r#type: u8, body: Vec<u8>) -> Self {
    RawResponse {
      version: 0,
      r#type,
      //body_len: body.len() as u16,
      body,
    }
  }
  pub async fn write_to_tcp_stream(self, mut tcp_stream: TcpStream) -> io::Result<()> {
    let mut to_write = Vec::new();

    to_write.push(self.version);
    to_write.push(self.r#type);
    to_write.extend((self.body.len() as u16).to_be_bytes());
    to_write.extend(self.body);

    tcp_stream.write_all(&to_write).await?;

    Ok(())
  }

  pub async fn from_tcp_stream(tcp_stream: &mut TcpStream) -> Result<Self, ParsingResponseError> {
    let version = tcp_stream.read_u8().await.map_err(|_| ParsingResponseError::InvalidVersion)?;
    let r#type = tcp_stream.read_u8().await.map_err(|_| ParsingResponseError::InvalidType)?;

    let body_len = tcp_stream.read_u16().await.map_err(|_| ParsingResponseError::InvalidBodyLen)?;

    let mut body: Vec<u8> = vec![0u8; body_len.into()];
    tcp_stream
      .read_exact(&mut body)
      .await
      .map_err(|_| ParsingResponseError::BodylengthMissmatch)?;

    let raw_req = RawResponse { version, r#type, body };

    Ok(raw_req)
  }
}

#[derive(Debug)]
pub enum ParsingResponseError {
  InvalidVersion,
  InvalidType,

  InvalidBodyLen,
  BodylengthMissmatch,
}
