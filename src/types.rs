use futures::future::BoxFuture;
use http::{Request as HttpRequest, Response as HttpResponse};
use std::{future::Future, pin::Pin, sync::Arc};

pub type Request = HttpRequest<Vec<u8>>;
pub type Response = HttpResponse<Vec<u8>>;

pub type BoxFutureResp = BoxFuture<'static, anyhow::Result<Response>>;

/// ハンドラは Request -> Future<Response>
pub type HandlerFn = Arc<dyn Fn(Request) -> BoxFutureResp + Send + Sync + 'static>;

/// ユーティリティ: 文字列レスポンス
pub fn text(body: &str, status: u16) -> Response {
    let mut resp = HttpResponse::builder()
        .status(status)
        .header("content-type", "text/plain; charset=utf-8")
        .body(body.as_bytes().to_vec())
        .unwrap();
    resp
}
