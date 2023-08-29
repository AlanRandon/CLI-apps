use super::internal_server_error;
use html_builder::prelude::*;
use http::{Method, Request, Response};
use hyper::Body;
use itertools::Itertools;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

mod delete;
mod messages;
mod rename;
mod send;

async fn chat(pool: &Pool<Sqlite>, id: Uuid) -> Result<impl Into<Node>, sqlx::Error> {
    let name = match sqlx::query!("SELECT name FROM chats WHERE id = ?", id)
        .fetch_one(pool)
        .await
    {
        Ok(record) => record.name,
        Err(_) => return Ok(html::text("Failed To Load Chat")),
    };

    Ok(div()
        .class("flex flex-col h-full gap-4")
        .child(
            div()
                .child(
                    input()
                        .attr("value", name)
                        .attr("hx-post", "/rename")
                        .attr("hx-vals", format!(r#"{{"id":"{id}"}}"#))
                        .attr("hx-trigger", "input delay:500ms")
                        .attr("hx-swap", "beforeend")
                        .attr("hx-target", "#notifications")
                        .attr("hx-include", "this")
                        .attr("name", "name")
                        .class("bg-transparent"),
                )
                .child(
                    button()
                        .attr("hx-delete", "/delete")
                        .attr("hx-vals", format!(r#"{{"id":"{id}"}}"#))
                        .attr("hx-target", "#chat")
                        .text("Delete"),
                ),
        )
        .child(
            div()
                .class("bg-white rounded-2xl flex flex-col grow min-h-0")
                .child(
                    div()
                        .class("flex flex-col-reverse overflow-y-auto grow min-h-0 p-4")
                        .child(
                            div()
                                .id("messages")
                                .class("w-full")
                                .attr("hx-get", format!("/messages?id={id}"))
                                .attr("hx-trigger", "reload-messages from:body, every 10s")
                                .child(
                                    match messages::messages(pool, messages::Params { id }).await {
                                        Ok(messages) => messages.into(),
                                        Err(_) => html::text("Failed To Get Messages"),
                                    },
                                ),
                        ),
                )
                .child(
                    form()
                        .class("flex gap-2 p-4")
                        .attr("hx-post", "/send")
                        .attr("hx-swap", "beforeend")
                        .attr("hx-target", "#notifications")
                        .attr("hx-on:submit", "this.querySelector('textarea').value = ''")
                        .id("send-message")
                        .child(
                            textarea()
                                .attr(
                                    "hx-on:keyup",
                                    "if (event.keyCode == 13 && !event.shiftKey) { this.parentElement.querySelector('input[type=submit]').click() }"
                                )
                                .class(
                                    "block p-2 h-[1.5em] box-content bg-white rounded-lg border border-slate-300 resize-none grow focus:shadow",
                                )
                                .id("content")
                                .attr("name", "content")
                                .attr("placeholder", "Your Message..."),
                        )
                        .child(
                            input()
                                .attr("type", "hidden")
                                .attr("value", id)
                                .attr("name", "id"),
                        )
                        .child(
                            input()
                                .class("btn")
                                .attr("type", "submit")
                                .attr("value", "Send"),
                        ),
                ),
        )
        .into())
}

pub async fn handler(
    pool: &Pool<Sqlite>,
    request: &mut Request<Body>,
    segments: &[&str],
) -> Option<Response<Body>> {
    if let Some(response) = messages::handler(pool, request, segments).await {
        return Some(response);
    };

    if let Some(response) = rename::handler(pool, request, segments).await {
        return Some(response);
    };

    if let Some(response) = delete::handler(pool, request, segments).await {
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
