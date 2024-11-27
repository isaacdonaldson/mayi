pub mod context;
pub mod error;
pub mod extractor;
pub mod macros;
pub mod permission;
pub mod traits;

pub use context::Context;
pub use error::{Error, ExtractionError};
pub use extractor::FromContext;
pub use permission::Permission;
pub use traits::AuthorizeAction;

// Re-export the mayi macro
pub use macros::mayi;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    Allow,
    Deny,
}

use crate::{Context, Error, Permission};
use async_trait::async_trait;

#[async_trait]
pub trait AuthorizeAction: Sized {
    fn check(&self, context: &Context) -> Permission;
    async fn balance(&self, context: &Context) -> Result<Permission, Error>;
}
