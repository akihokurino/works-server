use crate::ddb::schema::banks;
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
#[table_name = "banks"]
pub struct Entity {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub code: String,
    pub account_type: i32,
    pub account_number: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl TryFrom<Entity> for domain::bank::Bank {
    type Error = String;

    fn try_from(e: Entity) -> Result<Self, Self::Error> {
        Ok(domain::bank::Bank {
            id: e.id,
            user_id: e.user_id,
            name: e.name,
            code: e.code,
            account_type: domain::bank::AccountType::from(e.account_type),
            account_number: e.account_number,
            created_at: e.created_at,
            updated_at: e.updated_at,
        })
    }
}

impl From<domain::bank::Bank> for Entity {
    fn from(d: domain::bank::Bank) -> Entity {
        Entity {
            id: d.id,
            user_id: d.user_id,
            name: d.name,
            code: d.code,
            account_type: d.account_type.int(),
            account_number: d.account_number,
            created_at: d.created_at,
            updated_at: d.updated_at,
        }
    }
}

impl Dao<domain::bank::Bank> {
    pub fn get_all_by_user(
        &self,
        conn: &MysqlConnection,
        user_id: String,
    ) -> CoreResult<Vec<domain::bank::Bank>> {
        return banks::table
            .filter(banks::user_id.eq(user_id))
            .order(banks::created_at.desc())
            .load::<Entity>(conn)
            .map(|v: Vec<Entity>| {
                v.into_iter()
                    .map(|v| domain::bank::Bank::try_from(v).unwrap())
                    .collect::<Vec<_>>()
            })
            .map_err(CoreError::from);
    }

    pub fn get(&self, conn: &MysqlConnection, id: String) -> CoreResult<domain::bank::Bank> {
        banks::table
            .find(id)
            .first(conn)
            .map(|v: Entity| domain::bank::Bank::try_from(v).unwrap())
            .map_err(CoreError::from)
    }

    pub fn insert(&self, conn: &MysqlConnection, item: &domain::bank::Bank) -> CoreResult<()> {
        let e: Entity = item.clone().into();
        if let Err(e) = diesel::insert_into(banks::table)
            .values(e)
            .execute(conn)
            .map_err(CoreError::from)
        {
            return Err(e);
        }
        Ok(())
    }

    pub fn update(&self, conn: &MysqlConnection, item: &domain::bank::Bank) -> CoreResult<()> {
        let e: Entity = item.clone().into();
        if let Err(e) = diesel::update(banks::table.find(e.id.clone()))
            .set(&e)
            .execute(conn)
            .map_err(CoreError::from)
        {
            return Err(e);
        }
        Ok(())
    }

    pub fn delete(&self, conn: &MysqlConnection, id: String) -> CoreResult<()> {
        if let Err(e) = diesel::delete(banks::table.find(id))
            .execute(conn)
            .map_err(CoreError::from)
        {
            return Err(e);
        }
        Ok(())
    }
}
