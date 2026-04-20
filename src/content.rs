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
