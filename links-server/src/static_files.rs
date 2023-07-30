use hyper::{Body, Request, Response, StatusCode};
use lib_hyper_organizator::response_utils::IntoResultHyperResponse;
use tokio::fs::read;
use tracing::info;

use crate::router::CONFIG;
use crate::utils::Result;

pub async fn serve_file(req: Request<Body>) -> Result<Response<Body>> {
    info!("serve_file");
    if let Some(file_info) = CONFIG.static_files.get(req.uri().path()) {
        return serve_static_file(&file_info.file, &file_info.mime).await;
    }
    match req.uri().path() {
        "/links" => serve_links_file(req).await,
        _ => "invalid path".to_text_response_with_status(StatusCode::NOT_FOUND),
    }
}

/*
async fn serve_static_file(name: &str, mime: &str) -> Result<Response<Body>> {
    let file_name = format!("{}/{}", CONFIG.storage_dir, name);
    match fs::read_to_string(&file_name).await {
        Ok(content) => Ok(to_response(content, mime)),
        Err(e) => e
            .to_string()
            .to_text_response_with_status(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
*/
async fn serve_static_file(name: &str, mime: &str) -> Result<Response<Body>> {
    let file_name = format!("{}/{}", CONFIG.static_files_dir, name);
    match read(&file_name).await {
        Ok(content) => Ok(Response::builder()
            .header("Content-Type", mime)
            .body(Body::from(content))?),
        Err(e) => e
            .to_string()
            .to_text_response_with_status(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

fn to_response(content: String, mime: &str) -> Response<Body> {
    Response::builder()
        .header("Content-Type", mime)
        .body(Body::from(content))
        .unwrap()
}

async fn serve_links_file(req: Request<Body>) -> Result<Response<Body>> {
    // the parameter req is the uuid of the file name
    let Some(uuid) = req.uri().query() else {
        return "no uuid supplied".to_text_response_with_status(StatusCode::BAD_REQUEST);
    };
    // check the uuid is valid

    let file_name = format!("{}/{}.md", CONFIG.storage_dir, uuid);
    info!("serve_links_file: {}", file_name);
    match read(&file_name).await {
        Ok(content) => {
            let content = String::from_utf8(content)?;
            Ok(to_response(content, "text/markdown"))
        }
        Err(e) => e
            .to_string()
            .to_text_response_with_status(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
