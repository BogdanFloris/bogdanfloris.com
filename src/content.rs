use chrono::NaiveDate;
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use serde::Deserialize;
use std::sync::OnceLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::html::{styled_line_to_highlighted_html, IncludeBackground};
use syntect::parsing::SyntaxSet;

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

pub fn render_markdown(source: &str) -> String {
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
                    let mut highlighted = String::from("<pre class=\"code-block\"><code>");
                    for line in code_buffer.lines() {
                        let regions = highlighter
                            .highlight_line(line, &bundle.syntax_set)
                            .unwrap_or_default();
                        let line_html =
                            styled_line_to_highlighted_html(&regions[..], IncludeBackground::No)
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

    debug_assert!(in_code_block.is_none(), "unterminated code block");
    pulldown_cmark::html::push_html(&mut html_out, events.into_iter());
    html_out
}

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
}
