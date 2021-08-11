use crate::domain;
use crate::graphql::*;
use juniper_from_schema::{QueryTrail, Walked};

#[derive(Debug, Clone)]
pub struct Me {
    pub user: domain::user::User,
}
#[async_trait]
impl MeFields for Me {
    fn field_id(&self, _: &Executor<Context>) -> FieldResult<ID> {
        Ok(Into::into(self.user.id.clone()))
    }

    async fn field_suppliers<'s, 'r, 'a>(
        &'s self,
        exec: &Executor<'r, 'a, Context>,
        _: &QueryTrail<'r, supplier::SupplierConnection, Walked>,
    ) -> FieldResult<supplier::SupplierConnection> {
        let ctx = exec.context();
        let conn = ddb::establish_connection();
        let supplier_dao = ctx.ddb_dao::<domain::supplier::Supplier>();

        let suppliers = supplier_dao
            .get_all_by_user_with_invoices(&conn, self.user.id.clone())
            .map_err(FieldError::from)?;

        Ok(supplier::SupplierConnection(suppliers))
    }
}
