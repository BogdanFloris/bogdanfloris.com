use askama_axum::Template;
use axum::http::Uri;
use axum::routing::get;
use axum::{debug_handler, Router};
use clap::Parser;
use std::fs;
use std::net::{IpAddr, SocketAddr};
use tower_http::services::ServeDir;

use blog::{post, AppState, PageMeta, Post};

static BLOG_POSTS_DIR: &str = "./blog_posts";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'H', long, default_value = "0.0.0.0")]
    host: String,

    #[arg(short, long, default_value_t = 8080)]
    port: u16,
}

fn add_posts(state: &mut AppState) {
    let post_one_content =
        fs::read_to_string(format!("{BLOG_POSTS_DIR}/personal_website_blog.html")).unwrap();
    state.add_post(Post::new("Post #0".to_string(), post_one_content));
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    tracing_subscriber::fmt::init();
    let dist_service = ServeDir::new("./dist");
    let mut state = AppState::default();
    add_posts(&mut state);

    let app = Router::new()
        .nest_service("/dist", dist_service)
        .route("/", get(root))
        .route("/about", get(about))
        .route("/blog", get(blog))
        .route("/post/:id", get(post))
        .route("/resume", get(resume))
        .with_state(state)
        .fallback(not_found);

    let ip_addr: IpAddr = args.host.parse().unwrap();

    let addr = SocketAddr::from((ip_addr, args.port));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[debug_handler]
async fn root(uri: Uri) -> Index {
    let meta = PageMeta {
        page_title: "hello! | bogdan@web".to_string(),
        banner_title: "bogdan@web>".to_string(),
        path: uri.to_string(),
    };
    Index { meta }
}

async fn not_found(uri: Uri) -> NotFound {
    let meta = PageMeta {
        page_title: "¯\\_(ツ)_/¯ | bogdan@web".to_string(),
        banner_title: "not found :(".to_string(),
        path: uri.to_string(),
    };
    NotFound { meta }
}

async fn about(uri: Uri) -> About {
    let meta = PageMeta {
        page_title: "about | bogdan@web".to_string(),
        banner_title: "about me".to_string(),
        path: uri.to_string(),
    };
    About { meta }
}

async fn blog(uri: Uri) -> Blog {
    let meta = PageMeta {
        page_title: "blog | bogdan@web".to_string(),
        banner_title: "blog".to_string(),
        path: uri.to_string(),
    };
    Blog { meta }
}

async fn resume(uri: Uri) -> Resume {
    let meta = PageMeta {
        page_title: "resume | bogdan@web".to_string(),
        banner_title: "resume".to_string(),
        path: uri.to_string(),
    };
    Resume { meta }
}

#[derive(Template)]
#[template(path = "index.html")]
struct Index {
    meta: PageMeta,
}

#[derive(Template)]
#[template(path = "404.html")]
struct NotFound {
    meta: PageMeta,
}

#[derive(Template)]
#[template(path = "about.html")]
struct About {
    meta: PageMeta,
}

#[derive(Template)]
#[template(path = "blog.html")]
struct Blog {
    meta: PageMeta,
}

#[derive(Template)]
#[template(path = "resume.html")]
struct Resume {
    meta: PageMeta,
}
