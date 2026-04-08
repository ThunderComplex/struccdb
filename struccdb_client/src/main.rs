use serde::{Deserialize, Serialize};
use tonic::Request;

use crate::database::{
    FindQueryRequest, FindQueryResponse, InsertRequest, InsertResponse,
    db_service_client::DbServiceClient,
    db_service_server::{DbService, DbServiceServer},
};

pub mod database {
    tonic::include_proto!("database");
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AwesomeTest {
    id: u64,
    a_test: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = DbServiceClient::connect("http://[::1]:50051").await?;
    let mut awesome_instance = AwesomeTest::default();
    awesome_instance.id = 2;
    awesome_instance.a_test = "Wawo".into();

    let mut serialized_data: Vec<u8> = ron::to_string(&awesome_instance).unwrap().into_bytes();

    let request = Request::new(InsertRequest {
        struct_name: "AwesomeTest".into(),
        data: serialized_data,
    });

    let response = client.insert(request).await?;

    println!("RESPONSE={response:?}");

    awesome_instance.id = 4;
    awesome_instance.a_test = "damn".into();

    serialized_data = ron::to_string(&awesome_instance).unwrap().into_bytes();

    let request = Request::new(InsertRequest {
        struct_name: "AwesomeTest".into(),
        data: serialized_data,
    });

    let response = client.insert(request).await?;

    println!("RESPONSE={response:?}");

    let request = Request::new(FindQueryRequest {
        struct_name: "AwesomeTest".into(),
        field: "id".into(),
        value: "2".into(),
    });

    let response = client.find_query(request).await?;
    let inst: AwesomeTest = ron::from_str(
        String::from_utf8(response.into_inner().data)
            .unwrap()
            .as_str(),
    )
    .unwrap();

    println!("RESPONSE/INSTANCE={inst:?}");

    let request = Request::new(FindQueryRequest {
        struct_name: "AwesomeTest".into(),
        field: "id".into(),
        value: "3".into(),
    });

    let response = client.find_query(request).await?;

    println!("RESPONSE={response:?}");

    Ok(())
}
