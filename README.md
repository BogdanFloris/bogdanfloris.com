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
date: 3-11-2026
tags: [systems]
---
```

Dates can be day-first (`3-11-2026`, `3.11.2026`) or ISO (`2026-11-03`), and
the site shows them day-first.

Essays with two or more `##` headings get a contents sidebar. An `updates`
list in the frontmatter adds a change log next to it, and its newest date
shows as "updated":

```yaml
updates:
  - 1-11-2026 first draft
  - 3-11-2026 tightened the argument
```

Scribbles all go in `content/scribbles.md`, newest first. Every `## ` line
starts a new scribble and its heading shows on the site as written, so any
date format works:

```markdown
## 3-11-2026

The one line from today's notebook page.
```

The quickest way in, from any device, is GitHub's editor:
<https://github.com/BogdanFloris/bogdanfloris.com/edit/master/content/scribbles.md>.

`cargo run -- --drafts` also shows essays with `draft: true`.

## Deploying

Every push to `master` deploys through GitHub Actions
(`.github/workflows/fly-deploy.yml`). It needs a `FLY_API_TOKEN` repository
secret, created once with:

```sh
fly tokens create deploy | gh secret set FLY_API_TOKEN
```

`fly deploy` from a checkout still works too.
