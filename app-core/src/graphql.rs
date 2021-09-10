use std::sync::{Arc, Mutex, MutexGuard};

use diesel::MysqlConnection;
use juniper::*;
use juniper_from_schema::graphql_schema_from_file;

use crate::ddb;
use crate::graphql::bank::*;
use crate::graphql::invoice::*;
use crate::graphql::invoice_history::*;
use crate::graphql::me::*;
use crate::graphql::page_info::*;
use crate::graphql::sender::*;
use crate::graphql::supplier::*;
use crate::misoca;

use self::mutation::*;
use self::query::*;

mod bank;
mod get_misoca_token;
mod invoice;
mod invoice_history;
mod me;
mod mutation;
mod page_info;
mod query;
mod sender;
mod supplier;

#[allow(unused)]
graphql_schema_from_file!("src/graphql/schema.graphql", context_type: Context);

pub struct Context {
    pub authenticated_user_id: Option<String>,
    pub misoca_cli: misoca::Client,
    pub connection: Arc<Mutex<MysqlConnection>>,
    pub invoice_loader_by_supplier: ddb::invoice::LoaderBySupplier,
}

impl juniper::Context for Context {}

impl Context {
    pub fn new(authenticated_user_id: Option<String>, misoca_cli: misoca::Client) -> Self {
        let conn_ref = Arc::new(Mutex::new(ddb::establish_connection()));
        Self {
            authenticated_user_id,
            misoca_cli,
            connection: Arc::clone(&conn_ref),
            invoice_loader_by_supplier: ddb::invoice::BatcherBySupplier::new_loader(Arc::clone(
                &conn_ref,
            )),
        }
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
