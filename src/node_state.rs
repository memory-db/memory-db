use std::{path::Path, sync::Arc, time::Duration};

use tokio::{sync::Mutex, task, time::interval};

use crate::{
    io,
    log::DataChangeLog,
    prelude::DataStore,
    public_api::{
        dataquery::{DataQuery, HandleQuery as _},
        protocol::Request,
    },
};

const LOGFILE: &str = "memorydb.log";

// Clone: Both fields are behind Arcs.
#[derive(Clone)]
pub struct NodeState {
    pub store: DataStore,
    log: Arc<Mutex<Vec<DataChangeLog>>>,
}

impl NodeState {
    pub fn new() -> Self {
        Self {
            store: DataStore::default(),
            log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Reads the log and replays the data mutation queries. Also sets up thread for writing future data
    /// mutation queries.
    pub fn init(&mut self) -> std::io::Result<()> {
        // The last index of the vec that has been written to log. INCLUSIVE
        let mut index_file_written: Option<usize> = None;

        if Path::new(LOGFILE).exists() {
            let data_mutate_logs: Vec<DataChangeLog> =
                io::read_appended_structs_from_file(LOGFILE)?;

            let dataquery_log: Vec<DataQuery> = data_mutate_logs
                .iter()
                .map(|x| x.clone().query.into())
                .collect();
            let log_len = dataquery_log.len();

            self.log = Arc::new(Mutex::new(data_mutate_logs));

            for item in dataquery_log {
                println!("Applying: {:?}", &item);
                item.exec(self.store.clone());
            }

            if log_len != 0 {
                index_file_written = Some(log_len - 1);
            }
        }

        let log = self.log.clone();
        task::spawn(async move {
            let mut timing = interval(Duration::from_millis(250));

            // First tick completes immediately
            timing.tick().await;

            loop {
                timing.tick().await;
                let data_mutations_log = log.lock().await;

                // If mutation log is empty we don't do anything
                if data_mutations_log.is_empty() {
                    drop(data_mutations_log);
                    continue;
                }

                // This will run if, for example, user manually removes file.
                if !Path::new(LOGFILE).exists() {
                    index_file_written = None;
                }

                // If there is no newer records from the point of past written logs. Skip log
                // write.
                if let Some(index_of_written_logs) = index_file_written {
                    // index of written logs is inclusive.
                    if index_of_written_logs == data_mutations_log.len() - 1 {
                        drop(data_mutations_log);
                        continue;
                    }
                }

                // index_file_written is INCLUSIVE
                let index_to_start_log_write = match index_file_written {
                    None => 0,
                    Some(v) => v + 1,
                };

                // Data mutations log is guaranteed to be > 0
                let index_to_end_log_write = data_mutations_log.len() - 1;
                let log_to_write = data_mutations_log
                    [index_to_start_log_write..index_to_end_log_write + 1] // why + 1? End is non-inclusive
                    .to_vec();

                // Lock is released
                drop(data_mutations_log);

                for log_entry in log_to_write {
                    println!("Writing: {:?}", &log_entry);
                    io::append_struct_to_file(LOGFILE, &log_entry).unwrap();
                }
                index_file_written = Some(index_to_end_log_write);
            }
        });
        Ok(())
    }

    pub async fn handle_incoming(&mut self, incoming: Request) -> Vec<u8> {
        match incoming {
            Request::DataQuery(query) => self.handle_query(query).await,
            Request::Ping => "Pong".as_bytes().to_vec(),
        }
    }

    pub async fn handle_query(&mut self, query: DataQuery) -> Vec<u8> {
        if let Some(logs) = query.as_datachangelogs() {
            let mut _log = self.log.lock().await;
            _log.extend(logs);
        }
        query.exec(self.store.clone())
    }
}
