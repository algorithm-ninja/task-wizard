use std::default::Default;
use std::path::PathBuf;

use chrono::{DateTime, Local};
use diesel::{Connection, ConnectionResult, SqliteConnection};
use juniper::{FieldResult, FieldError};

use auth::JwtData;
use turingarena::problem::ProblemName;
use user::UserId;

use crate::{auth, contest, problem, Result, user};
use crate::args::ContestArgs;
use crate::submission;
use crate::contest::{ContestView, UserToken};

embed_migrations!();

pub struct MutationOk;

#[juniper::object]
impl MutationOk {
    fn ok() -> bool {
        true
    }
}

pub type Schema = juniper::RootNode<'static, ApiContext, ApiContext>;

/// API entry point.
/// The same struct is used as context, query and mutation type.
#[derive(Debug, Clone)]
pub struct ApiContext {
    /// Skip all authentication
    skip_auth: bool,

    /// Secret code to use for authenticating a JWT token.
    pub secret: Option<Vec<u8>>,

    /// JWT data of the token submitted to the server (if any)
    jwt_data: Option<JwtData>,

    /// Path of the database on the filesystem
    pub database_url: PathBuf,

    /// Path of the problems directory on the filesystem
    pub problems_dir: PathBuf,
}

impl Default for ApiContext {
    fn default() -> ApiContext {
        ApiContext {
            skip_auth: false,
            secret: None,
            jwt_data: None,
            database_url: PathBuf::default(),
            problems_dir: PathBuf::default(),
        }
    }
}

impl ApiContext {
    pub fn root_node(&self) -> Schema {
        Schema::new(self.clone(), self.clone())
    }

    pub fn with_args(self, args: ContestArgs) -> ApiContext {
        self.with_database_url(args.database_url).with_problems_dir(args.problems_dir)
    }

    /// Set the database URL
    pub fn with_database_url(self, database_url: PathBuf) -> ApiContext {
        ApiContext {
            database_url,
            ..self
        }
    }

    /// Set the problems directory
    pub fn with_problems_dir(self, problems_dir: PathBuf) -> ApiContext {
        ApiContext {
            problems_dir,
            ..self
        }
    }

    /// Sets a JWT data
    pub fn with_jwt_data(self, jwt_data: Option<JwtData>) -> ApiContext {
        ApiContext { jwt_data, ..self }
    }

    /// Sets a secret
    pub fn with_secret(self, secret: Option<Vec<u8>>) -> ApiContext {
        ApiContext { secret, ..self }
    }

    /// Sets if to skip authentication
    pub fn with_skip_auth(self, skip_auth: bool) -> ApiContext {
        ApiContext { skip_auth, ..self }
    }

    /// Authorize admin operations
    pub fn authorize_admin(&self) -> juniper::FieldResult<()> {
        if self.skip_auth {
            return Ok(());
        }
        return Err(juniper::FieldError::from("Forbidden"));
    }

    /// Authenticate user
    pub fn authorize_user(&self, user_id: &Option<UserId>) -> juniper::FieldResult<()> {
        if self.skip_auth {
            return Ok(());
        }

        if let Some(id) = user_id {
            if self.secret != None {
                if let Some(data) = &self.jwt_data {
                    if data.user != id.0 {
                        return Err(juniper::FieldError::from("Forbidden for the given user id"));
                    }
                } else {
                    return Err(juniper::FieldError::from("Authentication required"));
                }
            }
        }
        Ok(())
    }

    /// Open a connection to the database
    pub fn connect_db(&self) -> ConnectionResult<SqliteConnection> {
        let conn = SqliteConnection::establish(self.database_url.to_str().unwrap())?;
        conn.execute("PRAGMA busy_timeout = 5000;")
            .expect("Unable to set `busy_timeout`");
        Ok(conn)
    }

    // TODO: move the following methods in a more appropriate location

    /// Initialize the database
    pub fn init_db(&self) -> Result<()> {
        embedded_migrations::run_with_output(&self.connect_db()?, &mut std::io::stdout())?;
        contest::create_config(&self.connect_db()?)?;
        Ok(())
    }

    /// Add a user to the current contest
    pub fn add_user(&self, id: &str, display_name: &str, token: &str) -> Result<()> {
        user::insert(
            &self.connect_db()?,
            UserId(id.to_owned()),
            display_name,
            token,
        )?;
        Ok(())
    }

    /// Delete a user from the current contest
    pub fn delete_user(&self, id: &str) -> Result<()> {
        user::delete(&self.connect_db()?, UserId(id.to_owned()))?;
        Ok(())
    }

    /// Add a problem to the current contest
    pub fn add_problem(&self, name: &str) -> Result<()> {
        problem::insert(&self.connect_db()?, ProblemName(name.to_owned()))?;
        Ok(())
    }

    /// Delete a problem from the current contest
    pub fn delete_problem(&self, name: &str) -> Result<()> {
        problem::delete(&self.connect_db()?, ProblemName(name.to_owned()))?;
        Ok(())
    }

    /// Set the start time of the current contest
    pub fn set_start_time(&self, time: DateTime<Local>) -> Result<()> {
        contest::set_start_time(&self.connect_db()?, time)?;
        Ok(())
    }

    /// Set the end time of the current contest
    pub fn set_end_time(&self, time: DateTime<Local>) -> Result<()> {
        contest::set_end_time(&self.connect_db()?, time)?;
        Ok(())
    }
}

#[juniper::object(Context = ApiContext)]
impl ApiContext {
    /// Reset database
    fn init_db(&self, ctx: &ApiContext) -> FieldResult<MutationOk> {
        ctx.authorize_admin()?;
        ctx.init_db()?;
        Ok(MutationOk)
    }

    /// Get the view of a contest
    fn contest_view(&self, ctx: &ApiContext, user_id: Option<UserId>) -> FieldResult<ContestView> {
        ctx.authorize_user(&user_id)?;
        Ok(ContestView { user_id })
    }

    /// Get the submission with the specified id
    fn submission(
        &self,
        ctx: &ApiContext,
        submission_id: String,
    ) -> FieldResult<submission::Submission> {
        // TODO: check privilage
        Ok(submission::query(&ctx.connect_db()?, &submission_id)?)
    }

    /// Authenticate a user, generating a JWT authentication token
    fn auth(&self, ctx: &ApiContext, token: String) -> FieldResult<Option<UserToken>> {
        Ok(auth::auth(
            &ctx.connect_db()?,
            &token,
            ctx.secret
                .as_ref()
                .ok_or_else(|| FieldError::from("Authentication disabled"))?,
        )?)
    }

    /// Current time on the server as RFC3339 date
    fn server_time(&self) -> String {
        chrono::Local::now().to_rfc3339()
    }
}

impl juniper::Context for ApiContext {}