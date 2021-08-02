use crate::ddb::Tx;
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
        let now: DateTime<Utc> = Utc::now();
        let ctx = exec.context();
        let conn = ddb::establish_connection();
        let user_dao = ctx.ddb_dao::<domain::user::User>();
        let authorized_user_id = ctx
            .authorized_user_id
            .clone()
            .ok_or(FieldError::from("unauthorized"))?;

        let user = user_dao.get(&conn, authorized_user_id.clone());

        match user {
            CoreResult::Ok(user) => Ok(Me { user }),
            CoreResult::Err(err) => {
                if err != CoreError::NotFound {
                    return Err(FieldError::from(err));
                }

                let user = Tx::run(&conn, || {
                    let user = domain::user::User::new(authorized_user_id.clone(), now);
                    user_dao.insert(&conn, &user)?;
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
        let now: DateTime<Utc> = Utc::now();
        let ctx = exec.context();
        let conn = ddb::establish_connection();
        let user_dao = ctx.ddb_dao::<domain::user::User>();
        let supplier_dao = ctx.ddb_dao::<domain::supplier::Supplier>();
        let misoca_cli = &ctx.misoca_cli;
        let authorized_user_id = ctx
            .authorized_user_id
            .clone()
            .ok_or(FieldError::from("unauthorized"))?;

        let name = input.name;
        let subject = input.subject;
        let billing_amount = input.billing_amount;
        let billing_type = match input.billing_type {
            GraphQLBillingType::Monthly => domain::supplier::BillingType::Monthly,
            GraphQLBillingType::OneTime => domain::supplier::BillingType::OneTime,
        };

        let mut user = user_dao
            .get(&conn, authorized_user_id.clone())
            .map_err(FieldError::from)?;

        if user.misoca_refresh_token.is_empty() {
            return Err(FieldError::from("should connect misoca"));
        }

        let tokens = misoca_cli
            .refresh_tokens(misoca::tokens::refresh_tokens::Input {
                refresh_token: user.misoca_refresh_token.clone(),
            })
            .await
            .map_err(FieldError::from)?;

        let access_token = tokens.access_token;
        let refresh_token = tokens.refresh_token;

        user.update_misoca_refresh_token(refresh_token, now);
        Tx::run(&conn, || {
            user_dao.update(&conn, &user)?;
            Ok(())
        })
        .map_err(FieldError::from)?;

        let contacts = misoca_cli
            .get_contacts(misoca::contact::get_contacts::Input {
                access_token: access_token.clone(),
                page: 1,
                per_page: 100,
            })
            .await
            .map_err(FieldError::from)?;

        let mut contact_id = "".to_string();
        let mut contact_group_id = "".to_string();
        for contact in contacts {
            if contact.recipient_name.unwrap_or("".to_string()) == name {
                contact_id = contact.id.unwrap().to_string();
                contact_group_id = contact.contact_group_id.unwrap().to_string();
                break;
            }
        }

        if contact_id == "" {
            let contact = misoca_cli
                .create_contact(misoca::contact::create_contact::Input {
                    access_token: access_token.clone(),
                    name: name.clone(),
                })
                .await
                .map_err(FieldError::from)?;

            contact_id = contact.id.unwrap().to_string();
            contact_group_id = contact.contact_group_id.unwrap().to_string();
        }

        let supplier = Tx::run(&conn, || {
            let supplier = domain::supplier::Supplier::new(
                authorized_user_id,
                contact_id,
                contact_group_id,
                name,
                billing_amount,
                billing_type,
                subject,
                now,
            );
            supplier_dao.insert(&conn, &supplier)?;
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
        let now: DateTime<Utc> = Utc::now();
        let ctx = exec.context();
        let conn = ddb::establish_connection();
        let user_dao = ctx.ddb_dao::<domain::user::User>();
        let supplier_dao = ctx.ddb_dao::<domain::supplier::Supplier>();
        let misoca_cli = &ctx.misoca_cli;
        let authorized_user_id = ctx
            .authorized_user_id
            .clone()
            .ok_or(FieldError::from("unauthorized"))?;

        let id = input.id;
        let name = input.name;
        let subject = input.subject;
        let billing_amount = input.billing_amount;

        let mut user = user_dao
            .get(&conn, authorized_user_id.clone())
            .map_err(FieldError::from)?;

        if user.misoca_refresh_token.is_empty() {
            return Err(FieldError::from("should connect misoca"));
        }

        let tokens = misoca_cli
            .refresh_tokens(misoca::tokens::refresh_tokens::Input {
                refresh_token: user.misoca_refresh_token.clone(),
            })
            .await
            .map_err(FieldError::from)?;

        let access_token = tokens.access_token;
        let refresh_token = tokens.refresh_token;

        user.update_misoca_refresh_token(refresh_token, now);
        Tx::run(&conn, || {
            user_dao.update(&conn, &user)?;
            Ok(())
        })
        .map_err(FieldError::from)?;

        let contacts = misoca_cli
            .get_contacts(misoca::contact::get_contacts::Input {
                access_token: access_token.clone(),
                page: 1,
                per_page: 100,
            })
            .await
            .map_err(FieldError::from)?;

        let mut contact_id = "".to_string();
        let mut contact_group_id = "".to_string();
        for contact in contacts {
            if contact.recipient_name.unwrap_or("".to_string()) == name {
                contact_id = contact.id.unwrap().to_string();
                contact_group_id = contact.contact_group_id.unwrap().to_string();
                break;
            }
        }

        if contact_id == "" {
            return Err(FieldError::from("not found misoca contact"));
        }

        let supplier = Tx::run(&conn, || {
            let mut supplier = supplier_dao.get(&conn, id.clone())?;
            if supplier.user_id != authorized_user_id {
                return Err(CoreError::Forbidden);
            }

            supplier.update(
                contact_id,
                contact_group_id,
                name,
                billing_amount,
                subject,
                now,
            );
            supplier_dao.update(&conn, &supplier)?;
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
        let conn = ddb::establish_connection();
        let supplier_dao = ctx.ddb_dao::<domain::supplier::Supplier>();
        let invoice_dao = ctx.ddb_dao::<domain::invoice::Invoice>();
        let authorized_user_id = ctx
            .authorized_user_id
            .clone()
            .ok_or(FieldError::from("unauthorized"))?;

        let id = input.id;

        Tx::run(&conn, || {
            let supplier = supplier_dao.get(&conn, id.clone())?;
            if supplier.user_id != authorized_user_id {
                return Err(CoreError::Forbidden);
            }

            invoice_dao.delete_by_supplier(&conn, supplier.id.clone())?;
            supplier_dao.delete(&conn, supplier.id.clone())?;
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
        let now: DateTime<Utc> = Utc::now();
        let ctx = exec.context();
        let conn = ddb::establish_connection();
        let user_dao = ctx.ddb_dao::<domain::user::User>();
        let supplier_dao = ctx.ddb_dao::<domain::supplier::Supplier>();
        let invoice_dao = ctx.ddb_dao::<domain::invoice::Invoice>();
        let misoca_cli = &ctx.misoca_cli;
        let authorized_user_id = ctx
            .authorized_user_id
            .clone()
            .ok_or(FieldError::from("unauthorized"))?;

        let code = input.code;

        let mut user = user_dao
            .get(&conn, authorized_user_id.clone())
            .map_err(FieldError::from)?;

        let tokens = misoca_cli
            .get_tokens(misoca::tokens::get_tokens::Input { code })
            .await
            .map_err(FieldError::from)?;

        let access_token = tokens.access_token;
        let refresh_token = tokens.refresh_token;

        user.update_misoca_refresh_token(refresh_token, now);
        Tx::run(&conn, || {
            user_dao.update(&conn, &user)?;
            Ok(())
        })
        .map_err(FieldError::from)?;

        let suppliers = supplier_dao
            .get_all_by_user(&conn, authorized_user_id.clone())
            .map_err(FieldError::from)?;

        for supplier in suppliers {
            let invoices = misoca_cli
                .get_invoices(misoca::invoice::get_invoices::Input {
                    access_token: access_token.clone(),
                    page: 1,
                    per_page: 100,
                    supplier_id: supplier.id.clone(),
                    contact_group_id: supplier.contact_group_id.clone(),
                })
                .await
                .map_err(FieldError::from)?;

            Tx::run(&conn, || {
                for invoice in invoices {
                    match invoice_dao.get(&conn, invoice.id.clone()) {
                        Ok(current) => {
                            if current.should_update(&invoice) {
                                invoice_dao.update(&conn, &invoice)?;
                            }
                        }
                        Err(CoreError::NotFound) => {
                            invoice_dao.insert(&conn, &invoice)?;
                        }
                        Err(_) => {}
                    }
                }
                Ok(())
            })
            .map_err(FieldError::from)?;
        }

        Ok(true)
    }

    async fn field_refresh_misoca<'s, 'r, 'a>(
        &'s self,
        exec: &Executor<'r, 'a, Context>,
    ) -> FieldResult<bool> {
        let now: DateTime<Utc> = Utc::now();
        let ctx = exec.context();
        let conn = ddb::establish_connection();
        let user_dao = ctx.ddb_dao::<domain::user::User>();
        let supplier_dao = ctx.ddb_dao::<domain::supplier::Supplier>();
        let invoice_dao = ctx.ddb_dao::<domain::invoice::Invoice>();
        let misoca_cli = &ctx.misoca_cli;
        let authorized_user_id = ctx
            .authorized_user_id
            .clone()
            .ok_or(FieldError::from("unauthorized"))?;

        let mut user = user_dao
            .get(&conn, authorized_user_id.clone())
            .map_err(FieldError::from)?;

        if user.misoca_refresh_token.is_empty() {
            return Err(FieldError::from("should connect misoca"));
        }

        let tokens = misoca_cli
            .refresh_tokens(misoca::tokens::refresh_tokens::Input {
                refresh_token: user.misoca_refresh_token.clone(),
            })
            .await
            .map_err(FieldError::from)?;

        let access_token = tokens.access_token;
        let refresh_token = tokens.refresh_token;

        user.update_misoca_refresh_token(refresh_token, now);
        Tx::run(&conn, || {
            user_dao.update(&conn, &user)?;
            Ok(())
        })
        .map_err(FieldError::from)?;

        let suppliers = supplier_dao
            .get_all_by_user(&conn, authorized_user_id.clone())
            .map_err(FieldError::from)?;

        for supplier in suppliers {
            let invoices = misoca_cli
                .get_invoices(misoca::invoice::get_invoices::Input {
                    access_token: access_token.clone(),
                    page: 1,
                    per_page: 100,
                    supplier_id: supplier.id.clone(),
                    contact_group_id: supplier.contact_group_id.clone(),
                })
                .await
                .map_err(FieldError::from)?;

            Tx::run(&conn, || {
                for invoice in invoices {
                    match invoice_dao.get(&conn, invoice.id.clone()) {
                        Ok(current) => {
                            if current.should_update(&invoice) {
                                invoice_dao.update(&conn, &invoice)?;
                            }
                        }
                        Err(CoreError::NotFound) => {
                            invoice_dao.insert(&conn, &invoice)?;
                        }
                        Err(_) => {}
                    }
                }
                Ok(())
            })
            .map_err(FieldError::from)?;
        }

        Ok(true)
    }

    async fn field_download_invoice_pdf<'s, 'r, 'a>(
        &'s self,
        exec: &Executor<'r, 'a, Context>,
        input: DownloadInvoicePDFInput,
    ) -> FieldResult<String> {
        let now: DateTime<Utc> = Utc::now();
        let ctx = exec.context();
        let conn = ddb::establish_connection();
        let user_dao = ctx.ddb_dao::<domain::user::User>();
        let invoice_dao = ctx.ddb_dao::<domain::invoice::Invoice>();
        let misoca_cli = &ctx.misoca_cli;
        let authorized_user_id = ctx
            .authorized_user_id
            .clone()
            .ok_or(FieldError::from("unauthorized"))?;

        let invoice_id = input.invoice_id;

        let mut invoice = invoice_dao
            .get(&conn, invoice_id.clone())
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

        let mut user = user_dao
            .get(&conn, authorized_user_id.clone())
            .map_err(FieldError::from)?;

        if user.misoca_refresh_token.is_empty() {
            return Err(FieldError::from("should connect misoca"));
        }

        let tokens = misoca_cli
            .refresh_tokens(misoca::tokens::refresh_tokens::Input {
                refresh_token: user.misoca_refresh_token.clone(),
            })
            .await
            .map_err(FieldError::from)?;

        let access_token = tokens.access_token;
        let refresh_token = tokens.refresh_token;

        user.update_misoca_refresh_token(refresh_token, now);
        Tx::run(&conn, || {
            user_dao.update(&conn, &user)?;
            Ok(())
        })
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
        Tx::run(&conn, || {
            invoice_dao.update(&conn, &invoice)?;
            Ok(())
        })
        .map_err(FieldError::from)?;

        let download_url = object
            .download_url(INVOICE_PDF_DOWNLOAD_DURATION)
            .map_err(FieldError::from)?;
        Ok(download_url)
    }
}
