pub mod invoice;
mod schema;
pub mod supplier;
pub mod user;

use crate::errors;
use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use std::env;
use std::marker::PhantomData;

pub fn establish_connection() -> MysqlConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub struct Dao<T> {
    pub conn: MysqlConnection,
    _phantom: PhantomData<fn() -> T>,
}

impl<T> Dao<T> {
    pub fn new(conn: MysqlConnection) -> Self {
        Dao {
            conn,
            _phantom: PhantomData,
        }
    }

    pub fn tx<R, F>(&self, exec: F) -> errors::CoreResult<R>
    where
        F: FnOnce() -> errors::CoreResult<R>,
    {
        self.conn.transaction(|| exec())
    }
}
