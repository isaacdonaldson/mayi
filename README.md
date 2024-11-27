# mayi

Authorization library for Rust. Uses two approches to authorizing actions:

1. Authorizes wider actions (e.g. "read", "write") based on rules defined in the source code with Rust `Trait`s.
2. Authorizes more specific actions (e.g. "read:article", "write:article") based on async/remote checks to a database.

## Idea

```rust
// main.rs
let mayi_context = Mayi::Context::new()
    .add_connection(&mongo_conn);
    .add_connection(&postgres_conn);

```

```rust
// controllers/article.rs
fn article_create_router(
    State(state): State<AppContext>,
    User(user): User,
    Mayi::Context(context): Mayi::Context<AppContext>, // might not work?
    Json(params): Json<ArticleParams>,
) -> impl IntoResponse {


    // If extractor works
    mayi!(context, user, Auth::ReadArticle)
    // otherwise...
    mayi!(Mayi::context::new(state), user, Auth::ReadArticle)

}
```

Which the `mayi!` macro will roughly expand to:

```rust
match Auth::ReadArticle.check(&user, &params.article) {
    Err(_) => {
        // Have this error carry more information about the permission check
        return Err(MayiError::PermissionCheckDenied)
    },
    Ok(_) => {
        match Auth::ReadArticle.balance(&user, &params.article).await? {
            Ok(_) => {
                // Continue on, no issues with permissions
            },
            Err(e) => {
                return match e {
                    // Have these errors cover all the possible errors and return new ones
                    // and have the `MayiError::new(e)` method return an actual error with more information
                    MayiError::PermissionBalanceDenied => MayiError::new(e),
                    MayiError::PermissionBalanceNonOwner => MayiError::new(e),
                    //...
                }
            }
        }
    }
}
```

```rust
// auth/article.rs

#[derive(Mayi::Authorize)]
pub enum ArticleAuthAction {
    Read(
        resource_identifier: 'static &str, // table_name
        // columns you can read, can be filtered out
        permitted_identifiers: Vec<&'static str>, // table_name.[title, data, ...]
        owner_identifier: 'static &str,    // table_name.owner
    ),

    Write(
        resource_identifier: 'static &str, // table_name
        owner_identifier: 'static &str,    // table_name.owner
    ),

    Update(
        resource_identifier: 'static &str, // table_name
        // columns you can update, can be stopped from updating
        permitted_identifiers: Vec<&'static str>, // table_name.[title, data, ...]
        owner_identifier: 'static &str,    // table_name.owner
    ),

    Delete(
        resource_identifier: 'static &str, // table_name
        owner_identifier: 'static &str,    // table_name.owner
    ),
}

//rename? trait?
impl Mayi::AuthorizeAction for ArticleAuthAction {
    // check is a synchronous, local/wide check. It is intended to be
    // a quick check that uses information that is static and does not
    // need to be fetched from a database. Things like whether a user
    // can read/write/delete/update a resource, and is not intended to
    // be used to check if a user is the owner of a resource and similar
    // checks.
    pub fn check(self, User(user): User, Article(article): Article) -> Mayi::Permission> {
        match (user, self) {
            // Admin Role
            (User::Admin, _) => Mayi::Permission::Allow,
            // Writing Roles
            (User::Editor, action) => Mayi::Permission::Allow,
            (User::Author, action) => {
                match action {
                    ArticleAuthAction::Read(_, _) => Mayi::Permission::Allow,
                    // These 3 actions should only be allowed if the use is the owner
                    // of the article. That will be checked in the balance method though
                    // as it requires a database query.
                    ArticleAuthAction::Write(_, _) => Mayi::Permission::Allow,
                    ArticleAuthAction::Update(_, _, _) => Mayi::Permission::Allow,
                    // We don't want authors to delete any articles, only Editors and Admins
                    ArticleAuthAction::Delete(_, _, _) => Mayi::Permission::Deny,
                }
            }
            // Other Employee Roles
            // These roles should only be able to read articles
            // and not write, update, or delete them
            (User::Exec, ArticleAuthAction::Read(_, _, _)) => Mayi::Permission::Allow,
            (User::Hr, ArticleAuthAction::Read(_, _, _)) => Mayi::Permission::Allow,
            (User::Accounting, ArticleAuthAction::Read(_, _, _)) => Mayi::Permission::Allow,
            (User::Engineer, ArticleAuthAction::Read(_, _, _)) => Mayi::Permission::Allow,
            (User::Designer, ArticleAuthAction::Read(_, _, _)) => Mayi::Permission::Allow,
            // External Roles
            (User::Client, ArticleAuthAction::Read(_, _, _)) => Mayi::Permission::Allow,
            (User::Auditor, ArticleAuthAction::Read(_, _, _)) => Mayi::Permission::Allow,

            // This will return false for any other combination of user and action.
            // This is nice because if we forget to add a case it will default to deny
            _ => Mayi::Permission::Deny,
        }
    }

        // Since all the wider checks are done before here, these are the series of checks
        // that will take longer to resolve, and most likely require a query to a database
        // or another resource. This is an async method to support being able to do that.
    pub async fn balance(self, User(user): User, Article(article): Article) -> Result<Mayi::Permission, Mayi::Error> {
        // I have to figure out how to get the DB connection from the context in here,
        // in addition to how to query using the above defined actions, and support multiple
        // query backends.
        // Then I have to figure out how to make it ergonomic to use, and how to make it
        // work well with the `check` search, the `mayi` macro and the rest of the system.

    }
}

```

```rust
// models/user.rs

// rename to something better?
impl Mayi::FromContext for User {
    fn from_context(context: &Mayi::Context) -> Result<Self, Mayi::ExtractionError> {
        // Get the user from the context
        context.user.clone()
    }
}
```

```rust
// models/article.rs

// rename to something better?
impl Mayi::FromContext for Article {
    fn from_context(context: &Mayi::Context) -> Result<Self, Mayi::ExtractionError> {
        // Get the user from the context
        context.article.clone()
    }
}
```
