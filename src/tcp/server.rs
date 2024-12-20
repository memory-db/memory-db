use tokio::net::{TcpListener, TcpStream};

use crate::{
  public_api::dataquery::DataQuery,
  state::State,
  tcp::protocol::{RawRequest, RawResponse},
};

pub struct TcpServer {
  address: String,
  state: State,
}

#[derive(Debug, Copy, Clone)]
pub enum CommandV0 {
  /// 0
  Ping,
  /// 1
  Get,
  /// 2
  Put,
  /// 3
  Delete,
}

impl TryFrom<u8> for CommandV0 {
  type Error = ();
  fn try_from(value: u8) -> Result<Self, Self::Error> {
    let res = match value {
      0 => CommandV0::Ping,
      1 => CommandV0::Get,
      2 => CommandV0::Put,
      3 => CommandV0::Delete,
      _ => return Err(()),
    };

    Ok(res)
  }
}

impl TcpServer {
  pub fn new(address: &str, state: State) -> Self {
    TcpServer { address: address.to_string(), state }
  }

  pub async fn handle_conn(&mut self, mut stream: TcpStream) {
    let req = RawRequest::from_tcp_stream(&mut stream).await.unwrap();

    let cmd: CommandV0 = CommandV0::try_from(req.command).unwrap();
    let data_query: DataQuery = DataQuery::try_from((cmd, req.body)).unwrap();

    let response_bytes = self.state.handle_query(data_query).await;
    let response = RawResponse::new(0, response_bytes);
    response.write_to_tcp_stream(stream).await.unwrap();
  }

  pub async fn run(&mut self) {
    let listener = TcpListener::bind(&self.address)
      .await
      .unwrap_or_else(|_| panic!("Could not bind to address {}", self.address));

    tracing::info!("Server running on {}", self.address);

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
