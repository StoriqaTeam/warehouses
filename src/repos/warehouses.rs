use models::*;

use stq_acl::*;
use stq_db::repo::*;
use stq_db::sequence::*;

const TABLE: &str = "warehouses";
const SLUG_SEQUENCE: &str = "warehouse_slug_seq";

pub trait WarehouseRepo:
    DbRepo<DbWarehouse, DbWarehouse, WarehouseFilter, WarehouseUpdater, RepoError>
{
}

pub type WarehouseRepoImpl =
    DbRepoImpl<DbWarehouse, DbWarehouse, WarehouseFilter, WarehouseUpdater>;
impl WarehouseRepo for WarehouseRepoImpl {}

type Repo = WarehouseRepoImpl;

pub fn make_su_repo() -> Repo {
    Repo::new(TABLE)
}

type AclContext = (DbWarehouse, Action);

fn check_acl(login: UserLogin, (entry, _action): &mut AclContext) -> bool {
    use self::RepoLogin::*;
    use models::UserRole::*;

    if let User { caller_roles, .. } = login {
        for user_entry in caller_roles {
            match user_entry.role {
                // Superadmins can access in all cases.
                Superadmin => {
                    return true;
                }
                // Store managers can do anything to the warehouses of their stores.
                StoreManager(managed_store_id) => {
                    if managed_store_id == entry.0.store_id {
                        return true;
                    }
                }
            }
        }
    }

    false
}

pub fn make_repo(login: UserLogin) -> Repo {
    make_su_repo().with_afterop_acl_engine(InfallibleSyncACLFn(move |ctx: &mut AclContext| {
        check_acl(login.clone(), ctx)
    }))
}

pub trait WarehouseSlugSequence: Sequence<i64> {}
pub type WarehouseSlugSequenceImpl = SequenceImpl;
impl WarehouseSlugSequence for WarehouseSlugSequenceImpl {}

pub fn make_slug_sequence() -> WarehouseSlugSequenceImpl {
    WarehouseSlugSequenceImpl::new(SLUG_SEQUENCE)
}
