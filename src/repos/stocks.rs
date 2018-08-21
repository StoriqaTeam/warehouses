use models::*;

use failure;
use futures::prelude::*;
use std::rc::Rc;
use stq_acl::*;
use stq_api::warehouses::Warehouse;
use stq_db::repo::*;
use stq_roles::models::RepoLogin;
use stq_types::*;

const TABLE: &str = "stocks";

pub trait StocksRepo: DbRepo<DbStock, DbStock, StockFilter, StockUpdater, RepoError> {}

pub type StocksRepoImpl = DbRepoImpl<DbStock, DbStock, StockFilter, StockUpdater>;
impl StocksRepo for StocksRepoImpl {}

type Repo = StocksRepoImpl;

pub fn make_su_repo() -> Repo {
    Repo::new(TABLE)
}

fn check_acl(
    warehouse_source: &Rc<Fn(WarehouseId) -> Box<Future<Item = Warehouse, Error = failure::Error>>>,
    login: UserLogin,
    entry: DbStock,
    action: Action,
) -> Verdict<(DbStock, Action), failure::Error> {
    Box::new(
        (warehouse_source)(entry.0.warehouse_id)
            .map({
                move |warehouse| {
                    use self::RepoLogin::*;
                    use models::UserRole::*;

                    if let User { caller_roles, .. } = login {
                        for user_role in caller_roles {
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
    login: UserLogin,
    warehouse_source: Rc<Fn(WarehouseId) -> Box<Future<Item = Warehouse, Error = failure::Error>>>,
) -> Repo {
    make_su_repo().with_afterop_acl_engine(AsyncACLFn({
        move |(entry, action)| check_acl(&warehouse_source, login.clone(), entry, action)
    }))
}
