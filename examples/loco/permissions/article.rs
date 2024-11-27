use async_trait::async_trait;
use mayi::{AuthorizeAction, Context, Error, FromContext, Permission};

enum ArticleAuthAction {
    Read,
    Write,
    Update,
    Delete,
}

#[async_trait]
impl AuthorizeAction for ArticleAuthAction {
    fn check(&self, context: &Context) -> Permission {
        // Implement your synchronous permission check here
        Permission::Allow
    }

    async fn balance(&self, context: &Context) -> Result<Permission, Error> {
        // Implement your asynchronous permission check here
        // You can access database connections from the context
        let db =
            context
                .get_connection::<DatabaseConnection>("db")
                .ok_or(Error::ExtractionError(
                    "Database connection not found".to_string(),
                ))?;

        // Perform your database queries and checks
        Ok(Permission::Allow)
    }
}
