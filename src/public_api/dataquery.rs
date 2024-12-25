use crate::tcp::server::CommandV0;
use serde::{Deserialize, Serialize};

use crate::prelude::{DataStore, DataStoreKey};

pub trait HandleQuery {
  fn exec(self, datastore: DataStore) -> Vec<u8>;
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReadQuery {
  pub key: String,
}

impl HandleQuery for ReadQuery {
  fn exec(self, datastore: DataStore) -> Vec<u8> {
    let Self { key } = self;

    if let Some(value) = datastore.0.get(&key.as_str().into()) {
      let mut value = value.0.to_vec();
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
    datastore.0.insert(key.as_str().into(), value.into());
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

    datastore.0.remove(&DataStoreKey::from(key.as_str()));

    "OK\n".as_bytes().to_vec()
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum DataQuery {
  Read(ReadQuery),
  Put(PutQuery),
  Delete(DeleteQuery),
}

impl HandleQuery for Vec<DataQuery> {
  fn exec(self, datastore: DataStore) -> Vec<u8> {
    self.into_iter().flat_map(|query| query.exec(datastore.clone())).collect()
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

#[derive(Debug)]
pub struct InvalidBody;

impl TryFrom<(CommandV0, Vec<u8>)> for DataQuery {
  type Error = InvalidBody;
  fn try_from((cmd, body): (CommandV0, Vec<u8>)) -> Result<Self, Self::Error> {
    let value = match cmd {
      CommandV0::Get => {
        let query: ReadQuery = bincode::deserialize(&body).map_err(|_| InvalidBody)?;
        DataQuery::Read(query)
      }

      CommandV0::Put => {
        let query: PutQuery = bincode::deserialize(&body).map_err(|_| InvalidBody)?;
        DataQuery::Put(query)
      }
      CommandV0::Delete => {
        let query: DeleteQuery = bincode::deserialize(&body).map_err(|_| InvalidBody)?;
        DataQuery::Delete(query)
      }
      _ => unreachable!(),
    };
    Ok(value)
  }
}
