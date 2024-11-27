I want to create a Rust library that allows people to easily specify permissions and check them in an easy to use macro.

My general idea is to have a derive macro that will do a lot of the heavy lifting, in conjunction with a trait implementation that allows the user to specify the permissions.

The implemeentation will allow for 2 checks, one based on a User's role that can be arbitrary (called the check), and another based on some stored state in a repository (called the balance). The initial check (check) should be synchronous and deal with the broad permissions, and only if that fails will the more specific asynchronous check (balance) be run.

The library needs to be able to work with many different definitons of a 'User', and should be able to be used in many different contexts.

This is a rough idea of what I am thinking of:
```rust
// in file user.rs
pub enum UserRole {
  // Internal roles
  Admin,
  Editor,
  Author,
  Exec,
  Hr,
  Accounting,
  Engineer,
  Designer,
  // External roles
  Client,
  Auditor,
}

#[derive(Mayi::Identifier, Deserialize, Serialize)]
pub struct User {
  uuid: String,
  role: UserRole,
}

impl Mayi::Identifier for User {
  pub fn identify (&self) -> (String, UserRole) {
    (self.uuid, self.role)
  }
}
```

```rust
// in file article.rs
#[derive(Mayi::Permissions, Deserialize, Serialize)]
pub struct Article {
  pub title: String,
  pub body: String,
  pub author_id: String,
}

#[derive(Mayi::AuthActions, Deserialize, Serialize)]
pub enum ArticleAuthAction {
  Create,
  Read,
  Update,
  Delete,
  Publish,
  Revoke,
}

impl Mayi::AuthActions for Article {
    pub fn action_enum() -> Vec<ArticleAuthAction> {
      mayi_enum!(ArticleAuthAction)
    }
}

impl Mayi::PermissionChecks for Article {
  async fn fetch_resource(identifier: String) -> Result<Article, Error> {
    // Fetch the article from the database
    Entity.find(identifier).await
  }

  pub fn check(action: ArticleAuthAction, user: impl Mayi::Identifier) -> bool {
    let (user_id, user_role) = user.identify();

    match(user_role, action) {
          // Admin has all permissions
          (UserRole::Admin, _) => true,
          // More specific checks
          (UserRole::Editor, action) => {
              match action {
                ArticleAuthAction::Revoke => false,
                _ => true
              }
          }
          (UserRole::Author, action) => {
              match action {
                ArticleAuthAction::Create => true,
                ArticleAuthAction::Read => true,
                ArticleAuthAction::Update => true,
                ArticleAuthAction::Delete => true,
                _ => false
              }
          }

          // External roles
          (UserRole::Client, ArticleAuthAction::Read) => true,
          (UserRole::Audit, ArticleAuthAction::Read) => true,

          // Other internal roles
          (_, ArticleAuthAction::Read) => true,

          // Everything else is not permitted
          _ => false

    }
  }

  pub async fn balance(action: ArticleAuthAction, user: impl Mayi::Identifier) -> bool {
      let (user_id, user_role) = user.identify();
      let article = fetch_resource(self.uuid).await;
  }
}
```

Then if I have a http endpoint handler, I would like to do something like the follwing:
```rust
// In controllers/article.rs
fn article_create_router(
    State(state): State<AppContext>,
    User(user): User,
    Json(params): Json<ArticleParams>,
) -> impl IntoResponse {
    // If extractor works
    mayi!(user, ArticleAuthAction::Create);
    // ...
}
```

Which the mayi! macro would expand to something like:
```rust
match Article::check(ArticleAuthAction::Create, user) {
  true => (),
  false => {
    match Article::balance(ArticleAuthAction::Create, user).await {
      true => (),
      false => {
        // TODO: Add tracing logging, and more specific errors
        return HttpResponse::Unauthorized().finish();
      }
    }
  }
}
```

