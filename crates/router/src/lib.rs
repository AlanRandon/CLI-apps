pub use http;
pub use router_macros;

#[cfg(test)]
mod tests;

pub trait Body {
    type Body;
}

impl<'a, T, C> Body for Request<'a, T, C> {
    type Body = T;
}

impl<T> Body for http::Response<T> {
    type Body = T;
}

pub trait Context {
    type Context;
}

impl<'a, T, C> Context for Request<'a, T, C> {
    type Context = C;
}

#[derive(Debug)]
pub struct Request<'a, T, C> {
    pub request: &'a http::Request<T>,
    pub segments: Vec<&'a str>,
    pub context: &'a C,
}

impl<'a, T: 'a> Request<'a, T, ()> {
    pub fn from_http(request: &'a http::Request<T>) -> Self {
        let segments = request
            .uri()
            .path()
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        Self {
            segments,
            request,
            context: &(),
        }
    }
}

impl<'a, T, C> Request<'a, T, C> {
    pub fn ignore_segments(&self, count: usize) -> Self {
        Self {
            request: self.request,
            segments: self.segments[count..].to_vec(),
            context: self.context,
        }
    }

    pub fn with_context<U>(self, context: &'a U) -> Request<'a, T, U> {
        Request {
            request: self.request,
            segments: self.segments,
            context,
        }
    }

    pub fn from_http_with_context(request: &'a http::Request<T>, context: &'a C) -> Self {
        Request::from_http(request).with_context(context)
    }
}

extern crate self as router;

pub mod prelude {
    pub use super::Request;
    pub use router_macros::{any, delete, get, head, options, patch, post, put, router};
}
