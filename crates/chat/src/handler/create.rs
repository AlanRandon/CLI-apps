use super::internal_server_error;
use html_builder::prelude::*;
use http::{Method, Request, Response};
use hyper::Body;
use sqlx::{Pool, Sqlite};

pub async fn handler(
    pool: &Pool<Sqlite>,
    request: &Request<Body>,
    segments: &[&str],
) -> Option<Response<Body>> {
    let Some((&"create", segments)) = segments.split_first() else {
        return None;
    };

    if request.method() != Method::POST || !segments.is_empty() {
        return None;
    }

    let Ok(record) = sqlx::query!(
            "INSERT INTO chats (name) VALUES (?) RETURNING id",
            "New Chat"
        )
        .fetch_one(pool)
        .await else {
            return Some(internal_server_error(html::text("Failed To Create Chat")));
        };

    let id = uuid::Uuid::from_slice(&record.id).unwrap();

    Some(
        Response::builder()
            .header("HX-Trigger", "reload-chats")
            .body(Body::from(
                div()
                    .attr("hx-on:click", "this.remove()")
                    .text("Chat Created")
                    .child(
                        button()
                            .attr("hx-get", format!("chat/{id}"))
                            .attr("hx-target", "#chat")
                            .text("View"),
                    )
                    .to_string(),
            ))
            .unwrap(),
    )
}
