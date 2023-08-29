use crate::handler::{bad_request, internal_server_error};
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
    let Some((&"delete", segments)) = segments.split_first() else {
        return None;
    };

    if request.method() != Method::DELETE || !segments.is_empty() {
        return None;
    }

    #[derive(Deserialize, Debug)]
    struct Params {
        id: Uuid,
    }

    let body = request.body_mut();

    let Ok(body) = hyper::body::to_bytes(body).await else {
        return Some(bad_request(html::text("Failed To Construct Body")));
    };

    let Ok(body) = serde_urlencoded::from_bytes::<Params>(&body) else {
        return Some(bad_request(html::text("Body Was Malformed")));
    };

    let Ok(record) = sqlx::query!("DELETE FROM chats WHERE id = ? RETURNING name", body.id).fetch_one(pool).await else {
        return Some(internal_server_error(html::text("Failed To Send Message")));
    };

    Some(
        Response::builder()
            .header("HX-Trigger", "reload-chats")
            .body(Body::from(
                div()
                    .class("rounded-2xl bg-white h-full grid place-items-center")
                    .child(h2().text(format!("Chat {} deleted.", record.name)))
                    .to_string(),
            ))
            .unwrap(),
    )
}
