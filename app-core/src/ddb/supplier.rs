use crate::ddb::invoice;
use crate::ddb::schema::invoices;
use crate::ddb::schema::suppliers;
use crate::ddb::user;
use crate::ddb::Dao;
use crate::domain;
use crate::{CoreError, CoreResult};
use diesel::prelude::*;
use std::convert::TryFrom;

#[derive(
    Queryable, Insertable, Debug, Clone, Eq, PartialEq, Identifiable, Associations, AsChangeset,
)]
#[belongs_to(user::Entity, foreign_key = "user_id")]
#[table_name = "suppliers"]
pub struct Entity {
    pub id: String,
    pub user_id: String,
    pub contact_id: String,
    pub contact_group_id: String,
    pub name: String,
    pub billing_amount: i32,
    pub billing_type: i32,
    pub subject: String,
    pub subject_template: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl TryFrom<Entity> for domain::supplier::Supplier {
    type Error = String;

    fn try_from(e: Entity) -> Result<Self, Self::Error> {
        Ok(domain::supplier::Supplier {
            id: e.id,
            user_id: e.user_id,
            contact_id: e.contact_id,
            contact_group_id: e.contact_group_id,
            name: e.name,
            billing_amount: e.billing_amount,
            billing_type: domain::supplier::BillingType::from(e.billing_type),
            subject: e.subject,
            subject_template: e.subject_template,
            created_at: e.created_at,
            updated_at: e.updated_at,
        })
    }
}

impl From<domain::supplier::Supplier> for Entity {
    fn from(d: domain::supplier::Supplier) -> Entity {
        Entity {
            id: d.id,
            user_id: d.user_id,
            contact_id: d.contact_id,
            contact_group_id: d.contact_group_id,
            name: d.name,
            billing_amount: d.billing_amount,
            billing_type: d.billing_type.int(),
            subject: d.subject,
            subject_template: d.subject_template,
            created_at: d.created_at,
            updated_at: d.updated_at,
        }
    }
}

impl Dao<domain::supplier::Supplier> {
    pub fn get_all_by_user(
        &self,
        conn: &MysqlConnection,
        user_id: String,
    ) -> CoreResult<Vec<domain::supplier::Supplier>> {
        return suppliers::table
            .filter(suppliers::user_id.eq(user_id))
            .order(suppliers::created_at.desc())
            .load::<Entity>(conn)
            .map(|v: Vec<Entity>| {
                v.into_iter()
                    .map(|v| domain::supplier::Supplier::try_from(v).unwrap())
                    .collect::<Vec<_>>()
            })
            .map_err(CoreError::from);
    }

    pub fn get_all_by_user_with_invoices(
        &self,
        conn: &MysqlConnection,
        user_id: String,
    ) -> CoreResult<Vec<domain::supplier::SupplierWithInvoices>> {
        let supplier_entities: Vec<Entity> = suppliers::table
            .filter(suppliers::user_id.eq(user_id))
            .order(suppliers::created_at.desc())
            .load::<Entity>(conn)
            .map_err(CoreError::from)?;

        let invoice_entities: Vec<invoice::Entity> =
            invoice::Entity::belonging_to(&supplier_entities)
                .order(invoices::created_at.desc())
                .load::<invoice::Entity>(conn)
                .map_err(CoreError::from)?;

        let supplier_entities_with_invoices: Vec<(Entity, Vec<invoice::Entity>)> =
            supplier_entities
                .clone()
                .into_iter()
                .zip(invoice_entities.clone().grouped_by(&supplier_entities))
                .collect::<Vec<_>>();

        Ok(supplier_entities_with_invoices
            .into_iter()
            .map(
                |v: (Entity, Vec<invoice::Entity>)| domain::supplier::SupplierWithInvoices {
                    supplier: domain::supplier::Supplier::try_from(v.0).unwrap(),
                    invoices: v
                        .1
                        .into_iter()
                        .map(|v| domain::invoice::Invoice::try_from(v).unwrap())
                        .collect::<Vec<_>>(),
                },
            )
            .collect::<Vec<_>>())
    }

    pub fn get(
        &self,
        conn: &MysqlConnection,
        id: String,
    ) -> CoreResult<domain::supplier::Supplier> {
        suppliers::table
            .find(id)
            .first(conn)
            .map(|v: Entity| domain::supplier::Supplier::try_from(v).unwrap())
            .map_err(CoreError::from)
    }

    pub fn insert(
        &self,
        conn: &MysqlConnection,
        item: &domain::supplier::Supplier,
    ) -> CoreResult<()> {
        let e: Entity = item.clone().into();
        if let Err(e) = diesel::insert_into(suppliers::table)
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
        item: &domain::supplier::Supplier,
    ) -> CoreResult<()> {
        let e: Entity = item.clone().into();
        if let Err(e) = diesel::update(suppliers::table.find(e.id.clone()))
            .set(&e)
            .execute(conn)
            .map_err(CoreError::from)
        {
            return Err(e);
        }
        Ok(())
    }

    pub fn delete(&self, conn: &MysqlConnection, id: String) -> CoreResult<()> {
        if let Err(e) = diesel::delete(suppliers::table.find(id))
            .execute(conn)
            .map_err(CoreError::from)
        {
            return Err(e);
        }
        Ok(())
    }
}
