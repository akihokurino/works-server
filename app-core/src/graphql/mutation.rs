use crate::domain;
use crate::graphql::me::Me;
use crate::graphql::supplier::Supplier;
use crate::graphql::Context;
use crate::graphql::*;
use crate::misoca;
use crate::INVOICE_BUCKET;
use crate::INVOICE_PDF_DOWNLOAD_DURATION;
use crate::{CoreError, CoreResult};
use actix_web::web::Buf;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cloud_storage::Object;
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
            CoreResult::Ok(user) => Ok(Me { user }),
            CoreResult::Err(err) => {
                if err != CoreError::NotFound {
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
            GraphQLBillingType::Monthly => domain::supplier::BillingType::Monthly,
            GraphQLBillingType::OneTime => domain::supplier::BillingType::OneTime,
        };

        let supplier = supplier_dao
            .tx(|| {
                let supplier = domain::supplier::Supplier::new(
                    authorized_user_id,
                    "TODO".to_string(),
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
        let user_dao = ctx.ddb_dao::<domain::user::User>();
        let supplier_dao = ctx.ddb_dao::<domain::supplier::Supplier>();
        let misoca_cli = &ctx.misoca_cli;
        let authorized_user_id = ctx
            .authorized_user_id
            .clone()
            .ok_or(FieldError::from("unauthorized"))?;

        let now: DateTime<Utc> = Utc::now();
        let id = input.id;
        let name = input.name;
        let billing_amount = input.billing_amount;

        let access_token = domain::service::get_misoca_token::exec(
            user_dao,
            misoca_cli.clone(),
            authorized_user_id.clone(),
            now,
        )
        .await
        .map_err(FieldError::from)?;

        let supplier = supplier_dao
            .tx(|| {
                let mut supplier = supplier_dao.get(id.clone())?;
                if supplier.user_id != authorized_user_id {
                    return Err(CoreError::Forbidden);
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
        let invoice_dao = ctx.ddb_dao::<domain::invoice::Invoice>();
        let authorized_user_id = ctx
            .authorized_user_id
            .clone()
            .ok_or(FieldError::from("unauthorized"))?;

        let id = input.id;

        supplier_dao
            .tx(|| {
                let supplier = supplier_dao.get(id.clone())?;
                if supplier.user_id != authorized_user_id {
                    return Err(CoreError::Forbidden);
                }

                invoice_dao.delete_by_supplier(supplier.id.clone())?;
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
        let supplier_dao = ctx.ddb_dao::<domain::supplier::Supplier>();
        let invoice_dao = ctx.ddb_dao::<domain::invoice::Invoice>();
        let misoca_cli = &ctx.misoca_cli;
        let authorized_user_id = ctx
            .authorized_user_id
            .clone()
            .ok_or(FieldError::from("unauthorized"))?;

        let now: DateTime<Utc> = Utc::now();
        let code = input.code;

        let mut user = user_dao.get(authorized_user_id)?;

        let tokens = misoca_cli
            .get_tokens(misoca::tokens::get_tokens::Input { code })
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

        domain::service::sync_invoice::exec(
            supplier_dao,
            invoice_dao,
            misoca_cli.clone(),
            user.id.clone(),
            access_token.clone(),
        )
        .await
        .map_err(FieldError::from)?;

        Ok(true)
    }

    async fn field_refresh_misoca<'s, 'r, 'a>(
        &'s self,
        exec: &Executor<'r, 'a, Context>,
    ) -> FieldResult<bool> {
        let ctx = exec.context();
        let user_dao = ctx.ddb_dao::<domain::user::User>();
        let supplier_dao = ctx.ddb_dao::<domain::supplier::Supplier>();
        let invoice_dao = ctx.ddb_dao::<domain::invoice::Invoice>();
        let misoca_cli = &ctx.misoca_cli;
        let authorized_user_id = ctx
            .authorized_user_id
            .clone()
            .ok_or(FieldError::from("unauthorized"))?;

        let now: DateTime<Utc> = Utc::now();

        let access_token = domain::service::get_misoca_token::exec(
            user_dao,
            misoca_cli.clone(),
            authorized_user_id.clone(),
            now,
        )
        .await
        .map_err(FieldError::from)?;

        domain::service::sync_invoice::exec(
            supplier_dao,
            invoice_dao,
            misoca_cli.clone(),
            authorized_user_id.clone(),
            access_token.clone(),
        )
        .await
        .map_err(FieldError::from)?;

        Ok(true)
    }

    async fn field_download_invoice_pdf<'s, 'r, 'a>(
        &'s self,
        exec: &Executor<'r, 'a, Context>,
        input: DownloadInvoicePDFInput,
    ) -> FieldResult<String> {
        let ctx = exec.context();
        let user_dao = ctx.ddb_dao::<domain::user::User>();
        let invoice_dao = ctx.ddb_dao::<domain::invoice::Invoice>();
        let misoca_cli = &ctx.misoca_cli;
        let authorized_user_id = ctx
            .authorized_user_id
            .clone()
            .ok_or(FieldError::from("unauthorized"))?;

        let now: DateTime<Utc> = Utc::now();
        let invoice_id = input.invoice_id;

        let mut invoice = invoice_dao
            .get(invoice_id.clone())
            .map_err(FieldError::from)?;

        let next_path = format!(
            "invoice/{}_{}.pdf",
            invoice.id.clone(),
            invoice.updated_at.format("%Y%m%d%H%M%S")
        );

        if let Some(path) = invoice.pdf_path.clone() {
            if next_path == path {
                let download_url = Object::read(INVOICE_BUCKET, path.as_str())
                    .await
                    .map(|o| o.download_url(INVOICE_PDF_DOWNLOAD_DURATION))
                    .map_err(FieldError::from)?;
                return Ok(download_url.unwrap_or("".to_string()));
            }
        }

        let access_token = domain::service::get_misoca_token::exec(
            user_dao,
            misoca_cli.clone(),
            authorized_user_id.clone(),
            now,
        )
        .await
        .map_err(FieldError::from)?;

        let data = misoca_cli
            .get_pdf(misoca::invoice::get_pdf::Input {
                access_token,
                invoice_id: invoice.id.clone(),
            })
            .await
            .map_err(FieldError::from)?;

        let object = Object::create(
            INVOICE_BUCKET,
            data.bytes().to_vec(),
            next_path.as_str(),
            "application/pdf",
        )
        .await
        .map_err(FieldError::from)?;

        invoice.update_pdf_path(next_path);
        invoice_dao
            .tx(|| {
                invoice_dao.update(&invoice)?;
                Ok(())
            })
            .map_err(FieldError::from)?;

        let download_url = object
            .download_url(INVOICE_PDF_DOWNLOAD_DURATION)
            .map_err(FieldError::from)?;
        Ok(download_url)
    }
}
