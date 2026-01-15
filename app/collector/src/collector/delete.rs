use crate::collector::article::{is_article_deleted, DeletedArticle};
use crate::collector::entity;
use crate::collector::{article, delete_entity};
use anyhow::Context;

pub async fn collect_delete() -> anyhow::Result<()> {
    let articles = entity::get_monitor_target()
        .await
        .with_context(|| "fail get_monitor_target")?;

    println!("{} articles monitor.", articles.len());

    let articles = {
        let mut out = Vec::with_capacity(articles.len());
        for a in articles {
            let deleted_result = is_article_deleted(a).await?;
            if let article::CollectArticleResult::DeletedArticle(d) = deleted_result
            {
                out.push(d);
            }

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
        out
    };

    let len = articles.iter().count();

    println!("{} articles are deleted.", len);

    if len == 0 {
        return Ok(())
    }

    delete_entity::insert(articles.into_iter()).await?;

    Ok(())
}
