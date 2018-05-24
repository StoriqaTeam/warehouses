use models::*;

use stq_db::repo::*;

static TABLE: &'static str = "warehouse_users";

pub trait WarehouseUsersRepo:
    DbRepo<
        WarehouseUser,
        WarehouseUserInserter,
        WarehouseUserFilter,
        WarehouseUserUpdater,
        RepoError,
    >
{
}

pub type WarehouseUsersRepoImpl = DbRepoImpl;
impl WarehouseUsersRepo for WarehouseUsersRepoImpl {}

type Repo = WarehouseUsersRepoImpl;
pub fn make_repo() -> Repo {
    Repo::new(TABLE)
}
