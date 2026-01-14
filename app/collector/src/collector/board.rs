use once_cell::sync::Lazy;
use scraper::Selector;
use crate::collector::HTTP_CLIENT;

#[derive(Debug)]
pub struct PageMeta {
    pub id: u64,
    pub title: String,
    pub has_image: bool,
}
static PAGE_META_LIST_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse(".listwrap2 .ub-content.us-post").unwrap());
static PAGE_META_LINK_SELECT: Lazy<Selector> = Lazy::new(|| Selector::parse(".gall_tit.ub-word a").unwrap());
static PAGE_META_NUMBER_SELECT: Lazy<Selector> = Lazy::new(|| Selector::parse(".gall_num").unwrap());
static PAGE_META_HAS_IMAGE_SELECT: Lazy<Selector> = Lazy::new(|| Selector::parse(".icon_img.icon_pic").unwrap());

static URL: &'static str = "https://gall.dcinside.com/board/lists/?id=baseball_new13";

async fn http_list_page() -> Result<String, Box<dyn std::error::Error>> {
    let res = HTTP_CLIENT.get(URL).send().await?.text().await?;

    Ok(res)
}

pub async fn collect_list() -> Result<Vec<PageMeta>, Box<dyn std::error::Error>> {
    let html = http_list_page().await?;
    let dom = scraper::Html::parse_document(&html);
    let mut result = Vec::<PageMeta>::new();

    for el in dom.select(&PAGE_META_LIST_SELECTOR) {
        let link_sl = el.select(&PAGE_META_LINK_SELECT).next();
        let number_sl = el.select(&PAGE_META_NUMBER_SELECT).next();
        let has_image_sl = el.select(&PAGE_META_HAS_IMAGE_SELECT).next();
        let Some(link_el) = link_sl else { continue; };
        let Some(number_el) = number_sl else { continue; };

        let meta = PageMeta {
            id: number_el.text().collect::<String>().parse::<u64>()?,
            title: link_el.text().collect::<String>().replace("\n", "").replace("\t", ""),
            has_image: has_image_sl != None,
        };

        result.push(meta);
    }

    Ok(result)
}
