use crate::graphql::*;

#[derive(Debug, Clone)]
pub struct PageInfo {
    pub total_count: i64,
    pub has_next: bool,
}
#[async_trait]
impl PageInfoFields for PageInfo {
    fn field_total_count(&self, _: &Executor<Context>) -> FieldResult<i32> {
        Ok(self.total_count.clone() as i32)
    }

    fn field_has_next_page(&self, _: &Executor<Context>) -> FieldResult<bool> {
        Ok(self.has_next.clone())
    }
}
