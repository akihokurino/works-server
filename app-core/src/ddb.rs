pub mod invoice;
mod schema;
pub mod supplier;
pub mod user;

use crate::CoreResult;
use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use std::env;
use std::marker::PhantomData;

pub fn establish_connection() -> MysqlConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

#[derive(Clone)]
pub struct Dao<T> {
    _phantom: PhantomData<fn() -> T>,
}

impl<T> Dao<T> {
    pub fn new() -> Self {
        Dao {
            _phantom: PhantomData,
        }
    }
}

pub struct Tx {}

impl Tx {
    pub fn run<R, F>(conn: &MysqlConnection, exec: F) -> CoreResult<R>
    where
        F: FnOnce() -> CoreResult<R>,
    {
        conn.transaction(|| exec())
    }
}
