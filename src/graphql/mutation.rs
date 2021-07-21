use crate::ddb;
use crate::ddb::DaoError;
use crate::domain;
use crate::graphql::me::Me;
use crate::graphql::supplier::Supplier;
use crate::graphql::Context;
use crate::graphql::*;
use crate::misoca;
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

    async fn field_connect_misoca<'s, 'r, 'a>(
        &'s self,
        exec: &Executor<'r, 'a, Context>,
        input: ConnectMisocaInput,
    ) -> FieldResult<bool> {
        let ctx = exec.context();
        let user_dao = ctx.ddb_dao::<domain::user::User>();
        let misoca_cli = &ctx.misoca_cli;
        let authorized_user_id = ctx
            .authorized_user_id
            .clone()
            .ok_or(FieldError::from("unauthorized"))?;

        let now: DateTime<Utc> = Utc::now();
        let code = input.code;

        let tokens = misoca_cli
            .get_tokens(misoca::get_tokens::Input { code })
            .await
            .map_err(FieldError::from)?;

        let access_token = tokens.access_token;
        let refresh_token = tokens.refresh_token;

        user_dao
            .tx(|| {
                let mut user = user_dao.get(authorized_user_id)?;

                user.update_misoca_refresh_token(refresh_token, now);

                user_dao.update(&user)?;

                Ok(user)
            })
            .map_err(FieldError::from)?;

        let invoices = misoca_cli
            .get_invoices(misoca::get_invoices::Input {
                access_token,
                supplier_name: "Zidai株式会社".to_string(),
                page: 1,
                per_page: 10,
            })
            .await
            .map_err(FieldError::from)?;

        for invoice in invoices {
            println!("{:?}", invoice.subject)
        }

        Ok(true)
    }

    async fn field_refresh_misoca<'s, 'r, 'a>(
        &'s self,
        exec: &Executor<'r, 'a, Context>,
    ) -> FieldResult<bool> {
        let ctx = exec.context();
        let user_dao = ctx.ddb_dao::<domain::user::User>();
        let misoca_cli = &ctx.misoca_cli;
        let authorized_user_id = ctx
            .authorized_user_id
            .clone()
            .ok_or(FieldError::from("unauthorized"))?;

        let now: DateTime<Utc> = Utc::now();

        let mut user = user_dao.get(authorized_user_id).map_err(FieldError::from)?;

        let tokens = misoca_cli
            .refresh_tokens(misoca::refresh_tokens::Input {
                refresh_token: user.misoca_refresh_token.clone(),
            })
            .await
            .map_err(FieldError::from)?;

        let access_token = tokens.access_token;
        let refresh_token = tokens.refresh_token;

        user.update_misoca_refresh_token(refresh_token, now);

        user_dao
            .tx(|| {
                user_dao.update(&user)?;
                Ok(())
            })
            .map_err(FieldError::from)?;

        let invoices = misoca_cli
            .get_invoices(misoca::get_invoices::Input {
                access_token,
                supplier_name: "Zidai株式会社".to_string(),
                page: 1,
                per_page: 10,
            })
            .await
            .map_err(FieldError::from)?;

        for invoice in invoices {
            println!("{:?}", invoice.subject)
        }

        Ok(true)
    }
}
