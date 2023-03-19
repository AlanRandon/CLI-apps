use full_stack_web::app::{App, AppProps};

#[cfg(feature = "ssr")]
use {
    axum::{
        body::Body, extract::Extension, http::Request, response::IntoResponse, routing::post,
        Router,
    },
    full_stack_web::error_template::{AppError, ErrorTemplate, ErrorTemplateProps},
    leptos::{get_configuration, view},
    leptos_axum::{generate_route_list, LeptosRoutes},
    std::sync::Arc,
    tower::{ServiceBuilder, ServiceExt},
    tower_http::services::ServeDir,
};

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::{
        error_handling::HandleErrorLayer,
        handler::HandlerService,
        routing::{get, get_service, MethodRouter},
    };
    use leptos::LeptosOptions;
    use tower::{service_fn, Service};

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
            let handle_error = leptos_axum::render_app_to_stream(
                leptos_options.clone(),
                move |cx| view! {cx, <ErrorTemplate error=AppError::NotFound/>},
            );

            let dir_server = ServeDir::new(root).fallback(service_fn(move |request| async move {
                Ok(handle_error(request).await)
            }));
            get_service(dir_server)
        });

    log::info!("listening on http://{}", &addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
