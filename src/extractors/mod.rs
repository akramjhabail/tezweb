//! Request Extractors — Clean handler signatures

use std::collections::HashMap;
use crate::http::Request;

pub trait FromRequest: Sized {
    fn extract(req: &Request, params: &HashMap<String, String>) -> Option<Self>;
}

#[derive(Debug, Clone)]
pub struct Path<T>(pub T);

impl<T: std::str::FromStr> FromRequest for Path<T> {
    fn extract(_req: &Request, params: &HashMap<String, String>) -> Option<Self> {
        params.values().next()?.parse().ok().map(Path)
    }
}

impl<T> std::ops::Deref for Path<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target { &self.0 }
}

#[derive(Debug, Clone)]
pub struct Json<T>(pub T);

impl<T: serde::de::DeserializeOwned> FromRequest for Json<T> {
    fn extract(req: &Request, _params: &HashMap<String, String>) -> Option<Self> {
        serde_json::from_slice(&req.body).ok().map(Json)
    }
}

impl<T> std::ops::Deref for Json<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target { &self.0 }
}

#[derive(Debug, Clone)]
pub struct Query(pub HashMap<String, String>);

impl FromRequest for Query {
    fn extract(req: &Request, _params: &HashMap<String, String>) -> Option<Self> {
        if req.query.is_empty() { return None; }
        let map = req.query_all();
        if map.is_empty() { None } else { Some(Query(map)) }
    }
}

impl std::ops::Deref for Query {
    type Target = HashMap<String, String>;
    fn deref(&self) -> &Self::Target { &self.0 }
}

#[derive(Debug, Clone)]
pub struct ReqHeaders(pub Vec<(String, String)>);

impl FromRequest for ReqHeaders {
    fn extract(req: &Request, _params: &HashMap<String, String>) -> Option<Self> {
        if req.headers.is_empty() { return None; }
        Some(ReqHeaders(req.headers.clone()))
    }
}

impl std::ops::Deref for ReqHeaders {
    type Target = Vec<(String, String)>;
    fn deref(&self) -> &Self::Target { &self.0 }
}

#[derive(Debug, Clone)]
pub struct Body(pub Vec<u8>);

impl FromRequest for Body {
    fn extract(req: &Request, _params: &HashMap<String, String>) -> Option<Self> {
        if req.body.is_empty() { return None; }
        Some(Body(req.body.clone()))
    }
}

impl std::ops::Deref for Body {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl FromRequest for Request {
    fn extract(req: &Request, _params: &HashMap<String, String>) -> Option<Self> {
        Some(req.clone())
    }
}
