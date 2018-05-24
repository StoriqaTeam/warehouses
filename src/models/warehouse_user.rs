use super::ValueContainer;
use super::warehouse::WarehouseId;

use failure;
use std::fmt;
use std::str::FromStr;
use stq_db::repo::*;
use stq_db::statement::*;
use tokio_postgres::rows::Row;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct WarehouseUserId(pub i32);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct UserId(pub i32);

const ID_COLUMN: &'static str = "id";
const WAREHOUSE_ID_COLUMN: &'static str = "warehouse_id";
const USER_ID_COLUMN: &'static str = "user_id";
const ROLE_COLUMN: &'static str = "role";

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum WarehouseUserRole {
    #[serde(rename = "manager")]
    Manager,
    #[serde(rename = "admin")]
    Admin,
}

impl fmt::Display for WarehouseUserRole {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::WarehouseUserRole::*;

        write!(
            f,
            "{}",
            match *self {
                Manager => "manager",
                Admin => "admin",
            }
        )
    }
}

impl FromStr for WarehouseUserRole {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "manager" => Ok(WarehouseUserRole::Manager),
            "admin" => Ok(WarehouseUserRole::Admin),
            _ => Err(format_err!("Unknown warehouse user role: {}", s)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WarehouseUser {
    pub id: WarehouseUserId,
    pub warehouse_id: WarehouseId,
    pub user_id: UserId,
    pub role: WarehouseUserRole,
}

impl From<Row> for WarehouseUser {
    fn from(row: Row) -> Self {
        Self {
            id: WarehouseUserId(row.get::<_, _>(ID_COLUMN)),
            warehouse_id: WarehouseId(row.get::<_, _>(WAREHOUSE_ID_COLUMN)),
            user_id: UserId(row.get::<_, _>(USER_ID_COLUMN)),
            role: row.get::<String, _>(ROLE_COLUMN).parse().unwrap(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WarehouseUserMeta {
    pub role: WarehouseUserRole,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WarehouseUserInserter {
    pub warehouse_id: WarehouseId,
    pub user_id: UserId,
    pub role: WarehouseUserRole,
}

impl Inserter for WarehouseUserInserter {
    fn into_insert_builder(self, table: &'static str) -> InsertBuilder {
        InsertBuilder::new(table)
            .with_arg(WAREHOUSE_ID_COLUMN, self.warehouse_id.0)
            .with_arg(USER_ID_COLUMN, self.user_id.0)
            .with_arg(ROLE_COLUMN, self.role.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WarehouseUserFilter {
    pub id: Option<ValueContainer<WarehouseUserId>>,
    pub warehouse_id: Option<ValueContainer<WarehouseId>>,
    pub user_id: Option<ValueContainer<UserId>>,
    pub role: Option<ValueContainer<WarehouseUserRole>>,
}

impl Filter for WarehouseUserFilter {
    fn into_filtered_operation_builder(
        self,
        op: FilteredOperation,
        table: &'static str,
    ) -> FilteredOperationBuilder {
        let mut b = FilteredOperationBuilder::new(op, table);

        if let Some(id) = self.id {
            b = b.with_arg(ID_COLUMN, id.value.0);
        }

        if let Some(warehouse_id) = self.warehouse_id {
            b = b.with_arg(WAREHOUSE_ID_COLUMN, warehouse_id.value.0);
        }

        if let Some(user_id) = self.user_id {
            b = b.with_arg(USER_ID_COLUMN, user_id.value.0);
        }

        if let Some(role) = self.role {
            b = b.with_arg(ROLE_COLUMN, role.value.to_string());
        }

        b
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WarehouseUserUpdateData {
    pub role: Option<ValueContainer<WarehouseUserRole>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WarehouseUserUpdater {
    pub mask: WarehouseUserFilter,
    pub data: WarehouseUserUpdateData,
}

impl Updater for WarehouseUserUpdater {
    fn into_update_builder(self, table: &'static str) -> UpdateBuilder {
        let Self { mask, data } = self;

        let mut b = UpdateBuilder::from(mask.into_filtered_operation_builder(
            FilteredOperation::Select,
            table,
        ));

        if let Some(role) = data.role {
            b = b.with_value(ROLE_COLUMN, role.value.to_string());
        }

        b
    }
}
