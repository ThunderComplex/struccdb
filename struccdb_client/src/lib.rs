use serde::{Deserialize, Serialize};
use tonic::{Request, Status};

use crate::database::{FindQueryRequest, InsertRequest, db_service_client::DbServiceClient};

pub mod database {
    tonic::include_proto!("database");
}

pub trait StructName {
    fn get_struct_name() -> String;
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AwesomeTest {
    id: u64,
    a_test: String,
}

#[derive(Clone, Debug, Default)]
pub struct InsertError {}

#[derive(Clone, Debug, Default)]
pub struct FindError {
    pub message: String,
}

#[derive(Clone, Debug, Default)]
pub struct StruccDBConnection {}

#[derive(Clone, Debug)]
pub struct StruccDBORM {
    client: DbServiceClient<tonic::transport::Channel>,
}

impl From<Status> for FindError {
    fn from(value: Status) -> Self {
        Self {
            message: value.message().into(),
        }
    }
}

impl StruccDBConnection {
    pub async fn connect() -> StruccDBORM {
        StruccDBORM {
            client: DbServiceClient::connect("http://[::1]:50051")
                .await
                .unwrap(),
        }
    }
}

impl StruccDBORM {
    pub async fn insert<T: Serialize + StructName>(
        &mut self,
        struct_instance: T,
    ) -> Result<(), InsertError> {
        let request = Request::new(InsertRequest {
            struct_name: T::get_struct_name(),
            data: ron::to_string(&struct_instance).unwrap().into_bytes(),
        });

        self.client
            .insert(request)
            .await
            .and_then(|_| Ok(()))
            .or_else(|_| Err(InsertError::default()))
    }

    pub async fn find<T: for<'a> Deserialize<'a> + StructName>(
        &mut self,
        field: String,
        value: String,
    ) -> Result<Option<T>, FindError> {
        let request = Request::new(FindQueryRequest {
            struct_name: T::get_struct_name(),
            field,
            value,
        });

        let response = self.client.find_query(request).await;

        match response {
            Ok(found) => {
                let inst: T =
                    ron::from_str(String::from_utf8(found.into_inner().data).unwrap().as_str())
                        .unwrap();
                Ok(Some(inst))
            }
            Err(response_error) => {
                if response_error.message() == "No results"
                    || response_error.message() == "Struct not found"
                {
                    Ok(None)
                } else {
                    Err(response_error.into())
                }
            }
        }
    }
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
