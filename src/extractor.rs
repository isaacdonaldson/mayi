// extractor.rs

use crate::error::ExtractionError;
use crate::Context;

pub trait FromContext: Sized {
    fn from_context(context: &Context) -> Result<Self, ExtractionError>;
}
