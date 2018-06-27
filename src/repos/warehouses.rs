use models::*;

use failure;
use futures::future;
use futures::prelude::*;
use stq_acl::*;
use stq_db::repo::*;
use stq_db::sequence::*;

const TABLE: &'static str = "warehouses";
const SLUG_SEQUENCE: &'static str = "warehouse_slug_seq";

pub trait WarehouseRepo:
    DbRepo<Warehouse, Warehouse, WarehouseFilter, WarehouseUpdater, RepoError>
{
}

pub type WarehouseRepoImpl = DbRepoImpl<Warehouse, Warehouse, WarehouseFilter, WarehouseUpdater>;
impl WarehouseRepo for WarehouseRepoImpl {}

type Repo = WarehouseRepoImpl;

pub fn make_su_repo() -> Repo {
    Repo::new(TABLE)
}

fn check_acl(
    user_roles: Vec<Role>,
    entry: Warehouse,
    action: Action,
) -> Verdict<(Warehouse, Action), failure::Error> {
    Box::new(
        future::ok(())
            .map({
                let entry = entry.clone();
                move |_| {
                    use models::UserRole::*;

                    for user_entry in user_roles {
                        match user_entry.role {
                            // Superadmins can access in all cases.
                            Superadmin => {
                                return true;
                            }
                            // Store managers can do anything to the warehouses of their stores.
                            StoreManager(managed_store_id) => {
                                if managed_store_id == entry.store_id {
                                    return true;
                                }
                            }
                        }
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

pub fn make_repo(user_roles: Vec<Role>) -> Repo {
    make_su_repo().with_afterop_acl_engine({
        move |(entry, action)| check_acl(user_roles.clone(), entry, action)
    })
}

pub trait WarehouseSlugSequence: Sequence<i64> {}
pub type WarehouseSlugSequenceImpl = SequenceImpl;
impl WarehouseSlugSequence for WarehouseSlugSequenceImpl {}

pub fn make_slug_sequence() -> WarehouseSlugSequenceImpl {
    WarehouseSlugSequenceImpl::new(SLUG_SEQUENCE)
}
