mod invoice;
mod me;
mod mutation;
mod query;
mod supplier;

use self::mutation::*;
use self::query::*;
use crate::graphql::invoice::*;
use crate::graphql::me::*;
use crate::graphql::supplier::*;
use crate::misoca;
use juniper::*;
use juniper_from_schema::graphql_schema_from_file;

use crate::ddb;

#[allow(unused)]
graphql_schema_from_file!("src/graphql/schema.graphql", context_type: Context);

pub struct Context {
    pub authorized_user_id: Option<String>,
    pub misoca_cli: misoca::Client,
}

impl juniper::Context for Context {}

impl Context {
    pub fn ddb_dao<T>(&self) -> ddb::Dao<T> {
        let conn = ddb::establish_connection();
        ddb::Dao::new(conn)
    }
}

pub fn new_schema() -> Schema {
    Schema::new(Query {}, Mutation {}, EmptySubscription::new())
}
