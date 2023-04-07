use warp::Filter;

mod app;

#[tokio::main]
async fn main() {
    println!("serving on http://localhost:3030");

    warp::serve(
        warp::path::end()
            .map(|| {
                app::app_string(app::AppProps {
                    page: app::Page::Index,
                })
            })
            .map(warp::reply::html)
            .or(warp::filters::fs::dir(".dist"))
            .or(warp::any()
                .and(warp::path::full())
                .map(|uri: warp::path::FullPath| {
                    app::app_string(app::AppProps {
                        page: app::Page::NotFound {
                            uri: uri.as_str().to_string(),
                        },
                    })
                })
                .map(warp::reply::html)),
    )
    .run(([127, 0, 0, 1], 3030))
    .await;
}
