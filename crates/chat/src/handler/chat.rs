use super::internal_server_error;
use html_builder::prelude::*;
use http::{Method, Request, Response};
use hyper::Body;
use itertools::Itertools;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

mod messages;
mod send;

async fn chat(pool: &Pool<Sqlite>, id: Uuid) -> Result<impl Into<Node>, sqlx::Error> {
    Ok(div()
        .child(
            div()
                .id("messages")
                .attr("hx-get", format!("/messages?id={id}"))
                .attr("hx-trigger", "reload-messages from:body")
                .child(match messages::messages(pool, id).await {
                    Ok(messages) => messages.into(),
                    Err(_) => html::text("Failed To Get Messages"),
                }),
        )
        .child(
            form()
                .attr("hx-post", "/send")
                .attr("hx-swap", "beforeend")
                .attr("hx-target", "#notifications")
                .attr("hx-on:submit", "this.querySelector('textarea').value = ''")
                .id("send-message")
                .child(
                    textarea()
                        .id("content")
                        .attr("name", "content")
                        .attr("placeholder", "message"),
                )
                .child(
                    input()
                        .attr("type", "hidden")
                        .attr("value", id)
                        .attr("name", "id"),
                )
                .child(input().attr("type", "submit").attr("value", "Send")),
        ))
}

pub async fn handler(
    pool: &Pool<Sqlite>,
    request: &mut Request<Body>,
    segments: &[&str],
) -> Option<Response<Body>> {
    if let Some(response) = messages::handler(pool, request, segments).await {
        return Some(response);
    };

    if let Some(response) = send::handler(pool, request, segments).await {
        return Some(response);
    };

    let Some((&"chat", id)) = segments.iter().collect_tuple() else {
            return None;
        };

    if request.method() != Method::GET {
        return None;
    }

    let Ok(id) = id.parse::<Uuid>() else {
        return Some(internal_server_error(html::text("Failed To Parse Chat ID")));
    };

    let Ok(response) = chat(pool, id).await else {
        return Some(internal_server_error(html::text("Failed To Get Chat")));
    };

    Some(
        Response::builder()
            .body(Body::from(response.into().to_string()))
            .unwrap(),
    )
}
