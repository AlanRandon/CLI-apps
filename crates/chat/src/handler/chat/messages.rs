use super::internal_server_error;
use html_builder::prelude::*;
use http::{Method, Request, Response};
use hyper::Body;
use serde::Deserialize;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

pub async fn messages(pool: &Pool<Sqlite>, chat_id: Uuid) -> Result<impl Into<Node>, sqlx::Error> {
    let messages = sqlx::query!(
        "SELECT content FROM chats INNER JOIN messages ON chats.id = messages.chat WHERE chats.id = ?",
        chat_id
    ).fetch_all(pool).await?;

    Ok(ul().children(
        messages
            .into_iter()
            .map(|message| li().text(message.content)),
    ))
}

pub async fn handler(
    pool: &Pool<Sqlite>,
    request: &mut Request<Body>,
    segments: &[&str],
) -> Option<Response<Body>> {
    let Some((&"messages", segments)) = segments.split_first() else {
        return None;
    };

    if request.method() != Method::GET || !segments.is_empty() {
        return None;
    }

    #[derive(Deserialize)]
    struct Params {
        id: String,
    }

    let Some(query) = request.uri().query() else {
        return Some(internal_server_error(html::text("Missing Parameters")));
    };

    let Ok(body) = serde_urlencoded::from_str::<Params>(query) else {
        return Some(internal_server_error(html::text("Query Was Malformed")));
    };

    let Ok(id) = body.id.parse::<Uuid>() else {
        return Some(internal_server_error(html::text("Failed To Parse Chat ID")));
    };

    let Ok(response) = messages(pool, id).await else {
        return Some(internal_server_error(html::text("Failed To Get Messages")));
    };

    Some(
        Response::builder()
            .body(Body::from(response.into().to_string()))
            .unwrap(),
    )
}
