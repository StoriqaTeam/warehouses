use models::*;

use failure;
use futures::future;
use futures::prelude::*;
use stq_acl::*;
use stq_db::repo::*;
use stq_db::statement::{UpdateBuilder, Updater};

static TABLE: &'static str = "roles";

pub struct DummyRoleUpdater;

impl Updater for DummyRoleUpdater {
    fn into_update_builder(self, _table: &'static str) -> UpdateBuilder {
        unreachable!()
    }
}

pub trait RolesRepo: DbRepo<Role, Role, RoleFilter, DummyRoleUpdater, RepoError> {}

pub type RolesRepoImpl = DbRepoImpl<Role, Role, RoleFilter, DummyRoleUpdater>;
impl RolesRepo for RolesRepoImpl {}

type Repo = RolesRepoImpl;

pub fn make_su_repo() -> Repo {
    Repo::new(TABLE)
}

fn check_acl(
    user_roles: Vec<Role>,
    entry: Role,
    action: Action,
) -> Verdict<(Role, Action), failure::Error> {
    Box::new(
        future::ok(())
            .map({
                let entry = entry.clone();
                move |_| {
                    use models::UserRole::*;

                    for user_role in user_roles.into_iter() {
                        // Superadmins can access in all cases.
                        if user_role.role == Superadmin {
                            return true;
                        }

                        // Others can only view their roles.
                        if action == Action::Select {
                            if user_role.user_id == entry.user_id {
                                return true;
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
