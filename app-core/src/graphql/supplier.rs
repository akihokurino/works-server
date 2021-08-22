use crate::domain;
use crate::graphql::*;
use juniper_from_schema::{QueryTrail, Walked};

#[derive(Debug, Clone)]
pub struct Supplier {
    pub supplier: domain::supplier::Supplier,
    pub invoices: Vec<domain::invoice::Invoice>,
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

    fn field_subject(&self, _: &Executor<Context>) -> FieldResult<String> {
        Ok(self.supplier.subject.clone())
    }

    fn field_subject_template(&self, _: &Executor<Context>) -> FieldResult<String> {
        Ok(self.supplier.subject_template.clone())
    }

    fn field_invoice_list<'s, 'r>(
        &'s self,
        _: &Executor<Context>,
        _: &QueryTrail<'r, InvoiceConnection, Walked>,
    ) -> FieldResult<InvoiceConnection> {
        Ok(InvoiceConnection(self.invoices.clone()))
    }
}

#[derive(Debug, Clone)]
pub struct SupplierEdge(pub domain::supplier::SupplierWithInvoices);
#[async_trait]
impl SupplierEdgeFields for SupplierEdge {
    async fn field_node<'s, 'r, 'a>(
        &'s self,
        _exec: &Executor<'r, 'a, Context>,
        _: &QueryTrail<'r, Supplier, Walked>,
    ) -> FieldResult<Supplier> {
        Ok(Supplier {
            supplier: self.0.supplier.clone(),
            invoices: self.0.invoices.clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct SupplierConnection(pub Vec<domain::supplier::SupplierWithInvoices>);
#[async_trait]
impl SupplierConnectionFields for SupplierConnection {
    async fn field_edges<'s, 'r, 'a>(
        &'s self,
        _exec: &Executor<'r, 'a, Context>,
        _: &QueryTrail<'r, SupplierEdge, Walked>,
    ) -> FieldResult<Vec<SupplierEdge>> {
        let edges = self
            .0
            .clone()
            .into_iter()
            .map(|v| SupplierEdge(v.clone()))
            .collect::<Vec<_>>();
        Ok(edges)
    }
}
