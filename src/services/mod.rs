pub mod warehouse;
pub use self::warehouse::*;

use stq_db::repo::RepoFuture;

pub type ServiceFuture<T> = RepoFuture<T>;
