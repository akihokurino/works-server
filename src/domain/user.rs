use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct User {
    pub id: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl User {
    pub fn new(now: DateTime<Utc>) -> Self {
        User {
            id: Uuid::new_v4().to_string(),
            created_at: now.naive_utc(),
            updated_at: now.naive_utc(),
        }
    }
}
