// error.rs

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Permission check denied")]
    PermissionCheckDenied,
    #[error("Permission balance denied")]
    PermissionBalanceDenied,
    #[error("Permission balance non-owner")]
    PermissionBalanceNonOwner,
    #[error("Extraction error: {0}")]
    ExtractionError(String),
}

pub type ExtractionError = Error;
