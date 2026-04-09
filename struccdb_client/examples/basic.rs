use serde::{Deserialize, Serialize};
use struccdb_client::{StruccDBConnection, StructName};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AwesomeTest {
    id: u64,
    a_test: String,
}

impl StructName for AwesomeTest {
    fn get_struct_name() -> String {
        "AwesomeTest".into()
    }
}

#[tokio::main]
async fn main() {
    let inst = AwesomeTest {
        id: 23,
        a_test: "Hi".into(),
    };
    let inst2 = AwesomeTest {
        id: 57,
        a_test: "Hello".into(),
    };

    let mut orm = StruccDBConnection::connect().await;
    let _ = orm.insert(inst).await;
    let _ = orm.insert(inst2).await;

    let found: AwesomeTest = orm.find("id".into(), "23".into()).await.unwrap_or_default();
    let does_not_exist: AwesomeTest = orm.find("id".into(), "123456".into()).await.unwrap();

    dbg!(&found);
    dbg!(&does_not_exist);
}
