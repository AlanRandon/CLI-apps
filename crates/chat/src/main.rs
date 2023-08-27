use std::convert::Infallible;
use http::Uri;

mod handler;

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
                    let result = handler::handler(request, pool.clone()).await;

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

