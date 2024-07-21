use std::usize;

use askama_axum::Template;
use axum::{
    extract::{Path, State},
    http::Uri,
};

#[derive(Debug, Clone)]
pub struct Post {
    title: String,
    content: String,
}

impl Post {
    #[must_use]
    pub fn new(title: String, content: String) -> Self {
        Self { title, content }
    }
}

#[derive(Clone)]
pub struct AppState {
    posts: Vec<Post>,
}

impl AppState {
    #[must_use]
    pub fn new() -> Self {
        Self { posts: vec![] }
    }

    pub fn add_post(&mut self, post: Post) {
        self.posts.push(post);
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PageMeta {
    pub page_title: String,
    pub banner_title: String,
    pub path: String,
}

#[derive(Template)]
#[template(path = "blog_post.html")]
pub struct PostTemplate {
    meta: PageMeta,
    content: String,
}

#[allow(clippy::unused_async)]
pub async fn post_error(uri: Uri) -> PostTemplate {
    let meta = PageMeta {
        page_title: "Post not found | bogdan@web".to_string(),
        banner_title: "Post not found".to_string(),
        path: uri.to_string(),
    };
    PostTemplate {
        meta,
        content: "<p>Oops, there is nothing here.</p>".to_string(),
    }
}

/// Post handler.
///
/// # Panics
///
/// Panics if the blog post does not exist.
#[allow(clippy::unused_async)]
pub async fn post(
    Path(blog_id): Path<String>,
    State(state): State<AppState>,
    uri: Uri,
) -> PostTemplate {
    let blog_id = blog_id.parse::<usize>();
    if blog_id.is_err() {
        return post_error(uri).await;
    }
    let post = state.posts.get(blog_id.unwrap() - 1);
    if post.is_none() {
        return post_error(uri).await;
    }
    let post = post.unwrap();
    let title = post.title.to_lowercase();
    let meta = PageMeta {
        page_title: format!("{title} | bogdan@web").to_string(),
        banner_title: title.clone(),
        path: uri.to_string(),
    };
    PostTemplate {
        meta,
        content: post.content.clone(),
    }
}
