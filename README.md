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

Scribbles all go in `content/scribbles.md`, newest first, each under a
timestamp heading (the time is optional):

```markdown
## 2026-11-03 18:05

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
