use crate::domain;
use crate::graphql::*;
use juniper_from_schema::{QueryTrail, Walked};

#[derive(Debug, Clone)]
pub struct InvoiceHistory {
    pub invoice: domain::invoice::Invoice,
    pub supplier: domain::supplier::Supplier,
}
#[async_trait]
impl InvoiceHistoryFields for InvoiceHistory {
    fn field_id(&self, _: &Executor<Context>) -> FieldResult<ID> {
        Ok(Into::into(self.invoice.id.clone()))
    }

    fn field_invoice<'s, 'r>(
        &'s self,
        _: &Executor<Context>,
        _: &QueryTrail<'r, Invoice, Walked>,
    ) -> FieldResult<Invoice> {
        Ok(Invoice {
            invoice: self.invoice.clone(),
        })
    }

    fn field_supplier<'s, 'r>(
        &'s self,
        _: &Executor<Context>,
        _: &QueryTrail<'r, Supplier, Walked>,
    ) -> FieldResult<Supplier> {
        Ok(Supplier {
            supplier: self.supplier.clone(),
            invoices: vec![],
        })
    }
}

#[derive(Debug, Clone)]
pub struct InvoiceHistoryEdge(pub domain::invoice::Invoice, pub domain::supplier::Supplier);
#[async_trait]
impl InvoiceHistoryEdgeFields for InvoiceHistoryEdge {
    fn field_node<'s, 'r>(
        &'s self,
        _exec: &Executor<Context>,
        _: &QueryTrail<'r, InvoiceHistory, Walked>,
    ) -> FieldResult<InvoiceHistory> {
        Ok(InvoiceHistory {
            invoice: self.0.clone(),
            supplier: self.1.clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct InvoiceHistoryConnection(
    pub Vec<(domain::invoice::Invoice, domain::supplier::Supplier)>,
);
#[async_trait]
impl InvoiceHistoryConnectionFields for InvoiceHistoryConnection {
    fn field_edges<'s, 'r>(
        &'s self,
        _exec: &Executor<Context>,
        _: &QueryTrail<'r, InvoiceHistoryEdge, Walked>,
    ) -> FieldResult<Vec<InvoiceHistoryEdge>> {
        let edges = self
            .0
            .clone()
            .into_iter()
            .map(|v| InvoiceHistoryEdge(v.0.clone(), v.1.clone()))
            .collect::<Vec<_>>();
        Ok(edges)
    }
}
