use crate::domain;
use crate::graphql::*;

#[derive(Debug, Clone)]
pub struct Sender {
    pub sender: domain::sender::Sender,
}
#[async_trait]
impl SenderFields for Sender {
    fn field_id(&self, _: &Executor<Context>) -> FieldResult<ID> {
        Ok(Into::into(self.sender.id.clone()))
    }

    fn field_name(&self, _: &Executor<Context>) -> FieldResult<String> {
        Ok(self.sender.name.clone())
    }

    fn field_email(&self, _: &Executor<Context>) -> FieldResult<String> {
        Ok(self.sender.email.clone())
    }

    fn field_tel(&self, _: &Executor<Context>) -> FieldResult<String> {
        Ok(self.sender.tel.clone())
    }

    fn field_postal_code(&self, _: &Executor<Context>) -> FieldResult<String> {
        Ok(self.sender.postal_code.clone())
    }

    fn field_address(&self, _: &Executor<Context>) -> FieldResult<String> {
        Ok(self.sender.address.clone())
    }
}
