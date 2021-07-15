use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct User {
    pub id: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl User {
    pub fn new(id: String, now: DateTime<Utc>) -> Self {
        User {
            id,
            created_at: now.naive_utc(),
            updated_at: now.naive_utc(),
        }
    }
}
