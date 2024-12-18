use memory_db::node_state::NodeState;
use memory_db::public_api::dataquery::{DataQuery, PutQuery};
use memory_db::public_api::protocol::Request;
use memory_db::tcp_server::TcpServer;

#[tokio::main]
async fn main() {
    let mut state = NodeState::new();
    state.init().unwrap();

    let mut server = TcpServer::new("127.0.0.1:8000", state.clone());

    let queries = Vec::from_iter([
        Request::DataQuery(DataQuery::Put(PutQuery {
            key: "test".to_string(),
            value: "nice".to_string().into(),
        })),
        Request::DataQuery(DataQuery::Put(PutQuery {
            key: "test2".to_string(),
            value: "nice".to_string().into(),
        })),
    ]);

    for query in queries {
        let response = state.handle_incoming(query).await;
        println!("Response: {:?}", String::from_utf8(response).unwrap())
    }

    server.run().await;

    //std::thread::sleep(Duration::from_millis(400));
    //
    //let queries = Vec::from_iter([
    //    Request::DataQuery(DataQuery::Delete(DeleteQuery {
    //        key: "test2".to_string(),
    //    })),
    //    Request::DataQuery(DataQuery::Read(ReadQuery {
    //        key: "test2".to_string(),
    //    })),
    //    Request::DataQuery(DataQuery::Put(PutQuery {
    //        key: "tes2t".to_string(),
    //        value: "yes very nice".as_bytes().to_vec(),
    //    })),
    //    Request::DataQuery(DataQuery::Read(ReadQuery {
    //        key: "tes2t".to_string(),
    //    })),
    //]);
    //
    //for query in queries {
    //    let response = nice.handle_incoming(query).await;
    //    println!("Response: {:?}", unsafe {
    //        String::from_utf8_unchecked(response)
    //    })
    //}
    //std::thread::sleep(Duration::from_millis(400));
    //println!("store: {:?}", nice.store);
}
