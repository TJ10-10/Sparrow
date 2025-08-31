use bytes::Bytes;
use futures::future::BoxFuture;
use http::{Method, Request as HttpRequest, Response as HttpResponse};
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::Service;
use hyper::{Request as HyperReq, Response as HyperResp};
use hyper_util::rt::TokioIo;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::middleware::{Middleware, Next};
use crate::router::Router;
use crate::types::{BoxFutureResp, Request, Response};

/// ハンドラ型
type Handler = Arc<dyn Fn(Request) -> BoxFutureResp + Send + Sync>;

#[derive(Clone, Default)]
pub struct App {
    router: Router,
    chain: Vec<Arc<dyn Middleware>>,
    routes: HashMap<String, Handler>, // ミドルウェア・チェーン
}

impl App {
    pub fn new() -> Self {
        Self {
            router: Router::new(),
            chain: vec![],
            routes: HashMap::new(),
        }
    }
    pub fn with_router(mut self, router: Router) -> Self {
        self.router = router;
        self
    }
    pub fn use_middleware<M: Middleware + 'static>(&mut self, m: M) {
        self.chain.push(Arc::new(m));
    }
    pub fn at<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = anyhow::Result<Response>> + Send + 'static,
    {
        self.routes.insert(
            path.to_string(),
            Arc::new(move |req| Box::pin(handler(req))),
        );
    }

    /// サーバーを起動
    pub async fn listen(self, addr: &str) -> anyhow::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        println!("Listening on http://{}", addr);

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let svc = HttpService { app: self.clone() };

            tokio::spawn(async move {
                if let Err(e) = http1::Builder::new().serve_connection(io, svc).await {
                    eprintln!("serve_connection error: {:?}", e);
                }
            });
        }
    }
}

#[derive(Clone)]
struct HttpService {
    app: App,
}
impl Service<HyperReq<hyper::body::Incoming>> for HttpService {
    type Response = HyperResp<Full<Bytes>>;
    type Error = anyhow::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn call(&self, req: HyperReq<hyper::body::Incoming>) -> Self::Future {
        let app = self.app.clone();
        Box::pin(async move {
            // body 全読み (シンプル化)
            let (parts, body) = req.into_parts();
            let body_bytes = body.collect().await.unwrap().to_bytes().to_vec();
            let req2: Request = HttpRequest::from_parts(parts, body_bytes);

            // シンプルなルーティング
            let path = req2.uri().path().to_string();
            if let Some(handler) = app.routes.get(&path) {
                let resp = handler(req2).await?;
                let (parts, body_vec) = resp.into_parts();
                Ok(HyperResp::new(Full::new(Bytes::from(body_vec))))
            } else {
                let not_found = HttpResponse::builder()
                    .status(404)
                    .body(b"Not Found".to_vec())
                    .unwrap();
                Ok(HyperResp::new(Full::new(Bytes::from(
                    not_found.into_body(),
                ))))
            }
        })
    }
}
