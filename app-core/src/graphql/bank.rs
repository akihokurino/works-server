use crate::domain;
use crate::graphql::*;

#[derive(Debug, Clone)]
pub struct Bank {
    pub bank: domain::bank::Bank,
}
#[async_trait]
impl BankFields for Bank {
    fn field_id(&self, _: &Executor<Context>) -> FieldResult<ID> {
        Ok(Into::into(self.bank.id.clone()))
    }

    fn field_name(&self, _: &Executor<Context>) -> FieldResult<String> {
        Ok(self.bank.name.clone())
    }

    fn field_code(&self, _: &Executor<Context>) -> FieldResult<String> {
        Ok(self.bank.code.clone())
    }

    fn field_account_type(&self, _: &Executor<Context>) -> FieldResult<GraphQLBankAccountType> {
        Ok(match self.bank.account_type {
            domain::bank::AccountType::Savings => GraphQLBankAccountType::Savings,
            domain::bank::AccountType::Checking => GraphQLBankAccountType::Checking,
        })
    }

    fn field_account_number(&self, _: &Executor<Context>) -> FieldResult<String> {
        Ok(self.bank.account_number.clone())
    }
}
