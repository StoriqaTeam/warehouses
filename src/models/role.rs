use errors::*;

use failure;
use serde_json::{from_value, to_value, Value};
use stq_roles;
pub use stq_roles::models::RepoLogin;
use stq_types::*;

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

impl stq_roles::models::RoleModel for UserRole {
    fn is_su(&self) -> bool {
        use self::UserRole::*;

        match self {
            Superadmin => true,
            _ => false,
        }
    }

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
}

pub type RoleEntry = stq_roles::models::RoleEntry<UserRole>;
pub type RoleFilter = stq_roles::models::RoleFilter<UserRole>;

pub type UserLogin = stq_roles::models::RepoLogin<UserRole>;
