use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use tonic::{Request, Response, Status, transport::Server};

use crate::database::{
    FindQueryRequest, FindQueryResponse, InsertRequest, InsertResponse,
    db_service_server::{DbService, DbServiceServer},
};

pub mod database {
    tonic::include_proto!("database");
}

#[derive(Default)]
struct DatabaseService {
    data: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

fn ron_value_to_string(input_value: &ron::Value) -> Option<String> {
    match input_value.clone() {
        ron::Value::Bool(v) => {
            if v == true {
                Some("true".into())
            } else {
                Some("false".into())
            }
        }
        ron::Value::Char(v) => Some(String::from(v)),
        ron::Value::String(v) => Some(String::from(v)),
        ron::Value::Bytes(v) => String::from_utf8(v).ok(),
        ron::Value::Number(v) => Some(v.into_f64().to_string()),
        ron::Value::Option(v) => {
            if let Some(val) = v {
                ron_value_to_string(&val)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[tonic::async_trait]
impl DbService for DatabaseService {
    async fn insert(
        &self,
        request: Request<InsertRequest>,
    ) -> Result<Response<InsertResponse>, Status> {
        let request = request.into_inner();

        let mut db = self.data.lock().unwrap();

        match db.get_mut(&request.struct_name) {
            Some(entry) => {
                entry.push(String::from_utf8(request.data).unwrap());
            }
            None => {
                db.insert(
                    request.struct_name,
                    vec![String::from_utf8(request.data).unwrap()],
                );
            }
        }

        let response = InsertResponse {};
        Ok(Response::new(response))
    }

    async fn find_query(
        &self,
        request: Request<FindQueryRequest>,
    ) -> Result<Response<FindQueryResponse>, Status> {
        let request = request.into_inner();

        let db = self.data.lock().unwrap();

        match db.get(&request.struct_name) {
            Some(data) => {
                for entry in data.iter() {
                    let serialized: ron::Value = ron::from_str(entry).unwrap();

                    if let ron::Value::Map(entry_map) = serialized {
                        if let Some(entry_value) =
                            entry_map.get(&ron::Value::String(request.field.clone()))
                        {
                            let entry_string: Option<String>;
                            if let ron::Value::String(estr) = entry_value {
                                entry_string = Some(estr.clone());
                            } else {
                                entry_string = ron_value_to_string(entry_value);
                            }

                            if let Some(entry_string_value) = entry_string {
                                if entry_string_value == request.value {
                                    return Ok(Response::new(FindQueryResponse {
                                        data: entry.clone().into_bytes(),
                                    }));
                                }
                            }
                        }
                    }
                }
                Err(Status::not_found("No results"))
            }
            None => Err(Status::not_found("Struct not found")),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let db = DatabaseService::default();

    println!("Server listening on {addr}");

    Server::builder()
        .add_service(DbServiceServer::new(db))
        .serve(addr)
        .await?;

    Ok(())
}
