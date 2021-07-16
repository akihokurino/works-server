use crate::ddb;
use crate::ddb::DaoError;
use crate::domain;
use crate::graphql::me::Me;
use crate::graphql::supplier::Supplier;
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

    async fn field_create_supplier<'s, 'r, 'a>(
        &'s self,
        exec: &Executor<'r, 'a, Context>,
        _: &QueryTrail<'r, Supplier, Walked>,
        input: CreateSupplierInput,
    ) -> FieldResult<Supplier> {
        let ctx = exec.context();
        let supplier_dao = ctx.ddb_dao::<domain::supplier::Supplier>();
        let authorized_user_id = ctx
            .authorized_user_id
            .clone()
            .ok_or(FieldError::from("unauthorized"))?;

        let now: DateTime<Utc> = Utc::now();
        let name = input.name;
        let billing_amount = input.billing_amount;
        let billing_type = match input.billing_type {
            SupplierBillingType::Monthly => domain::supplier::BillingType::Monthly,
            SupplierBillingType::OneTime => domain::supplier::BillingType::OneTime,
        };

        let supplier = supplier_dao
            .tx(|| {
                let supplier = domain::supplier::Supplier::new(
                    authorized_user_id,
                    name,
                    billing_amount,
                    billing_type,
                    now,
                );
                supplier_dao.insert(&supplier)?;
                Ok(supplier)
            })
            .map_err(FieldError::from)?;

        Ok(Supplier { supplier })
    }

    async fn field_update_supplier<'s, 'r, 'a>(
        &'s self,
        exec: &Executor<'r, 'a, Context>,
        _: &QueryTrail<'r, Supplier, Walked>,
        input: UpdateSupplierInput,
    ) -> FieldResult<Supplier> {
        let ctx = exec.context();
        let supplier_dao = ctx.ddb_dao::<domain::supplier::Supplier>();
        let authorized_user_id = ctx
            .authorized_user_id
            .clone()
            .ok_or(FieldError::from("unauthorized"))?;

        let now: DateTime<Utc> = Utc::now();
        let id = input.id;
        let name = input.name;
        let billing_amount = input.billing_amount;

        let supplier = supplier_dao
            .tx(|| {
                let mut supplier = supplier_dao.get(id.clone())?;
                if supplier.user_id != authorized_user_id {
                    return Err(DaoError::Forbidden);
                }

                supplier.update(name, billing_amount, now);

                supplier_dao.update(&supplier)?;
                Ok(supplier)
            })
            .map_err(FieldError::from)?;

        Ok(Supplier { supplier })
    }

    async fn field_delete_supplier<'s, 'r, 'a>(
        &'s self,
        exec: &Executor<'r, 'a, Context>,
        input: DeleteSupplierInput,
    ) -> FieldResult<bool> {
        let ctx = exec.context();
        let supplier_dao = ctx.ddb_dao::<domain::supplier::Supplier>();
        let authorized_user_id = ctx
            .authorized_user_id
            .clone()
            .ok_or(FieldError::from("unauthorized"))?;

        let id = input.id;

        supplier_dao
            .tx(|| {
                let supplier = supplier_dao.get(id.clone())?;
                if supplier.user_id != authorized_user_id {
                    return Err(DaoError::Forbidden);
                }

                supplier_dao.delete(supplier.id.clone())?;
                Ok(())
            })
            .map_err(FieldError::from)?;

        Ok(true)
    }
}
