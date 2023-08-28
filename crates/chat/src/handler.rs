use super::UriExt;
use html_builder::prelude::*;
use http::{Method, Request, Response, StatusCode};
use hyper::Body;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

mod chat;
mod create;

pub fn document(body: impl IntoIterator<Item = impl Into<Node>>) -> String {
    let body = body.into_iter().map(Into::into).chain([Node::from(
        script().child(Node::RawHtml(include_str!("../.dist/init.js").to_string())),
    )]);

    html_builder::document::<Node, _>(
        [
            title().text("App").into(),
            style()
                .child(Node::RawHtml(
                    include_str!("../.dist/style.css").to_string(),
                ))
                .into(),
        ],
        body,
    )
}

async fn chats(pool: Pool<Sqlite>) -> Result<impl Into<Node>, sqlx::Error> {
    let chats = sqlx::query!("SELECT name, id FROM chats")
        .fetch_all(&pool)
        .await?;

    Ok(ul().children(chats.into_iter().map(|chat| {
        li().child(
            button()
                .text(chat.name)
                .attr(
                    "hx-get",
                    format!("/chat/{}", Uuid::from_slice(&chat.id).unwrap()),
                )
                .attr("hx-target", "#chat"),
        )
    })))
}

fn internal_server_error(message: impl Into<Node>) -> Response<Body> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(message.into().to_string()))
        .unwrap()
}

pub async fn handler(mut request: Request<Body>, pool: Pool<Sqlite>) -> Response<Body> {
    let uri = request.uri().clone();
    let segments = uri.segments();

    if segments.is_empty() && request.method() == Method::GET {
        return Response::builder()
            .body(Body::from(document([div()
                .class("flex gap-4 h-full")
                .child(
                    div()
                        .class("bg-slate-500 rounded-r h-full p-4 text-white overflow-auto")
                        .child(
                            button()
                                .attr("hx-post", "/create")
                                .attr("hx-swap", "beforeend")
                                .attr("hx-target", "#notifications")
                                .text("Create Chat"),
                        )
                        .child(
                            div()
                                .id("chats")
                                .attr("hx-get", "/chats")
                                .attr("hx-trigger", "reload-chats from:body delay:100ms")
                                .child(match chats(pool).await {
                                    Ok(chats) => chats.into(),
                                    Err(_) => html::text("Failed To Get Chats"),
                                }),
                        ),
                )
                .child(div().id("chat").class("grow"))
                .child(div().id("notifications"))])))
            .unwrap();
    }

    'a: {
        let Some((&"chats", segments)) = segments.split_first() else {
            break 'a;
        };

        if request.method() != Method::GET || !segments.is_empty() {
            break 'a;
        }

        return Response::builder()
            .body(Body::from(match chats(pool).await {
                Ok(chats) => chats.into().to_string(),
                Err(_) => {
                    return internal_server_error(html::text("Failed To Get Chats"));
                }
            }))
            .unwrap();
    }

    if let Some(response) = chat::handler(&pool, &mut request, &segments).await {
        return response;
    };

    if let Some(response) = create::handler(&pool, &request, &segments).await {
        return response;
    };

    Response::builder()
        .body(Body::from(document([
            h1().text(format!("Page {} not found", request.uri()))
        ])))
        .unwrap()
}
