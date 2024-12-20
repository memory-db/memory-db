use std::{
  collections::HashMap,
  fs::{self, File, OpenOptions},
  io::{Read as _, Write as _},
  path::Path,
  sync::Arc,
  time::Duration,
};

use super::utils;
use chrono::Utc;
use dashmap::DashMap;
use tokio::{task, time::interval};
use tracing::Level;

use crate::{
  log::DataChangeLog,
  prelude::{DataStore, DataStoreKey, DataStoreValue},
  public_api::dataquery::{DataQuery, HandleQuery as _},
};

// Clone: Both fields are behind Arcs.
#[derive(Clone, Default)]
pub struct State {
  pub store: DataStore,
}

impl State {
  /// Loads the newest snapshot it can find into memory.
  /// Then Reads the WAL and replays the data mutation queries. Also sets up thread for writing
  /// snapshots and WAL in background jobs.
  pub fn init(&mut self) -> std::io::Result<()> {
    if Path::new(super::SNAPSHOT_DIR).exists() {
      let mut files = utils::files_in_dir(super::SNAPSHOT_DIR).unwrap();

      if !files.is_empty() {
        utils::sort_snapshot_files(&mut files);

        let newest_snapshot = files[files.len() - 1].path();
        tracing::trace!("Loading snapshot into memory: {:?}", newest_snapshot);

        let mut snapshot_file = File::open(newest_snapshot)?;

        let mut snapshot_buf = Vec::new();
        snapshot_file.read_to_end(&mut snapshot_buf)?;

        let deserialized_snapshot: HashMap<DataStoreKey, DataStoreValue> =
          bincode::deserialize(&snapshot_buf).unwrap();

        let hash_map: DashMap<DataStoreKey, DataStoreValue> =
          deserialized_snapshot.into_iter().collect();

        self.store = Arc::new(hash_map);
      }
    }

    if Path::new(super::WAL_FILE).exists() {
      let data_mutate_logs: Vec<DataChangeLog> =
        utils::read_appended_structs_from_file(super::WAL_FILE)?;

      let dataquery_log: Vec<DataQuery> =
        data_mutate_logs.iter().map(|x| x.clone().query.into()).collect();

      for item in dataquery_log {
        tracing::trace!("Applying log: {:?}", &item);
        item.exec(self.store.clone());
      }
    }

    let store = self.store.clone();
    task::spawn(async move {
      let mut timing = interval(Duration::from_secs(super::SNAPSHOT_WRITE_INTERVAL_SEC));

      // First tick completes immediately
      timing.tick().await;

      loop {
        timing.tick().await;

        let _span = tracing::span!(Level::TRACE, "Snapshot");
        let _span = _span.enter();

        tracing::trace!("Starting snapshot");

        let hash_map: HashMap<DataStoreKey, DataStoreValue> =
          store.iter().map(|e| (e.key().clone(), e.value().clone())).collect();

        let snapshot_rawdata = match bincode::serialize(&hash_map) {
          Ok(data) => data,
          Err(err) => {
            tracing::error!("Binary serialization error: {:?}", err);
            continue;
          }
        };

        let current_date = Utc::now();
        let formatted_date = current_date.format(super::DATE_FMT).to_string();

        let file_name = format!("{formatted_date}-memorydb.dat");

        if let Err(err) = fs::create_dir_all(super::SNAPSHOT_DIR) {
          tracing::error!("File system access error: {:?}", err);
          continue;
        };

        let mut file = OpenOptions::new()
          .write(true)
          .create(true)
          .open(format!("{}/{file_name}", super::SNAPSHOT_DIR))
          .unwrap();

        if let Err(err) = file.write_all(&snapshot_rawdata) {
          tracing::error!("File write error: {:?}", err);
          continue;
        };

        if let Err(err) = file.flush() {
          tracing::error!("File flush error: {:?}", err);
          continue;
        };

        tracing::trace!("Success");

        tracing::trace!("Removed WAL");
        if Path::new(super::WAL_FILE).exists() {
          fs::remove_file(super::WAL_FILE).unwrap();
        }

        tracing::trace!("Cleaning old snapshots");
        let mut files = utils::files_in_dir(super::SNAPSHOT_DIR).unwrap();
        utils::sort_snapshot_files(&mut files);

        if files.len() > super::SNAPSHOT_KEEP_AMOUNT {
          let files_to_remove = files.len() - super::SNAPSHOT_KEEP_AMOUNT;

          for item in files.iter().take(files_to_remove) {
            if let Err(err) = std::fs::remove_file(item.path()) {
              tracing::error!("Old snapshot delete error: {:?}", err);
              continue;
            };
          }
        }
      }
    });

    Ok(())
  }

  //pub async fn handle_incoming(&mut self, incoming: Request) -> Vec<u8> {
  //    match incoming {
  //        Request::DataQuery(query) => self.handle_query(query).await,
  //        Request::Ping => "Pong".as_bytes().to_vec(),
  //    }
  //}

  pub async fn handle_query(&mut self, query: DataQuery) -> Vec<u8> {
    if let Some(logs) = query.as_datachangelogs() {
      for log in logs {
        utils::append_struct_to_file(super::WAL_FILE, &log).unwrap();
      }
    }
    query.exec(self.store.clone())
  }
}