use std::{
    collections::HashSet,
    fs::OpenOptions,
    io::{BufWriter, Write},
    str::from_utf8_unchecked,
    sync::Arc,
};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tracing::info;

use crate::{
    circular_string::CircularString,
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
    pub static ref CLICK_LOG: Arc<Mutex<DB>> = setup();
}

pub struct DB {
    file: File,
    buf:  CircularString,
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

fn setup() -> Arc<Mutex<DB>> {
    let file_name = format!("{}/click.log", CONFIG.storage_dir);
    // create a buffer and read the file into
    let mut cs = CircularString::with_capacity(CONFIG.click_buffer_size);
    read_click_log::read_click_log(&file_name, &mut cs);

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_name)
        .unwrap();

    Arc::new(Mutex::new(DB {
        file: file.into(),
        buf:  cs,
    }))
}

mod read_click_log {
    use super::info;
    use super::CircularString;
    use std::{
        fs::File,
        io::{BufRead, BufReader},
    };

    pub(super) fn read_click_log(file_name: &str, cs: &mut CircularString) {
        if let Ok(file) = File::open(file_name) {
            let reader = BufReader::new(file);
            let mut count = 0;
            for line in reader.lines().flatten() {
                cs.push(&line);
                count += 1;
            }
            info!("Read {} lines from {file_name}", count);
        }
    }
}

async fn write_click(click: Click, user: &str) -> Result<()> {
    let mut db = CLICK_LOG.lock().await;
    let mut buf = Vec::with_capacity(1020);
    writeln!(
        buf,
        "{} {} {} {}",
        get_epoch_ms(),
        user,
        click.uuid,
        click.href
    )?;
    db.buf.push(unsafe { from_utf8_unchecked(&buf) });
    db.file.write_all(&buf).await?;

    db.file.flush().await?;
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

fn compute_link_stats(buf: &CircularString) -> HashSet<&str> {
    let mut links = HashSet::new();
    buf.into_iter().for_each(|s| {
        if let Some(link) = s.rsplit_once(' ') {
            links.insert(link.1);
        }
    });
    links
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_compute_link_stats() {
        let mut buf = CircularString::with_capacity(1000);
        buf.push("pref https://www.google.com");
        buf.push("pref https://www.google.com");
        buf.push("pref https://www.microsoft.com");
        let stats = compute_link_stats(&buf);
        assert_eq!(stats.len(), 2);
    }
}

pub async fn get_link_stats(mut _request: Request<Body>) -> Result<Response<Body>> {
    let db = CLICK_LOG.lock().await;
    let stats = compute_link_stats(&db.buf);
    let json = serde_json::to_string(&stats)?;
    GenericMessage::json_string_response(json)
}
