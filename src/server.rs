use std::convert::Infallible;
use bytes::Buf;
use std::{ fs, fs::File, io::{BufWriter, Write}, fs::rename,};

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};

use config::{Config, File as ConfigFile, FileFormat};
use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use std::time::{SystemTime, UNIX_EPOCH};
use regex::Regex;
use thiserror::Error as ThisError;

use async_lock::Mutex;

use lazy_static::lazy_static;
type GenericError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, GenericError>;

lazy_static! {
    static ref CONFIG: ApConfig = ApConfig::read_config();
    static ref GLOBAL_LOCK: Mutex<usize> = Mutex::new(1);
}

#[derive(Serialize, Deserialize, Debug)]
struct Payload {
    uuid: String,
    content: String,
}

struct ApConfig {
    address: String,
    storage_dir: String,
}

impl ApConfig {
    fn read_config() -> ApConfig {
        let mut c = Config::new();
        c.merge(ConfigFile::new("settings", FileFormat::Toml).required(true)).unwrap();
        let address = c.get_str("address").unwrap();
        let storage_dir = c.get_str("storage_dir").unwrap();

        ApConfig { address, storage_dir, }
    }   
}

#[derive(ThisError, Debug)]
pub enum LinksError {
    #[error("Bad uuid {0}")]
    BadUuid(String),
    #[error("Bad user name {0}")]
    BadUserName(String),
    #[error("Content not changed")]
    ContentNotChanged,
    #[error("Rename failed")]
    RenameFailed,
}

macro_rules! err {
    ($a:expr) => {
        Err(Box::new($a))
    }
}

/** Equivalent of System.currentTimeMillis */
fn get_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

fn verify_uuid(uuid: &str) -> Result<()> {
    lazy_static! {
        static ref UUID: Regex = Regex::new(r#"^[\da-f]{8}-([\da-f]{4}-){3}[\da-f]{12}$"#).unwrap();
    }
    if !UUID.is_match(uuid) { 
        println!("Bad uuid");
        return err!(LinksError::BadUuid(String::from(uuid)));
    }
    Ok(())
}

fn verify_user(user: &str) -> Result<&str> {
    lazy_static! {
        static ref USER: Regex = Regex::new(r#"^CN=[a-zA-Z_-]+$"#).unwrap();
    }
    if !USER.is_match(user) {
        return err!(LinksError::BadUserName(String::from(user)));
    }
    Ok(&user[3..])
}

#[test]
fn test_verify_user() {
    assert_eq!("name", verify_user("CN=name").unwrap());
}

async fn do_work(p: Payload, cn: &str) -> Result<String> {
    //println!("Json received: {:#?}", p);

    let _guard = GLOBAL_LOCK.lock().await;

    // validate the uuid is the right format
    verify_uuid(&p.uuid)?;
    let user = verify_user(cn)?;
    let file_name = format!("{}/{}.md", CONFIG.storage_dir, p.uuid);
    // check if the file content is changed
    let current_content = fs::read_to_string(&file_name)?;
    if current_content == p.content {
        return err!(LinksError::ContentNotChanged);
    }

    // rename existing file, if present
    let bk_file_name = format!("{}/{}_{}_{}.md", CONFIG.storage_dir, p.uuid, get_epoch_ms(), user); 
    if let Err(_) = rename(&file_name, &bk_file_name) {
        println!("Could not rename {} to {}", &file_name, &bk_file_name);
        return err!(LinksError::RenameFailed);
    }

    let file = File::create(file_name)?;
    let mut out = BufWriter::new(&file);
    write!(out, "{}", p.content)?;

    Ok(String::from("Standard response"))
}

fn http_response(code: StatusCode, text: &str) -> Result<Response<Body>>{
    Ok(Response::builder()
        .status(code)
        .header("content-type", "text/plain")
        .header("server", "hyper")
        .body(Body::from(text.to_string()))
        .unwrap()
        )
}

async fn request_handler(req: Request<Body>) -> Result<Response<Body>> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/save_links") => {
            // check if the client is authenticated
            match req.headers().get("X-SSL-Client-Verify") {
                Some(ssl_client_verify) if ssl_client_verify == "SUCCESS" => true,
                Some(ssl_client_verify) => return http_response(StatusCode::UNAUTHORIZED, ssl_client_verify.to_str().unwrap_or("")),
                _ => return http_response(StatusCode::UNAUTHORIZED, "No security header"),
            };

            // get the user
            let user = String::from(match req.headers().get("X-SSL-Client-S-DN") {
                Some(dn) => dn.to_str().unwrap_or(""),
                _ => return http_response(StatusCode::FORBIDDEN, "No user specified"),
            });

            let whole_body = hyper::body::aggregate(req).await?;
            let p: Payload = serde_json::from_reader(whole_body.reader())?;

            match do_work(p, &user).await {
                Ok(content) => http_response(StatusCode::OK, &content),
                Err(e) => http_response(StatusCode::BAD_REQUEST, &e.to_string()),
            }
        }
        _ => http_response(StatusCode::NOT_FOUND, "No mapping for this request")
    }   
}

pub async fn start_server() -> Result<()> {
    pretty_env_logger::init();

    // For every connection, we must make a `Service` to handle all
    // incoming HTTP requests on said connection.
    let make_svc = make_service_fn(|_conn| {
        // This is the `Service` that will handle the connection.
        // `service_fn` is a helper to convert a function that
        // returns a Response into a `Service`.
        async { Ok::<_, Infallible>(service_fn(request_handler)) }
    }); 

    let socket_addr: SocketAddr = CONFIG.address.parse().expect("Unble to parse socket address");
    let server = Server::bind(&socket_addr).serve(make_svc);

    println!("Listening on http://{}", &CONFIG.address);

    server.await?;

    Ok(())
}

