pub mod content;
pub mod rss;

use askama_axum::Template;
use axum::{
    extract::{Path, State},
    http::{StatusCode, Uri},
    response::IntoResponse,
};
use chrono::NaiveDate;
use std::collections::HashMap;

pub use content::Post;

#[derive(Clone, Default)]
pub struct AppState {
    posts_by_slug: HashMap<String, Post>,
    slugs_newest_first: Vec<String>,
}

impl AppState {
    #[must_use]
    pub fn from_posts(posts: Vec<Post>) -> Self {
        let slugs_newest_first: Vec<String> = posts.iter().map(|p| p.slug.clone()).collect();
        let mut posts_by_slug = HashMap::with_capacity(posts.len());
        for post in posts {
            posts_by_slug.insert(post.slug.clone(), post);
        }
        Self { posts_by_slug, slugs_newest_first }
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

    /// Returns the `n` most recent posts.
    #[must_use]
    pub fn latest(&self, n: usize) -> Vec<&Post> {
        self.slugs_newest_first
            .iter()
            .filter_map(|s| self.posts_by_slug.get(s))
            .take(n)
            .collect()
    }
}

pub struct PageMeta {
    pub page_title: String,
    pub banner_title: String,
    pub path: String,
}

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
}

pub async fn post(
    Path(slug): Path<String>,
    State(state): State<AppState>,
    uri: Uri,
) -> axum::response::Response {
    match state.get(&slug) {
        Some(post) => {
            let meta = PageMeta {
                page_title: format!("{} | bogdan floris", post.title),
                banner_title: post.title.clone(),
                path: uri.to_string(),
            };
            PostTemplate {
                meta,
                title: post.title.clone(),
                date: format_date(&post.date),
                tags: post.tags.clone(),
                rendered_html: post.rendered_html.clone(),
            }
            .into_response()
        }
        None => (StatusCode::NOT_FOUND, "post not found").into_response(),
    }
}
