use http::Method;
use sparrow::{
    App, Router,
    middleware::Logging,
    types::{HandlerFn, Request, Response},
};
use std::{future::Future, pin::Pin, sync::Arc};

fn handler(fn_impl: fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send>>) -> HandlerFn {
    Arc::new(move |req| fn_impl(req))
}

fn hello(_req: Request) -> Pin<Box<dyn Future<Output = Response> + Send>> {
    Box::pin(async move { sparrow::types::text("hello world", 200) })
}

fn echo(req: Request) -> Pin<Box<dyn Future<Output = Response> + Send>> {
    Box::pin(async move {
        let body = req.body().clone();
        let mut resp = http::Response::builder()
            .status(200)
            .header("content-type", "application/octet-stream")
            .body(body)
            .unwrap();
        resp
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let router = Router::new()
        .route(Method::GET, "/hello", handler(hello))
        .route(Method::POST, "/echo", handler(echo));

    let mut app = App::new().with_router(router);
    app.use_middleware(Logging::default());
    app.run("127.0.0.1.8080").await
}
