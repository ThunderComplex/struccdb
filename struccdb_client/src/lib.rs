use std::fmt::Display;

use serde::{Deserialize, Serialize};
use tonic::{Request, Status};

use crate::database::{
    DeleteRequest, FindQueryRequest, InsertRequest, UpdateRequest,
    db_service_client::DbServiceClient,
};

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
pub struct DeleteError {}

#[derive(Clone, Debug, Default)]
pub struct UpdateError {}

#[derive(Clone, Debug)]
pub enum UpdateOperation {
    Set,
}

#[derive(Clone, Debug, Default)]
pub struct StruccDBConnection {}

#[derive(Clone, Debug)]
pub struct StruccDBORM {
    client: DbServiceClient<tonic::transport::Channel>,
}

impl Display for UpdateOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
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

    async fn find(
        &mut self,
        struct_name: String,
        field: String,
        value: String,
    ) -> Result<tonic::Response<database::FindQueryResponse>, Status> {
        let request = Request::new(FindQueryRequest {
            struct_name,
            field,
            value,
        });

        self.client.find_query(request).await
    }

    fn map_find_error<T>(&self, error: Status) -> Result<Option<T>, FindError> {
        if error.message() == "No results" || error.message() == "Struct not found" {
            Ok(None)
        } else {
            Err(error.into())
        }
    }

    pub async fn find_one<T: for<'a> Deserialize<'a> + StructName>(
        &mut self,
        field: String,
        value: String,
    ) -> Result<Option<T>, FindError> {
        let response = self.find(T::get_struct_name(), field, value).await;

        match response {
            Ok(found) => {
                let ron_str = String::from_utf8(found.into_inner().data).unwrap();
                Ok(Some(ron::from_str(ron_str.as_str()).unwrap()))
            }
            Err(response_error) => self.map_find_error(response_error),
        }
    }

    pub async fn find_many<T: for<'a> Deserialize<'a> + StructName>(
        &mut self,
        field: String,
        value: String,
    ) -> Result<Option<Vec<T>>, FindError> {
        let response = self.find(T::get_struct_name(), field, value).await;

        match response {
            Ok(found) => {
                let ron_str = String::from_utf8(found.into_inner().data).unwrap();
                let instances_str: Vec<&str> = ron_str.split("\0\0\0\0").collect();
                let instances: Vec<T> = instances_str
                    .iter()
                    .map(|s| ron::from_str(s).unwrap())
                    .collect();
                Ok(Some(instances))
            }
            Err(response_error) => self.map_find_error(response_error),
        }
    }

    pub async fn delete<T: StructName>(
        &mut self,
        field: String,
        value: String,
    ) -> Result<(), DeleteError> {
        let request = Request::new(DeleteRequest {
            struct_name: T::get_struct_name(),
            field,
            value,
        });

        match self.client.delete(request).await {
            Ok(_) => Ok(()),
            Err(_) => Err(DeleteError {}),
        }
    }

    pub async fn update<T: for<'a> Deserialize<'a> + StructName>(
        &mut self,
        field: String,
        search: String,
        operation: UpdateOperation,
        value: String,
    ) -> Result<(), UpdateError> {
        let request = Request::new(UpdateRequest {
            struct_name: T::get_struct_name(),
            field,
            search,
            operation: operation.to_string(),
            value,
        });

        match self.client.update(request).await {
            Ok(_) => Ok(()),
            Err(_) => Err(UpdateError {}),
        }
    }
}
