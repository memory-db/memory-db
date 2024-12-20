use std::sync::Arc;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};

pub type DataStore = Arc<DashMap<DataStoreKey, DataStoreValue>>;

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct DataStoreKey(pub Arc<str>);

impl<'a> From<&'a str> for DataStoreKey {
  fn from(value: &'a str) -> Self {
    DataStoreKey(Arc::from(value))
  }
}

impl Serialize for DataStoreKey {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(&self.0)
  }
}

impl<'a> Deserialize<'a> for DataStoreKey {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'a>,
  {
    let test: &str = Deserialize::deserialize(deserializer)?;
    Ok(test.into())
  }
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct DataStoreValue(pub Arc<[u8]>);

impl From<Vec<u8>> for DataStoreValue {
  fn from(value: Vec<u8>) -> Self {
    DataStoreValue(Arc::from(value.as_slice()))
  }
}

impl<'a> From<&'a [u8]> for DataStoreValue {
  fn from(value: &'a [u8]) -> Self {
    DataStoreValue(Arc::from(value))
  }
}

impl Serialize for DataStoreValue {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_bytes(&self.0)
  }
}

impl<'a> Deserialize<'a> for DataStoreValue {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'a>,
  {
    let test: &[u8] = Deserialize::deserialize(deserializer)?;
    Ok(test.into())
  }
}
