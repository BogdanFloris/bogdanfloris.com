# bogdanfloris.com revamp — design

**Date:** 2026-04-20
**Status:** Draft, pending user review

## Context

`bogdanfloris.com` is a Rust (Axum 0.6) + Askama + Tailwind personal site, currently using a Gruvbox terminal aesthetic with dark/light toggle. Two pain points motivate this revamp:

1. **Style.** The terminal metaphor (blinking cursor banner, `bogdan@web>` prompt) has run its course. The owner wants something more reading-first — "what a software engineer would read on a vertical display", in the spirit of jvns.ca but unique.
2. **Blog authoring.** Posts are currently hand-written HTML with inline Tailwind classes, hardcoded into `add_posts()` in `src/main.rs`. Every new post requires editing Rust code. There's no metadata, no dates, no tags, no syntax highlighting, no RSS.

The existing site has one blog post. The goal is to make writing the *next* post frictionless.

## Goals

- A distinctive, warm, reading-first visual built on the Gruvbox **light** palette.
- Authoring flow: drop a markdown file in a directory, restart the server, done.
- Syntax-highlighted code blocks that match the site's palette.
- Zero runtime cost per request (all parsing and highlighting happens at startup).

## Non-goals

- Dark mode. Removed entirely.
- Tag filter pages (`/tag/rust`). Revisit once the archive has a few dozen posts.
- A `/projects` page. GitHub does this better; a link on `/about` suffices.
- Comments, analytics, search. Not now.
- A general-purpose CMS. One author, files in a repo.

## Visual direction: "Notebook"

A calm, patient, reading-focused aesthetic. Serif body for long-form comfort; monospace for structure and metadata.

### Palette (Gruvbox light, light-only)

| Role | Token | Hex |
|---|---|---|
| Page background | `bg-primary` | `#fbf1c7` |
| Card/surface background | `bg-h` | `#f9f5d7` |
| Subtle dividers | `bg-2` | `#d5c4a1` |
| Body text | `fg` | `#282828` |
| Muted text (dates, tags) | `fg-4` / `gray` | `#7c6f64` / `#928374` |
| Links | `blue` | `#076678` |
| Visited links | `purple` | `#8f3f71` |
| Accent (section heads, labels) | `orange` | `#af3a03` |

No dark variants. Drop all `dark:` Tailwind classes and the theme toggle script.

### Typography

- **Body:** Font stack `Charter, "Source Serif 4", "Source Serif Pro", Georgia, serif`. Charter is system on Apple, absent on Linux/Windows — so serve **Source Serif 4** as the webfont fallback (via Google Fonts) and let the stack resolve naturally per platform.
- **Headings:** Same serif, heavier weight.
- **Metadata, code, section labels:** JetBrains Mono (already in the project's font config; served via Google Fonts).
- **Body size:** 17–18px. Line-height ~1.6. Reading column max-width **~680px**.

### Letterhead (replaces the banner)

A quiet top-of-page treatment: `bogdan floris` in small mono, a one-line mono tagline underneath, a thin divider. No box, no shadow, no blinking cursor.

### Post list (blog index)

A lab-notebook-style list: date flush-left in mono, title in serif, muted mono tags flush-right.

```
2025-06-24   Building a personal website with Rust and Axum        rust · web
2025-04-03   Notes on writing a tiny interpreter                    go · lang
2025-01-12   Zig, a week in                                         zig
```

Tags are decorative only for now (not links). All three columns align visually on wide screens; on narrow screens the tags wrap under the title.

### Code blocks

Syntax-highlighted via `syntect` using a Gruvbox-light theme. Background slightly darker than page (`#f2e5bc` or `#ebdbb2`) so blocks visually separate without shouting. Monospace, moderate padding, no heavy border.

## Information architecture

| Route | Purpose |
|---|---|
| `/` | Home. Short intro (name, one-line bio, links), the 3 most recent posts. |
| `/about` | The longer story (current `index.html` content merged in). |
| `/blog` | The full reverse-chronological post list. |
| `/post/:slug` | Individual post. **Slug-based URLs**, not numeric IDs. |
| `/resume` | Unchanged in content; restyled. |
| `/rss.xml` | RSS feed, generated from the same post data. |
| `*` | 404. |

**Removed:** `/projects` (was broken; GitHub link on `/about` replaces it).

**Nav:** `home · about · blog · resume`. RSS link lives in the footer, not the nav.

**Home/about merge:** The current `/` and `/about` overlap heavily. The new `/` becomes a landing card (bio + recent posts). `/about` keeps the longer-form biographical content.

## Blog authoring workflow

### File format

Markdown files in `/blog_posts/`, one per post, with YAML frontmatter:

```markdown
---
title: "Building a personal website with Rust and Axum"
date: 2025-06-24
tags: [rust, web]
slug: personal-website-rust
draft: false
---

I mainly use React at work, which I'm not a particularly big fan of...
```

Required fields: `title`, `date`. Optional: `slug` (derived from title via kebab-case if omitted), `tags` (list, defaults to empty), `draft` (bool, defaults to false).

### Discovery

At startup, scan `/blog_posts/*.md`. Parse frontmatter, parse body, render to HTML, store in `AppState`. No more editing `add_posts()` in `main.rs`.

### Rendering pipeline

- **Markdown:** `pulldown-cmark` (pure Rust, fast, battle-tested).
- **Frontmatter:** `serde_yaml` + a small split helper on `---`.
- **Syntax highlighting:** `syntect`, preloaded Gruvbox-light theme, applied to fenced code blocks during the markdown→HTML pass.
- **Result:** HTML pre-rendered and cached at startup. Runtime per-request cost is a hash-map lookup.

### Drafts

Posts with `draft: true` are skipped unless the binary is run with `--drafts`. In production, that flag is off.

### URLs

Slug-based: `/post/personal-website-rust`. Human-readable, shareable, stable if posts are reordered or deleted. The existing `/post/1` route is removed (only one post exists, to be migrated).

### Migration

The existing `blog_posts/personal_website_blog.html` will be rewritten as a markdown file with frontmatter. This is the sole content migration.

## Technical approach

### Dependencies to add

- `pulldown-cmark` — markdown parsing
- `syntect` — syntax highlighting
- `serde_yaml` — frontmatter parsing
- `quick-xml` or hand-rolled — RSS feed generation (TBD at implementation time)

### Shape of `AppState` and `Post`

Replace the current `Post { title, content }` with something richer:

```rust
pub struct Post {
    pub title: String,
    pub slug: String,
    pub date: NaiveDate,
    pub tags: Vec<String>,
    pub rendered_html: String, // pre-rendered at startup
}

pub struct AppState {
    posts_by_slug: HashMap<String, Post>,
    posts_sorted: Vec<String>, // slugs, newest first
}
```

`AppState` exposes helpers like `latest(n)`, `all()`, `get(slug)`.

### Files to touch

- `src/main.rs` — route updates (slug param, drop `/projects`), remove theme state, merge home/about.
- `src/blog.rs` — rewrite post loading, add markdown + syntect pipeline, add RSS generation.
- `tailwind.config.js` — drop dark-variant keys from the color palette (collapse each `{light, dark}` object to a single hex). Simpler config, smaller compiled CSS.
- `src/style.css` — replace terminal-styled component classes with Notebook equivalents.
- `templates/base.html` — drop theme toggle script, drop dark classes.
- `templates/partials/*` — replace banner with letterhead, drop theme button, update nav (drop `/projects`).
- `templates/index.html`, `about.html`, `blog.html`, `blog_post.html`, `resume.html`, `404.html` — restyle per Notebook.
- `Cargo.toml` — add deps; consider Axum upgrade (0.6 → latest) as a separate concern, flagged below.
- `blog_posts/*.md` — migrate the one existing post.

### Dependency upgrades

All dependencies get upgraded to current stable as part of this work, including the Axum 0.6 → latest bump. The Axum upgrade is the only one with meaningful API churn (handler signatures, `Server::bind` → `axum::serve`, Askama integration via `askama_axum` → current equivalent). It will be sequenced as the **first step** of the implementation plan so every subsequent change lands on the upgraded baseline.

## Success criteria

- New post workflow: drop `some-post.md` into `/blog_posts/`, restart the server, post is live at `/post/<slug>` and visible on `/blog`.
- No dark mode remnants in compiled CSS or rendered HTML.
- All existing routes (minus `/projects`) return 200 with the new look.
- The migrated post renders identically to today in content, with Gruvbox-highlighted code blocks.
- `/rss.xml` validates as RSS 2.0.
- Visual parity with the approved Notebook mockup on a vertical display (narrow column, flush-left dates, muted mono tags).
