use crate::ddb::schema::suppliers;
use crate::ddb::user;
use crate::ddb::{Dao, DaoError, DaoResult};
use crate::domain;
use diesel::prelude::*;
use std::convert::TryFrom;

#[derive(Queryable, Insertable, Debug, Clone, Eq, PartialEq, Identifiable, Associations)]
#[belongs_to(user::Entity, foreign_key = "user_id")]
#[table_name = "suppliers"]
pub struct Entity {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub billing_amount: i32,
    pub billing_type: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl TryFrom<Entity> for domain::supplier::Supplier {
    type Error = String;

    fn try_from(e: Entity) -> Result<Self, Self::Error> {
        Ok(domain::supplier::Supplier {
            id: e.id,
            user_id: e.user_id,
            name: e.name,
            billing_amount: e.billing_amount,
            billing_type: domain::supplier::BillingType::from(e.billing_type),
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
            name: d.name,
            billing_amount: d.billing_amount,
            billing_type: d.billing_type.int(),
            created_at: d.created_at,
            updated_at: d.updated_at,
        }
    }
}

impl Dao<domain::supplier::Supplier> {
    pub fn get_all_by_user(&self, user_id: String) -> DaoResult<Vec<domain::supplier::Supplier>> {
        return suppliers::table
            .filter(suppliers::user_id.eq(user_id))
            .order(suppliers::created_at.desc())
            .load::<Entity>(&self.conn)
            .map(|v: Vec<Entity>| {
                v.into_iter()
                    .map(|v| domain::supplier::Supplier::try_from(v).unwrap())
                    .collect::<Vec<_>>()
            })
            .map_err(DaoError::from);
    }

    pub fn get(&self, id: String) -> DaoResult<domain::supplier::Supplier> {
        suppliers::table
            .find(id)
            .first(&self.conn)
            .map(|v: Entity| domain::supplier::Supplier::try_from(v).unwrap())
            .map_err(DaoError::from)
    }

    pub fn insert(&self, item: &domain::supplier::Supplier) -> DaoResult<()> {
        let e: Entity = item.clone().into();
        if let Err(e) = diesel::insert_into(suppliers::table)
            .values(e)
            .execute(&self.conn)
            .map_err(DaoError::from)
        {
            return Err(e);
        }
        Ok(())
    }

    pub fn update(&self, item: &domain::supplier::Supplier) -> DaoResult<()> {
        let e: Entity = item.clone().into();
        if let Err(e) = diesel::update(suppliers::table.find(e.id))
            .set((
                suppliers::name.eq(e.name),
                suppliers::billing_amount.eq(e.billing_amount),
                suppliers::updated_at.eq(e.updated_at),
            ))
            .execute(&self.conn)
            .map_err(DaoError::from)
        {
            return Err(e);
        }
        Ok(())
    }

    pub fn delete(&self, id: String) -> DaoResult<()> {
        if let Err(e) = diesel::delete(suppliers::table.find(id))
            .execute(&self.conn)
            .map_err(DaoError::from)
        {
            return Err(e);
        }
        Ok(())
    }
}
