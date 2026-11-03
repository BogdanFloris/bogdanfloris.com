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

Everything I write lives in `content/`: essays and scribbles.

An essay is a markdown file with frontmatter. It's served at
`/essays/<slug>`, with the slug from `slug:` or the title:

```markdown
---
title: "Queues are everywhere"
date: 2026-11-03
tags: [systems]
---
```

Essays with two or more `##` headings get a contents sidebar. An `updates`
list in the frontmatter adds a change log next to it, and its newest date
shows as "updated":

```yaml
updates:
  - 2026-11-01 first draft
  - 2026-11-03 tightened the argument
```

Scribbles all go in `content/scribbles.md`, each under a timestamp heading,
in any order:

```markdown
## 2026-11-03 11:20

Trying out a notebook layout for the site.
```

`cargo run -- --drafts` also shows essays with `draft: true`.
