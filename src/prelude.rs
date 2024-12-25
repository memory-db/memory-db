use std::{collections::HashMap, sync::Arc};

use bytes::Bytes;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default)]
pub struct DataStore(pub Arc<DashMap<DataStoreKey, DataStoreValue>>);

impl TryFrom<DataStore> for Bytes {
  type Error = ();
  fn try_from(value: DataStore) -> Result<Self, Self::Error> {
    let hash_map: HashMap<DataStoreKey, DataStoreValue> =
      value.0.iter().map(|e| (e.key().clone(), e.value().clone())).collect();
    let bytes: Vec<u8> = bincode::serialize(&hash_map).map_err(|_| ())?;

    Ok(Bytes::from(bytes))
  }
}

impl TryFrom<Bytes> for DataStore {
  type Error = ();
  fn try_from(value: Bytes) -> Result<Self, Self::Error> {
    let hash_map: HashMap<DataStoreKey, DataStoreValue> =
      bincode::deserialize(&value.to_vec()).map_err(|_| ())?;

    let dash_map: DashMap<DataStoreKey, DataStoreValue> = DashMap::from_iter(hash_map.into_iter());

    Ok(DataStore(Arc::new(dash_map)))
  }
}

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
