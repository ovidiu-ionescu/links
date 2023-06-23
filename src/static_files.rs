use hyper::{Body, Request, Response, StatusCode};
use lib_hyper_organizator::response_utils::IntoResultHyperResponse;
use tokio::fs::read;
use tracing::info;

use crate::router::CONFIG;
use crate::utils::Result;

pub async fn serve_file(req: Request<Body>) -> Result<Response<Body>> {
    info!("serve_file");
    match req.uri().path() {
        "/" => serve_static_file("index.html", "text/html").await,
        "/pkg_test/concatenate.js" => serve_static_file("concatenate.js", "text/javascript").await,
        "/pkg_test/concatenate_bg.wasm" => {
            serve_static_file("concatenate_bg.wasm", "application/wasm").await
        }
        "/links-manifest.json" => {
            serve_static_file("links-manifest.json", "application/json").await
        }
        "/images/links.ico" => serve_static_file("links.ico", "image/x-icon").await,
        "/images/links-4x.png" => serve_static_file("links-4x.png", "image/png").await,
        "/329f4aef-f624-4ed1-8a89-bb9bb356a66a.md" => {
            serve_static_file("329f4aef-f624-4ed1-8a89-bb9bb356a66a.md", "text/markdown").await
        }
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
