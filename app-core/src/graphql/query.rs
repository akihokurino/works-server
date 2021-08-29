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
        let user_dao = ctx.ddb_dao::<domain::user::User>();
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
        _: &QueryTrail<'r, SupplierConnection, Walked>,
    ) -> FieldResult<SupplierConnection> {
        let ctx = exec.context();
        let conn = ctx.get_mutex_connection();
        let supplier_dao = ctx.ddb_dao::<domain::supplier::Supplier>();
        let authenticated_user_id = ctx
            .authenticated_user_id
            .clone()
            .ok_or(FieldErrorWithCode::from(CoreError::UnAuthenticate))?;

        let suppliers = supplier_dao
            .get_all_by_user_with_invoices(&conn, authenticated_user_id)
            .map_err(FieldErrorWithCode::from)?;

        Ok(SupplierConnection(suppliers))
    }

    async fn field_invoice_list<'s, 'r, 'a>(
        &'s self,
        exec: &Executor<'r, 'a, Context>,
        _: &QueryTrail<'r, InvoiceConnection, Walked>,
        supplier_id: String,
    ) -> FieldResult<InvoiceConnection> {
        let ctx = exec.context();
        let conn = ctx.get_mutex_connection();
        let supplier_dao = ctx.ddb_dao::<domain::supplier::Supplier>();
        let invoice_dao = ctx.ddb_dao::<domain::invoice::Invoice>();
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

        let invoices = invoice_dao
            .get_all_by_supplier(&conn, supplier.id)
            .map_err(FieldErrorWithCode::from)?;

        Ok(InvoiceConnection(invoices))
    }

    async fn field_invoice_history_list<'s, 'r, 'a>(
        &'s self,
        exec: &Executor<'r, 'a, Context>,
        _: &QueryTrail<'r, InvoiceHistoryConnection, Walked>,
    ) -> FieldResult<InvoiceHistoryConnection> {
        let ctx = exec.context();
        let conn = ctx.get_mutex_connection();
        let invoice_dao = ctx.ddb_dao::<domain::invoice::Invoice>();
        let authenticated_user_id = ctx
            .authenticated_user_id
            .clone()
            .ok_or(FieldErrorWithCode::from(CoreError::UnAuthenticate))?;

        let histories = invoice_dao
            .get_all_by_user_with_supplier(&conn, authenticated_user_id)
            .map_err(FieldErrorWithCode::from)?;

        Ok(InvoiceHistoryConnection(histories))
    }
}
