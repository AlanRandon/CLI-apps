use leptos::*;
use thiserror::Error;
use http::status::StatusCode;

#[derive(Clone, Debug, Error)]
pub enum AppError {
    #[error("Not Found")]
    NotFound,
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound => StatusCode::NOT_FOUND,
        }
    }
}

#[component]
pub fn ErrorTemplate(cx: Scope, error: AppError) -> impl IntoView {
    let status_code = error.status_code();

    #[cfg(feature = "ssr")]
    {
        let response = use_context::<leptos_axum::ResponseOptions>(cx);
        if let Some(response) = response {
            response.set_status(status_code);
        }
    }

    let error_string = error.to_string();

    view! {
        cx,
        <h2>"Error " {status_code.to_string()}</h2>
        <p>{error_string}</p>
    }
}
