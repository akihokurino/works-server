use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct User {
    pub id: String,
    pub misoca_refresh_token: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl User {
    pub fn new(id: String, now: DateTime<Utc>) -> Self {
        User {
            id,
            misoca_refresh_token: "".to_string(),
            created_at: now.naive_utc(),
            updated_at: now.naive_utc(),
        }
    }

    pub fn update_misoca_refresh_token(&mut self, token: String, now: DateTime<Utc>) {
        self.misoca_refresh_token = token;
        self.updated_at = now.naive_utc();
    }
}
