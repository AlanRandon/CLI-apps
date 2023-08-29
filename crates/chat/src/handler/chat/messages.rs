use crate::handler::{bad_request, internal_server_error};
use html_builder::prelude::*;
use http::{Method, Request, Response};
use hyper::Body;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct Params {
    pub id: Uuid,
}

pub async fn messages(pool: &Pool<Sqlite>, params: Params) -> Result<impl Into<Node>, sqlx::Error> {
    // TODO: limits?
    let messages = sqlx::query!(
        r#"
            SELECT content, (unixepoch() - unixepoch(creation_time)) as time_since
            FROM chats INNER JOIN messages
            ON chats.id = messages.chat
            WHERE chats.id = ?
            ORDER BY creation_time"#,
        params.id,
    )
    .fetch_all(pool)
    .await?;

    Ok(ul()
        .class("flex gap-4 flex-col")
        .children(messages.into_iter().map(|message| {
            li().class("flex flex-col")
                .child(
                    div()
                        .class("p-4 bg-slate-200 rounded-t flex flex-col")
                        .child(
                            pre()
                                .class("font-sans break-all hyphens-auto whitespace-pre-wrap")
                                .text(message.content),
                        )
                        .child(
                            div()
                                .class("text-xs text-black/80 min-w-[15ch] text-right")
                                .attr("x-show-time-since", message.time_since.unwrap_or(0)),
                        ),
                )
                .child(div().class(
                    "border-transparent border-t-slate-200 border-8 border-b-0 h-0 w-0 box-content",
                ))
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

    let Some(query) = request.uri().query() else {
        return Some(bad_request(html::text("Request Body Was Malformed")));
    };

    let Ok(body) = serde_urlencoded::from_str::<Params>(query) else {
        return Some(bad_request(html::text("Request Body Was Malformed")));
    };

    let Ok(response) = messages(pool, body).await else {
        return Some(internal_server_error(html::text("Failed To Get Messages")));
    };

    Some(
        Response::builder()
            .body(Body::from(response.into().to_string()))
            .unwrap(),
    )
}
