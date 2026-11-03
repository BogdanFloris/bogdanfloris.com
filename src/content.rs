use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};
use std::sync::OnceLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::html::{styled_line_to_highlighted_html, IncludeBackground};
use syntect::parsing::SyntaxSet;

/// The scribbles stream lives in this one file inside the content dir.
pub const SCRIBBLES_FILE: &str = "scribbles.md";

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
    #[serde(default)]
    pub updates: Vec<UpdateEntry>,
}

/// One `updates:` entry, written as `- 2026-11-01 promoted to essay` or
/// `- 2026-11-01: promoted to essay`.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum UpdateEntry {
    Line(String),
    Dated(BTreeMap<String, String>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Update {
    /// `YYYY-MM-DD`, or empty when the entry has no date.
    pub date: String,
    pub text: String,
}

/// Flattens `updates:` entries into a list sorted by date, oldest first.
#[must_use]
pub fn parse_updates(entries: Vec<UpdateEntry>) -> Vec<Update> {
    let is_date = |s: &str| NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok();
    let mut updates = Vec::new();
    for entry in entries {
        match entry {
            UpdateEntry::Line(line) => {
                let line = line.trim();
                let (date, text) = match line.split_once(' ') {
                    Some((date, text)) if is_date(date) => (date, text.trim()),
                    _ if is_date(line) => (line, ""),
                    _ => ("", line),
                };
                updates.push(Update {
                    date: date.to_string(),
                    text: text.to_string(),
                });
            }
            UpdateEntry::Dated(map) => {
                updates.extend(map.into_iter().map(|(date, text)| Update { date, text }));
            }
        }
    }
    updates.sort_by(|a, b| a.date.cmp(&b.date));
    updates
}

/// A `##` (level 2) or `###` (level 3) heading, for the contents sidebar.
#[derive(Debug, Clone, PartialEq)]
pub struct TocEntry {
    pub level: u8,
    pub id: String,
    pub text: String,
}

/// Splits a markdown file's contents into (`frontmatter_yaml`, `body_markdown`).
/// The frontmatter is delimited by `---` on its own line at the start of the file
/// and a closing `---` on its own line.
///
/// # Errors
///
/// Returns an error when either fence is missing.
pub fn split_frontmatter(source: &str) -> Result<(&str, &str), String> {
    let rest = source
        .strip_prefix("---\n")
        .or_else(|| source.strip_prefix("---\r\n"))
        .ok_or_else(|| "missing opening --- fence".to_string())?;
    let (end_marker, end_len) = rest
        .find("\n---\n")
        .map(|i| (i, "\n---\n".len()))
        .or_else(|| rest.find("\n---\r\n").map(|i| (i, "\n---\r\n".len())))
        .ok_or_else(|| "missing closing --- fence".to_string())?;
    let yaml = &rest[..end_marker];
    let body = &rest[end_marker + end_len..];
    Ok((yaml, body))
}

/// Derives a url-safe kebab-case slug from a post title.
/// Strips non-alphanumerics (except spaces and hyphens), lowercases, and
/// collapses runs of whitespace/dashes into single hyphens.
#[must_use]
pub fn derive_slug(title: &str) -> String {
    let mut slug = String::with_capacity(title.len());
    let mut prev_dash = true;
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

#[must_use]
pub fn render_markdown(source: &str) -> String {
    render(source).html
}

pub struct Rendered {
    pub html: String,
    pub toc: Vec<TocEntry>,
}

/// Renders markdown to HTML with highlighted code blocks, gives every heading an id,
/// and collects the `##`/`###` headings for the contents sidebar.
#[must_use]
pub fn render(source: &str) -> Rendered {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(source, options);
    let bundle = syntect();
    // InspiredGitHub is a light theme that ships with syntect defaults.
    // A proper Gruvbox-light theme swap is a follow-up (spec notes it).
    let theme = &bundle.theme_set.themes["InspiredGitHub"];

    let mut html_out = String::new();
    let mut in_code_block: Option<Option<String>> = None;
    let mut code_buffer = String::new();
    let mut events: Vec<Event> = Vec::new();
    // Index of the open heading's Start event in `events`, and its text so far.
    let mut heading: Option<(usize, String)> = None;
    let mut used_ids = HashSet::new();
    let mut toc = Vec::new();

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(ref lang))) => {
                in_code_block = Some(if lang.is_empty() {
                    None
                } else {
                    Some(lang.to_string())
                });
                code_buffer.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some(lang_opt) = in_code_block.take() {
                    let syntax = lang_opt
                        .as_deref()
                        .and_then(|l| bundle.syntax_set.find_syntax_by_token(l))
                        .unwrap_or_else(|| bundle.syntax_set.find_syntax_plain_text());
                    let mut highlighter = HighlightLines::new(syntax, theme);
                    let mut block_html = String::from("<pre class=\"code-block\"><code>");
                    for line in code_buffer.lines() {
                        let regions = highlighter
                            .highlight_line(line, &bundle.syntax_set)
                            .unwrap_or_default();
                        let line_html =
                            styled_line_to_highlighted_html(&regions[..], IncludeBackground::No)
                                .unwrap_or_else(|_| line.to_string());
                        block_html.push_str(&line_html);
                        block_html.push('\n');
                    }
                    block_html.push_str("</code></pre>");
                    events.push(Event::Html(block_html.into()));
                }
            }
            Event::Text(text) if in_code_block.is_some() => {
                code_buffer.push_str(&text);
            }
            Event::Start(Tag::Heading { .. }) => {
                heading = Some((events.len(), String::new()));
                events.push(event);
            }
            Event::End(TagEnd::Heading(level)) => {
                if let Some((start, text)) = heading.take() {
                    let id = unique_id(&derive_slug(&text), &mut used_ids);
                    if let Event::Start(Tag::Heading {
                        level,
                        classes,
                        attrs,
                        ..
                    }) = std::mem::replace(&mut events[start], Event::SoftBreak)
                    {
                        events[start] = Event::Start(Tag::Heading {
                            level,
                            id: Some(id.clone().into()),
                            classes,
                            attrs,
                        });
                    }
                    let toc_level = match level {
                        HeadingLevel::H2 => Some(2),
                        HeadingLevel::H3 => Some(3),
                        _ => None,
                    };
                    if let Some(level) = toc_level {
                        let text = text.trim().to_string();
                        toc.push(TocEntry { level, id, text });
                    }
                }
                events.push(Event::End(TagEnd::Heading(level)));
            }
            other => {
                if in_code_block.is_none() {
                    if let (Some((_, text)), Event::Text(t) | Event::Code(t)) =
                        (heading.as_mut(), &other)
                    {
                        text.push_str(t);
                    }
                    events.push(other);
                }
            }
        }
    }

    debug_assert!(in_code_block.is_none(), "unterminated code block");
    pulldown_cmark::html::push_html(&mut html_out, events.into_iter());
    Rendered {
        html: html_out,
        toc,
    }
}

#[derive(Debug, Clone)]
pub struct Post {
    pub title: String,
    pub slug: String,
    pub date: NaiveDate,
    pub tags: Vec<String>,
    pub rendered_html: String,
    pub draft: bool,
    pub toc: Vec<TocEntry>,
    pub updates: Vec<Update>,
}

impl Post {
    #[must_use]
    pub fn url(&self) -> String {
        format!("/essays/{}", self.slug)
    }
}

/// Reads every `*.md` file in `dir` except the scribbles file, parses frontmatter,
/// renders the body, and returns posts sorted newest-first. Drafts are included iff
/// `include_drafts` is true.
/// Errors on individual files are logged via `tracing::warn!` and the file is skipped.
pub fn load_posts(dir: &std::path::Path, include_drafts: bool) -> Vec<Post> {
    let mut posts = Vec::new();
    let read = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("cannot read posts dir {:?}: {}", dir, e);
            return posts;
        }
    };
    for entry in read {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!("skipping dir entry in {:?}: {}", dir, e);
                continue;
            }
        };
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md")
            || path.file_name().and_then(|s| s.to_str()) == Some(SCRIBBLES_FILE)
        {
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
        let rendered = render(body);
        posts.push(Post {
            title: fm.title,
            slug,
            date: fm.date,
            tags: fm.tags,
            rendered_html: rendered.html,
            draft: fm.draft,
            toc: rendered.toc,
            updates: parse_updates(fm.updates),
        });
    }
    posts.sort_by(|a, b| b.date.cmp(&a.date));
    posts
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scribble {
    /// Anchor id, e.g. `2026-11-03-1120`.
    pub id: String,
    /// Display form of the heading, e.g. `2026-11-03 11:20`.
    pub stamp: String,
    pub at: NaiveDateTime,
    pub html: String,
}

/// Splits the scribbles file into entries at `## YYYY-MM-DD HH:MM` (or `## YYYY-MM-DD`)
/// headings and returns them newest-first. Text before the first heading is ignored.
#[must_use]
pub fn parse_scribbles(source: &str) -> Vec<Scribble> {
    let mut entries: Vec<(NaiveDateTime, bool, String)> = Vec::new();
    for line in source.lines() {
        if let Some((at, has_time)) = line.strip_prefix("## ").and_then(parse_stamp) {
            entries.push((at, has_time, String::new()));
        } else if let Some((_, _, body)) = entries.last_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    entries.sort_by(|a, b| b.0.cmp(&a.0));

    let mut used_ids = HashSet::new();
    entries
        .into_iter()
        .map(|(at, has_time, body)| {
            let (id, stamp) = if has_time {
                (at.format("%Y-%m-%d-%H%M"), at.format("%Y-%m-%d %H:%M"))
            } else {
                (at.format("%Y-%m-%d"), at.format("%Y-%m-%d"))
            };
            Scribble {
                id: unique_id(&id.to_string(), &mut used_ids),
                stamp: stamp.to_string(),
                at,
                html: render_markdown(body.trim()),
            }
        })
        .collect()
}

fn parse_stamp(heading: &str) -> Option<(NaiveDateTime, bool)> {
    let heading = heading.trim();
    if let Ok(at) = NaiveDateTime::parse_from_str(heading, "%Y-%m-%d %H:%M") {
        return Some((at, true));
    }
    NaiveDate::parse_from_str(heading, "%Y-%m-%d")
        .ok()
        .map(|d| (d.and_time(NaiveTime::MIN), false))
}

/// Returns `base`, or `base-2`, `base-3`, ... if it was already handed out.
fn unique_id(base: &str, used: &mut HashSet<String>) -> String {
    let base = if base.is_empty() { "section" } else { base };
    let mut id = base.to_string();
    let mut n = 1;
    while !used.insert(id.clone()) {
        n += 1;
        id = format!("{base}-{n}");
    }
    id
}

/// Reads `dir/scribbles.md`. A missing file just means no scribbles yet.
pub fn load_scribbles(dir: &std::path::Path) -> Vec<Scribble> {
    let path = dir.join(SCRIBBLES_FILE);
    match std::fs::read_to_string(&path) {
        Ok(source) => parse_scribbles(&source),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(e) => {
            tracing::warn!("cannot read {:?}: {}", path, e);
            Vec::new()
        }
    }
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
        assert_eq!(
            derive_slug("Building a Personal Website"),
            "building-a-personal-website"
        );
    }

    #[test]
    fn derive_slug_strips_punctuation() {
        assert_eq!(
            derive_slug("Notes on Rust, Zig & Go!"),
            "notes-on-rust-zig-go"
        );
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

    #[test]
    fn split_frontmatter_handles_crlf_line_endings() {
        let src = "---\r\ntitle: Hi\r\ndate: 2025-01-01\r\n---\r\nBody text.\r\n";
        let (fm, body) = split_frontmatter(src).unwrap();
        assert!(fm.contains("title: Hi"));
        // Body must not start with a stray \r from the closing fence.
        assert!(!body.starts_with('\r'));
        assert_eq!(body, "Body text.\r\n");
    }

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

    #[test]
    fn render_markdown_falls_back_to_plain_text_for_unknown_language() {
        let html = render_markdown("```xyzzy\nsome code\n```");
        assert!(html.contains("<pre"));
        assert!(html.contains("some code"));
    }

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

    #[test]
    fn post_url_is_under_essays() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("e.md"),
            "---\ntitle: Big Idea\ndate: 2025-01-01\n---\nBody.\n",
        )
        .unwrap();
        let posts = load_posts(tmp.path(), false);
        assert_eq!(posts[0].url(), "/essays/big-idea");
    }

    #[test]
    fn load_posts_skips_the_scribbles_file() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join(SCRIBBLES_FILE),
            "## 2026-11-03 10:00\n\nHi.\n",
        )
        .unwrap();
        std::fs::write(
            tmp.path().join("n.md"),
            "---\ntitle: N\ndate: 2025-01-01\n---\nBody.\n",
        )
        .unwrap();
        let posts = load_posts(tmp.path(), false);
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].title, "N");
    }

    #[test]
    fn parse_scribbles_splits_on_stamps_newest_first() {
        let src = "intro that is ignored\n\n\
                   ## 2026-11-01 09:05\n\nOlder *one*.\n\n\
                   ## 2026-11-03 11:20\n\nNewer one.\n\n## Not a stamp\n\nStill the newer one.\n";
        let scribbles = parse_scribbles(src);
        assert_eq!(scribbles.len(), 2);
        assert_eq!(scribbles[0].id, "2026-11-03-1120");
        assert_eq!(scribbles[0].stamp, "2026-11-03 11:20");
        assert!(scribbles[0].html.contains("Still the newer one."));
        assert!(scribbles[0].html.contains(">Not a stamp</h2>"));
        assert_eq!(scribbles[1].id, "2026-11-01-0905");
        assert!(scribbles[1].html.contains("<em>one</em>"));
        assert!(!scribbles[1].html.contains("intro"));
    }

    #[test]
    fn parse_scribbles_accepts_date_only_stamps_and_dedupes_ids() {
        let src = "## 2026-11-02\n\nA.\n\n## 2026-11-02\n\nB.\n";
        let scribbles = parse_scribbles(src);
        assert_eq!(scribbles[0].id, "2026-11-02");
        assert_eq!(scribbles[0].stamp, "2026-11-02");
        assert_eq!(scribbles[1].id, "2026-11-02-2");
    }

    #[test]
    fn load_scribbles_without_a_file_is_empty() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(load_scribbles(tmp.path()).is_empty());
    }

    #[test]
    fn render_gives_headings_ids_and_collects_h2_h3() {
        let md = "# Top\n\n## Why *not* surrender?\n\n### Vibe limit\n\n#### Too deep\n\n## Why not surrender\n";
        let rendered = render(md);
        assert!(rendered.html.contains("<h2 id=\"why-not-surrender\">"));
        assert!(rendered.html.contains("<h2 id=\"why-not-surrender-2\">"));
        assert!(rendered.html.contains("<h4 id=\"too-deep\">"));
        let toc: Vec<(u8, &str, &str)> = rendered
            .toc
            .iter()
            .map(|e| (e.level, e.id.as_str(), e.text.as_str()))
            .collect();
        assert_eq!(
            toc,
            [
                (2, "why-not-surrender", "Why not surrender?"),
                (3, "vibe-limit", "Vibe limit"),
                (2, "why-not-surrender-2", "Why not surrender"),
            ]
        );
    }

    #[test]
    fn render_keeps_inline_code_in_heading_text() {
        let rendered = render("## The `kind` field\n");
        assert_eq!(rendered.toc[0].text, "The kind field");
        assert_eq!(rendered.toc[0].id, "the-kind-field");
    }

    #[test]
    fn updates_accept_lines_and_pairs_sorted_by_date() {
        let yaml = "title: Hi\ndate: 2026-11-03\nupdates:\n  - 2026-11-03 promoted to essay\n  - 2026-10-25: first draft\n  - tightened the argument\n";
        let fm: Frontmatter = serde_yaml::from_str(yaml).unwrap();
        let updates = parse_updates(fm.updates);
        let flat: Vec<(&str, &str)> = updates
            .iter()
            .map(|u| (u.date.as_str(), u.text.as_str()))
            .collect();
        assert_eq!(
            flat,
            [
                ("", "tightened the argument"),
                ("2026-10-25", "first draft"),
                ("2026-11-03", "promoted to essay"),
            ]
        );
    }
}
