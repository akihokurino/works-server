use crate::ddb::pager::Pager;
use crate::ddb::Dao;
use crate::graphql::invoice::InvoiceConnection;
use crate::graphql::Context;
use crate::graphql::*;
use crate::{domain, CoreError, FieldErrorWithCode};
use async_trait::async_trait;
use juniper::{Executor, FieldResult};
use juniper_from_schema::{QueryTrail, Walked};

pub struct Query;
#[async_trait]
impl QueryFields for Query {
    async fn field_me<'s, 'r, 'a>(
        &'s self,
        exec: &Executor<'r, 'a, Context>,
        _: &QueryTrail<'r, Me, Walked>,
    ) -> FieldResult<Me> {
        let ctx = exec.context();
        let conn = ctx.get_mutex_connection();
        let user_dao: Dao<domain::user::User> = Dao::new();
        let authenticated_user_id = ctx
            .authenticated_user_id
            .clone()
            .ok_or(FieldErrorWithCode::from(CoreError::UnAuthenticate))?;

        let user = user_dao
            .get(&conn, authenticated_user_id)
            .map_err(FieldErrorWithCode::from)?;

        Ok(Me { user })
    }

    async fn field_supplier_list<'s, 'r, 'a>(
        &'s self,
        exec: &Executor<'r, 'a, Context>,
        _: &QueryTrail<'r, Supplier, Walked>,
    ) -> FieldResult<Vec<Supplier>> {
        let ctx = exec.context();
        let conn = ctx.get_mutex_connection();
        let supplier_dao: Dao<domain::supplier::Supplier> = Dao::new();
        let authenticated_user_id = ctx
            .authenticated_user_id
            .clone()
            .ok_or(FieldErrorWithCode::from(CoreError::UnAuthenticate))?;

        let suppliers = supplier_dao
            .get_all_by_user(&conn, authenticated_user_id)
            .map_err(FieldErrorWithCode::from)?;

        Ok(suppliers
            .iter()
            .map(|v| Supplier {
                supplier: v.to_owned(),
            })
            .collect())
    }

    async fn field_invoice_list<'s, 'r, 'a>(
        &'s self,
        exec: &Executor<'r, 'a, Context>,
        _: &QueryTrail<'r, InvoiceConnection, Walked>,
        supplier_id: String,
        page: i32,
        limit: i32,
    ) -> FieldResult<InvoiceConnection> {
        let ctx = exec.context();
        let conn = ctx.get_mutex_connection();
        let supplier_dao: Dao<domain::supplier::Supplier> = Dao::new();
        let invoice_dao: Dao<domain::invoice::Invoice> = Dao::new();
        let authenticated_user_id = ctx
            .authenticated_user_id
            .clone()
            .ok_or(FieldErrorWithCode::from(CoreError::UnAuthenticate))?;

        let supplier = supplier_dao
            .get(&conn, supplier_id)
            .map_err(FieldErrorWithCode::from)?;

        if supplier.user_id != authenticated_user_id {
            return Err(FieldErrorWithCode::from(CoreError::UnAuthenticate).into());
        }

        let pager = Pager::new(page, limit);

        let invoices = invoice_dao
            .get_all_by_supplier(&conn, supplier.id.clone(), &pager)
            .map_err(FieldErrorWithCode::from)?;

        let total_count = invoice_dao
            .get_count_by_supplier(&conn, supplier.id.clone())
            .map_err(FieldErrorWithCode::from)?;

        let has_next = total_count > pager.get_offset() + invoices.len() as i64;

        Ok(InvoiceConnection {
            invoices,
            total_count,
            has_next,
        })
    }

    async fn field_invoice_history_list<'s, 'r, 'a>(
        &'s self,
        exec: &Executor<'r, 'a, Context>,
        _: &QueryTrail<'r, InvoiceHistoryConnection, Walked>,
        page: i32,
        limit: i32,
    ) -> FieldResult<InvoiceHistoryConnection> {
        let ctx = exec.context();
        let conn = ctx.get_mutex_connection();
        let invoice_dao: Dao<domain::invoice::Invoice> = Dao::new();
        let authenticated_user_id = ctx
            .authenticated_user_id
            .clone()
            .ok_or(FieldErrorWithCode::from(CoreError::UnAuthenticate))?;

        let pager = Pager::new(page, limit);

        let histories = invoice_dao
            .get_all_by_user(&conn, authenticated_user_id.clone(), &pager)
            .map_err(FieldErrorWithCode::from)?;

        let total_count = invoice_dao
            .get_count_by_user(&conn, authenticated_user_id.clone())
            .map_err(FieldErrorWithCode::from)?;

        let has_next = total_count > pager.get_offset() + histories.len() as i64;

        Ok(InvoiceHistoryConnection {
            histories,
            total_count,
            has_next,
        })
    }
}
