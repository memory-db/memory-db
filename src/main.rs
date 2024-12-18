use std::collections::HashMap;

trait HandleQuery {
    fn should_log(&self) -> bool;
    fn exec(self, datastore: &mut DataStore) -> Vec<u8>;
}

#[derive(Clone)]
struct ReadQuery {
    key: String,
}

impl HandleQuery for ReadQuery {
    fn should_log(&self) -> bool {
        false
    }
    fn exec(self, datastore: &mut DataStore) -> Vec<u8> {
        let Self { key } = self;

        if let Some(value) = datastore.get(&key) {
            let mut value = value.to_vec();
            value.extend("\n".as_bytes().to_vec());
            value
        } else {
            "NOT FOUND\n".as_bytes().to_vec()
        }
    }
}

#[derive(Clone)]
struct PutQuery {
    key: String,
    value: Vec<u8>,
}

impl HandleQuery for PutQuery {
    fn should_log(&self) -> bool {
        true
    }
    fn exec(self, datastore: &mut DataStore) -> Vec<u8> {
        let Self { key, value } = self;
        datastore.upsert(key, value);
        "OK\n".as_bytes().to_vec()
    }
}

#[derive(Clone)]
struct DeleteQuery {
    key: String,
}

impl HandleQuery for DeleteQuery {
    fn should_log(&self) -> bool {
        true
    }
    fn exec(self, datastore: &mut DataStore) -> Vec<u8> {
        let Self { key } = self;

        datastore.delete(&key);

        "OK\n".as_bytes().to_vec()
    }
}

#[derive(Clone)]
enum DataQuery {
    Read(ReadQuery),
    Put(PutQuery),
    Delete(DeleteQuery),
    Batch(Vec<DataQuery>),
}

impl HandleQuery for Vec<DataQuery> {
    fn should_log(&self) -> bool {
        self.iter().all(|x| !matches!(x, DataQuery::Read(_)))
    }
    fn exec(self, datastore: &mut DataStore) -> Vec<u8> {
        self.into_iter()
            .flat_map(|query| {
                if let DataQuery::Batch(_) = query {
                    unimplemented!("Can't nest batches");
                };

                query.exec(datastore)
            })
            .collect()
    }
}

impl HandleQuery for DataQuery {
    fn should_log(&self) -> bool {
        if let DataQuery::Read(_) = self {
            return false;
        };

        if let DataQuery::Batch(batch) = self {
            return batch.should_log();
        };

        true
    }
    fn exec(self, datastore: &mut DataStore) -> Vec<u8> {
        match self {
            DataQuery::Batch(query) => query.exec(datastore),
            DataQuery::Put(query) => query.exec(datastore),
            DataQuery::Read(query) => query.exec(datastore),
            DataQuery::Delete(query) => query.exec(datastore),
        }
    }
}

enum IncomingMessage {
    DataQuery(DataQuery),
    Ping,
}

impl IncomingMessage {
    fn handle(self, store: &mut DataStore) -> Vec<u8> {
        let mut nice = match self {
            IncomingMessage::Ping => return "Pong".as_bytes().to_vec(),
            IncomingMessage::DataQuery(query) => query.exec(store),
        };

        nice.extend("\n".as_bytes().to_vec());
        nice
    }
}

#[derive(Default)]
struct DataStore(HashMap<String, Vec<u8>>);

impl DataStore {
    fn get(&self, key: &str) -> Option<&Vec<u8>> {
        self.0.get(key)
    }

    fn get_mut(&mut self, key: &str) -> Option<&mut Vec<u8>> {
        self.0.get_mut(key)
    }

    fn upsert(&mut self, key: String, value: Vec<u8>) {
        self.0.insert(key, value);
    }

    fn delete(&mut self, key: &str) {
        self.0.remove(key);
    }
}

#[derive(Default)]
struct NodeState {
    store: DataStore,
    mem_log: Vec<DataQuery>,
}

fn main() {
    let mut nice = NodeState::default();

    let queries = Vec::from_iter([
        IncomingMessage::DataQuery(DataQuery::Put(PutQuery {
            key: "test".to_string(),
            value: "nice".to_string().into(),
        })),
        IncomingMessage::DataQuery(DataQuery::Put(PutQuery {
            key: "test2".to_string(),
            value: "nice".to_string().into(),
        })),
        IncomingMessage::DataQuery(DataQuery::Read(ReadQuery {
            key: "test2".to_string(),
        })),
        IncomingMessage::DataQuery(DataQuery::Batch(Vec::from_iter([
            DataQuery::Put(PutQuery {
                key: "test".to_string(),
                value: "nice".to_string().into(),
            }),
            DataQuery::Put(PutQuery {
                key: "test2".to_string(),
                value: "nice2".to_string().into(),
            }),
            DataQuery::Read(ReadQuery {
                key: "test2".to_string(),
            }),
        ]))),
    ]);

    for query in queries {
        if let IncomingMessage::DataQuery(ref dataquery) = query {
            nice.mem_log.push(dataquery.clone());
        }
        let svar = query.handle(&mut nice.store);
        println!("{:?}", unsafe {
            String::from_utf8_unchecked(svar.to_vec())
        });
    }
}

impl From<String> for IncomingMessage {
    fn from(value: String) -> Self {
        if value == "Ping" {
            return Self::Ping;
        };
        unimplemented!()
    }
}
