use std::{
  error::Error,
  future::{Future, IntoFuture},
  pin::Pin,
  time::Duration,
};

use tokio::time::interval;

use crate::{
  state::State,
  storage::{MyStorage, RaftNode},
  tcp::server::TcpServer,
};

pub struct App {
  tcp: TcpServer,
  raft_node: RaftNode,
  state: State,
}

impl App {
  fn new() -> Result<App, Box<dyn Error>> {
    let mut state = State::default();
    state.init();
    let tcp = TcpServer::new("0.0.0.0:8080", state);

    let storage = MyStorage::default();
    let config = raft::Config { id: rand::random(), ..Default::default() }.validate()?;
    let raft_node = RaftNode::new(&config, &storage)?;

    Ok(Self { tcp, state, raft_node })
  }
}

impl IntoFuture for App {
  type Output = Result<(), Box<dyn Error>>;
  type IntoFuture = Pin<Box<dyn Future<Output = Self::Output>>>;
  fn into_future(self) -> Self::IntoFuture {
    Box::pin(async move {
      let ping = interval(Duration::from_millis(100));

      ping.tick().await;

      loop {
        ping.tick().await;

        // https://docs.rs/raft/latest/raft/index.html#processing-the-ready-state
        if let Some(mut payload) = self.raft_node.tick() {
          if !payload.messages().is_empty() {
            for msg in payload.take_messages() {
              unimplemented!("Send msgs to other peers.");
            }
          }

          if !payload.snapshot().is_empty() {
            // This is a snapshot, we need to apply the snapshot at first.
            //self.raft_node.mut_store().wl().apply_snapshot(ready.snapshot().clone()).unwrap();
          }
        }
      }

      Ok(())
    })
  }
}
