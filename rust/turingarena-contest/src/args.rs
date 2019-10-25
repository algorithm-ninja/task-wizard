use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "turingarena",
    about = "CLI to manage the turingarena contest server"
)]
pub struct Args {
    /// url of the database
    #[structopt(long, env = "DATABASE_URL", default_value = "./database.sqlite3")]
    pub database_url: String,

    /// command  
    #[structopt(subcommand)]
    pub subcommand: Command,
}

#[derive(StructOpt, Debug)]
pub enum Command {
    /// start a contest HTTP server
    Serve {
        /// host to bind the server to
        #[structopt(short = "H", long, default_value = "localhost")]
        host: String,

        /// port for the server to listen
        #[structopt(short, long, default_value = "8080")]
        port: u16,

        /// secret key for the webserver
        #[structopt(long, short, env = "SECRET")]
        secret_key: Option<String>,

        /// skip authentication (DANGEROUS: only for debug!)
        #[structopt(long, env = "SKIP_AUTH")]
        skip_auth: bool,
    },
    /// generate GraphQL schema
    GenerateSchema {},
    /// add a new user to the contest database
    AddUser {
        /// name of the user
        username: String,

        /// display name, e.g. the full name of the user
        display_name: String,

        /// password for the new user
        password: String,
    },
    /// removes a user from the contest database
    DeleteUser {
        /// name of the user to remove
        username: String,
    },
    /// add a problem to the contest database
    AddProblem {
        /// name of the problem to add
        name: String,
        /// name of the problem to add
        path: String,
    },
    /// removes a problem from the contest database
    DeleteProblem {
        /// name of the problem to remove
        name: String,
    },
    /// initializes the database
    InitDb {},
}
