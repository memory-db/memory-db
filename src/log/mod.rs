use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::public_api::dataquery::{DataQuery, DeleteQuery, PutQuery};

impl DataQuery {
    pub fn as_datachangelogs(&self) -> Option<Vec<DataChangeLog>> {
        let date = Utc::now().timestamp();
        match self {
            DataQuery::Read(_) => None,
            DataQuery::Put(put_query) => Some(Vec::from_iter([DataChangeLog {
                query: DataChangeQuery::Put(put_query.clone()),
                date,
            }])),
            DataQuery::Delete(delete_query) => Some(Vec::from_iter([DataChangeLog {
                query: DataChangeQuery::Delete(delete_query.clone()),
                date,
            }])),
        }
    }
}

impl From<DataChangeQuery> for DataQuery {
    fn from(value: DataChangeQuery) -> Self {
        match value {
            DataChangeQuery::Delete(query) => DataQuery::Delete(query),
            DataChangeQuery::Put(query) => DataQuery::Put(query),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DataChangeLog {
    pub query: DataChangeQuery,
    date: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DataChangeQuery {
    Put(PutQuery),
    Delete(DeleteQuery),
}
