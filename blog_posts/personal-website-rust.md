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
