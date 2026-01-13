use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use scraper::Selector;
use crate::collector::board::PageMeta;
use crate::collector::HTTP_CLIENT;

static CREATED_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("#container header .gall_date").unwrap()); // 작성일시
static AUTHOR_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("#container header .nickname em").unwrap()); // 작성자
static SUBJECT_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("#container header .gallview_head .title_subject").unwrap()); // 제목
static CONTENT_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("#container .writing_view_box .write_div").unwrap()); // 본문

#[derive(Debug)]
pub struct Article {
    number: String,
    timestamp: DateTime<Utc>,
    author: String,
    subject: String,
    content: String,
    attach: Vec<Attach>
}

#[derive(Debug)]
pub struct Attach {

}

#[derive(Debug, thiserror::Error)]
#[error("collect article error")]
struct CollectArticleErr {
    message: &'static str
}

impl CollectArticleErr {
    fn new (msg: &'static str) -> Self {
        Self {
            message: msg
        }
    }
}

pub async fn collect_article(meta: &PageMeta) -> Result<Article, Box<dyn std::error::Error>> {
    let PageMeta {
        number: page_number,
        ..
    } = meta;

    let html = http_page(page_number).await?;
    let dom = scraper::Html::parse_document(&html);

    let created_sl = dom.select(&CREATED_SELECTOR).next();
    let author_sl = dom.select(&AUTHOR_SELECTOR).next();
    let subject_sl = dom.select(&SUBJECT_SELECTOR).next();
    let content_sl = dom.select(&CONTENT_SELECTOR).next();

    let created_el = match created_sl {
        Some(v) => v,
        None => return Err(Box::new(CollectArticleErr::new("not found created_el"))),
    };
    let author_el = match author_sl {
        Some(v) => v,
        None => return Err(Box::new(CollectArticleErr::new("not found author_el"))),
    };
    let subject_el = match subject_sl {
        Some(v) => v,
        None => return Err(Box::new(CollectArticleErr::new("not found subject_el"))),
    };
    let content_el = match content_sl {
        Some(v) => v,
        None => return Err(Box::new(CollectArticleErr::new("not found content_el"))),
    };

    content_el

    Ok(Article {
        number: page_number.to_string(),
        timestamp: Utc::now(), // todo date parse
        author: author_el.text().collect::<String>(),
        subject: subject_el.text().collect::<String>(),
        content: content_el.text().collect::<String>().replace("\n", "").replace("\t", ""),
        attach: vec![],
    })

}

async fn http_page(page_number: &String) -> Result<String, Box<dyn std::error::Error>> {
    let res = HTTP_CLIENT.get(format!("https://gall.dcinside.com/board/view/?id=baseball_new13&no={page_number}&page=1"))
        .send()
        .await?
        .text()
        .await?;

    Ok(res)
}