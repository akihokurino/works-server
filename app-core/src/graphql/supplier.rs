use crate::domain;
use crate::graphql::*;
use juniper_from_schema::{QueryTrail, Walked};

#[derive(Debug, Clone)]
pub struct Supplier {
    pub supplier: domain::supplier::Supplier,
}
#[async_trait]
impl SupplierFields for Supplier {
    fn field_id(&self, _: &Executor<Context>) -> FieldResult<ID> {
        Ok(Into::into(self.supplier.id.clone()))
    }

    fn field_name(&self, _: &Executor<Context>) -> FieldResult<String> {
        Ok(self.supplier.name.clone())
    }

    fn field_billing_amount_include_tax(&self, _: &Executor<Context>) -> FieldResult<i32> {
        Ok(self.supplier.billing_amount_include_tax().clone())
    }

    fn field_billing_amount_exclude_tax(&self, _: &Executor<Context>) -> FieldResult<i32> {
        Ok(self.supplier.billing_amount.clone())
    }

    fn field_billing_type(&self, _: &Executor<Context>) -> FieldResult<GraphQLBillingType> {
        Ok(match self.supplier.billing_type {
            domain::supplier::BillingType::Monthly => GraphQLBillingType::Monthly,
            domain::supplier::BillingType::OneTime => GraphQLBillingType::OneTime,
        })
    }

    fn field_end_ym(&self, _: &Executor<Context>) -> FieldResult<Option<String>> {
        if !self.supplier.end_ym.is_empty() {
            return Ok(Some(self.supplier.end_ym.to_string()));
        }
        Ok(None)
    }

    fn field_subject(&self, _: &Executor<Context>) -> FieldResult<String> {
        Ok(self.supplier.subject.clone())
    }

    fn field_subject_template(&self, _: &Executor<Context>) -> FieldResult<String> {
        Ok(self.supplier.subject_template.clone())
    }

    async fn field_latest_invoice_list<'s, 'r, 'a>(
        &'s self,
        exec: &Executor<'r, 'a, Context>,
        _: &QueryTrail<'r, Invoice, Walked>,
    ) -> FieldResult<Vec<Invoice>> {
        let ctx = exec.context();

        let invoices: Vec<domain::invoice::Invoice> = ctx
            .invoice_loader_by_supplier
            .load(self.supplier.id.clone())
            .await?;

        Ok(invoices
            .iter()
            .map(|v| Invoice {
                invoice: v.to_owned(),
            })
            .collect())
    }
}
