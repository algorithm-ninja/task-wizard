use std::default::Default;
use std::env::temp_dir;
use std::path::{Path, PathBuf};

use diesel::{Connection, RunQueryDsl, SqliteConnection};
use juniper::FieldResult;
use structopt::StructOpt;

use auth::JwtData;
use contest::{ContestView, UserToken};
use formats::ImportFileInput;
use problem::ProblemName;
use schema::blobs;
use user::UserId;
use user::UserInput;

use crate::api::contest::{Contest, ContestUpdateInput};
use crate::api::contest_evaluation::Evaluation;
use crate::api::contest_problem::{Problem, ProblemInput, ProblemUpdateInput};
use crate::api::formats::Import;
use crate::api::user::{User, UserUpdateInput};

use super::*;

embed_migrations!();

pub struct MutationOk;

#[juniper::object]
impl MutationOk {
    fn ok() -> bool {
        true
    }
}

pub type RootNode<'a> = juniper::RootNode<'static, Query<'a>, Mutation<'a>>;

#[derive(StructOpt, Debug)]
pub struct ContestArgs {
    /// url of the database
    #[structopt(long, env = "DATABASE", default_value = "./database.sqlite3")]
    pub database_url: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// Skip all authentication
    skip_auth: bool,

    /// Secret code to use for authenticating a JWT token.
    pub secret: Option<Vec<u8>>,

    /// Path of the database on the filesystem
    pub database_url: PathBuf,
}

pub struct ApiContext {
    pub config: ApiConfig,
    /// JWT data of the token submitted to the server (if any)
    pub jwt_data: Option<JwtData>,
    pub database: SqliteConnection,
    pub workspace_path: PathBuf,
}

impl juniper::Context for ApiContext {}

impl Default for ApiConfig {
    fn default() -> ApiConfig {
        ApiConfig {
            skip_auth: false,
            secret: None,
            database_url: PathBuf::default(),
        }
    }
}

impl ApiConfig {
    pub fn create_context(&self, jwt_data: Option<JwtData>) -> ApiContext {
        // FIXME: should not create directory here
        let workspace_path = temp_dir().join("turingarena");
        std::fs::create_dir_all(&workspace_path).expect("Unable to create workspace dir");

        ApiContext {
            config: self.clone(),
            database: self.connect_db(),
            jwt_data,
            workspace_path,
        }
    }

    fn connect_db(&self) -> SqliteConnection {
        let conn = SqliteConnection::establish(self.database_url.to_str().unwrap())
            .expect("Unable to establish connection");
        conn.execute("PRAGMA busy_timeout = 5000;")
            .expect("Unable to set `busy_timeout`");
        conn
    }

    pub fn with_args(self, args: ContestArgs) -> ApiConfig {
        self.with_database_url(args.database_url)
    }

    /// Set the database URL
    pub fn with_database_url(self, database_url: PathBuf) -> ApiConfig {
        ApiConfig {
            database_url,
            ..self
        }
    }

    /// Sets a secret
    pub fn with_secret(self, secret: Option<Vec<u8>>) -> ApiConfig {
        ApiConfig { secret, ..self }
    }

    /// Sets if to skip authentication
    pub fn with_skip_auth(self, skip_auth: bool) -> ApiConfig {
        ApiConfig { skip_auth, ..self }
    }
}

impl ApiContext {
    pub fn root_node(&self) -> RootNode {
        RootNode::new(Query { context: &self }, Mutation { context: &self })
    }

    pub fn workspace_path(&self) -> &Path {
        &self.workspace_path
    }

    pub fn create_blob(&self, content: &[u8]) -> FieldResult<String> {
        use diesel::ExpressionMethods;
        let integrity = archive::compute_integrity(content);
        diesel::insert_into(blobs::table)
            .values((
                blobs::dsl::integrity.eq(&integrity),
                blobs::dsl::content.eq(content),
            ))
            .execute(&self.database)?;
        Ok(integrity)
    }

    pub fn unpack_archive(&self, integrity: &str, prefix: &str) -> FieldResult<PathBuf> {
        use diesel::QueryDsl;
        let workspace_path = &self.workspace_path().to_owned();

        archive::unpack_archive(workspace_path, prefix, integrity, || {
            Ok(blobs::table
                .find(integrity)
                .select(blobs::content)
                .first::<Vec<u8>>(&self.database)?)
        })
    }

    pub fn default_contest(&self) -> FieldResult<Contest> {
        Contest::new(&self)
    }

    /// Authorize admin operations
    #[must_use = "Error means forbidden"]
    pub fn authorize_admin(&self) -> juniper::FieldResult<()> {
        if self.config.skip_auth {
            return Ok(());
        }
        return Err(juniper::FieldError::from("Forbidden"));
    }

    /// Authenticate user
    #[must_use = "Error means forbidden"]
    pub fn authorize_user(&self, user_id: &Option<UserId>) -> juniper::FieldResult<()> {
        if self.config.skip_auth {
            return Ok(());
        }

        if let Some(id) = user_id {
            if self.config.secret != None {
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
}

pub struct Query<'a> {
    context: &'a ApiContext,
}

#[juniper_ext::graphql(Context = ApiContext)]
impl Query<'_> {
    fn contest(&self) -> FieldResult<Contest> {
        Contest::new(&self.context)
    }

    fn user(&self, user_id: Option<UserId>) -> FieldResult<Option<User>> {
        Ok(if let Some(user_id) = user_id {
            Some(User::by_id(&self.context, user_id.clone())?)
        } else {
            None
        })
    }

    /// Get the submission with the specified id
    fn submission(&self, submission_id: String) -> FieldResult<contest_submission::Submission> {
        let submission = contest_submission::Submission::by_id(&self.context, &submission_id)?;
        self.context.authorize_user(&Some(submission.user_id()))?;
        Ok(submission)
    }

    /// Current time on the server as RFC3339 date
    fn server_time(&self) -> String {
        chrono::Local::now().to_rfc3339()
    }
}

pub struct Mutation<'a> {
    context: &'a ApiContext,
}

#[juniper_ext::graphql(Context = ApiContext)]
impl Mutation<'_> {
    /// Reset database
    fn init_db(&self) -> FieldResult<MutationOk> {
        self.context.authorize_admin()?;

        embedded_migrations::run_with_output(&self.context.database, &mut std::io::stdout())?;

        Contest::init(&self.context)?;

        Ok(MutationOk)
    }

    /// Authenticate a user, generating a JWT authentication token
    fn auth(&self, token: String) -> FieldResult<Option<UserToken>> {
        Ok(auth::auth(&self.context, &token)?)
    }

    /// Current time on the server as RFC3339 date
    fn server_time() -> String {
        chrono::Local::now().to_rfc3339()
    }

    /// Add a user to the current contest
    pub fn update_contest(&self, input: ContestUpdateInput) -> FieldResult<MutationOk> {
        self.context.authorize_admin()?;
        self.context.default_contest()?.update(input)?;

        Ok(MutationOk)
    }

    /// Add a user to the current contest
    pub fn add_users(&self, inputs: Vec<UserInput>) -> FieldResult<MutationOk> {
        self.context.authorize_admin()?;
        User::insert(&self.context, inputs)?;
        Ok(MutationOk)
    }

    pub fn update_users(&self, inputs: Vec<UserUpdateInput>) -> FieldResult<MutationOk> {
        self.context.authorize_admin()?;
        User::update(&self.context, inputs)?;
        Ok(MutationOk)
    }

    /// Delete a user from the current contest
    pub fn delete_users(&self, ids: Vec<String>) -> FieldResult<MutationOk> {
        self.context.authorize_admin()?;
        User::delete(&self.context, ids)?;
        Ok(MutationOk)
    }

    /// Add problems to the current contest
    pub fn add_problems(&self, inputs: Vec<ProblemInput>) -> FieldResult<MutationOk> {
        self.context.authorize_admin()?;
        Problem::insert(&self.context, inputs)?;
        Ok(MutationOk)
    }

    /// Update problems in the current contest
    pub fn update_problems(&self, inputs: Vec<ProblemUpdateInput>) -> FieldResult<MutationOk> {
        self.context.authorize_admin()?;
        Problem::update(&self.context, inputs)?;
        Ok(MutationOk)
    }

    /// Delete problems from the current contest
    pub fn delete_problems(&self, names: Vec<String>) -> FieldResult<MutationOk> {
        self.context.authorize_admin()?;
        Problem::delete(&self.context, names)?;
        Ok(MutationOk)
    }

    /// Import a file
    pub fn import(
        &self,
        inputs: Vec<ImportFileInput>,
        dry_run: Option<bool>,
    ) -> FieldResult<Import> {
        self.context.authorize_admin()?;
        let import = Import::load(inputs)?;
        if dry_run.is_some() && dry_run.unwrap() {
            // no-op
        } else {
            import.apply(&self.context)?;
        }
        Ok(import)
    }

    /// Submit a solution to the problem
    fn submit(
        &self,
        user_id: UserId,
        problem_name: ProblemName,
        files: Vec<contest_submission::FileInput>,
    ) -> FieldResult<contest_submission::Submission> {
        let submission = contest_submission::Submission::insert(
            &self.context,
            &user_id.0,
            &problem_name.0,
            files,
        )?;
        Evaluation::start_new(&submission, &self.context.config)?;
        Ok(submission)
    }

    fn evaluate(&self, submission_ids: Vec<String>) -> FieldResult<MutationOk> {
        self.context.authorize_admin()?;
        for id in submission_ids {
            let submission = contest_submission::Submission::by_id(&self.context, &id)?;
            Evaluation::start_new(&submission, &self.context.config)?;
        }
        Ok(MutationOk)
    }
}
