// This type would obviously be more complex in a real application, maybe an sqlx connection
pub type DBConnection = String;

#[derive(Debug, Copy, Clone)]
pub enum User {
    Patient,
    PlaceUser,
    Admin,
}

#[derive(Debug, Copy, Clone)]
pub enum AuthError {
    Permission,
}

#[derive(Debug, Copy, Clone)]
pub enum Auth {
    Allow,
    Deny,
}

#[derive(Debug, Copy, Clone)]
pub enum AuthAction<'a> {
    // Could have more, just a few for now
    // Could also add column_name to more of them
    Delete {
        table_name: &'a str,
    },
    Insert {
        table_name: &'a str,
    },
    Update {
        table_name: &'a str,
        column_name: &'a str,
    },
    Read {
        table_name: &'a str,
    },
}

#[derive(Debug, Clone)]
pub struct SqlStatement<'a> {
    pub sql: String,          // Just an easy way to represent SQL for a demo
    pub params: Vec<&'a str>, // For the SQL string interpolation
    pub action: AuthAction<'a>,
}

// This is our shim over the actual database connection, ideally Clone would use an internal Arc to a connection pool
#[derive(Debug, Clone)]
pub struct Connection {
    pub conn: DBConnection,
    authorizer: fn(User, AuthAction) -> Auth,
}

impl Drop for Connection {
    fn drop(&mut self) {
        // This is where we can close the connection and do any cleanup using the actual db connection
    }
}

impl Connection {
    // This is the function where a users permissions would be checked
    // ideally it would be better to split this fn up to be in a different file
    fn authorizer(user: User, action: AuthAction) -> Auth {
        match (user, action) {
            (User::Admin, _) => Auth::Allow, // Admins can do anything
            (User::PlaceUser, action) => {
                match action {
                    AuthAction::Delete {
                        table_name: "forms" | "resources",
                    }
                    | AuthAction::Insert {
                        table_name: "forms" | "checkins" | "action_plans",
                    }
                    | AuthAction::Update {
                        // Can have multiple update definitions
                        table_name: "forms",
                        column_name: "due_at", // Just a few for now
                    }
                    | AuthAction::Update {
                        table_name: "action_plans",
                        column_name: "name" | "owner" | "status", // Just a few for now
                    }
                    | AuthAction::Read {
                        table_name: "forms" | "checkins" | "users" | "places" | "action_plans",
                    } => Auth::Allow,
                    // Deny everything else
                    _ => Auth::Deny,
                }
            }
            (User::Patient, action) => {
                match action {
                    AuthAction::Insert {
                        table_name: "form_response" | "checkins",
                    }
                    | AuthAction::Update {
                        table_name: "checkins",
                        column_name: "status",
                    }
                    | AuthAction::Read {
                        table_name: "form_response" | "forms" | "checkins" | "action_plans",
                    } => Auth::Allow,
                    // Deny everything else
                    _ => Auth::Deny,
                }
            }
        }
    }

    fn connect(self) -> Result<Self, ()> {
        // This is where we would actually connect to the database with proper authentication
        Ok(self)
    }

    pub fn trusted(conn: String) -> Result<Self, ()> {
        // Our auth fn here just allows everything
        let conn = Connection {
            conn,
            authorizer: |_, _| Auth::Allow,
        };
        conn.connect()
    }

    pub fn untrusted(conn: String) -> Result<Self, ()> {
        let conn = Connection {
            conn,
            authorizer: Connection::authorizer,
        };
        conn.connect()
    }

    pub fn exec(self, current_user: User, statement: SqlStatement) -> Result<(), AuthError> {
        // The trick here would be to generate the AuthAction from the SQL Statement of the library
        // But having to pass the desired AuthAction in to the function is not too bad either
        match (self.authorizer)(current_user, statement.action) {
            Auth::Allow => {
                // This is where we would actually execute the query on the database connection we hold
                Ok(())
            }
            Auth::Deny => Err(AuthError::Permission),
        }
    }
}

fn main() {
    // truseted connections could happen in cron jobs, cli tools, etc
    let trusted_conn = Connection::trusted("localhost".to_string()).unwrap();

    // untrested connections could happen in the web server, or anywhere users interact
    let untrusted_conn = Connection::untrusted("localhost".to_string()).unwrap();

    // imagine we are in a route handler to read places
    let sql_read_statment = SqlStatement {
        sql: "SELECT * FROM places WHERE created_at < $1::timestamptz".to_string(),
        params: vec!["2024-04-17T12:00:00.000000-04:00"], // SQL params
        // This is the action we want to check for
        action: AuthAction::Read {
            table_name: "places",
        },
    };

    /////// UNTRUSTED CONNECTION ///////
    // This should be successful because PlaceUsers can read places
    match untrusted_conn
        .clone()
        .exec(User::PlaceUser, sql_read_statment.clone())
    {
        Ok(_) => println!("Success"),
        Err(e) => println!("Error: {:?}", e),
    }

    // This should fail because Patients cannot read places
    match untrusted_conn
        .clone()
        .exec(User::Patient, sql_read_statment.clone())
    {
        Ok(_) => println!("Success"),
        Err(e) => println!("Error: {:?}", e),
    }

    /////// TRUSTED CONNECTION ///////
    // Since this is a trusted connection, everything should be allowed
    match trusted_conn
        .clone()
        .exec(User::Patient, sql_read_statment.clone())
    {
        Ok(_) => println!("Success"),
        Err(e) => println!("Error: {:?}", e),
    }

    // Now imagine we are in a route handler to create an action plan
    let sql_insert_statment = SqlStatement {
        sql: "INSERT INTO action_plans (name, owner, status) VALUES ($1::text, $2::text, $3::text)"
            .to_string(),
        params: vec!["Help Get Better", "Brandon Merrel", "draft"],
        // Action to check against
        action: AuthAction::Insert {
            table_name: "action_plans",
        },
    };

    /////// UNTRUSTED CONNECTION ///////
    // This should be successful because PlaceUsers can create action plans
    match untrusted_conn
        .clone()
        .exec(User::PlaceUser, sql_insert_statment.clone())
    {
        Ok(_) => println!("Success"),
        Err(e) => println!("Error: {:?}", e),
    }

    // This should fail because Patient cannot create action plans
    match untrusted_conn
        .clone()
        .exec(User::Patient, sql_insert_statment.clone())
    {
        Ok(_) => println!("Success"),
        Err(e) => println!("Error: {:?}", e),
    }

    /////// TRUSTED CONNECTION ///////
    // Since this is a trusted connection, everything should be allowed
    match trusted_conn
        .clone()
        .exec(User::Patient, sql_insert_statment.clone())
    {
        Ok(_) => println!("Success"),
        Err(e) => println!("Error: {:?}", e),
    }
}
