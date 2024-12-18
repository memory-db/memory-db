use serde::{Deserialize, Serialize};

use crate::prelude::DataStore;

pub trait HandleQuery {
    fn exec(self, datastore: DataStore) -> Vec<u8>;
}

#[derive(Clone, Debug)]
pub struct ReadQuery {
    pub key: String,
}

impl HandleQuery for ReadQuery {
    fn exec(self, datastore: DataStore) -> Vec<u8> {
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PutQuery {
    pub key: String,
    pub value: Vec<u8>,
}

impl HandleQuery for PutQuery {
    fn exec(self, datastore: DataStore) -> Vec<u8> {
        let Self { key, value } = self;
        datastore.insert(key, value);
        "OK\n".as_bytes().to_vec()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeleteQuery {
    pub key: String,
}

impl HandleQuery for DeleteQuery {
    fn exec(self, datastore: DataStore) -> Vec<u8> {
        let Self { key } = self;

        datastore.remove(&key);

        "OK\n".as_bytes().to_vec()
    }
}

#[derive(Clone, Debug)]
pub enum DataQuery {
    Read(ReadQuery),
    Put(PutQuery),
    Delete(DeleteQuery),
}

impl HandleQuery for Vec<DataQuery> {
    fn exec(self, datastore: DataStore) -> Vec<u8> {
        self.into_iter()
            .flat_map(|query| query.exec(datastore.clone()))
            .collect()
    }
}

impl HandleQuery for DataQuery {
    fn exec(self, datastore: DataStore) -> Vec<u8> {
        match self {
            DataQuery::Put(query) => query.exec(datastore),
            DataQuery::Read(query) => query.exec(datastore),
            DataQuery::Delete(query) => query.exec(datastore),
        }
    }
}
