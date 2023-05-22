use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
};

use crate::{
    router::CONFIG,
    utils::{get_epoch_ms, get_user_name, Result},
};
use hyper::{Body, Request, Response, StatusCode};
use lib_hyper_organizator::response_utils::{
    parse_body, GenericMessage, PolymorphicGenericMessage,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Click {
    uuid: String,
    href: String,
}

async fn write_click(click: Click, user: &str) -> Result<()> {
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
