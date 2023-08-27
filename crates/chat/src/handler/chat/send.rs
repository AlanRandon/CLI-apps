use super::internal_server_error;
use html_builder::prelude::*;
use http::{Method, Request, Response};
use hyper::Body;
use serde::Deserialize;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

pub async fn handler(
    pool: &Pool<Sqlite>,
    request: &mut Request<Body>,
    segments: &[&str],
) -> Option<Response<Body>> {
    let Some((&"send", segments)) = segments.split_first() else {
        return None;
    };

    if request.method() != Method::POST || !segments.is_empty() {
        return None;
    }

    #[derive(Deserialize, Debug)]
    struct Params {
        content: String,
        id: String,
    }

    let body = request.body_mut();

    // TODO: malformed request not internal server error
    let Ok(body) = hyper::body::to_bytes(body).await else {
        return Some(internal_server_error(html::text("Failed To Construct Body")));
    };

    let Ok(body) = serde_urlencoded::from_bytes::<Params>(&body) else {
        return Some(internal_server_error(html::text("Body Was Malformed")));
    };

    let Ok(id) = body.id.parse::<Uuid>() else {
        return Some(internal_server_error(html::text("Failed To Parse Chat ID")));
    };

    let Ok(_) = sqlx::query!("INSERT INTO messages (chat, content) VALUES (?, ?)", id, body.content).execute(pool).await else {
        return Some(internal_server_error(html::text("Failed To Send Message")));
    };

    Some(
        Response::builder()
            .header("HX-Trigger", "reload-messages")
            .body(Body::empty())
            .unwrap(),
    )
}
