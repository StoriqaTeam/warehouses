use bb8;
use bb8_postgres;
use std::sync::Arc;

pub type DbPool = Arc<bb8::Pool<bb8_postgres::PostgresConnectionManager>>;
