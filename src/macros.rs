// macros.rs

#[macro_export]
macro_rules! mayi {
    ($context:expr, $user:expr, $action:expr) => {{
        use $crate::{AuthorizeAction, Error, Permission};

        match $action.check(&$context) {
            Permission::Deny => Err(Error::PermissionCheckDenied),
            Permission::Allow => match $action.balance(&$context).await {
                Ok(Permission::Allow) => Ok(()),
                Ok(Permission::Deny) => Err(Error::PermissionBalanceDenied),
                Err(e) => Err(e),
            },
        }
    }};
}
