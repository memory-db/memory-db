use std::{
  error::Error,
  future::{Future, IntoFuture},
  pin::Pin,
  sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use raft::{
  prelude::{Entry, EntryType, HardState, Snapshot},
  Config, RaftState, RawNode, Ready, Storage,
};
use std::time::Duration;
use tokio::time::interval;

// TODO: Handle raft lifecycle and another thread and networking and whatnot wtf.
pub struct RaftNode {
  node: RawNode<MyStorage>,
}

impl RaftNode {
  pub fn new(config: Config, storage: MyStorage) -> Result<Self, Box<dyn Error>> {
    let drain = tracing_slog::TracingSlogDrain;
    let logger = slog::Logger::root(drain, slog::o!());
    let node = RawNode::new(&config, storage, &logger)?;

    Ok(Self { node })
  }

  pub fn tick(&mut self) -> Option<Ready> {
    if self.node.tick() {
      if self.node.has_ready() {
        return None;
      }
      return Some(self.node.ready());
    }
    None
  }
}

impl IntoFuture for RaftNode {
  type Output = Result<(), Box<dyn Error>>;
  type IntoFuture = Pin<Box<dyn Future<Output = Self::Output>>>;
  fn into_future(mut self) -> Self::IntoFuture {
    Box::pin(async move {
      let mut ping = interval(Duration::from_millis(100));

      loop {
        ping.tick().await;

        let Some(mut payload) = self.tick() else {
          continue;
        };

        // https://docs.rs/raft/latest/raft/index.html#processing-the-ready-state

        // Step 1.
        //
        // Check whether messages is empty or not. If not, it means that the node will send messages to other nodes:
        if !payload.messages().is_empty() {
          for msg in payload.take_messages() {
            unimplemented!("Send msgs to other peers.");
          }
        }

        // Step 2.
        //
        // Check whether snapshot is empty or not. If not empty, it means that
        // the Raft node has received a Raft snapshot from the leader and
        // we must apply the snapshot:
        if !payload.snapshot().is_empty() {
          // This is a snapshot, we need to apply the snapshot at first.
          self.node.mut_store().wl().apply_snapshot(payload.snapshot().clone()).unwrap();
        }

        // Step 3.
        //
        // Check whether committed_entries is empty or not.
        // If not, it means that there are some newly committed log entries
        // which you must apply to the state machine.
        //
        // Of course, after applying, you need to update the applied index and resume apply
        // later.
        //
        //
        //
        // NOTE: although Raft guarentees only persisted committed entries will be applied,
        // but it doesn’t guarentee commit index is persisted before being applied.
        // For example, if application is restarted after applying committed entries
        // before persisting commit index, apply index can be larger than commit index and cause panic.
        //
        // To solve the problem, persisting commit index with or before applying entries.
        // You can also always assign commit index to the max(commit_index, applied_index) after restarting,
        // it may work but potential log loss may also be ignored silently.
        let mut _last_apply_index = 0;
        for entry in payload.take_committed_entries() {
          _last_apply_index = entry.index;

          if entry.data.is_empty() {
            // Emtpy entry, when the peer becomes Leader it will send an empty entry.
            continue;
          }

          todo!("This bit:")
          //match entry.get_entry_type() {
          //  EntryType::EntryNormal => handle_normal(entry),
          //  // It's recommended to always use `EntryType::EntryConfChangeV2.
          //  EntryType::EntryConfChange => handle_conf_change(entry),
          //  EntryType::EntryConfChangeV2 => handle_conf_change_v2(entry),
          //}
        }

        // Step 4.
        //
        // Check whether entries is empty or not.
        // If not empty, it means that there are newly added entries but have not been committed yet,
        // we must append the entries to the Raft log
        if !payload.entries().is_empty() {
          self.node.mut_store().wl().append(payload.entries()).unwrap();
        }

        // Step 5.
        //
        // Check whether hs is empty or not. If not empty,
        // it means that the HardState of the node has changed.
        // For example, the node may vote for a new leader, or the commit index has been increased.
        //
        // We must persist the changed HardState
        if let Some(hs) = payload.hs() {
          self.node.mut_store().wl().set_hardstate(hs.clone());
        }

        // Step 6.
        //
        // Check whether persisted_messages is empty or not.
        // If not, it means that the node will send messages to other nodes after persisting hardstate,
        // entries and snapshot
        if !payload.persisted_messages().is_empty() {
          for msg in payload.take_persisted_messages() {
            todo!("Todo what the fuck to do here")
            // Send persisted messages to other peers.
          }
        }

        // Step 7.
        //
        // Call advance to notify that the previous work is completed.
        // Get the return value LightReady and handle its messages and committed_entries like step 1 and step 3 does.
        // Then call advance_apply to advance the applied index inside.
        let mut light_rd = self.node.advance(payload);

        // WHAT THE FUCK

        //handle_messages(light_rd.take_messages());
        //handle_committed_entries(light_rd.take_committed_entries());
        self.node.advance_apply();
      }
    })
  }
}

#[derive(Default)]
pub struct MyStorage {
  core: Arc<RwLock<MyStorageCore>>,
}

#[derive(Default)]
pub struct MyStorageCore {
  entries: Vec<Entry>,
}

impl MyStorageCore {
  /// Example implementation: https://docs.rs/raft/latest/src/raft/storage.rs.html#243
  pub fn apply_snapshot(&mut self, snapshot: Snapshot) -> raft::Result<()> {
    let _ = snapshot;
    unimplemented!()
  }

  pub fn append(&mut self, entries: &[Entry]) -> raft::Result<()> {
    self.entries.extend(entries.to_vec());

    Ok(())
  }

  pub fn set_hardstate(&mut self, hardstate: HardState) {
    unimplemented!()
  }
}

impl MyStorage {
  pub fn wl(&self) -> RwLockWriteGuard<'_, MyStorageCore> {
    self.core.write().unwrap()
  }

  pub fn rl(&self) -> RwLockReadGuard<'_, MyStorageCore> {
    self.core.read().unwrap()
  }
}

impl Storage for MyStorage {
  fn initial_state(&self) -> raft::Result<raft::RaftState> {
    let raftstate = RaftState::default();

    Ok(raftstate)
  }

  fn term(&self, idx: u64) -> raft::Result<u64> {
    let rl_self = self.rl();
    let entry = rl_self
      .entries
      .get(idx as usize)
      .ok_or(raft::Error::Store(raft::StorageError::Unavailable))?
      .term;
    drop(rl_self);

    Ok(entry)
  }

  fn entries(
    &self,
    low: u64,
    high: u64,
    max_size: impl Into<Option<u64>>,
    _context: raft::GetEntriesContext,
  ) -> raft::Result<Vec<raft::prelude::Entry>> {
    let rl_self = self.rl();
    let mut slice: Vec<Entry> = rl_self.entries[low as usize..high as usize].to_vec();
    drop(rl_self);

    if let Some(max_size) = max_size.into() {
      slice.truncate(max_size as usize);
    };

    Ok(slice)
  }

  fn snapshot(&self, request_index: u64, to: u64) -> raft::Result<raft::prelude::Snapshot> {
    unimplemented!()
  }

  fn last_index(&self) -> raft::Result<u64> {
    let rl_self = self.rl();
    Ok(rl_self.entries.len() as u64 - 1)
  }

  fn first_index(&self) -> raft::Result<u64> {
    Ok(0)
  }
}

//impl Default for RaftNode {
//  fn default() -> Self {
//    let config = Config::new(1);
//
//    let decorator = slog_term::TermDecorator::new().build();
//    let drain = slog_term::FullFormat::new(decorator).build().fuse();
//    let drain = slog_async::Async::new(drain)
//      .chan_size(4096)
//      .overflow_strategy(slog_async::OverflowStrategy::Block)
//      .build()
//      .fuse();
//    let logger = slog::Logger::root(drain, o!());
//    let raw_node = RawNode::new(&config, MyStorage::default(), &logger).unwrap();
//
//    Raft { node: raw_node }
//  }
//}
