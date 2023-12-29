use crate::prelude::*;

struct Context;
type Request<'req> = crate::Request<'req, (), Context>;

// Static path segments can be matched
#[get("hello")]
fn hello(_: &Request<'req>) -> http::Response<String> {
    http::Response::new("Hello World!".to_string())
}

// Dynamic path segments may be bound
#[get("hello" / name)]
fn hello_name(_: &Request<'req>, name: &str) -> http::Response<String> {
    http::Response::new(format!("Hello {name}!"))
}

// There may be multiple dynamic path segments
#[get("add" / a / b)]
fn add(_: &Request<'req>, a: &str, b: &str) -> http::Response<String> {
    str::parse::<u32>(a)
        .and_then(|a| Ok(a + str::parse::<u32>(b)?))
        .map(|result| http::Response::new(format!("{result}")))
        .unwrap_or_else(|_| {
            http::Response::builder()
                .status(http::StatusCode::BAD_REQUEST)
                .body(format!("Cannot add {a} to {b}"))
                .unwrap()
        })
}

// Globs can match all unmatched segments in a path
// The request recieved has the segments matched, except from those in the glob
// This allows routers to be nested
#[get("api" / version / *path)]
fn api(request: &Request<'req>, version: &str) -> http::Response<String> {
    if version == "v1" {
        let router = routes![add, hello];
        router(request).unwrap_or_else(|| not_found(request))
    } else {
        http::Response::builder()
            .status(http::StatusCode::NOT_FOUND)
            .body(format!("Version {version} of the api does not exist"))
            .unwrap()
    }
}

fn not_found(request: &Request) -> http::Response<String> {
    http::Response::builder()
        .status(http::StatusCode::NOT_FOUND)
        .body(format!("Not Found {}", request.request.uri().path()))
        .unwrap()
}

#[test]
fn router_works() {
    let routes = routes![hello, hello_name, api];
    for (uri, expected_body) in [
        ("/hello", "Hello World!"),
        ("/hello/", "Hello World!"),
        ("/hello/Alice", "Hello Alice!"),
        ("/hello/Bob", "Hello Bob!"),
        ("/api/v1/add/3/4", "7"),
        ("/not-real", "Not Found /not-real"),
    ] {
        let request = http::Request::builder().uri(uri).body(()).unwrap();
        let request = Request::from_http_with_context(&request, &Context);

        let response = routes(&request).unwrap_or_else(|| not_found(&request));
        let body = response.body().as_str();
        assert_eq!(expected_body, body);
    }
}
