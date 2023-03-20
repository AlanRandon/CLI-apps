use full_stack_web::app::{App, AppProps};
use http::{HeaderValue, StatusCode};
use std::{future::Future, path::Path, pin::Pin};

#[cfg(feature = "ssr")]
use {
    axum::{
        body::{Body, HttpBody},
        error_handling::HandleErrorLayer,
        extract::Extension,
        http::Request,
        response::{IntoResponse, Response},
        routing::post,
        routing::{get, get_service, MethodRouter},
        BoxError, RequestExt, Router,
    },
    full_stack_web::error_template::{AppError, ErrorTemplate, ErrorTemplateProps},
    leptos::{get_configuration, view, LeptosOptions},
    leptos_axum::{generate_route_list, LeptosRoutes},
    std::sync::Arc,
    tower::{service_fn, util::ServiceFn, Service, ServiceBuilder, ServiceExt},
    tower_http::services::ServeDir,
};

#[cfg(feature = "ssr")]
fn serve_dir(
    root: &str,
    fallback: impl Fn(Request<Body>) -> Pin<Box<dyn Future<Output = Response>>>,
) -> ServiceFn<impl Fn(Request<Body>) -> Pin<Box<dyn Future<Output = Response>>>> {
    service_fn(move |request: Request<Body>| {
        Box::<dyn Future<Output = Response>>::pin(async move {
            let uri = request.uri().clone();
            let Ok(body) = tokio::fs::read_to_string(&format!("{root}{uri}")).await else {
                return fallback(request).await;
            };
            Response::builder()
                .header(
                    "Content-Type",
                    HeaderValue::from_str(
                        mime_guess::from_path(Path::new(uri.path()))
                            .first_raw()
                            .unwrap_or("application/octet-stream"),
                    )
                    .unwrap(),
                )
                .body(body)
        })
    })
}

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

                match dir_service.try_call(request).await {
                    Ok(response) => {
                        dbg!(uri);
                        response.into_response()
                    }
                    _ => handle_error(Request::default()).await.into_response(),
                }
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
