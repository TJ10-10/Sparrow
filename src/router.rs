use crate::types::HandlerFn;
use http::Method;
use std::collections::HashMap;

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct RouteKey {
    pub method: Method,
    pub path: String,
}

#[derive(Default, Clone)]
pub struct Router {
    routes: HashMap<RouteKey, HandlerFn>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub fn route(mut self, method: Method, path: &str, handler: HandlerFn) -> Self {
        self.routes.insert(
            RouteKey {
                method,
                path: path.to_string(),
            },
            handler,
        );
        self
    }

    pub fn find(&self, method: &Method, path: &str) -> Option<HandlerFn> {
        self.routes
            .get(&RouteKey {
                method: method.clone(),
                path: path.to_string(),
            })
            .cloned()
    }
}
