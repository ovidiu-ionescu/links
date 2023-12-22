use crate::utils::Result;
use hyper::{Body, Request, Response, StatusCode};
use lib_hyper_organizator::response_utils::IntoResultHyperResponse;
use tokio::fs::{read_dir, DirEntry};

use crate::router::CONFIG;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

pub async fn get_catalog(_req: Request<Body>) -> Result<Response<Body>> {
    match build_catalog(&CONFIG.storage_dir).await {
        Ok(catalog) => catalog.to_text_response(),
        Err(e) => e
            .to_string()
            .to_text_response_with_status(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

fn trim(s: &str) -> &str {
    s.trim_start_matches('#')
        .trim_matches(' ')
        .trim_end_matches(|c| c == '\n' || c == '\r' || c == ' ' || c == '\t')
}

async fn build_catalog(dir: &str) -> Result<String> {
    let mut catalog = String::from(indoc::indoc! {r#"# Catalog

           <link rel="stylesheet" href="/memo.css" >
           
        "#});
    let mut titles = Vec::<String>::new();
    let mut file_names = read_dir(dir).await?;
    while let Some(dir_entry) = file_names.next_entry().await? {
        let file_name = dir_entry.file_name();
        let file_name = file_name.to_str().unwrap();
        if file_name.ends_with(".md") {
            let uuid = file_name.trim_end_matches(".md");
            // read the first line in the file
            if let Ok(line) = read_line(&dir_entry).await {
                titles.push(format!("- [{}](/?{})", trim(&line), uuid));
            } else {
                titles.push(format!("- [{}](/?{})", uuid, uuid));
            }
        }
    }
    titles.sort_by_cached_key(|a| a.to_lowercase());
    catalog.push_str(titles.join("\n").as_str());
    Ok(catalog)
}

async fn read_line(dir_entry: &DirEntry) -> Result<String> {
    let file = File::open(dir_entry.path()).await?;
    let mut lines = BufReader::new(file).lines();
    if let Some(line) = lines.next_line().await? {
        Ok(line)
    } else {
        Err("empty file".into())
    }
}

// test for build_catalog
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_build_catalog() {
        let s = build_catalog("../data").await.unwrap();
        println!("{s}");
    }
}
