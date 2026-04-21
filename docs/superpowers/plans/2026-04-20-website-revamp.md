# Website Revamp Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebuild bogdanfloris.com as a Notebook-style Gruvbox-light reading site with a markdown-driven blog pipeline.

**Architecture:** Keep the Rust + Axum + Askama + Tailwind stack, but modernize: upgrade Axum 0.6 → latest, replace hand-written HTML blog posts with markdown + YAML frontmatter parsed at startup (`pulldown-cmark` + `syntect` for syntax highlighting), split pure content logic into `src/content.rs` for testability, drop dark mode entirely, restyle under a calm serif-body/mono-metadata "Notebook" aesthetic.

**Tech Stack:** Rust, Axum (latest), Askama 0.12+, Tailwind CSS, pulldown-cmark, syntect, serde_yaml.

**Spec:** `docs/superpowers/specs/2026-04-20-website-revamp-design.md`

**Commit style:** This repo uses `jj` (not git). Every commit step uses `jj commit <paths> -m "..."` scoped to specific paths so unrelated working-copy changes don't get swept in.

---

## File Structure

**New files:**
- `src/content.rs` — pure content logic: frontmatter parsing, slug derivation, markdown+syntect rendering, post loading. All testable without a server.
- `src/rss.rs` — RSS 2.0 feed generation. Pure function, testable.
- `templates/partials/letterhead.html` — replaces the current banner.
- `templates/partials/footer.html` — site footer with RSS link.
- `blog_posts/personal-website-rust.md` — migrated post.

**Modified files:**
- `Cargo.toml` — dep upgrades, new deps.
- `src/main.rs` — new routes (slug-based), `--drafts` flag, merged home/about handler, drop `/projects`.
- `src/blog.rs` — use `content` module; new `Post` struct with date/tags/slug/rendered_html; `AppState` with `HashMap<slug, Post>` + sorted slug list.
- `tailwind.config.js` — collapse light/dark color pairs to single values, drop `darkMode: "class"`, keep font config.
- `src/style.css` — Notebook-era component classes (`.post-list`, `.post-row`, `.letterhead`, `.tag`, etc.), drop `.animate-blink` and terminal bits.
- `templates/base.html` — load Source Serif 4 + JetBrains Mono from Google Fonts, drop theme-toggle script, drop dark classes on `<body>`.
- `templates/index.html` — landing card (bio + recent posts).
- `templates/about.html` — longer biography content (current home content absorbs here).
- `templates/blog.html` — lab-notebook post list (date · title · tags).
- `templates/blog_post.html` — new article layout.
- `templates/resume.html`, `templates/404.html` — restyle under Notebook.
- `templates/partials/nav.html` — drop `/projects`.

**Deleted files:**
- `templates/partials/banner.html` (replaced by letterhead).
- `templates/partials/theme-button.html` (no dark mode).
- `blog_posts/personal_website_blog.html` (migrated to markdown).

---

## Task 1: Upgrade Axum and related dependencies

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/main.rs:30-55` (Server::bind → axum::serve)
- Modify: `src/blog.rs` (check askama_axum integration still works)

- [ ] **Step 1: Read the current Cargo.toml to confirm starting state**

Run: `cargo tree --depth 0`
Expected: shows `axum v0.6.x`, `askama v0.12.x`, `askama_axum v0.3.x`, `tower-http v0.4.x`.

- [ ] **Step 2: Upgrade Axum, askama_axum, tower-http, tokio, and other deps via cargo add**

Run:
```bash
cargo add axum --features macros
cargo add askama_axum
cargo add tower-http --features fs
cargo add tokio --features full
cargo add tracing tracing-subscriber clap --features derive chrono serde --features derive
```

Open `Cargo.toml` and confirm deps updated (versions will be whatever is current on crates.io). If askama had a major bump to 0.13 that removed `with-axum`, adjust the `askama` feature list accordingly — `askama_axum` may now be standalone.

- [ ] **Step 3: Run `cargo check` and fix breakage**

Run: `cargo check`
Expected: compile errors around `axum::Server::bind` (removed in 0.7).

Fix `src/main.rs` — replace the server bootstrap block:

Old (around `src/main.rs:50-55`):
```rust
let addr = SocketAddr::from((ip_addr, args.port));
tracing::info!("listening on {}", addr);
axum::Server::bind(&addr)
    .serve(app.into_make_service())
    .await
    .unwrap();
```

New:
```rust
let addr = SocketAddr::from((ip_addr, args.port));
tracing::info!("listening on {}", addr);
let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
axum::serve(listener, app).await.unwrap();
```

If `askama_axum` API changed, the `#[derive(Template)]` structs may need to implement `IntoResponse` explicitly via an `askama_axum::IntoResponse` re-export or via `into_response()`. Check the latest askama_axum docs if errors mention response conversion.

- [ ] **Step 4: Run `cargo check` again — must pass**

Run: `cargo check`
Expected: compiles cleanly, warnings OK.

- [ ] **Step 5: Run the server and hit `/` to verify nothing is broken**

Run (in background): `cargo run -- -p 3000`
Then: `curl -sS http://localhost:3000/ | head -30`
Expected: HTML response starts with `<!DOCTYPE html>` and contains "Hello, there!".

Kill the server.

- [ ] **Step 6: Commit**

```bash
jj commit Cargo.toml Cargo.lock src/main.rs src/blog.rs -m "chore: upgrade axum and related deps to latest stable"
```

---

## Task 2: Add markdown pipeline dependencies

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Add the new runtime deps**

Run:
```bash
cargo add pulldown-cmark
cargo add syntect --no-default-features --features default-fancy
cargo add serde_yaml
cargo add quick-xml --features serialize
```

`syntect` with `default-fancy` gives regex-based syntax detection plus HTML output without pulling the onig C dependency. `quick-xml` generates the RSS XML.

- [ ] **Step 2: Verify compilation still passes**

Run: `cargo check`
Expected: compiles cleanly.

- [ ] **Step 3: Commit**

```bash
jj commit Cargo.toml Cargo.lock -m "chore: add markdown, syntect, yaml, and xml deps"
```

---

## Task 3: Create `src/content.rs` with frontmatter parsing + slug derivation (TDD)

**Files:**
- Create: `src/content.rs`
- Modify: `src/blog.rs` (declare `pub mod content;`)

- [ ] **Step 1: Declare the module in `src/blog.rs` so tests can run**

Add to the top of `src/blog.rs`:
```rust
pub mod content;
```

- [ ] **Step 2: Write failing tests for frontmatter splitting + slug derivation**

Create `src/content.rs` with tests only (implementation functions unimplemented):

```rust
use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Frontmatter {
    pub title: String,
    pub date: NaiveDate,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub draft: bool,
}

/// Splits a markdown file's contents into (frontmatter_yaml, body_markdown).
/// The frontmatter is delimited by `---` on its own line at the start of the file
/// and a closing `---` on its own line.
pub fn split_frontmatter(source: &str) -> Result<(&str, &str), String> {
    unimplemented!()
}

/// Derives a url-safe kebab-case slug from a post title.
/// Strips non-alphanumerics (except spaces and hyphens), lowercases, and
/// collapses runs of whitespace/dashes into single hyphens.
pub fn derive_slug(title: &str) -> String {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_frontmatter_extracts_yaml_and_body() {
        let src = "---\ntitle: Hi\ndate: 2025-01-01\n---\nBody text here.\n";
        let (fm, body) = split_frontmatter(src).unwrap();
        assert!(fm.contains("title: Hi"));
        assert_eq!(body.trim(), "Body text here.");
    }

    #[test]
    fn split_frontmatter_rejects_files_without_opening_fence() {
        let src = "no fence here\ntitle: Hi\n";
        assert!(split_frontmatter(src).is_err());
    }

    #[test]
    fn split_frontmatter_rejects_files_without_closing_fence() {
        let src = "---\ntitle: Hi\nbody never closes";
        assert!(split_frontmatter(src).is_err());
    }

    #[test]
    fn derive_slug_handles_normal_title() {
        assert_eq!(derive_slug("Building a Personal Website"), "building-a-personal-website");
    }

    #[test]
    fn derive_slug_strips_punctuation() {
        assert_eq!(derive_slug("Notes on Rust, Zig & Go!"), "notes-on-rust-zig-go");
    }

    #[test]
    fn derive_slug_collapses_multiple_spaces() {
        assert_eq!(derive_slug("Hello    world"), "hello-world");
    }

    #[test]
    fn frontmatter_parses_with_serde_yaml() {
        let yaml = "title: Hi\ndate: 2025-06-24\ntags: [rust, web]\n";
        let fm: Frontmatter = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(fm.title, "Hi");
        assert_eq!(fm.tags, vec!["rust".to_string(), "web".to_string()]);
        assert!(!fm.draft);
        assert_eq!(fm.slug, None);
    }
}
```

- [ ] **Step 3: Run tests — must fail with `unimplemented!`**

Run: `cargo test --lib split_frontmatter`
Expected: test failures with `not yet implemented` panic.

- [ ] **Step 4: Implement `split_frontmatter` and `derive_slug`**

Replace the two `unimplemented!()` bodies in `src/content.rs`:

```rust
pub fn split_frontmatter(source: &str) -> Result<(&str, &str), String> {
    let rest = source
        .strip_prefix("---\n")
        .or_else(|| source.strip_prefix("---\r\n"))
        .ok_or_else(|| "missing opening --- fence".to_string())?;
    let end_marker = rest
        .find("\n---\n")
        .or_else(|| rest.find("\n---\r\n"))
        .ok_or_else(|| "missing closing --- fence".to_string())?;
    let yaml = &rest[..end_marker];
    let body_start = end_marker + "\n---\n".len();
    let body = &rest[body_start..];
    Ok((yaml, body))
}

pub fn derive_slug(title: &str) -> String {
    let mut slug = String::with_capacity(title.len());
    let mut prev_dash = true; // suppress leading dash
    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            for c in ch.to_lowercase() {
                slug.push(c);
            }
            prev_dash = false;
        } else if !prev_dash {
            slug.push('-');
            prev_dash = true;
        }
    }
    if slug.ends_with('-') {
        slug.pop();
    }
    slug
}
```

- [ ] **Step 5: Run the tests — must pass**

Run: `cargo test --lib content::`
Expected: all 7 tests pass.

- [ ] **Step 6: Commit**

```bash
jj commit src/blog.rs src/content.rs -m "feat(content): add frontmatter splitter and slug deriver with tests"
```

---

## Task 4: Add markdown-to-HTML rendering with syntect syntax highlighting (TDD)

**Files:**
- Modify: `src/content.rs`

- [ ] **Step 1: Write failing test for `render_markdown`**

Append to the `tests` module in `src/content.rs`:

```rust
#[test]
fn render_markdown_produces_html_paragraph() {
    let html = render_markdown("Hello **world**");
    assert!(html.contains("<p>"));
    assert!(html.contains("<strong>world</strong>"));
}

#[test]
fn render_markdown_highlights_rust_code_blocks() {
    let md = "```rust\nfn main() {}\n```";
    let html = render_markdown(md);
    // syntect wraps tokens in <span> with inline styles
    assert!(html.contains("<span"));
    assert!(html.contains("fn"));
    assert!(html.contains("main"));
}

#[test]
fn render_markdown_leaves_plain_code_blocks_as_pre() {
    let md = "```\nplain text\n```";
    let html = render_markdown(md);
    assert!(html.contains("<pre"));
    assert!(html.contains("plain text"));
}
```

Add the function signature above the tests module:

```rust
pub fn render_markdown(source: &str) -> String {
    unimplemented!()
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib render_markdown`
Expected: 3 failing tests (`unimplemented!` panics).

- [ ] **Step 3: Implement `render_markdown` using pulldown-cmark + syntect**

Replace the body:

```rust
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::html::{styled_line_to_highlighted_html, IncludeBackground};
use syntect::parsing::SyntaxSet;
use std::sync::OnceLock;

struct SyntectBundle {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

fn syntect() -> &'static SyntectBundle {
    static BUNDLE: OnceLock<SyntectBundle> = OnceLock::new();
    BUNDLE.get_or_init(|| SyntectBundle {
        syntax_set: SyntaxSet::load_defaults_newlines(),
        theme_set: ThemeSet::load_defaults(),
    })
}

pub fn render_markdown(source: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(source, options);
    let bundle = syntect();
    // InspiredGitHub is a light theme that ships with syntect defaults.
    // We'll swap to a proper Gruvbox theme at config time later; for now it gives us
    // light-background highlighting that approximates the target.
    let theme = &bundle.theme_set.themes["InspiredGitHub"];

    let mut html_out = String::new();
    let mut in_code_block: Option<Option<String>> = None; // Some(Some(lang)) if inside a fenced block
    let mut code_buffer = String::new();
    let mut events: Vec<Event> = Vec::new();

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(ref lang))) => {
                in_code_block = Some(if lang.is_empty() { None } else { Some(lang.to_string()) });
                code_buffer.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some(lang_opt) = in_code_block.take() {
                    let syntax = lang_opt
                        .as_deref()
                        .and_then(|l| bundle.syntax_set.find_syntax_by_token(l))
                        .unwrap_or_else(|| bundle.syntax_set.find_syntax_plain_text());
                    let mut highlighter = HighlightLines::new(syntax, theme);
                    let mut highlighted = String::from("<pre class=\"code-block\"><code>");
                    for line in code_buffer.lines() {
                        let regions = highlighter
                            .highlight_line(line, &bundle.syntax_set)
                            .unwrap_or_default();
                        let line_html = styled_line_to_highlighted_html(
                            &regions[..],
                            IncludeBackground::No,
                        )
                        .unwrap_or_else(|_| line.to_string());
                        highlighted.push_str(&line_html);
                        highlighted.push('\n');
                    }
                    highlighted.push_str("</code></pre>");
                    events.push(Event::Html(highlighted.into()));
                }
            }
            Event::Text(text) if in_code_block.is_some() => {
                code_buffer.push_str(&text);
            }
            other => {
                if in_code_block.is_none() {
                    events.push(other);
                }
            }
        }
    }

    pulldown_cmark::html::push_html(&mut html_out, events.into_iter());
    html_out
}
```

- [ ] **Step 4: Run tests — must pass**

Run: `cargo test --lib render_markdown`
Expected: all 3 tests pass. The full `cargo test` should show 10 passing tests total.

- [ ] **Step 5: Commit**

```bash
jj commit src/content.rs -m "feat(content): render markdown with syntect syntax highlighting"
```

---

## Task 5: Add post-loading function that reads a directory of markdown files (TDD)

**Files:**
- Modify: `src/content.rs`

- [ ] **Step 1: Define `Post` struct and `load_posts` signature**

Append to `src/content.rs` (above the tests module):

```rust
#[derive(Debug, Clone)]
pub struct Post {
    pub title: String,
    pub slug: String,
    pub date: NaiveDate,
    pub tags: Vec<String>,
    pub rendered_html: String,
    pub draft: bool,
}

/// Reads every `*.md` file in `dir`, parses frontmatter, renders the body,
/// and returns posts sorted newest-first. Drafts are included iff `include_drafts` is true.
/// Errors on individual files are logged via `tracing::warn!` and the file is skipped.
pub fn load_posts(dir: &std::path::Path, include_drafts: bool) -> Vec<Post> {
    unimplemented!()
}
```

- [ ] **Step 2: Write failing integration-style tests using a tempdir**

Add `tempfile` as a dev-dep:

Run: `cargo add --dev tempfile`

Append to the `tests` module in `src/content.rs`:

```rust
#[test]
fn load_posts_reads_a_directory_of_markdown_files() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(
        tmp.path().join("a.md"),
        "---\ntitle: First\ndate: 2025-01-10\ntags: [rust]\n---\nHello.\n",
    )
    .unwrap();
    std::fs::write(
        tmp.path().join("b.md"),
        "---\ntitle: Second\ndate: 2025-02-20\n---\nWorld.\n",
    )
    .unwrap();

    let posts = load_posts(tmp.path(), false);
    assert_eq!(posts.len(), 2);
    // Sorted newest first:
    assert_eq!(posts[0].title, "Second");
    assert_eq!(posts[1].title, "First");
    assert_eq!(posts[1].slug, "first");
    assert!(posts[0].rendered_html.contains("<p>World."));
}

#[test]
fn load_posts_excludes_drafts_by_default() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(
        tmp.path().join("draft.md"),
        "---\ntitle: Draft\ndate: 2025-05-01\ndraft: true\n---\nWIP.\n",
    )
    .unwrap();
    std::fs::write(
        tmp.path().join("real.md"),
        "---\ntitle: Real\ndate: 2025-05-01\n---\nBody.\n",
    )
    .unwrap();

    let without = load_posts(tmp.path(), false);
    assert_eq!(without.len(), 1);
    assert_eq!(without[0].title, "Real");

    let with = load_posts(tmp.path(), true);
    assert_eq!(with.len(), 2);
}

#[test]
fn load_posts_uses_explicit_slug_when_provided() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(
        tmp.path().join("custom.md"),
        "---\ntitle: A Very Long Title\nslug: short\ndate: 2025-01-01\n---\nBody.\n",
    )
    .unwrap();
    let posts = load_posts(tmp.path(), false);
    assert_eq!(posts[0].slug, "short");
}
```

- [ ] **Step 3: Run tests — must fail**

Run: `cargo test --lib load_posts`
Expected: `unimplemented!` panics.

- [ ] **Step 4: Implement `load_posts`**

Replace the `unimplemented!()` body:

```rust
pub fn load_posts(dir: &std::path::Path, include_drafts: bool) -> Vec<Post> {
    let mut posts = Vec::new();
    let read = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("cannot read posts dir {:?}: {}", dir, e);
            return posts;
        }
    };
    for entry in read.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let source = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("skipping {:?}: {}", path, e);
                continue;
            }
        };
        let (fm_yaml, body) = match split_frontmatter(&source) {
            Ok(pair) => pair,
            Err(e) => {
                tracing::warn!("frontmatter error in {:?}: {}", path, e);
                continue;
            }
        };
        let fm: Frontmatter = match serde_yaml::from_str(fm_yaml) {
            Ok(fm) => fm,
            Err(e) => {
                tracing::warn!("yaml error in {:?}: {}", path, e);
                continue;
            }
        };
        if fm.draft && !include_drafts {
            continue;
        }
        let slug = fm.slug.clone().unwrap_or_else(|| derive_slug(&fm.title));
        let rendered = render_markdown(body);
        posts.push(Post {
            title: fm.title,
            slug,
            date: fm.date,
            tags: fm.tags,
            rendered_html: rendered,
            draft: fm.draft,
        });
    }
    posts.sort_by(|a, b| b.date.cmp(&a.date));
    posts
}
```

- [ ] **Step 5: Run all tests — must pass**

Run: `cargo test --lib`
Expected: all content tests pass (13 total).

- [ ] **Step 6: Commit**

```bash
jj commit Cargo.toml Cargo.lock src/content.rs -m "feat(content): load_posts reads and renders a directory of markdown"
```

---

## Task 6: Rewrite `AppState` in `src/blog.rs` to use markdown-backed posts

**Files:**
- Modify: `src/blog.rs`
- Modify: `templates/blog_post.html` (expects new field names)

- [ ] **Step 1: Replace the body of `src/blog.rs` with markdown-backed state**

New content for `src/blog.rs`:

```rust
pub mod content;

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
        self.all().into_iter().take(n).collect()
    }
}

pub struct PageMeta {
    pub page_title: String,
    pub banner_title: String,
    pub path: String,
}

/// A tiny helper that mirrors a `Post` for use in templates where we only need
/// display fields. Askama can traverse `&Post` directly; this is here so future
/// filters (e.g., date formatting) have a single place to live.
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
```

- [ ] **Step 2: Run `cargo check` — expect compile errors in main.rs (next task fixes)**

Run: `cargo check`
Expected: errors about `Post::new`, `add_post`, or old API usage in `src/main.rs`. That's intentional — the next task fixes main.rs. Do NOT commit yet.

- [ ] **Step 3: Skip to Task 7 which fixes main.rs. Don't commit here.**

This task is bundled with Task 7 for a single green-build commit.

---

## Task 7: Wire up `main.rs` for slug routes, `--drafts` flag, and merged home

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Rewrite `src/main.rs` end-to-end**

Replace the file contents:

```rust
use askama_axum::Template;
use axum::http::Uri;
use axum::routing::get;
use axum::{extract::State, Router};
use clap::Parser;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use tower_http::services::ServeDir;

use blog::content::{load_posts, Post};
use blog::{format_date, post, AppState, PageMeta};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'H', long, default_value = "0.0.0.0")]
    host: String,

    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// Include posts marked `draft: true` in the frontmatter.
    #[arg(long)]
    drafts: bool,

    /// Directory containing blog post markdown files.
    #[arg(long, default_value = "./blog_posts")]
    posts_dir: PathBuf,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    tracing_subscriber::fmt::init();

    let posts = load_posts(&args.posts_dir, args.drafts);
    tracing::info!("loaded {} post(s) from {:?}", posts.len(), args.posts_dir);
    let state = AppState::from_posts(posts);

    let dist_service = ServeDir::new("./dist");

    let app = Router::new()
        .nest_service("/dist", dist_service)
        .route("/", get(home))
        .route("/about", get(about))
        .route("/blog", get(blog_index))
        .route("/post/:slug", get(post))
        .route("/resume", get(resume))
        .route("/rss.xml", get(rss))
        .with_state(state)
        .fallback(not_found);

    let ip_addr: IpAddr = args.host.parse().expect("invalid host");
    let addr = SocketAddr::from((ip_addr, args.port));
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind failed");
    axum::serve(listener, app).await.expect("serve failed");
}

async fn home(uri: Uri, State(state): State<AppState>) -> Home {
    let meta = PageMeta {
        page_title: "bogdan floris".to_string(),
        banner_title: "bogdan floris".to_string(),
        path: uri.to_string(),
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

async fn about(uri: Uri) -> About {
    let meta = PageMeta {
        page_title: "about | bogdan floris".to_string(),
        banner_title: "about".to_string(),
        path: uri.to_string(),
    };
    About { meta }
}

async fn blog_index(uri: Uri, State(state): State<AppState>) -> BlogIndex {
    let meta = PageMeta {
        page_title: "blog | bogdan floris".to_string(),
        banner_title: "blog".to_string(),
        path: uri.to_string(),
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

async fn resume(uri: Uri) -> Resume {
    let meta = PageMeta {
        page_title: "resume | bogdan floris".to_string(),
        banner_title: "resume".to_string(),
        path: uri.to_string(),
    };
    Resume { meta }
}

async fn not_found(uri: Uri) -> NotFound {
    let meta = PageMeta {
        page_title: "not found | bogdan floris".to_string(),
        banner_title: "not found".to_string(),
        path: uri.to_string(),
    };
    NotFound { meta }
}

async fn rss(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    let xml = blog::rss::build_feed(&state.all());
    (
        [(axum::http::header::CONTENT_TYPE, "application/rss+xml; charset=utf-8")],
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
#[template(path = "about.html")]
struct About {
    meta: PageMeta,
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
```

Note: `blog::rss` doesn't exist yet — Task 8 adds it. Compilation will fail on the `rss` handler until then.

- [ ] **Step 2: Leave compilation broken until Task 8 adds the RSS module.**

Do not commit yet. Proceed to Task 8.

---

## Task 8: Add RSS feed generation (TDD)

**Files:**
- Create: `src/rss.rs`
- Modify: `src/blog.rs` (declare `pub mod rss;`)

- [ ] **Step 1: Declare the module**

Add to the top of `src/blog.rs`, below `pub mod content;`:
```rust
pub mod rss;
```

- [ ] **Step 2: Write a failing test**

Create `src/rss.rs`:

```rust
use crate::content::Post;

const SITE_URL: &str = "https://bogdanfloris.com";
const SITE_TITLE: &str = "bogdan floris";
const SITE_DESCRIPTION: &str = "Notes on software, systems, and side quests.";

/// Renders an RSS 2.0 feed for the given posts.
pub fn build_feed(posts: &[&Post]) -> String {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::Post;
    use chrono::NaiveDate;

    fn sample_post(title: &str, slug: &str, date: &str) -> Post {
        Post {
            title: title.to_string(),
            slug: slug.to_string(),
            date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            tags: vec![],
            rendered_html: "<p>body</p>".to_string(),
            draft: false,
        }
    }

    #[test]
    fn build_feed_contains_channel_metadata() {
        let posts = vec![sample_post("Hi", "hi", "2025-01-01")];
        let refs: Vec<&Post> = posts.iter().collect();
        let xml = build_feed(&refs);
        assert!(xml.starts_with("<?xml"));
        assert!(xml.contains("<rss"));
        assert!(xml.contains("<channel>"));
        assert!(xml.contains("<title>bogdan floris</title>"));
        assert!(xml.contains("<link>https://bogdanfloris.com</link>"));
    }

    #[test]
    fn build_feed_includes_one_item_per_post() {
        let posts = vec![
            sample_post("First", "first", "2025-01-01"),
            sample_post("Second", "second", "2025-02-01"),
        ];
        let refs: Vec<&Post> = posts.iter().collect();
        let xml = build_feed(&refs);
        assert_eq!(xml.matches("<item>").count(), 2);
        assert!(xml.contains("https://bogdanfloris.com/post/first"));
        assert!(xml.contains("https://bogdanfloris.com/post/second"));
    }

    #[test]
    fn build_feed_escapes_html_in_titles() {
        let posts = vec![sample_post("A & B <c>", "a", "2025-01-01")];
        let refs: Vec<&Post> = posts.iter().collect();
        let xml = build_feed(&refs);
        assert!(xml.contains("A &amp; B &lt;c&gt;"));
        assert!(!xml.contains("A & B <c>"));
    }
}
```

- [ ] **Step 3: Run tests — must fail**

Run: `cargo test --lib rss::`
Expected: 3 failures (`unimplemented!`).

- [ ] **Step 4: Implement `build_feed`**

Replace the `unimplemented!()` body:

```rust
pub fn build_feed(posts: &[&Post]) -> String {
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\">\n");
    out.push_str("<channel>\n");
    push_tag(&mut out, "title", SITE_TITLE);
    push_tag(&mut out, "link", SITE_URL);
    push_tag(&mut out, "description", SITE_DESCRIPTION);
    out.push_str("<language>en</language>\n");

    for post in posts {
        out.push_str("<item>\n");
        push_tag(&mut out, "title", &post.title);
        let url = format!("{}/post/{}", SITE_URL, post.slug);
        push_tag(&mut out, "link", &url);
        push_tag(&mut out, "guid", &url);
        let pub_date = post
            .date
            .and_hms_opt(0, 0, 0)
            .map(|dt| dt.format("%a, %d %b %Y %H:%M:%S +0000").to_string())
            .unwrap_or_default();
        push_tag(&mut out, "pubDate", &pub_date);
        // RSS `description` can carry HTML wrapped in CDATA; pick that route.
        out.push_str("<description><![CDATA[");
        out.push_str(&post.rendered_html);
        out.push_str("]]></description>\n");
        out.push_str("</item>\n");
    }

    out.push_str("</channel>\n</rss>\n");
    out
}

fn push_tag(out: &mut String, name: &str, value: &str) {
    out.push('<');
    out.push_str(name);
    out.push('>');
    xml_escape(value, out);
    out.push_str("</");
    out.push_str(name);
    out.push_str(">\n");
}

fn xml_escape(input: &str, out: &mut String) {
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            c => out.push(c),
        }
    }
}
```

- [ ] **Step 5: Run tests — all pass**

Run: `cargo test --lib`
Expected: all 16 tests pass (content: 13, rss: 3).

- [ ] **Step 6: Run `cargo check` — full build passes**

Run: `cargo check`
Expected: clean build.

- [ ] **Step 7: Commit the big state change (blog.rs + main.rs + rss.rs together)**

```bash
jj commit src/blog.rs src/main.rs src/rss.rs -m "feat(blog): slug-based routes, drafts flag, rss feed, home shows recent posts"
```

---

## Task 9: Migrate the existing HTML post to markdown

**Files:**
- Create: `blog_posts/personal-website-rust.md`
- Delete: `blog_posts/personal_website_blog.html`

- [ ] **Step 1: Create the markdown file**

Write to `blog_posts/personal-website-rust.md`:

```markdown
---
title: "Building a Personal Website with Rust and Axum"
date: 2025-06-24
tags: [rust, web]
slug: personal-website-rust
---

I mainly use React at work, which I'm not a particularly big fan of for a multitude of reasons I won't go into much detail here. So, when choosing a stack to build my personal website in, React was out of the question. I wanted to try out Svelte because I have heard good things about it from my co-workers, but Svelte is still JavaScript frontend framework number #92321223.

So, after not so much consideration, I went with what I knew when I first started building websites, templating. But with a bit of a twist. I am learning Rust in pursuit of my goal to deepen my knowledge on systems engineering, so it seemed like a natural choice. As for why I chose Axum? ¯\_(ツ)_/¯. Reddit recommends it and it seemed simple enough. And it really is!

## Getting Started

First, let's set up a basic Axum server. You'll need to add the following dependencies to your `Cargo.toml` file:

```toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tower = "0.4"
```

## Creating the Server

Seems simple enough:

- define some paths on the router
- get a host and a port on which to run this bad boy on
- launch the server

```rust
use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/about", get(about));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn about() -> &'static str {
    "About page"
}
```

### Running the Application

To run your application, simply use `cargo run` in your terminal. The server will start listening on port 3000.

## Styling with Tailwind and Gruvbox

For styling, I combined Tailwind CSS with the Gruvbox color scheme. I am basically obsessed with Gruvbox in general, I run it on everything, so it seemed natural to also use it for the personal website.

## Adding Templates with Askama

For templating, I chose Askama because it provides compile-time template checking. This means template errors are caught during compilation rather than at runtime:

```rust
use askama_axum::Template;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    title: String,
    content: String,
}
```

## Future Improvements

I plan to add some interactivity to the website besides the small dark/light mode toggle to test "frameworks" like HTMX. I've used it in another templating projects slightly, but I want to see how far a thing like HTMX could be pushed until one needs to crawl back in React's arm.

You can find the complete source code for this website on [GitHub](https://github.com/BogdanFloris/bogdanfloris.com). Feel free to use it as inspiration for your own Rust web projects!
```

- [ ] **Step 2: Delete the old HTML file**

Run: `rm blog_posts/personal_website_blog.html`

- [ ] **Step 3: Run the server and verify the post loads**

Run (background): `cargo run -- -p 3000`
Then: `curl -sS http://localhost:3000/post/personal-website-rust | grep -c "Building a Personal Website"`
Expected: output `1` or more.

Also check: `curl -sS http://localhost:3000/rss.xml | head -5`
Expected: starts with `<?xml version="1.0" ...` and contains the post.

Kill the server.

- [ ] **Step 4: Commit**

```bash
jj commit blog_posts/ -m "content: migrate the existing post to markdown with frontmatter"
```

---

## Task 10: Drop dark mode — tailwind config, style.css, and template classes

**Files:**
- Modify: `tailwind.config.js`
- Modify: `src/style.css`
- Modify: `templates/base.html`
- Delete: `templates/partials/theme-button.html`
- Modify: every template file referencing `dark:*` classes

- [ ] **Step 1: Simplify `tailwind.config.js`**

Replace with:

```js
/** @type {import('tailwindcss').Config} */
const defaultTheme = require("tailwindcss/defaultTheme");

module.exports = {
  content: ["./templates/**/*.html"],
  theme: {
    fontFamily: {
      sans: ["Inter", ...defaultTheme.fontFamily.sans],
      serif: ['"Source Serif 4"', '"Source Serif Pro"', "Charter", "Georgia", "serif"],
      mono: ['"JetBrains Mono"', ...defaultTheme.fontFamily.mono],
    },
    colors: {
      "bg-h": "#f9f5d7",
      "bg-primary": "#fbf1c7",
      "bg-s": "#f2e5bc",
      "bg-1": "#ebdbb2",
      "bg-2": "#d5c4a1",
      "bg-3": "#bdae93",
      "bg-4": "#a89984",
      fg: "#282828",
      "fg-1": "#3c3836",
      "fg-2": "#504945",
      "fg-3": "#665c54",
      "fg-4": "#7c6f64",
      red: "#9d0006",
      green: "#79740e",
      yellow: "#b57614",
      blue: "#076678",
      purple: "#8f3f71",
      aqua: "#427b58",
      orange: "#af3a03",
      gray: "#928374",
      "red-dim": "#cc2412",
      "green-dim": "#98971a",
      "yellow-dim": "#d79921",
      "blue-dim": "#458598",
      "purple-dim": "#b16286",
      "aqua-dim": "#689d6a",
      "orange-dim": "#d65d0e",
      "gray-dim": "#7c6f64",
      white: "#ffffff",
      transparent: "transparent",
    },
  },
  plugins: [],
};
```

- [ ] **Step 2: Rewrite `src/style.css` for the Notebook aesthetic**

Replace the file:

```css
@tailwind base;
@tailwind components;
@tailwind utilities;

/* ---------- Base ---------- */
html {
  font-size: 17px;
}
body {
  @apply bg-bg-primary text-fg font-serif leading-relaxed;
}

/* ---------- Typography ---------- */
.h1 {
  @apply text-3xl font-bold font-serif;
}
.h2 {
  @apply text-2xl font-bold font-serif;
}
.h3 {
  @apply text-xl font-semibold font-serif;
}
.body-text {
  @apply leading-relaxed;
}

/* ---------- Links ---------- */
.link {
  @apply text-blue hover:text-blue-dim visited:text-purple;
  @apply underline decoration-1 underline-offset-2 hover:decoration-2 transition-all duration-150;
}

/* ---------- Letterhead ---------- */
.letterhead {
  @apply font-mono text-sm text-fg-3 border-b border-bg-2 pb-4 mb-10;
}
.letterhead .name {
  @apply text-fg font-semibold;
}
.letterhead .tagline {
  @apply text-fg-4 mt-1;
}

/* ---------- Nav ---------- */
.site-nav {
  @apply font-mono text-sm text-fg-3 mb-12;
}
.site-nav a {
  @apply text-fg-2 hover:text-orange no-underline;
}
.site-nav .sep {
  @apply mx-2 text-fg-4;
}
.site-nav a.active {
  @apply text-orange font-semibold;
}

/* ---------- Post list (lab notebook style) ---------- */
.post-list {
  @apply list-none p-0 m-0 space-y-3;
}
.post-row {
  @apply grid gap-x-4 items-baseline;
  grid-template-columns: auto 1fr auto;
}
@media (max-width: 640px) {
  .post-row {
    grid-template-columns: auto 1fr;
  }
  .post-row .tags {
    grid-column: 1 / -1;
    padding-left: 5.75rem;
  }
}
.post-row .date {
  @apply font-mono text-sm text-fg-4 whitespace-nowrap;
}
.post-row .title a {
  @apply text-blue hover:text-blue-dim no-underline font-semibold;
}
.post-row .tags {
  @apply font-mono text-xs text-gray whitespace-nowrap;
}

/* ---------- Tags ---------- */
.tag-list {
  @apply font-mono text-xs text-gray;
}
.tag-list .tag + .tag::before {
  content: " · ";
}

/* ---------- Article ---------- */
.article h1 {
  @apply h1 mb-3;
}
.article .meta {
  @apply font-mono text-sm text-fg-4 mb-10;
}
.article h2 {
  @apply h2 mt-10 mb-4;
}
.article h3 {
  @apply h3 mt-8 mb-3;
}
.article p {
  @apply my-4 leading-relaxed;
}
.article ul,
.article ol {
  @apply my-4 ml-8;
}
.article ul {
  @apply list-disc;
}
.article ol {
  @apply list-decimal;
}
.article li {
  @apply my-1;
}
.article a {
  @apply link;
}
.article code {
  @apply bg-bg-1 px-1.5 py-0.5 rounded-sm font-mono text-sm;
}
.article pre.code-block {
  @apply bg-bg-s border border-bg-2 p-4 my-6 overflow-x-auto rounded-sm;
}
.article pre.code-block code {
  @apply bg-transparent p-0 text-sm;
}

/* ---------- Footer ---------- */
.site-footer {
  @apply mt-24 pt-6 border-t border-bg-2 font-mono text-xs text-fg-4 flex justify-between;
}
.site-footer a {
  @apply text-fg-3 hover:text-orange no-underline;
}
```

- [ ] **Step 3: Delete the theme toggle partial**

Run: `rm templates/partials/theme-button.html`

- [ ] **Step 4: Strip theme script + dark classes from `templates/base.html`**

Replace the file:

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <meta http-equiv="X-UA-Compatible" content="ie=edge" />
    <meta name="description" content="The personal website and blog of Bogdan Floris" />
    <meta name="keywords" content="bogdan floris, software engineer, rust, blog" />
    <link rel="icon" type="image/svg+xml" href="/dist/favicon.svg" />
    <link rel="alternate" type="application/rss+xml" title="RSS" href="/rss.xml" />
    <link rel="preconnect" href="https://fonts.googleapis.com" />
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
    <link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;600&family=Source+Serif+4:ital,wght@0,400;0,600;0,700;1,400&display=swap" rel="stylesheet" />
    <link href="/dist/css/output.css" rel="stylesheet" />
    <title>{% block title %}{% endblock %}</title>
  </head>
  <body>
    {% block body %}{% endblock %}
  </body>
</html>
```

- [ ] **Step 5: Rebuild CSS to check for unused classes**

Run: `tailwindcss -i src/style.css -o dist/css/output.css`
Expected: succeeds. Output file exists.

Run: `cargo check`
Expected: still compiles (templates still reference old partials — next tasks fix them).

- [ ] **Step 6: Commit this intermediate state**

Don't commit yet — templates still reference `theme-button.html`. Fold into the next task's commit. Move on.

---

## Task 11: Replace banner with letterhead, add footer, fix nav

**Files:**
- Create: `templates/partials/letterhead.html`
- Create: `templates/partials/footer.html`
- Delete: `templates/partials/banner.html`
- Modify: `templates/partials/nav.html`

- [ ] **Step 1: Create `templates/partials/letterhead.html`**

```html
<div class="letterhead">
  <div class="name">bogdan floris</div>
  <div class="tagline">software engineer · bucharest · writes about rust &amp; systems</div>
</div>
```

- [ ] **Step 2: Create `templates/partials/footer.html`**

```html
<footer class="site-footer">
  <span>© bogdan floris</span>
  <span>
    <a href="/rss.xml">rss</a>
    <span class="mx-2">·</span>
    <a href="https://github.com/BogdanFloris">github</a>
  </span>
</footer>
```

- [ ] **Step 3: Rewrite `templates/partials/nav.html` — no projects, no pipes, mono dots**

```html
<nav class="site-nav">
  <a href="/"{% if meta.path == "/" %} class="active"{% endif %}>home</a>
  <span class="sep">·</span>
  <a href="/about"{% if meta.path == "/about" %} class="active"{% endif %}>about</a>
  <span class="sep">·</span>
  <a href="/blog"{% if meta.path.starts_with("/blog") || meta.path.starts_with("/post") %} class="active"{% endif %}>blog</a>
  <span class="sep">·</span>
  <a href="/resume"{% if meta.path == "/resume" %} class="active"{% endif %}>resume</a>
</nav>
```

- [ ] **Step 4: Delete the old banner**

Run: `rm templates/partials/banner.html`

- [ ] **Step 5: Do not commit yet — templates still include the deleted partials.**

Next task rewrites page templates to use letterhead + footer.

---

## Task 12: Restyle all page templates under Notebook

**Files:**
- Modify: `templates/index.html`
- Modify: `templates/about.html`
- Modify: `templates/blog.html`
- Modify: `templates/blog_post.html`
- Modify: `templates/resume.html`
- Modify: `templates/404.html`

- [ ] **Step 1: Rewrite `templates/index.html` (landing card + recent posts)**

```html
{% extends "base.html" %}
{% block title %}{{ meta.page_title }}{% endblock %}
{% block body %}
<main class="max-w-[680px] mx-auto px-6 py-16">
  {% include "partials/letterhead.html" %}
  {% include "partials/nav.html" %}

  <section class="mb-12 body-text">
    <p class="mb-4">
      I'm Bogdan, a software engineer in Bucharest with six years of experience. I currently work at <a class="link" href="https://datacamp.com">DataCamp</a> on the Content Platform team, enabling new languages and building tooling around course authoring.
    </p>
    <p class="mb-4">
      Before that I worked at <a class="link" href="https://www.bloomberg.com/engineering">Bloomberg</a> in London on the MARS risk platform. I write here mostly about Rust, systems, and the occasional side quest.
    </p>
    <p class="mb-4"><a class="link" href="/about">More about me →</a></p>
  </section>

  {% if !recent.is_empty() %}
  <section class="mb-12">
    <h2 class="font-mono text-sm text-orange uppercase tracking-wider mb-4">recent</h2>
    <ul class="post-list">
      {% for post in recent %}
      <li class="post-row">
        <span class="date">{{ post.date }}</span>
        <span class="title"><a href="/post/{{ post.slug }}">{{ post.title }}</a></span>
        <span class="tags">
          {% for tag in post.tags %}{% if !loop.first %} · {% endif %}{{ tag }}{% endfor %}
        </span>
      </li>
      {% endfor %}
    </ul>
    <p class="mt-4"><a class="link font-mono text-sm" href="/blog">all posts →</a></p>
  </section>
  {% endif %}

  {% include "partials/footer.html" %}
</main>
{% endblock %}
```

- [ ] **Step 2: Rewrite `templates/about.html`**

```html
{% extends "base.html" %}
{% block title %}{{ meta.page_title }}{% endblock %}
{% block body %}
<main class="max-w-[680px] mx-auto px-6 py-16">
  {% include "partials/letterhead.html" %}
  {% include "partials/nav.html" %}

  <article class="article">
    <h1>About</h1>
    <p>
      I'm Bogdan, a software engineer with six years of experience, from Bucharest, Romania. I currently work at <a class="link" href="https://datacamp.com">DataCamp</a>, where we build a platform people of all backgrounds use to learn data and programming skills. I'm on the Content Platform team — we enable new technologies in the platform (anything a course can be built in: Python, SQL, Azure, PowerBI) and maintain the tooling content developers use to author those courses.
    </p>
    <p>
      Before DataCamp I worked at <a class="link" href="https://www.bloomberg.com/engineering">Bloomberg</a> in the MARS team in London, building a suite of risk-management tools that help professionals analyze their portfolios.
    </p>
    <p>
      I've been programming most of my life — C and C++ in middle school, Java through my Bachelor's, Python through my Master's, modern C++ at Bloomberg, and Python + TypeScript/React at DataCamp. In my own time I like wandering through languages: this site is Rust, I've written a tiny interpreter in Go, my Neovim config is Lua, and I've recently fallen for Zig.
    </p>
    <p>
      Outside work I ski a lot and play acoustic guitar (badly, but happily). This site is my small piece on the internet — I keep it simple and, naturally, Gruvbox.
    </p>
    <p class="font-mono text-sm mt-8">
      email &middot; <a class="link" href="mailto:bogdan.floris@gmail.com">bogdan.floris@gmail.com</a><br />
      github &middot; <a class="link" href="https://github.com/BogdanFloris">BogdanFloris</a><br />
      linkedin &middot; <a class="link" href="https://www.linkedin.com/in/bogdan-floris">bogdan-floris</a><br />
    </p>
  </article>

  {% include "partials/footer.html" %}
</main>
{% endblock %}
```

- [ ] **Step 3: Rewrite `templates/blog.html` (full post list)**

```html
{% extends "base.html" %}
{% block title %}{{ meta.page_title }}{% endblock %}
{% block body %}
<main class="max-w-[680px] mx-auto px-6 py-16">
  {% include "partials/letterhead.html" %}
  {% include "partials/nav.html" %}

  <section class="mb-12">
    <h1 class="h1 mb-6">writing</h1>
    {% if posts.is_empty() %}
    <p class="body-text text-fg-3">Nothing here yet. Soon.</p>
    {% else %}
    <ul class="post-list">
      {% for post in posts %}
      <li class="post-row">
        <span class="date">{{ post.date }}</span>
        <span class="title"><a href="/post/{{ post.slug }}">{{ post.title }}</a></span>
        <span class="tags">
          {% for tag in post.tags %}{% if !loop.first %} · {% endif %}{{ tag }}{% endfor %}
        </span>
      </li>
      {% endfor %}
    </ul>
    {% endif %}
  </section>

  {% include "partials/footer.html" %}
</main>
{% endblock %}
```

- [ ] **Step 4: Rewrite `templates/blog_post.html`**

```html
{% extends "base.html" %}
{% block title %}{{ meta.page_title }}{% endblock %}
{% block body %}
<main class="max-w-[680px] mx-auto px-6 py-16">
  {% include "partials/letterhead.html" %}
  {% include "partials/nav.html" %}

  <article class="article">
    <h1>{{ title }}</h1>
    <div class="meta">
      {{ date }}
      {% if !tags.is_empty() %}
      &middot;
      {% for tag in tags %}{% if !loop.first %} · {% endif %}{{ tag }}{% endfor %}
      {% endif %}
    </div>
    {{ rendered_html|safe }}
  </article>

  <p class="mt-12 font-mono text-sm"><a class="link" href="/blog">← all posts</a></p>

  {% include "partials/footer.html" %}
</main>
{% endblock %}
```

- [ ] **Step 5: Update `templates/resume.html` wrapper**

```html
{% extends "base.html" %}
{% block title %}{{ meta.page_title }}{% endblock %}
{% block body %}
<main class="max-w-[680px] mx-auto px-6 py-16">
  {% include "partials/letterhead.html" %}
  {% include "partials/nav.html" %}
  <article class="article">
    {% include "partials/resume-body.html" %}
  </article>
  {% include "partials/footer.html" %}
</main>
{% endblock %}
```

The existing `partials/resume-body.html` has its own classes — they'll render fine under the new palette. Leave it alone for this pass; if anything looks off after verification, fix in Task 13.

- [ ] **Step 6: Rewrite `templates/404.html`**

```html
{% extends "base.html" %}
{% block title %}{{ meta.page_title }}{% endblock %}
{% block body %}
<main class="max-w-[680px] mx-auto px-6 py-16">
  {% include "partials/letterhead.html" %}
  {% include "partials/nav.html" %}
  <section class="my-20 text-center">
    <h1 class="h1 mb-4">not found</h1>
    <p class="body-text text-fg-3">The page you're looking for doesn't exist.</p>
    <p class="mt-6"><a class="link font-mono" href="/">← home</a></p>
  </section>
  {% include "partials/footer.html" %}
</main>
{% endblock %}
```

- [ ] **Step 7: Compile Tailwind and build the project**

Run: `tailwindcss -i src/style.css -o dist/css/output.css`
Expected: produces output CSS without errors.

Run: `cargo check`
Expected: clean build.

- [ ] **Step 8: Commit the full visual revamp**

```bash
jj commit tailwind.config.js src/style.css templates/ -m "feat(ui): notebook-style gruvbox-light redesign; drop dark mode"
```

---

## Task 13: End-to-end verification

**Files:** none (verification only).

- [ ] **Step 1: Build production CSS and run the server**

Run: `tailwindcss -i src/style.css -o dist/css/output.css`
Run (background): `cargo run -- -p 3000`

- [ ] **Step 2: Verify each route returns 200 with expected content**

```bash
for path in / /about /blog /post/personal-website-rust /resume /rss.xml /nope-does-not-exist; do
  echo "== $path =="
  curl -sS -o /dev/null -w "%{http_code}\n" http://localhost:3000$path
done
```

Expected status codes: `200 200 200 200 200 200 404` (RSS returns 200; the nonsense path returns 404).

- [ ] **Step 3: Spot-check content**

```bash
curl -sS http://localhost:3000/ | grep -c "recent"
# Expected: at least 1 (the "recent" heading on home).

curl -sS http://localhost:3000/post/personal-website-rust | grep -c "class=\"code-block\""
# Expected: at least 1 — syntect produced a highlighted code block.

curl -sS http://localhost:3000/rss.xml | head -1
# Expected: <?xml version="1.0" encoding="UTF-8"?>
```

- [ ] **Step 4: Tell the user to load `http://localhost:3000` in a browser**

Since I cannot see rendered pixels, hand off for visual verification. Ask the user to:
1. Open `http://localhost:3000/` and confirm the Notebook look.
2. Visit `/blog` and `/post/personal-website-rust` — confirm date/title/tags align, code block is highlighted, serif body reads well on a narrow viewport.
3. Resize the browser narrow (~420px) to confirm the post list wraps gracefully.
4. Confirm no dark-mode flash on reload.

Kill the server once verified.

- [ ] **Step 5: Cleanup commit (if any stray files or formatting)**

Run: `cargo fmt`
Run: `jj status`

If anything changed:
```bash
jj commit <paths> -m "chore: fmt"
```

Otherwise skip.

---

## Self-Review Notes

Completed inline during plan writing:

- **Spec coverage:** Every spec requirement has a task — Notebook visual (Tasks 10–12), Gruvbox-light palette (Task 10), drop dark mode (Task 10), merged home+about (Tasks 7, 12), drop `/projects` (Task 7), markdown + frontmatter (Tasks 3–5), syntect (Task 4), slug URLs (Tasks 5, 7), drafts flag (Task 7), RSS (Task 8), migrate existing post (Task 9), dep upgrade sequenced first (Task 1).
- **No placeholders:** Every step has exact code/commands/paths. One soft spot: `InspiredGitHub` theme is used in Task 4 for highlighting. The spec calls for "Gruvbox-light" highlighting; that's a post-implementation polish item — either swap to a bundled Gruvbox theme for syntect or generate one from a `.tmTheme` file. Tracked as a known polish item, not a blocker.
- **Type consistency:** `Post` fields, `AppState::from_posts`/`get`/`all`/`latest`, `PageMeta`, `RecentPost`, `PostTemplate` field names are all consistent across tasks.
- **Scope:** One plan, one cohesive revamp. Testable at the end as a whole.

## Known follow-ups (out of scope)

- Real Gruvbox-light syntect theme (swap `InspiredGitHub` for a proper Gruvbox `.tmTheme`).
- Tag filter pages (`/tag/:name`) — deferred until the archive is larger.
- Hot-reload for markdown during `cargo watch` — currently requires a server restart.
