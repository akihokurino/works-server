use crate::ddb::schema::senders;
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
#[table_name = "senders"]
pub struct Entity {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub email: String,
    pub tel: String,
    pub postal_code: String,
    pub address: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl TryFrom<Entity> for domain::sender::Sender {
    type Error = String;

    fn try_from(e: Entity) -> Result<Self, Self::Error> {
        Ok(domain::sender::Sender {
            id: e.id,
            user_id: e.user_id,
            name: e.name,
            email: e.email,
            tel: e.tel,
            postal_code: e.postal_code,
            address: e.address,
            created_at: e.created_at,
            updated_at: e.updated_at,
        })
    }
}

impl From<domain::sender::Sender> for Entity {
    fn from(d: domain::sender::Sender) -> Entity {
        Entity {
            id: d.id,
            user_id: d.user_id,
            name: d.name,
            email: d.email,
            tel: d.tel,
            postal_code: d.postal_code,
            address: d.address,
            created_at: d.created_at,
            updated_at: d.updated_at,
        }
    }
}

impl Dao<domain::sender::Sender> {
    pub fn get_all_by_user(
        &self,
        conn: &MysqlConnection,
        user_id: String,
    ) -> CoreResult<Vec<domain::sender::Sender>> {
        return senders::table
            .filter(senders::user_id.eq(user_id))
            .order(senders::created_at.desc())
            .load::<Entity>(conn)
            .map(|v: Vec<Entity>| {
                v.into_iter()
                    .map(|v| domain::sender::Sender::try_from(v).unwrap())
                    .collect::<Vec<_>>()
            })
            .map_err(CoreError::from);
    }

    pub fn get(&self, conn: &MysqlConnection, id: String) -> CoreResult<domain::sender::Sender> {
        senders::table
            .find(id)
            .first(conn)
            .map(|v: Entity| domain::sender::Sender::try_from(v).unwrap())
            .map_err(CoreError::from)
    }

    pub fn insert(&self, conn: &MysqlConnection, item: &domain::sender::Sender) -> CoreResult<()> {
        let e: Entity = item.clone().into();
        if let Err(e) = diesel::insert_into(senders::table)
            .values(e)
            .execute(conn)
            .map_err(CoreError::from)
        {
            return Err(e);
        }
        Ok(())
    }

    pub fn update(&self, conn: &MysqlConnection, item: &domain::sender::Sender) -> CoreResult<()> {
        let e: Entity = item.clone().into();
        if let Err(e) = diesel::update(senders::table.find(e.id.clone()))
            .set(&e)
            .execute(conn)
            .map_err(CoreError::from)
        {
            return Err(e);
        }
        Ok(())
    }

    pub fn delete(&self, conn: &MysqlConnection, id: String) -> CoreResult<()> {
        if let Err(e) = diesel::delete(senders::table.find(id))
            .execute(conn)
            .map_err(CoreError::from)
        {
            return Err(e);
        }
        Ok(())
    }
}
