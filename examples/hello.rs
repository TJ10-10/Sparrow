use http::response::Builder;
use sparrow::app::App;
use sparrow::types::Response;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut app = App::new();

    // シンプルなハンドラ
    app.at("/", |_req| {
        let resp = Builder::new()
            .status(200)
            .body("Hello, Sparrow!".into())
            .unwrap();
        async move { Ok(resp) }
    });

    // listen の引数を修正
    app.listen("127.0.0.1:3000").await?;

    Ok(())
}
