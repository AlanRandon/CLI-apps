use super::internal_server_error;
use html_builder::prelude::*;
use http::{Method, Request, Response};
use hyper::Body;
use serde::Deserialize;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

pub async fn messages(pool: &Pool<Sqlite>, chat_id: Uuid) -> Result<impl Into<Node>, sqlx::Error> {
    let messages = sqlx::query!(
        r#"
            SELECT content, (unixepoch() - unixepoch(creation_time)) as time_since
            FROM chats INNER JOIN messages
            ON chats.id = messages.chat
            WHERE chats.id = ?"#,
        chat_id
    )
    .fetch_all(pool)
    .await?;

    Ok(ul()
        .class("flex gap-4 flex-col")
        .children(messages.into_iter().map(|message| {
            li().class("rounded-[100vmax] rounded-bl-none bg-slate-200 p-4 w-fit")
                .child(pre().class("font-sans").text(message.content))
                .child(
                    div()
                        .class("text-sm")
                        .attr("x-show-time-since", message.time_since.unwrap_or(0)),
                )
        })))
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
