use anyhow::Context;
use once_cell::sync::Lazy;
use reqwest::Client;
use scraper::Selector;

#[derive(Debug)]
pub struct ArticleHeader {
    pub id: u64,
    pub title: String,
    pub has_image: bool,
}
static PAGE_META_LIST_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse(".listwrap2 .ub-content.us-post").unwrap());
static PAGE_META_LINK_SELECT: Lazy<Selector> = Lazy::new(|| Selector::parse(".gall_tit.ub-word a").unwrap());
static PAGE_META_NUMBER_SELECT: Lazy<Selector> = Lazy::new(|| Selector::parse(".gall_num").unwrap());
static PAGE_META_HAS_IMAGE_SELECT: Lazy<Selector> = Lazy::new(|| Selector::parse(".icon_img.icon_pic").unwrap());

static URL: &'static str = "https://gall.dcinside.com/board/lists/?id=baseball_new13";

#[tracing::instrument]
async fn http_list_page(client: Client) -> anyhow::Result<String> {
    let response = client
        .get(URL)
        .send()
        .await
        .context(format!("fail to request {}", URL))?;
    
    let res = response.text().await
        .context(format!("fail to read response body from {}", URL))?;

    Ok(res)
}

#[tracing::instrument]
pub async fn collect_list(client: Client) -> anyhow::Result<Vec<ArticleHeader>> {
    let html = http_list_page(client).await.context("fail to collect list page")?;
    let dom = scraper::Html::parse_document(&html);
    let mut result = Vec::<ArticleHeader>::new();

    for el in dom.select(&PAGE_META_LIST_SELECTOR) {
        let link_sl = el.select(&PAGE_META_LINK_SELECT).next();
        let number_sl = el.select(&PAGE_META_NUMBER_SELECT).next();
        let has_image_sl = el.select(&PAGE_META_HAS_IMAGE_SELECT).next();
        let Some(link_el) = link_sl else { continue; };
        let Some(number_el) = number_sl else { continue; };

        let meta = ArticleHeader {
            id: number_el.text().collect::<String>().parse::<u64>()?,
            title: link_el.text().collect::<String>().replace("\n", "").replace("\t", ""),
            has_image: has_image_sl != None,
        };

        result.push(meta);
    }

    Ok(result)
}
