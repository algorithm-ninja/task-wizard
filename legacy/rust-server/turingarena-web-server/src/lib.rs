#![feature(proc_macro_hygiene, decl_macro)]

use rocket::fairing::AdHoc;
use rocket::http::hyper::header::AccessControlAllowOrigin;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::response::content;
use rocket::response::Response;
use rocket::State;
use std::path::{Path, PathBuf};

#[cfg(feature = "embed")]
use turingarena_web_client::WebContent;

use rocket::http::ContentType;
use std::error::Error;
use std::ffi::OsStr;
use std::io::Cursor;
use structopt::StructOpt;
use turingarena_core::api::root::ContestArgs;
use turingarena_core::api::{auth, root::ApiConfig};

/// Args for the server
#[derive(StructOpt, Debug)]
#[structopt(
    name = "turingarena-serve",
    about = "CLI to start a turingarena server"
)]

pub struct ServerArgs {
    #[structopt(flatten)]
    contest: ContestArgs,

    /// host to bind the server to
    #[structopt(short = "H", long, env = "HOST", default_value = "localhost")]
    host: String,

    /// port for the server to listen
    #[structopt(short, long, env = "PORT", default_value = "8080")]
    port: u16,

    /// secret key for the webserver
    #[structopt(long, short, env = "SECRET")]
    secret_key: Option<String>,

    /// skip authentication (DANGEROUS: only for debug!)
    #[structopt(long, env = "SKIP_AUTH")]
    skip_auth: bool,

    /// Skip authentication on endpoint `/dmz/graphql` (DANGEROUS: only use if under a proxy!)
    #[structopt(long)]
    enable_dmz: bool,
}

struct Authorization(Option<String>);

impl<'a, 'r> FromRequest<'a, 'r> for Authorization {
    type Error = String;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        request::Outcome::Success(Authorization(
            match request.headers().get_one("Authorization") {
                Some(token) => Some(token.to_owned()),
                None => None,
            },
        ))
    }
}

#[rocket::get("/graphiql")]
fn graphiql() -> content::Html<String> {
    juniper_rocket::graphiql_source("/graphql")
}

#[rocket::options("/<_path..>")]
fn options_all(_path: PathBuf) {}

#[rocket::post("/graphql", data = "<request>")]
fn post_graphql_handler(
    request: juniper_rocket::GraphQLRequest,
    auth: Authorization,
    config: State<ApiConfig>,
) -> juniper_rocket::GraphQLResponse {
    let claims = config.secret.as_ref().and_then(|secret| {
        auth.0
            .and_then(|token| match auth::validate(&token, secret) {
                Ok(claims) => Some(claims),
                Err(_) => panic!("Invalid token"),
            })
    });
    let context = config.create_context(claims);
    request.execute(&context.root_node(), &context)
}

#[rocket::post("/dmz/graphql", data = "<request>")]
fn post_graphql_dmz_handler(
    request: juniper_rocket::GraphQLRequest,
    config: State<ApiConfig>,
    args: State<ServerArgs>,
) -> juniper_rocket::GraphQLResponse {
    if args.enable_dmz {
        let context = config.clone().with_skip_auth(true).create_context(None);
        request.execute(&context.root_node(), &context)
    } else {
        juniper_rocket::GraphQLResponse::error("DMZ not enabled".into())
    }
}

#[rocket::get("/")]
fn index<'r>() -> rocket::response::Result<'r> {
    dist(PathBuf::from("index.html"))
}

#[rocket::get("/<file..>")]
fn dist<'r>(file: PathBuf) -> rocket::response::Result<'r> {
    let ext = file
        .as_path()
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("html");
    let content_type = ContentType::from_extension(ext)
        .ok_or(Status::new(400, "Could not get file content type"))?;
    Response::build()
        .header(content_type)
        .sized_body(Cursor::new(get_content(&file)))
        .ok()
}

#[cfg(feature = "embed")]
fn get_content(file: &Path) -> Vec<u8> {
    let filename = file.display().to_string();
    Vec::from(
        WebContent::get(&filename)
            .or(WebContent::get("index.html"))
            .unwrap(),
    )
}

#[cfg(not(feature = "embed"))]
fn get_content(file: &Path) -> Vec<u8> {
    let base = Path::new("/usr/share/turingarena/web");
    std::fs::read(base.join(file))
        .or(std::fs::read(base.join("index.html")))
        .unwrap()
}

/// Run the server
pub fn run_server(args: ServerArgs) -> Result<(), Box<dyn Error>> {
    if args.skip_auth {
        eprintln!("WARNING: authentication disabled");
    } else if args.secret_key == None {
        eprintln!("ERROR: provide a secret OR set skip-auth");
        return Err("Secret not provided".to_owned().into());
    }

    let api_config = ApiConfig::default()
        .with_args(args.contest.clone())
        .with_skip_auth(args.skip_auth)
        .with_secret(args.secret_key.as_ref().map(|s| s.as_bytes().to_owned()));

    let config = rocket::Config::build(rocket::config::Environment::active()?)
        .port(args.port)
        .address(&args.host)
        .finalize()?;

    rocket::custom(config)
        .manage(api_config)
        .manage(args)
        .attach(AdHoc::on_response("Cors header", |_, res| {
            res.set_header(AccessControlAllowOrigin::Any);
            res.set_raw_header("Access-Control-Allow-Methods", "OPTIONS, POST");
            res.set_raw_header(
                "Access-Control-Allow-Headers",
                "Content-Type, Authorization",
            );
        }))
        .mount(
            "/",
            rocket::routes![
                graphiql,
                options_all,
                post_graphql_handler,
                post_graphql_dmz_handler,
                index,
                dist
            ],
        )
        .launch();
    Ok(())
}