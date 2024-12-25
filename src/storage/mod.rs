use std::{
  error::Error,
  sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use bytes::Bytes;
use raft::{
  prelude::{Entry, HardState, Snapshot, SnapshotMetadata},
  Config, RaftState, RawNode, Ready, Storage,
};

use crate::prelude::DataStore;

// TODO: Handle raft lifecycle and another thread and networking and whatnot wtf.
pub struct RaftNode {
  node: RawNode<DatabaseStorage>,
}

impl RaftNode {
  pub fn new(config: &Config, storage: DatabaseStorage) -> Result<Self, Box<dyn Error>> {
    let drain = tracing_slog::TracingSlogDrain;
    let logger = slog::Logger::root(drain, slog::o!());
    let node = RawNode::new(config, storage, &logger)?;

    Ok(Self { node })
  }

  fn raft_tick(&mut self) -> Option<Ready> {
    if self.node.tick() {
      if self.node.has_ready() {
        return None;
      }
      return Some(self.node.ready());
    }
    None
  }

  // returntype () should be some sort of action that later is queued.
  pub async fn tick(&mut self) -> Result<Option<Vec<()>>, ()> {
    let Some(mut payload) = self.raft_tick() else {
      return Ok(None);
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
      self.node.mut_store().apply_snapshot(payload.snapshot().clone()).unwrap();
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
        return Ok(None);
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
    let light_rd = self.node.advance(payload);

    // WHAT THE FUCK

    //handle_messages(light_rd.take_messages());
    //handle_committed_entries(light_rd.take_committed_entries());
    self.node.advance_apply();
    // temporary
    Ok(None)
  }
}

#[derive(Default)]
pub struct DatabaseStorage {
  core: Arc<RwLock<MyStorageCore>>,
  store: DataStore,
}

#[derive(Default)]
pub struct MyStorageCore {
  raft_state: RaftState,
  entries: Vec<Entry>,
  next_snapshot_metadata: SnapshotMetadata,
}

impl MyStorageCore {
  /// Example implementation: https://docs.rs/raft/latest/src/raft/storage.rs.html#243

  pub fn append(&mut self, entries: &[Entry]) -> raft::Result<()> {
    self.entries.extend_from_slice(entries);

    Ok(())
  }

  pub fn set_hardstate(&mut self, hardstate: HardState) {
    self.raft_state.hard_state = hardstate;
  }
  fn first_index(&self) -> u64 {
    match self.entries.first() {
      Some(e) => e.index,
      None => self.next_snapshot_metadata.index + 1,
    }
  }
  fn last_index(&self) -> u64 {
    match self.entries.last() {
      Some(e) => e.index,
      None => self.next_snapshot_metadata.index,
    }
  }
}

// TODO: Implement startup behavior. for example WAL and load snapshots from fs.
impl DatabaseStorage {
  pub fn apply_snapshot(&mut self, snapshot: Snapshot) -> raft::Result<()> {
    self.wl().next_snapshot_metadata = snapshot.get_metadata().clone();
    self.store = DataStore::try_from(snapshot.data).unwrap();

    Ok(())
  }
  pub fn wl(&self) -> RwLockWriteGuard<'_, MyStorageCore> {
    self.core.write().unwrap()
  }

  pub fn rl(&self) -> RwLockReadGuard<'_, MyStorageCore> {
    self.core.read().unwrap()
  }

  pub fn last_entry_index(&self) -> Option<u64> {
    self.rl().entries.iter().last().map(|x| x.index)
  }
}

impl Storage for DatabaseStorage {
  fn initial_state(&self) -> raft::Result<raft::RaftState> {
    Ok(self.rl().raft_state.clone())
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
    let mut slice: Vec<Entry> = self.rl().entries[low as usize..high as usize].to_vec();

    if let Some(max_size) = max_size.into() {
      slice.truncate(max_size as usize);
    };

    Ok(slice)
  }

  fn snapshot(
    &self,
    request_index: u64,
    _to_peer_id: u64,
  ) -> raft::Result<raft::prelude::Snapshot> {
    if self.rl().next_snapshot_metadata.index < request_index {
      return Err(raft::Error::Store(raft::StorageError::SnapshotTemporarilyUnavailable));
    }

    let mut snapshot = Snapshot::default();

    *snapshot.mut_metadata() = self.rl().next_snapshot_metadata.clone();
    *snapshot.mut_data() = Bytes::try_from(self.store.clone()).unwrap();

    Ok(snapshot)
  }

  fn last_index(&self) -> raft::Result<u64> {
    Ok(self.rl().last_index())
  }

  fn first_index(&self) -> raft::Result<u64> {
    Ok(self.rl().first_index())
  }
}
