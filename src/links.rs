use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    sync::Arc,
};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

use crate::{
    router::CONFIG,
    utils::{get_epoch_ms, get_user_name, Result},
};
use hyper::{Body, Request, Response, StatusCode};
use lazy_static::lazy_static;
use lib_hyper_organizator::response_utils::{
    parse_body, GenericMessage, PolymorphicGenericMessage,
};
use serde::{Deserialize, Serialize};

lazy_static! {
    pub static ref CLICK_LOG: Arc<Mutex<File>> = setup();
}

#[derive(Serialize, Deserialize, Debug)]
struct Click {
    uuid: String,
    href: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct LinkMessage {
    time:  u128,
    user:  String,
    click: Click,
}

fn setup() -> Arc<Mutex<File>> {
    let file_name = format!("{}/click.log", CONFIG.storage_dir);
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_name)
        .unwrap();
    Arc::new(Mutex::new(file.into()))
}

async fn write_click(click: Click, user: &str) -> Result<()> {
    let mut file = CLICK_LOG.lock().await;
    let mut buf = Vec::with_capacity(1020);
    writeln!(
        buf,
        "{} {} {} {}",
        get_epoch_ms(),
        user,
        click.uuid,
        click.href
    )?;
    file.write_all(&buf).await?;

    file.flush().await?;
    Ok(())
}

async fn _write_click(click: Click, user: &str) -> Result<()> {
    let file_name = format!("{}/click.log", CONFIG.storage_dir);
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_name)?;
    let line = format!(
        "{} {} {} {}\n",
        get_epoch_ms(),
        user,
        click.uuid,
        click.href
    );
    let mut out = BufWriter::new(&file);
    write!(out, "{}", line)?;
    out.flush()?;
    Ok(())
}

pub async fn register_click(mut request: Request<Body>) -> Result<Response<Body>> {
    let click = parse_body(&mut request).await?;
    let user = get_user_name(&request)?;
    write_click(click, user).await?;
    Ok(GenericMessage::text(StatusCode::OK, "Click registered"))
}
