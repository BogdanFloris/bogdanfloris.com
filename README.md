# bogdanfloris.com

This is my personal website [bogdanfloris.com](https://bogdanfloris.com).

It's written in Rust using the Axum framework.

## Running locally

In two separate terminals run:

```sh
tailwind -i src/style.css -o dist/css/output.css --watch
```

and then,

```sh
cargo watch -x run
```

Or run them in detached mode by appending `&` at the end of the command.

## Writing

Everything I write lives in `content/`. A page is a markdown file with
frontmatter, and `kind` decides where it shows up:

```markdown
---
title: "Queues are everywhere"
date: 2026-11-03
kind: note
tags: [systems]
---
```

- `note` (the default): rough, one idea. Listed at `/notes`.
- `log`: a running journal of a project. Listed at `/logs`.
- `essay`: a finished piece. Listed at `/essays` and on the home page, and
  the only kind in the RSS feed.

A page lives at `/<kind>s/<slug>`, with the slug from `slug:` or the title.
To promote a note, change `kind: note` to `kind: essay`; the old URL redirects.

Scribbles all go in `content/scribbles.md`, each under a timestamp heading,
in any order:

```markdown
## 2026-11-03 11:20

Trying out a notebook layout for the site.
```

`cargo run -- --drafts` also shows pages with `draft: true`.
