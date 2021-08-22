use crate::domain::invoice::Invoice;
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use uuid::Uuid;

const CONSUMPTION_TAX_RATE: f64 = 0.1;
const DATE_PLACEHOLDER: &str = "{D}";
const SUBJECT_PLACEHOLDER: &str = "{S}";

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Supplier {
    pub id: String,
    pub user_id: String,
    pub contact_id: String,
    pub contact_group_id: String,
    pub name: String,
    pub billing_amount: i32,
    pub billing_type: BillingType,
    pub subject: String,
    pub subject_template: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SupplierWithInvoices {
    pub supplier: Supplier,
    pub invoices: Vec<Invoice>,
}

impl Supplier {
    pub fn new(
        user_id: String,
        contact_id: String,
        contact_group_id: String,
        name: String,
        billing_amount: i32,
        billing_type: BillingType,
        subject: String,
        subject_template: String,
        now: DateTime<Utc>,
    ) -> Self {
        Supplier {
            id: Uuid::new_v4().to_string(),
            user_id,
            contact_id,
            contact_group_id,
            name,
            billing_amount,
            billing_type,
            subject,
            subject_template,
            created_at: now.naive_utc(),
            updated_at: now.naive_utc(),
        }
    }

    pub fn update(
        &mut self,
        contact_id: String,
        contact_group_id: String,
        name: String,
        billing_amount: i32,
        subject: String,
        subject_template: String,
        now: DateTime<Utc>,
    ) {
        self.contact_id = contact_id;
        self.contact_group_id = contact_group_id;
        self.name = name;
        self.billing_amount = billing_amount;
        self.subject = subject;
        self.subject_template = subject_template;
        self.updated_at = now.naive_utc();
    }

    pub fn billing_amount_include_tax(&self) -> i32 {
        let tmp =
            f64::from(self.billing_amount) + f64::from(self.billing_amount) * CONSUMPTION_TAX_RATE;
        tmp.floor() as i32
    }

    pub fn subject_in_this_month(&self, now: DateTime<Utc>) -> String {
        let first_day_in_last_month = NaiveDate::from_ymd(now.year(), now.month() - 1, 1);
        let target_year = first_day_in_last_month.year();
        let target_month = first_day_in_last_month.month();

        if self.subject_template.is_empty() {
            return format!(
                "{} ({}年{}月分)",
                self.subject.clone(),
                target_year,
                target_month
            );
        }

        self.subject_template
            .replace(SUBJECT_PLACEHOLDER, &self.subject)
            .replace(
                DATE_PLACEHOLDER,
                format!("{}年{}月分", target_year, target_month).as_str(),
            )
    }

    pub fn payment_date_in_this_month(&self, now: DateTime<Utc>) -> (String, String) {
        let issue_date = now.format("%Y-%m-%d").to_string();
        let first_day_in_next_month = NaiveDate::from_ymd(now.year(), now.month() + 1, 1);
        let last_day_in_month = first_day_in_next_month - Duration::hours(24);
        let payment_due_on = last_day_in_month.format("%Y-%m-%d").to_string();
        return (issue_date, payment_due_on);
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

#[cfg(test)]
mod supplier_tests {
    use crate::domain::supplier::{BillingType, Supplier};
    use chrono::{NaiveDateTime, TimeZone, Utc};

    #[test]
    fn subject_in_this_month() {
        let dt = NaiveDateTime::parse_from_str("2021/09/01 12:00:00", "%Y/%m/%d %H:%M:%S").unwrap();
        let now = Utc.from_local_datetime(&dt).unwrap();

        let supplier1 = Supplier {
            id: "".to_string(),
            user_id: "".to_string(),
            contact_id: "".to_string(),
            contact_group_id: "".to_string(),
            name: "".to_string(),
            billing_amount: 0,
            billing_type: BillingType::OneTime,
            subject: "通常の件名テスト".to_string(),
            subject_template: "".to_string(),
            created_at: now.naive_utc(),
            updated_at: now.naive_utc(),
        };

        assert_eq!(
            supplier1.subject_in_this_month(now),
            "通常の件名テスト (2021年8月分)"
        );

        let supplier2 = Supplier {
            id: "".to_string(),
            user_id: "".to_string(),
            contact_id: "".to_string(),
            contact_group_id: "".to_string(),
            name: "".to_string(),
            billing_amount: 0,
            billing_type: BillingType::OneTime,
            subject: "テンプレートの件名テスト".to_string(),
            subject_template: "{D} {S}".to_string(),
            created_at: now.naive_utc(),
            updated_at: now.naive_utc(),
        };

        assert_eq!(
            supplier2.subject_in_this_month(now),
            "2021年8月分 テンプレートの件名テスト"
        );
    }
}
