use std::sync::Arc;

use dashmap::DashMap;

pub type DataStore = Arc<DashMap<String, Vec<u8>>>;
