use chrono::{DateTime, Utc};
use uuid::Uuid;

const CONSUMPTION_TAX_RATE: f64 = 0.1;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Supplier {
    pub id: String,
    pub user_id: String,
    pub contact_id: String,
    pub name: String,
    pub billing_amount: i32,
    pub billing_type: BillingType,
    pub subject: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl Supplier {
    pub fn new(
        user_id: String,
        contact_id: String,
        name: String,
        billing_amount: i32,
        billing_type: BillingType,
        subject: String,
        now: DateTime<Utc>,
    ) -> Self {
        Supplier {
            id: Uuid::new_v4().to_string(),
            user_id,
            contact_id,
            name,
            billing_amount,
            billing_type,
            subject,
            created_at: now.naive_utc(),
            updated_at: now.naive_utc(),
        }
    }

    pub fn update(
        &mut self,
        contact_id: String,
        name: String,
        billing_amount: i32,
        subject: String,
        now: DateTime<Utc>,
    ) {
        self.contact_id = contact_id;
        self.name = name;
        self.billing_amount = billing_amount;
        self.subject = subject;
        self.updated_at = now.naive_utc();
    }

    pub fn billing_amount_include_tax(&self) -> i32 {
        let tmp =
            f64::from(self.billing_amount) + f64::from(self.billing_amount) * CONSUMPTION_TAX_RATE;
        tmp.floor() as i32
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BillingType {
    Monthly,
    OneTime,
}

impl BillingType {
    pub fn int(&self) -> i32 {
        match self {
            Self::Monthly => 0,
            Self::OneTime => 1,
        }
    }
}

impl Default for BillingType {
    fn default() -> Self {
        Self::Monthly
    }
}

impl From<i32> for BillingType {
    fn from(v: i32) -> BillingType {
        match v {
            0 => Self::Monthly,
            1 => Self::OneTime,
            _ => Self::default(),
        }
    }
}
