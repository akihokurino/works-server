use crate::ddb::Dao;
use crate::graphql::*;
use crate::{domain, FieldErrorWithCode};
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

    async fn field_supplier_list<'s, 'r, 'a>(
        &'s self,
        exec: &Executor<'r, 'a, Context>,
        _: &QueryTrail<'r, supplier::SupplierConnection, Walked>,
    ) -> FieldResult<supplier::SupplierConnection> {
        let ctx = exec.context();
        let conn = ctx.get_mutex_connection();
        let supplier_dao: Dao<domain::supplier::Supplier> = Dao::new();

        let suppliers = supplier_dao
            .get_all_by_user_with_invoices(&conn, self.user.id.clone())
            .map_err(FieldErrorWithCode::from)?;

        Ok(supplier::SupplierConnection(suppliers))
    }

    async fn field_sender<'s, 'r, 'a>(
        &'s self,
        exec: &Executor<'r, 'a, Context>,
        _: &QueryTrail<'r, sender::Sender, Walked>,
    ) -> FieldResult<Option<sender::Sender>> {
        let ctx = exec.context();
        let conn = ctx.get_mutex_connection();
        let sender_dao: Dao<domain::sender::Sender> = Dao::new();

        let senders = sender_dao
            .get_all_by_user(&conn, self.user.id.clone())
            .map_err(FieldErrorWithCode::from)?;

        if let Some(sender) = senders.first() {
            return Ok(Some(sender::Sender {
                sender: sender.clone(),
            }));
        }
        Ok(None)
    }

    async fn field_bank<'s, 'r, 'a>(
        &'s self,
        exec: &Executor<'r, 'a, Context>,
        _: &QueryTrail<'r, bank::Bank, Walked>,
    ) -> FieldResult<Option<bank::Bank>> {
        let ctx = exec.context();
        let conn = ctx.get_mutex_connection();
        let bank_dao: Dao<domain::bank::Bank> = Dao::new();

        let banks = bank_dao
            .get_all_by_user(&conn, self.user.id.clone())
            .map_err(FieldErrorWithCode::from)?;

        if let Some(bank) = banks.first() {
            return Ok(Some(bank::Bank { bank: bank.clone() }));
        }
        Ok(None)
    }
}
