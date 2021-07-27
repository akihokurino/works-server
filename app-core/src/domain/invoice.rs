use crate::domain::YMD;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Invoice {
    pub id: String,
    pub supplier_id: String,
    pub issue_ymd: YMD,
    pub payment_due_on_ymd: YMD,
    pub invoice_number: String,
    pub payment_status: PaymentStatus,
    pub invoice_status: InvoiceStatus,
    pub recipient_name: String,
    pub subject: String,
    pub total_amount: i32,
    pub tax: i32,
    pub pdf_path: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl Invoice {
    pub fn update_pdf_path(&mut self, path: String) {
        self.pdf_path = Some(path);
    }

    pub fn should_update(&self, other: &Invoice) -> bool {
        self.updated_at != other.updated_at
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PaymentStatus {
    UnPaid,
    Paid,
}

impl PaymentStatus {
    pub fn int(&self) -> i32 {
        match self {
            Self::UnPaid => 0,
            Self::Paid => 1,
        }
    }
}

impl Default for PaymentStatus {
    fn default() -> Self {
        Self::UnPaid
    }
}

impl From<i32> for PaymentStatus {
    fn from(v: i32) -> PaymentStatus {
        match v {
            0 => Self::UnPaid,
            1 => Self::Paid,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum InvoiceStatus {
    UnSubmitted,
    Submitted,
}

impl InvoiceStatus {
    pub fn int(&self) -> i32 {
        match self {
            Self::UnSubmitted => 0,
            Self::Submitted => 1,
        }
    }
}

impl Default for InvoiceStatus {
    fn default() -> Self {
        Self::UnSubmitted
    }
}

impl From<i32> for InvoiceStatus {
    fn from(v: i32) -> InvoiceStatus {
        match v {
            0 => Self::UnSubmitted,
            1 => Self::Submitted,
            _ => Self::default(),
        }
    }
}
