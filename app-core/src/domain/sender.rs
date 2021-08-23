use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Sender {
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

impl Sender {
    pub fn new(
        user_id: String,
        name: String,
        email: String,
        tel: String,
        postal_code: String,
        address: String,
        now: DateTime<Utc>,
    ) -> Self {
        Sender {
            id: Uuid::new_v4().to_string(),
            user_id,
            name,
            email,
            tel,
            postal_code,
            address,
            created_at: now.naive_utc(),
            updated_at: now.naive_utc(),
        }
    }

    pub fn update(
        &mut self,
        name: String,
        email: String,
        tel: String,
        postal_code: String,
        address: String,
        now: DateTime<Utc>,
    ) {
        self.name = name;
        self.email = email;
        self.tel = tel;
        self.postal_code = postal_code;
        self.address = address;
        self.updated_at = now.naive_utc();
    }
}
