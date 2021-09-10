use crate::domain;
use crate::graphql::*;
use juniper_from_schema::{QueryTrail, Walked};

#[derive(Debug, Clone)]
pub struct Invoice {
    pub invoice: domain::invoice::Invoice,
}
#[async_trait]
impl InvoiceFields for Invoice {
    fn field_id(&self, _: &Executor<Context>) -> FieldResult<ID> {
        Ok(Into::into(self.invoice.id.clone()))
    }

    fn field_issue_ymd(&self, _: &Executor<Context>) -> FieldResult<String> {
        Ok(self.invoice.issue_ymd.to_string().clone())
    }

    fn field_payment_due_on_ymd(&self, _: &Executor<Context>) -> FieldResult<String> {
        Ok(self.invoice.payment_due_on_ymd.to_string().clone())
    }

    fn field_invoice_number(&self, _: &Executor<Context>) -> FieldResult<String> {
        Ok(self.invoice.invoice_number.clone())
    }

    fn field_payment_status(&self, _: &Executor<Context>) -> FieldResult<GraphQLPaymentStatus> {
        Ok(match self.invoice.payment_status {
            domain::invoice::PaymentStatus::UnPaid => GraphQLPaymentStatus::UnPaid,
            domain::invoice::PaymentStatus::Paid => GraphQLPaymentStatus::Paid,
        })
    }

    fn field_invoice_status(&self, _: &Executor<Context>) -> FieldResult<GraphQLInvoiceStatus> {
        Ok(match self.invoice.invoice_status {
            domain::invoice::InvoiceStatus::UnSubmitted => GraphQLInvoiceStatus::UnSubmitted,
            domain::invoice::InvoiceStatus::Submitted => GraphQLInvoiceStatus::Submitted,
        })
    }

    fn field_recipient_name(&self, _: &Executor<Context>) -> FieldResult<String> {
        Ok(self.invoice.recipient_name.clone())
    }

    fn field_subject(&self, _: &Executor<Context>) -> FieldResult<String> {
        Ok(self.invoice.subject.clone())
    }

    fn field_total_amount(&self, _: &Executor<Context>) -> FieldResult<i32> {
        Ok(self.invoice.total_amount.clone())
    }

    fn field_tax(&self, _: &Executor<Context>) -> FieldResult<i32> {
        Ok(self.invoice.tax.clone())
    }
}

#[derive(Debug, Clone)]
pub struct InvoiceEdge(pub domain::invoice::Invoice);
#[async_trait]
impl InvoiceEdgeFields for InvoiceEdge {
    fn field_node<'s, 'r>(
        &'s self,
        _exec: &Executor<Context>,
        _: &QueryTrail<'r, Invoice, Walked>,
    ) -> FieldResult<Invoice> {
        Ok(Invoice {
            invoice: self.0.to_owned(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct InvoiceConnection {
    pub invoices: Vec<domain::invoice::Invoice>,
    pub total_count: i64,
    pub has_next: bool,
}
#[async_trait]
impl InvoiceConnectionFields for InvoiceConnection {
    fn field_edges<'s, 'r>(
        &'s self,
        _exec: &Executor<Context>,
        _: &QueryTrail<'r, InvoiceEdge, Walked>,
    ) -> FieldResult<Vec<InvoiceEdge>> {
        let edges = self
            .invoices
            .iter()
            .map(|v| InvoiceEdge(v.to_owned()))
            .collect::<Vec<_>>();
        Ok(edges)
    }

    fn field_page_info<'s, 'r>(
        &'s self,
        _exec: &Executor<Context>,
        _: &QueryTrail<'r, PageInfo, Walked>,
    ) -> FieldResult<PageInfo> {
        Ok(PageInfo {
            total_count: self.total_count.to_owned(),
            has_next: self.has_next.to_owned(),
        })
    }
}
