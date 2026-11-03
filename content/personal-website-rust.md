---
title: "A Personal Website in Rust and Tailwind"
date: 2026-04-21
tags: [rust, web, design]
slug: personal-website-rust
---

I mainly used React at work, which I'm not a particularly big fan of for a
multitude of reasons I won't get into here. So when I went to build my
personal website, React was out of the question. I thought about Svelte
because my past coworkers swore by it, but Svelte is still JavaScript frontend
framework number #92321223.

Instead I went with what I already knew — templating on the server — with one
twist: I'm learning Rust to get more comfortable with systems engineering, so
it seemed like a natural pick for the backend. As for why Axum? ¯\_(ツ)_/¯.
Reddit recommends it and it seemed simple enough. It really is.

## The look

I took a hard look at sites I actually enjoy reading —
[jvns.ca](https://jvns.ca) being the obvious one — and what they have in
common is that they look like paper. Narrow reading column, serif body, mono
metadata, and not a lot else going on.

So that's what this is. A ~680px column, [Source Serif
4](https://fonts.google.com/specimen/Source+Serif+4) for the body,
[JetBrains Mono](https://www.jetbrains.com/lp/mono/) for dates and tags. No
shadows, no rounded cards, no hero section. Gruvbox light, one theme. I am
basically obsessed with Gruvbox, I run it on everything, so it seemed natural
to use it here too.

## The stack

```toml
[dependencies]
axum = "0.7"
askama = "0.12"
tokio = { version = "1", features = ["full"] }
tower-http = { version = "0.5", features = ["fs"] }
pulldown-cmark = "0.13"
syntect = "5"
serde_yaml = "0.9"
chrono = { version = "0.4", features = ["serde"] }
```

Axum for routing, Askama for compile-time checked templates, Tailwind v4 for
styling. The whole site is a single Rust binary plus a static `dist/` folder
for CSS and images.

```rust
let app = Router::new()
    .nest_service("/dist", ServeDir::new("./dist"))
    .route("/", get(home))
    .route("/blog", get(blog_index))
    .route("/post/:slug", get(post))
    .route("/rss.xml", get(rss))
    .with_state(state)
    .fallback(not_found);
```

## Markdown posts

I write posts as markdown with a small YAML frontmatter:

```markdown
---
title: "A Personal Website in Rust and Tailwind"
date: 2026-04-21
tags: [rust, web, design]
---

I mainly use React at work...
```

At startup, the server walks `blog_posts/`, parses frontmatter, runs the body
through [pulldown-cmark](https://docs.rs/pulldown-cmark) for HTML and
[syntect](https://docs.rs/syntect) for syntax highlighting, and stores the
result in an `AppState`. Routes look posts up by slug.

Drafts are skipped by default. Passing `--drafts` on the CLI flips a flag that
includes `draft: true` posts — so I can run a local instance with
works-in-progress visible without ever shipping them.

## Tailwind v4

The theme lives in CSS, not JavaScript:

```css
@import "tailwindcss";
@source "../templates/**/*.html";

@theme {
  --color-bg-primary: #fbf1c7;
  --color-fg: #282828;
  --font-serif: "Source Serif 4", Charter, Georgia, serif;
  --font-mono: "JetBrains Mono", ui-monospace, monospace;
}
```

The whole Gruvbox palette lives in one block, and class names like
`bg-bg-primary` and `font-serif` come out of those variables automatically.
One fewer config file to read.

## RSS

There is an RSS feed at [/rss.xml](/rss.xml). Hand-rolled XML in about 60
lines of Rust, which came out shorter than pulling in another dependency.
If you still read RSS, you know what to do.

## Closing thoughts

Nice weekend project with Claude.

Source is on [GitHub](https://github.com/BogdanFloris/bogdanfloris.com).
Steal what you like.
