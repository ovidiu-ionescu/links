use bytes::Buf;
use lib_hyper_organizator::response_utils::read_full_body;
use std::{
    fs,
    fs::rename,
    fs::File,
    io::{BufWriter, Write},
};

use hyper::{Body, Method, Request, Response, StatusCode};

use config::Config;

use serde::{Deserialize, Serialize};

use regex::Regex;
use thiserror::Error as ThisError;

use async_lock::Mutex;

use crate::utils::Result;
use lazy_static::lazy_static;
use lib_hyper_organizator::response_utils::{GenericMessage, PolymorphicGenericMessage};

use crate::{
    links::register_click,
    utils::{get_epoch_ms, get_user_name},
};

lazy_static! {
    pub static ref CONFIG: ApConfig = ApConfig::read_config();
    static ref GLOBAL_LOCK: Mutex<usize> = Mutex::new(1);
}

#[derive(Serialize, Deserialize, Debug)]
struct Payload {
    uuid:    String,
    content: String,
}

#[derive(Deserialize, Debug)]
pub struct ApConfig {
    pub storage_dir:       String,
    pub click_buffer_size: usize,
}

impl ApConfig {
    fn read_config() -> ApConfig {
        let config = Config::builder()
            .add_source(config::File::with_name("settings").required(true))
            .build()
            .unwrap();
        let ap_config: ApConfig = config.get("application").unwrap();
        ap_config
    }
}

#[derive(ThisError, Debug, PartialEq)]
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
    };
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
        static ref USER: Regex = Regex::new(r#"^[a-zA-Z0-9_-]+$"#).unwrap();
    }
    if !USER.is_match(user) {
        return err!(LinksError::BadUserName(String::from(user)));
    }
    Ok(user)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_verify_user() {
        assert_eq!("name", verify_user("name").unwrap());

        let name_with_slash = "embedded/slash";
        let res = verify_user(name_with_slash).unwrap_err();
        assert_eq!(
            res.downcast_ref(),
            Some(&LinksError::BadUserName(String::from(name_with_slash)))
        );

        let name_with_spaces = "name with spaces";
        let res = verify_user(name_with_spaces).unwrap_err();
        assert_eq!(
            res.downcast_ref(),
            Some(&LinksError::BadUserName(String::from(name_with_spaces)))
        );
    }
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
    let bk_file_name = format!(
        "{}/{}_{}_{}.md",
        CONFIG.storage_dir,
        p.uuid,
        get_epoch_ms(),
        user
    );
    if rename(&file_name, &bk_file_name).is_err() {
        println!("Could not rename {} to {}", &file_name, &bk_file_name);
        return err!(LinksError::RenameFailed);
    }

    let file = File::create(file_name)?;
    let mut out = BufWriter::new(&file);
    write!(out, "{}", p.content)?;

    Ok(String::from("Standard response"))
}

async fn save_links(mut request: Request<Body>) -> Result<Response<Body>> {
    //let whole_body = hyper::body::aggregate(request).await?;
    let whole_body = read_full_body(&mut request).await?;
    let p: Payload = match serde_json::from_reader(whole_body.reader()) {
        Ok(p) => p,
        Err(e) => {
            return GenericMessage::text(
                StatusCode::BAD_REQUEST,
                format!("Error parsing json: {}", e).as_str(),
            );
        }
    };

    let user = get_user_name(&request)?;
    match do_work(p, user).await {
        Ok(content) => GenericMessage::text(StatusCode::OK, &content),
        Err(e) if e.downcast_ref() == Some(&LinksError::ContentNotChanged) => GenericMessage::text(
            StatusCode::from_u16(254).unwrap(),
            "Content has not changed since last save",
        ),
        Err(e) => GenericMessage::text(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn request_handler(req: Request<Body>) -> Result<Response<Body>> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/save_links") => save_links(req).await,
        (&Method::POST, "/register_click") => register_click(req).await,
        _ => GenericMessage::not_found(),
    }
}
