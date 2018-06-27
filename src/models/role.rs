use super::{StoreId, UserId, ValueContainer};
use errors::*;

use failure;
use serde_json::{from_value, to_value, Value};
use stq_db::statement::*;
use tokio_postgres::rows::Row;
use uuid::Uuid;

const ID_COLUMN: &'static str = "id";
const USER_ID_COLUMN: &'static str = "user_id";
const ROLE_NAME_COLUMN: &'static str = "name";
const ROLE_DATA_COLUMN: &'static str = "data";

#[derive(Clone, Copy, Debug, Display, PartialEq, FromStr, Hash, Serialize, Deserialize)]
pub struct RoleId(pub Uuid);

impl RoleId {
    pub fn new() -> Self {
        RoleId(Uuid::new_v4())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserRoleName {
    Superadmin,
    StoreManager,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "name", content = "data")]
pub enum UserRole {
    Superadmin,
    StoreManager(StoreId),
}

impl UserRole {
    fn from_db(variant: &str, data: Value) -> Result<Self, failure::Error> {
        use self::UserRole::*;

        match variant {
            "superadmin" => Ok(Superadmin),
            "store_manager" => Ok(StoreManager(from_value(data)?)),
            other => Err(format_err!("Unknown variant {}", other)
                .context(Error::ParseError)
                .into()),
        }
    }

    fn into_db(self) -> (String, Value) {
        use self::UserRole::*;

        match self {
            Superadmin => ("superadmin".into(), Value::Null),
            StoreManager(data) => ("store_manager".into(), to_value(data).unwrap()),
        }
    }

    pub fn into_tuple(self) -> (UserRoleName, Value) {
        use self::UserRole::*;

        match self {
            Superadmin => (UserRoleName::Superadmin, Value::Null),
            StoreManager(data) => (UserRoleName::StoreManager, to_value(data).unwrap()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Role {
    pub id: RoleId,
    pub user_id: UserId,
    pub role: UserRole,
}

impl From<Row> for Role {
    fn from(row: Row) -> Self {
        Self {
            id: RoleId(row.get(ID_COLUMN)),
            user_id: UserId(row.get(USER_ID_COLUMN)),
            role: UserRole::from_db(row.get(ROLE_NAME_COLUMN), row.get(ROLE_DATA_COLUMN)).unwrap(),
        }
    }
}

impl Inserter for Role {
    fn into_insert_builder(self, table: &'static str) -> InsertBuilder {
        let (role_name, role_data) = UserRole::into_db(self.role);
        InsertBuilder::new(table)
            .with_arg(ID_COLUMN, self.id.0)
            .with_arg(USER_ID_COLUMN, self.user_id.0)
            .with_arg(ROLE_NAME_COLUMN, role_name)
            .with_arg(ROLE_DATA_COLUMN, role_data)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RoleFilter {
    pub id: Option<ValueContainer<RoleId>>,
    pub user_id: Option<ValueContainer<UserId>>,
    pub role: Option<ValueContainer<UserRole>>,
}

impl Filter for RoleFilter {
    fn into_filtered_operation_builder(self, table: &'static str) -> FilteredOperationBuilder {
        let mut b = FilteredOperationBuilder::new(table);

        if let Some(id) = self.id {
            b = b.with_filter(ID_COLUMN, id.value.0);
        }

        if let Some(user_id) = self.user_id {
            b = b.with_filter(USER_ID_COLUMN, user_id.value.0);
        }

        if let Some(role) = self.role {
            let (role_name, role_data) = UserRole::into_db(role.value);
            b = b.with_filter(ROLE_NAME_COLUMN, role_name)
                .with_filter(ROLE_DATA_COLUMN, role_data);
        }

        b
    }
}
