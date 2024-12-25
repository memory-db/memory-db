use std::{error::Error, time::Duration};

use tokio::{
  sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
  task,
  time::interval,
};

use crate::{
  storage::{DataBaseStorage, RaftNode},
  tcp::server::TcpServer,
};

pub struct App {
  tcp: TcpServer,
}

impl App {
  fn spawn_raft_statemachine(storage: DataBaseStorage, sender: UnboundedSender<()>) {
    task::spawn(async move {
      let config = raft::Config { id: rand::random(), ..Default::default() };

      config.validate().unwrap();

      let mut raft_node = RaftNode::new(&config, storage).unwrap();
      let mut schedule = interval(Duration::from_millis(100));

      loop {
        schedule.tick().await;

        match raft_node.tick().await {
          Ok(thing) => {
            if let Some(stuff) = thing {
              for item in stuff {
                sender.send(item).unwrap();
              }
            }
          }
          Err(err) => tracing::error!("Raft state machine error: {:?}", err),
        }
      }
    });
  }
  fn new() -> Result<App, Box<dyn Error>> {
    let storage = DataBaseStorage::default();
    let tcp = TcpServer::new("0.0.0.0:8080", state);

    Ok(Self { tcp })
  }

  pub async fn run(&mut self) {
    let (sender, mut receiver) = mpsc::unbounded_channel::<()>();
    App::spawn_raft_statemachine(sender);

    while let Some(value) = receiver.recv().await {
      // TODO Handle value here
      let _: () = value;
    }
  }
}
