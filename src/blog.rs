pub mod content;
pub mod rss;

use askama_axum::Template;
use axum::{
    extract::{Path, State},
    http::{StatusCode, Uri},
    response::{IntoResponse, Redirect},
};
use chrono::NaiveDate;
use std::collections::HashMap;
use std::sync::Arc;

pub use content::{Post, Scribble, TocEntry, Update};

/// Short hex hash of the CSS bundle, used to cache-bust the `<link>` in `base.html`.
/// Returns `"dev"` if the file can't be read, so local dev still renders.
#[must_use]
pub fn compute_css_version(path: &std::path::Path) -> String {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let Ok(bytes) = std::fs::read(path) else {
        return "dev".to_string();
    };
    let mut h = DefaultHasher::new();
    bytes.hash(&mut h);
    format!("{:016x}", h.finish())[..8].to_string()
}

#[derive(Clone, Default)]
pub struct AppState {
    posts_by_slug: HashMap<String, Post>,
    slugs_newest_first: Vec<String>,
    scribbles: Vec<Scribble>,
    pub css_version: String,
}

impl AppState {
    #[must_use]
    pub fn from_posts(posts: Vec<Post>) -> Self {
        let slugs_newest_first: Vec<String> = posts.iter().map(|p| p.slug.clone()).collect();
        let mut posts_by_slug = HashMap::with_capacity(posts.len());
        for post in posts {
            posts_by_slug.insert(post.slug.clone(), post);
        }
        Self {
            posts_by_slug,
            slugs_newest_first,
            scribbles: Vec::new(),
            css_version: String::new(),
        }
    }

    #[must_use]
    pub fn with_css_version(mut self, v: String) -> Self {
        self.css_version = v;
        self
    }

    #[must_use]
    pub fn with_scribbles(mut self, scribbles: Vec<Scribble>) -> Self {
        self.scribbles = scribbles;
        self
    }

    #[must_use]
    pub fn get(&self, slug: &str) -> Option<&Post> {
        self.posts_by_slug.get(slug)
    }

    /// Returns all posts, newest-first.
    #[must_use]
    pub fn all(&self) -> Vec<&Post> {
        self.slugs_newest_first
            .iter()
            .filter_map(|s| self.posts_by_slug.get(s))
            .collect()
    }

    /// Returns the scribbles, newest-first.
    #[must_use]
    pub fn scribbles(&self) -> &[Scribble] {
        &self.scribbles
    }
}

pub struct PageMeta {
    pub page_title: String,
    pub banner_title: String,
    pub path: String,
    pub css_version: String,
}

#[must_use]
pub fn format_date(date: &NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

#[derive(Template)]
#[template(path = "blog_post.html")]
pub struct PostTemplate {
    pub meta: PageMeta,
    pub title: String,
    pub date: String,
    pub tags: Vec<String>,
    pub rendered_html: String,
    pub toc: Vec<TocEntry>,
    pub updates: Vec<Update>,
    /// The newest update's date, or empty when it adds nothing to `date`.
    pub updated: String,
}

#[derive(Template)]
#[template(path = "404.html")]
pub struct NotFound {
    pub meta: PageMeta,
}

#[must_use]
pub fn not_found_page(state: &AppState, uri: &Uri) -> axum::response::Response {
    let meta = PageMeta {
        page_title: "not found | bogdan floris".to_string(),
        banner_title: "not found".to_string(),
        path: uri.to_string(),
        css_version: state.css_version.clone(),
    };
    (StatusCode::NOT_FOUND, NotFound { meta }).into_response()
}

/// Serves `/essays/<slug>`, and redirects the old `/post/<slug>` there.
#[allow(clippy::unused_async)]
pub async fn post(
    Path(slug): Path<String>,
    State(state): State<Arc<AppState>>,
    uri: Uri,
) -> axum::response::Response {
    let Some(post) = state.get(&slug) else {
        return not_found_page(&state, &uri);
    };
    if let Some(encoded_slug) = uri.path().strip_prefix("/post/") {
        return Redirect::permanent(&format!("/essays/{encoded_slug}")).into_response();
    }
    let meta = PageMeta {
        page_title: format!("{} | bogdan floris", post.title),
        banner_title: post.title.clone(),
        path: uri.to_string(),
        css_version: state.css_version.clone(),
    };
    let date = format_date(&post.date);
    let updated = post
        .updates
        .iter()
        .map(|u| u.date.as_str())
        .filter(|d| *d > date.as_str())
        .max()
        .unwrap_or_default()
        .to_string();
    PostTemplate {
        meta,
        title: post.title.clone(),
        date,
        tags: post.tags.clone(),
        rendered_html: post.rendered_html.clone(),
        toc: post.toc.clone(),
        updates: post.updates.clone(),
        updated,
    }
    .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::header::LOCATION;

    fn sample() -> Post {
        Post {
            title: "Queues".to_string(),
            slug: "queues".to_string(),
            date: NaiveDate::from_ymd_opt(2026, 11, 3).unwrap(),
            tags: vec![],
            rendered_html: "<p>body</p>".to_string(),
            draft: false,
            toc: vec![],
            updates: vec![],
        }
    }

    fn state() -> Arc<AppState> {
        Arc::new(AppState::from_posts(vec![sample()]))
    }

    async fn get(state: Arc<AppState>, path: &str) -> axum::response::Response {
        let slug = path.rsplit('/').next().unwrap().to_string();
        post(Path(slug), State(state), path.parse().unwrap()).await
    }

    #[tokio::test]
    async fn serves_an_essay() {
        let res = get(state(), "/essays/queues").await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn old_post_urls_redirect_to_essays() {
        let res = get(state(), "/post/queues").await;
        assert_eq!(res.status(), StatusCode::PERMANENT_REDIRECT);
        assert_eq!(res.headers()[LOCATION], "/essays/queues");
    }

    #[tokio::test]
    async fn unknown_slugs_get_the_404_page() {
        let res = get(state(), "/essays/nope").await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn shows_the_newest_update_after_the_date() {
        let update = |date: &str, text: &str| Update {
            date: date.to_string(),
            text: text.to_string(),
        };
        let mut post = sample();
        post.updates = vec![
            update("2026-11-01", "first draft"),
            update("2026-11-05", "tightened"),
        ];
        let state = Arc::new(AppState::from_posts(vec![post]));
        let res = get(state, "/essays/queues").await;
        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("updated 2026-11-05"));
        assert!(html.contains("first draft"));
    }
}
