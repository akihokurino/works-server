use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Bank {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub code: String,
    pub account_type: AccountType,
    pub account_number: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl Bank {
    pub fn new(
        user_id: String,
        name: String,
        code: String,
        account_type: AccountType,
        account_number: String,
        now: DateTime<Utc>,
    ) -> Self {
        Bank {
            id: Uuid::new_v4().to_string(),
            user_id,
            name,
            code,
            account_type,
            account_number,
            created_at: now.naive_utc(),
            updated_at: now.naive_utc(),
        }
    }

    pub fn update(
        &mut self,
        name: String,
        code: String,
        account_type: AccountType,
        account_number: String,
        now: DateTime<Utc>,
    ) {
        self.name = name;
        self.code = code;
        self.account_type = account_type;
        self.account_number = account_number;
        self.updated_at = now.naive_utc();
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AccountType {
    Savings,
    Checking,
}

impl AccountType {
    pub fn int(&self) -> i32 {
        match self {
            Self::Savings => 0,
            Self::Checking => 1,
        }
    }
}

impl Default for AccountType {
    fn default() -> Self {
        Self::Savings
    }
}

impl From<i32> for AccountType {
    fn from(v: i32) -> AccountType {
        match v {
            0 => Self::Savings,
            1 => Self::Checking,
            _ => Self::default(),
        }
    }
}
