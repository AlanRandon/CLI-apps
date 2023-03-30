use game_of_life_leptos::app::{App, AppProps};
use http::StatusCode;

#[cfg(feature = "ssr")]
use {
    axum::{body::Body, http::Request, response::IntoResponse, routing::post, Router},
    game_of_life_leptos::error_template::{AppError, ErrorTemplate, ErrorTemplateProps},
    leptos::{get_configuration, view},
    leptos_axum::{generate_route_list, LeptosRoutes},
    tower_http::services::ServeDir,
};

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Info).expect("couldn't initialize logging");

    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(|cx| view! { cx, <App/> }).await;

    let app = Router::new()
        // handle leptos server fns
        .route("/api/*fn_name", post(leptos_axum::handle_server_fns))
        // handle leptos routes
        .leptos_routes(leptos_options.clone(), routes, |cx| view! { cx, <App/> })
        .fallback({
            let root = leptos_options.site_root.clone();
            let handle_error = leptos_axum::render_app_async(
                leptos_options.clone(),
                move |cx| view! {cx, <ErrorTemplate error=AppError::NotFound/>},
            );

            let mut dir_service = ServeDir::new(root);
            move |request: Request<Body>| async move {
                let uri = request.uri().clone();

                {
                    let request = Request::builder()
                        .uri(uri.clone())
                        .body(Body::empty())
                        .unwrap();
                    if let Ok(response) = dir_service.try_call(request).await {
                        if response.status() == StatusCode::OK {
                            return response.into_response();
                        }
                    }
                }

                handle_error(Request::default()).await.into_response()
            }
        });

    log::info!("listening on http://{}", &addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
