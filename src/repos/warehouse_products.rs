use models::*;

use stq_db::repo::*;

static TABLE: &'static str = "warehouse_products";

pub trait WarehouseProductsRepo:
    DbRepo<
        WarehouseProduct,
        WarehouseProductInserter,
        WarehouseProductFilter,
        WarehouseProductUpdater,
        RepoError,
    >
{
}

pub type WarehouseProductsRepoImpl = DbRepoImpl;
impl WarehouseProductsRepo for WarehouseProductsRepoImpl {}

type Repo = WarehouseProductsRepoImpl;
pub fn make_repo() -> Repo {
    Repo::new(TABLE)
}
