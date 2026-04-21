use askama_axum::Template;
use axum::http::{StatusCode, Uri};
use axum::routing::get;
use axum::{extract::State, Router};
use clap::Parser;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use tower_http::services::ServeDir;

use blog::content::load_posts;
use blog::{compute_css_version, format_date, post, AppState, PageMeta};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'H', long, default_value = "0.0.0.0")]
    host: String,
    #[arg(short, long, default_value_t = 8080)]
    port: u16,
    #[arg(long)]
    drafts: bool,
    #[arg(long, default_value = "./blog_posts")]
    posts_dir: PathBuf,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    tracing_subscriber::fmt::init();

    let posts = load_posts(&args.posts_dir, args.drafts);
    tracing::info!("loaded {} post(s) from {:?}", posts.len(), args.posts_dir);
    let css_version = compute_css_version(std::path::Path::new("./dist/css/output.css"));
    tracing::info!("css version {}", css_version);
    let state = AppState::from_posts(posts).with_css_version(css_version);

    let dist_service = ServeDir::new("./dist");

    let app = Router::new()
        .nest_service("/dist", dist_service)
        .route("/", get(home))
        .route("/blog", get(blog_index))
        .route("/post/:slug", get(post))
        .route("/resume", get(resume))
        .route("/rss.xml", get(rss))
        .fallback(not_found)
        .with_state(state);

    let ip_addr: IpAddr = args.host.parse().expect("invalid host");
    let addr = SocketAddr::from((ip_addr, args.port));
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind failed");
    axum::serve(listener, app).await.expect("serve failed");
}

async fn home(uri: Uri, State(state): State<AppState>) -> Home {
    let meta = PageMeta {
        page_title: "bogdan floris".to_string(),
        banner_title: "bogdan floris".to_string(),
        path: uri.to_string(),
        css_version: state.css_version.clone(),
    };
    let recent: Vec<RecentPost> = state
        .latest(3)
        .into_iter()
        .map(|p| RecentPost {
            title: p.title.clone(),
            slug: p.slug.clone(),
            date: format_date(&p.date),
            tags: p.tags.clone(),
        })
        .collect();
    Home { meta, recent }
}

async fn blog_index(uri: Uri, State(state): State<AppState>) -> BlogIndex {
    let meta = PageMeta {
        page_title: "blog | bogdan floris".to_string(),
        banner_title: "blog".to_string(),
        path: uri.to_string(),
        css_version: state.css_version.clone(),
    };
    let posts: Vec<RecentPost> = state
        .all()
        .into_iter()
        .map(|p| RecentPost {
            title: p.title.clone(),
            slug: p.slug.clone(),
            date: format_date(&p.date),
            tags: p.tags.clone(),
        })
        .collect();
    BlogIndex { meta, posts }
}

async fn resume(uri: Uri, State(state): State<AppState>) -> Resume {
    let meta = PageMeta {
        page_title: "resume | bogdan floris".to_string(),
        banner_title: "resume".to_string(),
        path: uri.to_string(),
        css_version: state.css_version.clone(),
    };
    Resume { meta }
}

async fn not_found(uri: Uri, State(state): State<AppState>) -> (StatusCode, NotFound) {
    let meta = PageMeta {
        page_title: "not found | bogdan floris".to_string(),
        banner_title: "not found".to_string(),
        path: uri.to_string(),
        css_version: state.css_version.clone(),
    };
    (StatusCode::NOT_FOUND, NotFound { meta })
}

async fn rss(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    let xml = blog::rss::build_feed(&state.all());
    (
        [(
            axum::http::header::CONTENT_TYPE,
            "application/rss+xml; charset=utf-8",
        )],
        xml,
    )
}

#[derive(Clone)]
pub struct RecentPost {
    pub title: String,
    pub slug: String,
    pub date: String,
    pub tags: Vec<String>,
}

#[derive(Template)]
#[template(path = "index.html")]
struct Home {
    meta: PageMeta,
    recent: Vec<RecentPost>,
}

#[derive(Template)]
#[template(path = "blog.html")]
struct BlogIndex {
    meta: PageMeta,
    posts: Vec<RecentPost>,
}

#[derive(Template)]
#[template(path = "resume.html")]
struct Resume {
    meta: PageMeta,
}

#[derive(Template)]
#[template(path = "404.html")]
struct NotFound {
    meta: PageMeta,
}
