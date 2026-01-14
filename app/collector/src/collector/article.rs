use crate::collector::HTTP_CLIENT;
use crate::collector::board::PageMeta;
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use reqwest::StatusCode;
use scraper::{ElementRef, Selector};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

static DELETED_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("#container").unwrap()); // 삭제여부
static CREATED_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("#container header .gall_date").unwrap()); // 작성일시
static AUTHOR_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("#container header .nickname em").unwrap()); // 작성자
static SUBJECT_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("#container header .gallview_head .title_subject").unwrap()); // 제목
static CONTENT_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("#container .writing_view_box .write_div").unwrap()); // 본문
static IMG_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("img").unwrap()); // 본문

static IMAGE_DIR: Lazy<std::path::PathBuf> = Lazy::new(|| {
    let dir = std::env::var("IMAGE_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("./data/attach"));

    std::fs::create_dir_all(&dir).expect("failed to create IMAGE_DIR");

    dir
});

#[derive(Debug)]
pub struct Article {
    pub id: u64,
    pub timestamp: DateTime<Utc>,
    pub author: String,
    pub subject: String,
    pub content: String,
    pub attach: Vec<Attach>,
}

#[derive(Debug)]
pub struct Attach {
    pub origin_src: String,
    pub copied_path: std::path::PathBuf,
}

pub enum CollectArticleResult {
    Article(Article),
    DeletedArticle(DeletedArticle),
}

pub struct DeletedArticle {
    pub id: u64,
    pub timestamp: DateTime<Utc>,
}

impl From<DeletedArticle> for CollectArticleResult {
    fn from(v: DeletedArticle) -> Self {
        Self::DeletedArticle(v)
    }
}

impl From<Article> for CollectArticleResult {
    fn from(v: Article) -> Self {
        Self::Article(v)
    }
}

pub async fn is_article_deleted(article: Article) -> anyhow::Result<CollectArticleResult> {
    let (code, _, _) = http_page(article.id).await?;

    Ok(match code {
        StatusCode::NOT_FOUND => DeletedArticle {
            id: article.id,
            timestamp: Utc::now(),
        }.into(),
        _ => article.into(),
    })
}

pub async fn collect_article(meta: PageMeta) -> anyhow::Result<CollectArticleResult> {
    let PageMeta {
        id: page_id,
        ..
    } = meta;

    let (code, html, referer) = http_page(page_id).await?;

    if code == StatusCode::NOT_FOUND || html.len() > 0 {
        return Ok(
            DeletedArticle {
                id: page_id,
                timestamp: Utc::now(),
            }.into()
        );
    }


    let (author, subject, content, attach_src, referer) = {

        let dom = scraper::Html::parse_document(&html);

        let created_sl = dom.select(&CREATED_SELECTOR).next();
        let author_sl = dom.select(&AUTHOR_SELECTOR).next();
        let subject_sl = dom.select(&SUBJECT_SELECTOR).next();
        let content_sl = dom.select(&CONTENT_SELECTOR).next();

        let created_el = match created_sl {
            Some(v) => v,
            None => return Err(anyhow::format_err!("not found created_el id: {}", page_id)),
        };
        let author_el = match author_sl {
            Some(v) => v,
            None => return Err(anyhow::format_err!("not found author_el id: {}", page_id)),
        };
        let subject_el = match subject_sl {
            Some(v) => v,
            None => return Err(anyhow::format_err!("not found subject_el id: {}", page_id)),
        };
        let content_el = match content_sl {
            Some(v) => v,
            None => return Err(anyhow::format_err!("not found content_el id: {}", page_id)),
        };

        (
            author_el.text().collect::<String>(),
            subject_el.text().collect::<String>(),
            content_el.text().collect::<String>().replace("\n", "").replace("\t", ""),
            collect_attach(content_el),
            referer
        )
    };

    let mut attach = Vec::<Attach>::new();
    for src in attach_src {
        let path = save_to_attach(&src, &referer).await?;
        attach.push(Attach {
            origin_src: src.to_string(),
            copied_path: path,
        });
    }

    Ok(
        Article {
            id: page_id,
            timestamp: Utc::now(),
            author,
            subject,
            content,
            attach,
        }.into()
    )
}

fn collect_attach(element: ElementRef<'_>) -> Vec<String> {
    let mut result = Vec::<String>::new();

    for img_el in element.select(&IMG_SELECTOR) {
        let Some(src) = img_el.value().attr("src") else { continue; };

        result.push(src.to_string());
    }

    result
}

async fn save_to_attach(
    src: &str,
    referer: &str,
) -> anyhow::Result<std::path::PathBuf> {
    let response = HTTP_CLIENT
        .get(src)
        .header(reqwest::header::REFERER, referer)
        .send()
        .await?
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

async fn http_page(page_number: u64) -> anyhow::Result<(StatusCode, String, String)> {
    let url = format!(
        "https://gall.dcinside.com/board/view/?id=baseball_new13&no={page_number}&page=1"
    );

    let response = HTTP_CLIENT
        .get(url.as_str())
        .send()
        .await?;

    let status = response.status();
    let response_body = response.text().await?;

    let log = response_body.as_str().chars().take(30).collect::<String>();

    println!("id {} ,body: {}...", page_number, log);

    Ok((status, response_body, url))
}
