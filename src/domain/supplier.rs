use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Supplier {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl Supplier {
    pub fn new(user_id: String, name: String, now: DateTime<Utc>) -> Self {
        Supplier {
            id: Uuid::new_v4().to_string(),
            user_id,
            name,
            created_at: now.naive_utc(),
            updated_at: now.naive_utc(),
        }
    }

    pub fn update(&mut self, name: String, now: DateTime<Utc>) {
        self.name = name;
        self.updated_at = now.naive_utc();
    }
}
