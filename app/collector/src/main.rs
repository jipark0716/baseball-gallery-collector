mod collector;

use futures::future;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let metas = collector::collect_list().await.unwrap();

    // let articles = future::try_join_all(metas.iter().map(|meta| collector::collect_article(meta)))
    //     .await
    //     .unwrap();
    //
    // println!("{:#?}", articles);

    for meta in metas {
        let article = collector::collect_article(&meta).await.unwrap();
        println!("{:#?}", article);
    }

    Ok(())
}
