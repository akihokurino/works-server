pub mod bank;
pub mod invoice;
mod schema;
pub mod sender;
pub mod supplier;
pub mod user;

use crate::CoreResult;
use diesel::connection::TransactionManager;
use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use std::env;
use std::future::Future;
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
    pub fn run<R, F>(conn: &MysqlConnection, f: F) -> CoreResult<R>
    where
        F: FnOnce() -> CoreResult<R>,
    {
        conn.transaction(|| f())
    }

    pub async fn run_async<R, F>(conn: &MysqlConnection, f: F) -> CoreResult<R>
    where
        F: Future<Output = CoreResult<R>>,
    {
        let transaction_manager = conn.transaction_manager();
        transaction_manager.begin_transaction(conn)?;
        match f.await {
            Ok(value) => {
                transaction_manager.commit_transaction(conn)?;
                Ok(value)
            }
            Err(e) => {
                transaction_manager.rollback_transaction(conn)?;
                Err(e)
            }
        }
    }
}
