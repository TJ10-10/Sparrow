use crate::types::{Request, Response};
use async_trait::async_trait;
use std::time::Instant;

/// ミドルウェアは "next" を包む関数として表現
pub type BoxFutureResp =
    std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<Response>> + Send>>;

pub struct Next {
    pub handler: Box<dyn Fn(Request) -> BoxFutureResp + Send + Sync>,
}

impl Next {
    pub async fn run(&self, req: Request) -> anyhow::Result<Response> {
        (self.handler)(req).await
    }
}

#[async_trait]
pub trait Middleware: Send + Sync {
    async fn handle(&self, req: Request, next: Next) -> anyhow::Result<Response>;
}

pub struct Logger;

#[async_trait]
impl Middleware for Logger {
    async fn handle(&self, req: Request, next: Next) -> anyhow::Result<Response> {
        let method = req.method().clone();
        let path = req.uri().path().to_string();
        let start = Instant::now();

        // ハンドラ実行
        let resp_result = next.run(req).await;
        let ms = start.elapsed().as_millis();

        match &resp_result {
            Ok(resp) => {
                tracing::info!(%method, %path, %ms, status = resp.status().as_u16(), "request");
            }
            Err(err) => {
                tracing::error!(%method, %path, %ms, error = %err, "request failed");
            }
        }

        resp_result
    }
}
