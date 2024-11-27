use async_trait::async_trait;
use mayi::{AuthorizeAction, Context, Error, FromContext, Permission};

struct User;

impl FromContext for User {
    fn from_context(context: &Context) -> Result<Self, ExtractionError> {
        context.get::<User>("user").cloned().ok_or_else(|| {
            ExtractionError::ExtractionError("User not found in context".to_string())
        })
    }
}
