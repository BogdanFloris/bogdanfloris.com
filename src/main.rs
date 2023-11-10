use askama::Template;
use axum::http::{StatusCode, Uri};
use axum::routing::get;
use axum::{debug_handler, Router};
use std::net::SocketAddr;
use tower_http::services::ServeDir;

struct PageMeta<'a> {
    page_title: &'a str,
    banner_title: &'a str,
    path: String,
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // add the dist folder
    let dist_service = ServeDir::new("./dist");

    // create the app
    let app = Router::new()
        .nest_service("/dist", dist_service)
        .route("/", get(root))
        .route("/health", get(|| async { StatusCode::NO_CONTENT }))
        .route("/about", get(about))
        .fallback(not_found);

    // run the app
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[debug_handler]
async fn root(uri: Uri) -> Index<'static> {
    let meta = PageMeta {
        page_title: "hello! | bogdan@web",
        banner_title: "bogdan@web>",
        path: uri.to_string(),
    };
    Index { meta }
}

async fn not_found(uri: Uri) -> NotFound<'static> {
    let meta = PageMeta {
        page_title: "¯\\_(ツ)_/¯ | bogdan@web",
        banner_title: "not found :(",
        path: uri.to_string(),
    };
    NotFound { meta }
}

async fn about(uri: Uri) -> About<'static> {
    let meta = PageMeta {
        page_title: "about | bogdan@web",
        banner_title: "about me",
        path: uri.to_string(),
    };
    About { meta }
}

#[derive(Template)]
#[template(path = "index.html")]
struct Index<'a> {
    meta: PageMeta<'a>,
}

#[derive(Template)]
#[template(path = "404.html")]
struct NotFound<'a> {
    meta: PageMeta<'a>,
}

#[derive(Template)]
#[template(path = "about.html")]
struct About<'a> {
    meta: PageMeta<'a>,
}
