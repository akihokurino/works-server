use crate::ddb::schema::suppliers;
use crate::ddb::schema::users;
use crate::ddb::supplier;
use crate::ddb::Dao;
use crate::domain;
use crate::{CoreError, CoreResult};
use diesel::prelude::*;
use std::convert::TryFrom;

#[derive(Queryable, Insertable, Debug, Clone, Eq, PartialEq, Identifiable, AsChangeset)]
#[table_name = "users"]
pub struct Entity {
    pub id: String,
    pub misoca_refresh_token: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl TryFrom<Entity> for domain::user::User {
    type Error = String;

    fn try_from(e: Entity) -> Result<Self, Self::Error> {
        Ok(domain::user::User {
            id: e.id.to_string(),
            misoca_refresh_token: e.misoca_refresh_token,
            created_at: e.created_at,
            updated_at: e.updated_at,
        })
    }
}

impl From<domain::user::User> for Entity {
    fn from(d: domain::user::User) -> Entity {
        Entity {
            id: d.id,
            misoca_refresh_token: d.misoca_refresh_token,
            created_at: d.created_at,
            updated_at: d.updated_at,
        }
    }
}

impl Dao<domain::user::User> {
    pub fn get_all_with_suppliers(
        &self,
        conn: &MysqlConnection,
    ) -> CoreResult<Vec<(domain::user::User, Vec<domain::supplier::Supplier>)>> {
        let user_entities = users::table
            .order(users::created_at.desc())
            .load::<Entity>(conn)
            .map_err(CoreError::from)?;

        let supplier_entities = supplier::Entity::belonging_to(&user_entities)
            .order(suppliers::created_at.desc())
            .load::<supplier::Entity>(conn)
            .map_err(CoreError::from)?
            .grouped_by(&user_entities);

        let zipped = user_entities
            .into_iter()
            .zip(supplier_entities)
            .map(|v: (Entity, Vec<supplier::Entity>)| {
                (
                    domain::user::User::try_from(v.0).unwrap(),
                    v.1.into_iter()
                        .map(|v| domain::supplier::Supplier::try_from(v).unwrap())
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();

        Ok(zipped)
    }

    pub fn get(&self, conn: &MysqlConnection, id: String) -> CoreResult<domain::user::User> {
        users::table
            .find(id)
            .first(conn)
            .map(|v: Entity| domain::user::User::try_from(v).unwrap())
            .map_err(CoreError::from)
    }

    pub fn insert(&self, conn: &MysqlConnection, item: &domain::user::User) -> CoreResult<()> {
        let e: Entity = item.clone().into();
        if let Err(e) = diesel::insert_into(users::table)
            .values(e)
            .execute(conn)
            .map_err(CoreError::from)
        {
            return Err(e);
        }
        Ok(())
    }

    pub fn update(&self, conn: &MysqlConnection, item: &domain::user::User) -> CoreResult<()> {
        let e: Entity = item.clone().into();
        if let Err(e) = diesel::update(users::table.find(e.id.clone()))
            .set(&e)
            .execute(conn)
            .map_err(CoreError::from)
        {
            return Err(e);
        }
        Ok(())
    }
}
