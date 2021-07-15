use crate::ddb;
use crate::domain;
use crate::graphql::me::Me;
use crate::graphql::Context;
use crate::graphql::*;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use juniper::{Executor, FieldResult};
use juniper_from_schema::{QueryTrail, Walked};

pub struct Mutation;
#[async_trait]
impl MutationFields for Mutation {
    async fn field_authenticate<'s, 'r, 'a>(
        &'s self,
        exec: &Executor<'r, 'a, Context>,
        _: &QueryTrail<'r, Me, Walked>,
    ) -> FieldResult<Me> {
        let ctx = exec.context();
        let user_dao = ctx.ddb_dao::<domain::user::User>();
        let authorized_user_id = ctx
            .authorized_user_id
            .clone()
            .ok_or(FieldError::from("unauthorized"))?;
        let now: DateTime<Utc> = Utc::now();

        let user = user_dao.get(authorized_user_id.clone());

        match user {
            ddb::DaoResult::Ok(user) => Ok(Me { user }),
            ddb::DaoResult::Err(err) => {
                if err != ddb::DaoError::NotFound {
                    return Err(FieldError::from(err));
                }

                let user = user_dao
                    .tx(|| {
                        let user = domain::user::User::new(authorized_user_id.clone(), now);
                        user_dao.insert(&user)?;
                        Ok(user)
                    })
                    .map_err(FieldError::from)?;

                Ok(Me { user })
            }
        }
    }
}
