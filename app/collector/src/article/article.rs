use anyhow::{bail};
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use reqwest::{Client, StatusCode};
use scraper::{ElementRef, Selector};

static _DELETED_SELECTOR: Lazy<Selector> =
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
#[derive(Debug)]
pub struct Article {
    pub id: u64,
    pub timestamp: DateTime<Utc>,
    pub author: String,
    pub subject: String,
    pub content: String,
    pub url: String,
    pub attach_src: Vec<String>,
}

#[derive(Debug)]
pub struct Attach {
    pub origin_src: String,
    pub copied_path: std::path::PathBuf,
}

#[derive(Debug)]
pub enum ArticleResult {
    Article(Article),
    DeletedArticle,
}

#[derive(Debug)]
pub struct DeletedArticle {
    pub id: u64,
    pub timestamp: DateTime<Utc>,
}

impl From<Article> for ArticleResult {
    fn from(v: Article) -> Self {
        Self::Article(v)
    }
}

#[tracing::instrument]
pub async fn get_article(client: Client, id: u64) -> anyhow::Result<ArticleResult> {
    let (code, html, url) = http_page(client, id).await?;

    if code == StatusCode::NOT_FOUND {
        return Ok(ArticleResult::DeletedArticle);
    }

    let dom = scraper::Html::parse_document(&html);

    let _created_el = match dom.select(&CREATED_SELECTOR).next() {
        Some(v) => v,
        None => bail!("not found created_el id: {}", id),
    };
    let author_el = match dom.select(&AUTHOR_SELECTOR).next() {
        Some(v) => v,
        None => bail!("not found author_el id: {}", id),
    };
    let subject_el = match dom.select(&SUBJECT_SELECTOR).next() {
        Some(v) => v,
        None => bail!("not found subject_el id: {}", id),
    };
    let content_el = match dom.select(&CONTENT_SELECTOR).next() {
        Some(v) => v,
        None => bail!("not found content_el id: {}", id),
    };

    Ok(
        Article {
            id,
            timestamp: Utc::now(),
            author: author_el.text().collect::<String>(),
            subject: subject_el.text().collect::<String>(),
            content: content_el.text().collect::<String>().replace("\n", "").replace("\t", ""),
            attach_src: collect_attach_src(content_el),
            url,
        }.into()
    )
}

fn collect_attach_src(element: ElementRef<'_>) -> Vec<String> {
    let mut result = Vec::<String>::new();

    for img_el in element.select(&IMG_SELECTOR) {
        let Some(src) = img_el.value().attr("src") else { continue; };

        result.push(src.to_string());
    }

    result
}

#[tracing::instrument]
async fn http_page(client: Client, page_number: u64) -> anyhow::Result<(StatusCode, String, String)> {
    let url = format!(
        "https://gall.dcinside.com/board/view/?id=baseball_new13&no={page_number}&page=1"
    );

    let response = client
        .get(url.as_str())
        .send()
        .await?;

    let status = response.status();
    let response_body = response.text().await?;

    if response_body.len() == 0 {
        bail!("empty response body");
    }

    Ok((status, response_body, url))
}
