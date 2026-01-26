use anyhow::Context;
use once_cell::sync::Lazy;
use reqwest::Client;
use tokio::io::AsyncWriteExt;
use tracing::instrument;
use uuid::Uuid;

static IMAGE_DIR: Lazy<std::path::PathBuf> = Lazy::new(|| {
    let dir = std::env::var("IMAGE_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("./data/attach"));

    std::fs::create_dir_all(&dir).expect("failed to create IMAGE_DIR");

    dir
});

pub struct ArticleFile {
    pub origin_src: String,
    pub copied_path: std::path::PathBuf,
}

pub async fn save_files(client: Client, attach_src: Vec<String>, referer: &str) -> anyhow::Result<Vec<ArticleFile>> {
    let mut attach = Vec::<ArticleFile>::new();
    for src in attach_src {
        let path = save_file(client.clone(), &src, &referer)
            .await
            .context(format!("failed to save attach file {}", src))?;

        attach.push(ArticleFile {
            origin_src: src.to_string(),
            copied_path: path,
        });
    }

    Ok(attach)
}

#[instrument]
async fn save_file(
    client: Client,
    src: &str,
    referer: &str,
) -> anyhow::Result<std::path::PathBuf> {
    let response = client
        .get(src)
        .header(reqwest::header::REFERER, referer)
        .send()
        .await
        .context(format!("failed to download attach {}", src))?
        .error_for_status()?;

    let ext = extract_ext_from_cd(response.headers());
    let bytes = response.bytes().await?;

    let filename = format!("{}.{}", Uuid::now_v7(), ext);
    let path = IMAGE_DIR.join(filename);

    let mut file = tokio::fs::File::create(&path).await?;
    file.write_all(&bytes).await?;

    Ok(path)
}

fn extract_ext_from_cd(headers: &reqwest::header::HeaderMap) -> String {
    headers
        .get(reqwest::header::CONTENT_DISPOSITION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| {
            v.split(';')
                .find(|s| s.trim_start().starts_with("filename="))
        })
        .and_then(|s| {
            let filename = s.trim().trim_start_matches("filename=").trim_matches('"');
            std::path::Path::new(filename)
                .extension()
                .and_then(|e| e.to_str())
        })
        .map(|s| s.to_string())
        .unwrap_or_else(|| "jpg".to_string())
}