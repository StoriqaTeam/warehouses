use models::*;

use stq_db::repo::*;

static TABLE: &'static str = "warehouses";

pub trait WarehouseRepo:
    DbRepo<Warehouse, WarehouseInserter, WarehouseFilter, WarehouseUpdater, RepoError>
{
}

pub type WarehouseRepoImpl = DbRepoImpl;
impl WarehouseRepo for WarehouseRepoImpl {}

type Repo = WarehouseRepoImpl;
pub fn make_repo() -> Repo {
    Repo::new(TABLE)
}
