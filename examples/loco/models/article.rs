use async_trait::async_trait;
use mayi::{AuthorizeAction, Context, Error, FromContext, Permission};

struct Article;

impl FromContext for Article {
    fn from_context(context: &Context) -> Result<Self, ExtractionError> {
        context.get::<Article>("article").cloned().ok_or_else(|| {
            ExtractionError::ExtractionError("Article not found in context".to_string())
        })
    }
}
