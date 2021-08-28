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
use std::sync::{Arc, Mutex, MutexGuard};

#[allow(unused)]
graphql_schema_from_file!("src/graphql/schema.graphql", context_type: Context);

pub struct Context {
    pub authenticated_user_id: Option<String>,
    pub misoca_cli: misoca::Client,
    pub connection: Arc<Mutex<MysqlConnection>>,
}

impl juniper::Context for Context {}

impl Context {
    pub fn new(authenticated_user_id: Option<String>, misoca_cli: misoca::Client) -> Self {
        Self {
            authenticated_user_id,
            misoca_cli,
            connection: Arc::new(Mutex::new(ddb::establish_connection())),
        }
    }

    pub fn ddb_dao<T>(&self) -> ddb::Dao<T> {
        ddb::Dao::new()
    }

    pub fn get_mutex_connection(&self) -> MutexGuard<MysqlConnection> {
        self.connection.lock().unwrap()
    }

    pub fn get_new_connection(&self) -> MysqlConnection {
        ddb::establish_connection()
    }
}

pub fn new_schema() -> Schema {
    Schema::new(Query {}, Mutation {}, EmptySubscription::new())
}
