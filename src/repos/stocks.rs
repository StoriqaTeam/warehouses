use models::*;

use failure;
use futures::prelude::*;
use std::rc::Rc;
use stq_acl::*;
use stq_db::repo::*;
use stq_types::*;

const TABLE: &str = "stocks";

pub trait StocksRepo: DbRepo<Stock, Stock, StockFilter, StockUpdater, RepoError> {}

pub type StocksRepoImpl = DbRepoImpl<Stock, Stock, StockFilter, StockUpdater>;
impl StocksRepo for StocksRepoImpl {}

type Repo = StocksRepoImpl;

pub fn make_su_repo() -> Repo {
    Repo::new(TABLE)
}

fn check_acl(
    warehouse_source: &Rc<Fn(WarehouseId) -> Box<Future<Item = Warehouse, Error = failure::Error>>>,
    user_roles: Vec<RoleEntry>,
    entry: Stock,
    action: Action,
) -> Verdict<(Stock, Action), failure::Error> {
    use models::UserRole::*;

    Box::new(
        (warehouse_source)(entry.warehouse_id)
            .map({
                move |warehouse| {
                    for user_role in user_roles {
                        match user_role.role {
                            // Superadmins can access in all cases.
                            Superadmin => {
                                return true;
                            }
                            // Store managers can change products of the warehouses that belong to the stores that they manage.
                            StoreManager(managed_store_id) => {
                                if managed_store_id == warehouse.store_id {
                                    return true;
                                }
                            }
                        }
                    }

                    // Allow read-only access for everyone
                    if action == Action::Select {
                        return true;
                    }

                    false
                }
            })
            .then(move |v| match v {
                Ok(d) => Ok((d, (entry, action))),
                Err(e) => Err((e, (entry, action))),
            }),
    )
}

pub fn make_repo(
    user_roles: Vec<RoleEntry>,
    warehouse_source: Rc<Fn(WarehouseId) -> Box<Future<Item = Warehouse, Error = failure::Error>>>,
) -> Repo {
    make_su_repo().with_afterop_acl_engine({
        move |(entry, action)| check_acl(&warehouse_source, user_roles.clone(), entry, action)
    })
}
