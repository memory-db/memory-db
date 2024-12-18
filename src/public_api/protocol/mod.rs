use super::dataquery::DataQuery;

#[derive(Clone, Debug)]
pub enum Request {
    DataQuery(DataQuery),
    Ping,
}

impl From<Vec<u8>> for Request {
    fn from(_bytes_from_request: Vec<u8>) -> Self {
        unimplemented!("")
    }
}

pub struct Response;

impl From<Response> for Vec<u8> {
    fn from(response: Response) -> Self {
        unimplemented!()
    }
}
