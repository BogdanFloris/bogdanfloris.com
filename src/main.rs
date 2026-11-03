use askama_axum::Template;
use axum::http::Uri;
use axum::response::{Redirect, Response};
use axum::routing::get;
use axum::{extract::State, Router};
use clap::Parser;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::services::ServeDir;

use blog::content::{load_posts, load_scribbles};
use blog::{
    compute_css_version, format_date, not_found_page, post, AppState, PageMeta, Post, Scribble,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'H', long, default_value = "0.0.0.0")]
    host: String,
    #[arg(short, long, default_value_t = 8080)]
    port: u16,
    #[arg(long)]
    drafts: bool,
    #[arg(long, default_value = "./content")]
    content_dir: PathBuf,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    tracing_subscriber::fmt::init();

    let posts = load_posts(&args.content_dir, args.drafts);
    let scribbles = load_scribbles(&args.content_dir);
    tracing::info!(
        "loaded {} page(s) and {} scribble(s) from {:?}",
        posts.len(),
        scribbles.len(),
        args.content_dir
    );
    let css_version = compute_css_version(std::path::Path::new("./dist/css/output.css"));
    tracing::info!("css version {}", css_version);
    let state = AppState::from_posts(posts)
        .with_scribbles(scribbles)
        .with_css_version(css_version);

    let dist_service = ServeDir::new("./dist");

    let app = Router::new()
        .nest_service("/dist", dist_service)
        .route("/", get(home))
        .route("/essays", get(essays_index))
        .route("/essays/:slug", get(post))
        .route("/scribbles", get(scribbles_page))
        .route("/post/:slug", get(post))
        .route("/blog", get(|| async { Redirect::permanent("/essays") }))
        .route("/resume", get(resume))
        .route("/rss.xml", get(rss))
        .fallback(not_found)
        .with_state(Arc::new(state));

    let ip_addr: IpAddr = args.host.parse().expect("invalid host");
    let addr = SocketAddr::from((ip_addr, args.port));
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind failed");
    axum::serve(listener, app).await.expect("serve failed");
}

async fn home(uri: Uri, State(state): State<Arc<AppState>>) -> Home {
    let meta = PageMeta {
        page_title: "bogdan floris".to_string(),
        banner_title: "bogdan floris".to_string(),
        path: uri.to_string(),
        css_version: state.css_version.clone(),
    };
    Home {
        meta,
        posts: rows(&state.all()),
        latest_scribble: state.scribbles().first().cloned(),
    }
}

async fn essays_index(uri: Uri, State(state): State<Arc<AppState>>) -> EssaysIndex {
    let meta = PageMeta {
        page_title: "essays | bogdan floris".to_string(),
        banner_title: "essays".to_string(),
        path: uri.to_string(),
        css_version: state.css_version.clone(),
    };
    EssaysIndex {
        meta,
        posts: rows(&state.all()),
    }
}

async fn scribbles_page(uri: Uri, State(state): State<Arc<AppState>>) -> ScribblesPage {
    let meta = PageMeta {
        page_title: "scribbles | bogdan floris".to_string(),
        banner_title: "scribbles".to_string(),
        path: uri.to_string(),
        css_version: state.css_version.clone(),
    };
    ScribblesPage {
        meta,
        scribbles: state.scribbles().to_vec(),
    }
}

async fn resume(uri: Uri, State(state): State<Arc<AppState>>) -> Resume {
    let meta = PageMeta {
        page_title: "resume | bogdan floris".to_string(),
        banner_title: "resume".to_string(),
        path: uri.to_string(),
        css_version: state.css_version.clone(),
    };
    Resume { meta }
}

async fn not_found(uri: Uri, State(state): State<Arc<AppState>>) -> Response {
    not_found_page(&state, &uri)
}

async fn rss(State(state): State<Arc<AppState>>) -> impl axum::response::IntoResponse {
    let xml = blog::rss::build_feed(&state.all());
    (
        [(
            axum::http::header::CONTENT_TYPE,
            "application/rss+xml; charset=utf-8",
        )],
        xml,
    )
}

fn rows(posts: &[&Post]) -> Vec<PostRow> {
    posts
        .iter()
        .map(|p| PostRow {
            title: p.title.clone(),
            url: p.url(),
            date: format_date(&p.date),
            tags: p.tags.clone(),
        })
        .collect()
}

#[derive(Clone)]
pub struct PostRow {
    pub title: String,
    pub url: String,
    pub date: String,
    pub tags: Vec<String>,
}

#[derive(Template)]
#[template(path = "index.html")]
struct Home {
    meta: PageMeta,
    posts: Vec<PostRow>,
    latest_scribble: Option<Scribble>,
}

#[derive(Template)]
#[template(path = "essays.html")]
struct EssaysIndex {
    meta: PageMeta,
    posts: Vec<PostRow>,
}

#[derive(Template)]
#[template(path = "scribbles.html")]
struct ScribblesPage {
    meta: PageMeta,
    scribbles: Vec<Scribble>,
}

#[derive(Template)]
#[template(path = "resume.html")]
struct Resume {
    meta: PageMeta,
}
