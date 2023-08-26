use html_builder::prelude::*;
use http::{Method, Request, Response, Uri};
use hyper::Body;
use sqlx::{Pool, Sqlite};
use std::convert::Infallible;

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

#[tokio::main]
async fn main() {
    use hyper::service::{make_service_fn, service_fn};

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite://data.db")
        .await
        .unwrap();

    sqlx::migrate!().run(&pool).await.unwrap();

    let addr = format!(
        "127.0.0.1:{}",
        std::env::var("PORT").unwrap_or("8000".to_string())
    )
    .parse()
    .unwrap();

    println!("Listening on http://{addr}");

    hyper::Server::bind(&addr)
        .serve(make_service_fn(move |_connection| {
            let pool = pool.clone();
            let service = service_fn(move |request| {
                let pool = pool.clone();
                async move {
                    let pool = pool.clone();
                    let result = handler(request, pool.clone()).await;

                    Ok::<_, Infallible>(result)
                }
            });
            async move { Ok::<_, Infallible>(service) }
        }))
        .await
        .unwrap();
}

trait UriExt {
    fn segments(&self) -> Vec<&str>;
}

impl UriExt for Uri {
    fn segments(&self) -> Vec<&str> {
        let mut segments = Vec::new();
        for segment in self.path().split('/') {
            match segment {
                "." | "" => {}
                ".." => {
                    segments.pop();
                }
                segment => segments.push(segment),
            }
        }
        segments
    }
}

async fn handler(request: Request<Body>, pool: Pool<Sqlite>) -> Response<Body> {
    let segments = request.uri().segments();

    if segments.is_empty() && request.method() == Method::GET {
        return Response::builder()
            .body(Body::from(document([button()
                .attr("hx-post", "/create")
                .text("Create Chat")])))
            .unwrap();
    }

    match segments.split_first() {
        Some((&"create", segments)) if segments.is_empty() && request.method() == Method::POST => {
            // let Ok(record) = sqlx::query!(
            //     "INSERT INTO chats (id, name) VALUES (NULL, ?) RETURNING id",
            //     "New Chat"
            // )
            // .fetch_one(&pool)
            // .await else {
            //     todo!();
            // };

            // return Response::builder()
            //     .body(Body::from(format!("Chat {} created", record.id)))
            //     .unwrap();
        }
        _ => {}
    }

    Response::builder()
        .body(Body::from(document([
            h1().text(format!("Page {} not found", request.uri()))
        ])))
        .unwrap()
}
