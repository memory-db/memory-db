use std::io;

use tokio::{
  io::{AsyncReadExt as _, AsyncWriteExt as _},
  net::TcpStream,
};
#[derive(Debug)]
pub enum ParsingRequestError {
  InvalidVersion,
  InvalidCommand,
  BodylengthMissmatch,
}

/// This is the raw request that is received from tcp. This doesn't change based on request
/// versions, since it operates with raw bytes. This acts as the first formating.
#[derive(Debug)]
pub struct RawRequest {
  pub version: u8,
  pub command: u8,
  pub body: Vec<u8>,
}

impl RawRequest {
  pub fn new(command: u8, body: Vec<u8>) -> Self {
    Self { command, body, version: 0 }
  }
  pub async fn write_to_tcp_stream(self, tcp_stream: &mut TcpStream) -> io::Result<()> {
    let mut to_write = Vec::new();

    to_write.push(self.version);
    to_write.push(self.command);
    to_write.extend((self.body.len() as u16).to_be_bytes());
    to_write.extend(self.body);

    tcp_stream.write_all(&to_write).await?;

    Ok(())
  }

  pub async fn from_tcp_stream(tcp_stream: &mut TcpStream) -> Result<Self, ParsingRequestError> {
    let version = tcp_stream.read_u8().await.map_err(|_| ParsingRequestError::InvalidVersion)?;
    let command = tcp_stream.read_u8().await.map_err(|_| ParsingRequestError::InvalidCommand)?;

    let body_len = tcp_stream.read_u16().await.map_err(|_| ParsingRequestError::InvalidCommand)?;

    let mut body: Vec<u8> = vec![0u8; body_len.into()];
    tcp_stream.read_exact(&mut body).await.map_err(|_| ParsingRequestError::BodylengthMissmatch)?;

    let raw_req = RawRequest { version, command, body };

    Ok(raw_req)
  }
}
