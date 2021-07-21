use crate::ddb::schema::invoices;
use crate::ddb::supplier;
use crate::ddb::{Dao, DaoError, DaoResult};
use crate::domain;
use diesel::prelude::*;
use std::convert::TryFrom;
use std::str::FromStr;

#[derive(Queryable, Insertable, Debug, Clone, Eq, PartialEq, Identifiable, Associations)]
#[belongs_to(supplier::Entity, foreign_key = "supplier_id")]
#[table_name = "invoices"]
pub struct Entity {
    pub id: String,
    pub supplier_id: String,
    pub issue_ymd: String,
    pub issue_at: Option<chrono::NaiveDate>,
    pub payment_due_on_ymd: String,
    pub payment_due_on_at: Option<chrono::NaiveDate>,
    pub invoice_number: String,
    pub payment_status: i32,
    pub invoice_status: i32,
    pub recipient_name: String,
    pub subject: String,
    pub total_amount: i32,
    pub tax: i32,
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
            issue_at: d.issue_ymd.to_date(),
            payment_due_on_ymd: d.payment_due_on_ymd.to_string(),
            payment_due_on_at: d.payment_due_on_ymd.to_date(),
            invoice_number: d.invoice_number,
            payment_status: d.payment_status.int(),
            invoice_status: d.invoice_status.int(),
            recipient_name: d.recipient_name,
            subject: d.subject,
            total_amount: d.total_amount,
            tax: d.tax,
            created_at: d.created_at,
            updated_at: d.updated_at,
        }
    }
}

impl Dao<domain::invoice::Invoice> {
    pub fn insert(&self, item: &domain::invoice::Invoice) -> DaoResult<()> {
        let e: Entity = item.clone().into();
        if let Err(e) = diesel::insert_into(invoices::table)
            .values(e)
            .execute(&self.conn)
            .map_err(DaoError::from)
        {
            return Err(e);
        }
        Ok(())
    }
}
