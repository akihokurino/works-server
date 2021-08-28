mod bank;
mod invoice;
mod invoice_history;
mod me;
mod mutation;
mod query;
mod sender;
mod supplier;

use self::mutation::*;
use self::query::*;
use crate::ddb;
use crate::graphql::bank::*;
use crate::graphql::invoice::*;
use crate::graphql::invoice_history::*;
use crate::graphql::me::*;
use crate::graphql::sender::*;
use crate::graphql::supplier::*;
use crate::misoca;
use diesel::MysqlConnection;
use juniper::*;
use juniper_from_schema::graphql_schema_from_file;
use std::sync::{Arc, Mutex};

#[allow(unused)]
graphql_schema_from_file!("src/graphql/schema.graphql", context_type: Context);

pub struct Context {
    pub authenticated_user_id: Option<String>,
    pub misoca_cli: misoca::Client,
    pub connection: SingleCache<Mutex<MysqlConnection>>,
}

impl juniper::Context for Context {}

impl Context {
    pub fn new(authenticated_user_id: Option<String>, misoca_cli: misoca::Client) -> Self {
        Self {
            authenticated_user_id,
            misoca_cli,
            connection: SingleCache::new(),
        }
    }

    pub fn ddb_dao<T>(&self) -> ddb::Dao<T> {
        ddb::Dao::new()
    }

    pub fn get_connection(&self) -> Arc<Mutex<MysqlConnection>> {
        self.connection
            .get_or_create(|| Mutex::new(ddb::establish_connection()))
    }
}

pub fn new_schema() -> Schema {
    Schema::new(Query {}, Mutation {}, EmptySubscription::new())
}

pub struct SingleCache<T> {
    cache: Arc<Mutex<Option<Arc<T>>>>,
}

impl<T> SingleCache<T> {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_or_create<F: Fn() -> T>(&self, create_fn: F) -> Arc<T> {
        let mut mutex = self.cache.lock().unwrap();
        if let Some(v) = mutex.as_ref() {
            Arc::clone(v)
        } else {
            *mutex = Some(Arc::new(create_fn()));
            Arc::clone(mutex.as_ref().unwrap())
        }
    }
}
