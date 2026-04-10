use serde::{Deserialize, Serialize};
use struccdb_client::{FindError, StruccDBConnection, StructName};
use tonic::Status;

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
    let inst3 = AwesomeTest {
        id: 21,
        a_test: "Hi".into(),
    };

    let mut orm = StruccDBConnection::connect().await;
    let _ = orm.insert(inst).await;
    let _ = orm.insert(inst2).await;
    let _ = orm.insert(inst3).await;

    let found: Result<Option<AwesomeTest>, FindError> =
        orm.find_one("id".into(), "23".into()).await;
    let does_not_exist: Result<Option<AwesomeTest>, FindError> =
        orm.find_one("id".into(), "123456".into()).await;
    let duplicate: Result<Option<Vec<AwesomeTest>>, FindError> =
        orm.find_many("a_test".into(), "Hi".into()).await;

    dbg!(&found);
    dbg!(&does_not_exist);
    dbg!(&duplicate);

    let _ = orm.delete::<AwesomeTest>("id".into(), "23".into()).await;

    let found_again: Result<Option<AwesomeTest>, FindError> =
        orm.find_one("id".into(), "23".into()).await;

    dbg!(&found_again);

    let _ = orm
        .update::<AwesomeTest>(
            "a_test".into(),
            "Hello".into(),
            struccdb_client::UpdateOperation::Set,
            "Very cool".into(),
        )
        .await;

    let found_after_update: Result<Option<AwesomeTest>, FindError> =
        orm.find_one("a_test".into(), "Very cool".into()).await;

    dbg!(&found_after_update);
}
