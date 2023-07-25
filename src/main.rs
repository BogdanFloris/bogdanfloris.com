use askama::Template;
use axum::http::Uri;
use axum::routing::get;
use axum::{debug_handler, Router};
use std::net::SocketAddr;
use tower_http::services::ServeDir;

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
        .fallback(not_found);

    // run the app
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[debug_handler]
async fn root(uri: Uri) -> Index<'static> {
    Index {
        page_title: "hello! | bogdan@web",
        banner_title: "bogdan@web>",
        path: uri.to_string(),
    }
}

async fn not_found(uri: Uri) -> NotFound<'static> {
    NotFound {
        page_title: "¯\\_(ツ)_/¯ | bogdan@web",
        banner_title: "not found :(",
        path: uri.to_string(),
    }
}

#[derive(Template)]
#[template(path = "index.html")]
struct Index<'a> {
    page_title: &'a str,
    banner_title: &'a str,
    path: String,
}

#[derive(Template)]
#[template(path = "404.html")]
struct NotFound<'a> {
    page_title: &'a str,
    banner_title: &'a str,
    path: String,
}
