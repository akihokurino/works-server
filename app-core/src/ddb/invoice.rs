use crate::ddb::schema::invoices;
use crate::ddb::schema::suppliers;
use crate::ddb::supplier;
use crate::ddb::Dao;
use crate::domain;
use crate::{CoreError, CoreResult};
use diesel::dsl::*;
use diesel::prelude::*;
use std::convert::TryFrom;
use std::str::FromStr;

#[derive(
    Queryable, Insertable, Debug, Clone, Eq, PartialEq, Identifiable, Associations, AsChangeset,
)]
#[belongs_to(supplier::Entity, foreign_key = "supplier_id")]
#[table_name = "invoices"]
pub struct Entity {
    pub id: String,
    pub supplier_id: String,
    pub issue_ymd: String,
    pub issue_at: Option<chrono::NaiveDateTime>,
    pub payment_due_on_ymd: String,
    pub payment_due_on_at: Option<chrono::NaiveDateTime>,
    pub invoice_number: String,
    pub payment_status: i32,
    pub invoice_status: i32,
    pub recipient_name: String,
    pub subject: String,
    pub total_amount: i32,
    pub tax: i32,
    pub pdf_path: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl TryFrom<Entity> for domain::invoice::Invoice {
    type Error = String;

    fn try_from(e: Entity) -> Result<Self, Self::Error> {
        Ok(domain::invoice::Invoice {
            id: e.id,
            supplier_id: e.supplier_id,
            issue_ymd: domain::YMD::from_str(e.issue_ymd.as_str())
                .map_err(|_e| "parse ymd error".to_string())?,
            payment_due_on_ymd: domain::YMD::from_str(e.payment_due_on_ymd.as_str())
                .map_err(|_e| "parse ymd error".to_string())?,
            invoice_number: e.invoice_number,
            payment_status: domain::invoice::PaymentStatus::from(e.payment_status),
            invoice_status: domain::invoice::InvoiceStatus::from(e.invoice_status),
            recipient_name: e.recipient_name,
            subject: e.subject,
            total_amount: e.total_amount,
            tax: e.tax,
            pdf_path: e.pdf_path,
            created_at: e.created_at,
            updated_at: e.updated_at,
        })
    }
}

impl From<domain::invoice::Invoice> for Entity {
    fn from(d: domain::invoice::Invoice) -> Entity {
        Entity {
            id: d.id,
            supplier_id: d.supplier_id,
            issue_ymd: d.issue_ymd.to_string(),
            issue_at: d.issue_ymd.to_datetime(),
            payment_due_on_ymd: d.payment_due_on_ymd.to_string(),
            payment_due_on_at: d.payment_due_on_ymd.to_datetime(),
            invoice_number: d.invoice_number,
            payment_status: d.payment_status.int(),
            invoice_status: d.invoice_status.int(),
            recipient_name: d.recipient_name,
            subject: d.subject,
            total_amount: d.total_amount,
            tax: d.tax,
            pdf_path: d.pdf_path,
            created_at: d.created_at,
            updated_at: d.updated_at,
        }
    }
}

impl Dao<domain::invoice::Invoice> {
    pub fn get_all_by_supplier(
        &self,
        conn: &MysqlConnection,
        supplier_id: String,
    ) -> CoreResult<Vec<domain::invoice::Invoice>> {
        return invoices::table
            .filter(invoices::supplier_id.eq(supplier_id))
            .order(invoices::issue_at.desc())
            .load::<Entity>(conn)
            .map(|v: Vec<Entity>| {
                v.into_iter()
                    .map(|v| domain::invoice::Invoice::try_from(v).unwrap())
                    .collect::<Vec<_>>()
            })
            .map_err(CoreError::from);
    }

    pub fn get_all_by_user_with_supplier(
        &self,
        conn: &MysqlConnection,
        user_id: String,
    ) -> CoreResult<Vec<(domain::invoice::Invoice, domain::supplier::Supplier)>> {
        return invoices::table
            .inner_join(suppliers::table)
            .filter(suppliers::user_id.eq(user_id))
            .order(invoices::issue_at.desc())
            .load::<(Entity, supplier::Entity)>(conn)
            .map(|v: Vec<(Entity, supplier::Entity)>| {
                v.into_iter()
                    .map(|v| {
                        (
                            domain::invoice::Invoice::try_from(v.0).unwrap(),
                            domain::supplier::Supplier::try_from(v.1).unwrap(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .map_err(CoreError::from);
    }

    pub fn get(&self, conn: &MysqlConnection, id: String) -> CoreResult<domain::invoice::Invoice> {
        invoices::table
            .find(id)
            .first(conn)
            .map(|v: Entity| domain::invoice::Invoice::try_from(v).unwrap())
            .map_err(CoreError::from)
    }

    pub fn exist_by_subject(
        &self,
        conn: &MysqlConnection,
        supplier_id: String,
        subject: String,
    ) -> CoreResult<bool> {
        select(exists(
            invoices::table.filter(
                invoices::subject
                    .eq(subject)
                    .and(invoices::supplier_id.eq(supplier_id)),
            ),
        ))
        .get_result(conn)
        .map_err(CoreError::from)
    }

    pub fn insert(
        &self,
        conn: &MysqlConnection,
        item: &domain::invoice::Invoice,
    ) -> CoreResult<()> {
        let e: Entity = item.clone().into();
        if let Err(e) = diesel::insert_into(invoices::table)
            .values(e)
            .execute(conn)
            .map_err(CoreError::from)
        {
            return Err(e);
        }
        Ok(())
    }

    pub fn update(
        &self,
        conn: &MysqlConnection,
        item: &domain::invoice::Invoice,
    ) -> CoreResult<()> {
        let e: Entity = item.clone().into();
        if let Err(e) = diesel::update(invoices::table.find(e.id.clone()))
            .set(&e)
            .execute(conn)
            .map_err(CoreError::from)
        {
            return Err(e);
        }
        Ok(())
    }

    pub fn delete_by_supplier(
        &self,
        conn: &MysqlConnection,
        supplier_id: String,
    ) -> CoreResult<()> {
        if let Err(e) = diesel::delete(invoices::table)
            .filter(invoices::supplier_id.eq(supplier_id))
            .execute(conn)
            .map_err(CoreError::from)
        {
            return Err(e);
        }
        Ok(())
    }
}
