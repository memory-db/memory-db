use memory_db::{state::State, tcp::server::TcpServer};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
  let subscriber = FmtSubscriber::builder().with_max_level(Level::TRACE).finish();

  tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
  let mut state = State::default();
  state.init().unwrap();

  TcpServer::new("127.0.0.1:8000", state.clone()).run().await;
}
