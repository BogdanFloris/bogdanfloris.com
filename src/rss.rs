use crate::content::Post;

const SITE_URL: &str = "https://bogdanfloris.com";
const SITE_TITLE: &str = "bogdan floris";
const SITE_DESCRIPTION: &str = "Notes on software, systems, and side quests.";

/// Renders an RSS 2.0 feed for the given posts.
pub fn build_feed(posts: &[&Post]) -> String {
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\">\n");
    out.push_str("<channel>\n");
    push_tag(&mut out, "title", SITE_TITLE);
    push_tag(&mut out, "link", SITE_URL);
    push_tag(&mut out, "description", SITE_DESCRIPTION);
    out.push_str("<language>en</language>\n");

    for post in posts {
        out.push_str("<item>\n");
        push_tag(&mut out, "title", &post.title);
        let url = format!("{}/post/{}", SITE_URL, post.slug);
        push_tag(&mut out, "link", &url);
        push_tag(&mut out, "guid", &url);
        let pub_date = post
            .date
            .and_hms_opt(0, 0, 0)
            .map(|dt| dt.format("%a, %d %b %Y %H:%M:%S +0000").to_string())
            .unwrap_or_default();
        push_tag(&mut out, "pubDate", &pub_date);
        out.push_str("<description><![CDATA[");
        out.push_str(&post.rendered_html);
        out.push_str("]]></description>\n");
        out.push_str("</item>\n");
    }

    out.push_str("</channel>\n</rss>\n");
    out
}

fn push_tag(out: &mut String, name: &str, value: &str) {
    out.push('<');
    out.push_str(name);
    out.push('>');
    xml_escape(value, out);
    out.push_str("</");
    out.push_str(name);
    out.push_str(">\n");
}

fn xml_escape(input: &str, out: &mut String) {
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            c => out.push(c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::Post;
    use chrono::NaiveDate;

    fn sample_post(title: &str, slug: &str, date: &str) -> Post {
        Post {
            title: title.to_string(),
            slug: slug.to_string(),
            date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            tags: vec![],
            rendered_html: "<p>body</p>".to_string(),
            draft: false,
        }
    }

    #[test]
    fn build_feed_contains_channel_metadata() {
        let posts = [sample_post("Hi", "hi", "2025-01-01")];
        let refs: Vec<&Post> = posts.iter().collect();
        let xml = build_feed(&refs);
        assert!(xml.starts_with("<?xml"));
        assert!(xml.contains("<rss"));
        assert!(xml.contains("<channel>"));
        assert!(xml.contains("<title>bogdan floris</title>"));
        assert!(xml.contains("<link>https://bogdanfloris.com</link>"));
    }

    #[test]
    fn build_feed_includes_one_item_per_post() {
        let posts = [
            sample_post("First", "first", "2025-01-01"),
            sample_post("Second", "second", "2025-02-01"),
        ];
        let refs: Vec<&Post> = posts.iter().collect();
        let xml = build_feed(&refs);
        assert_eq!(xml.matches("<item>").count(), 2);
        assert!(xml.contains("https://bogdanfloris.com/post/first"));
        assert!(xml.contains("https://bogdanfloris.com/post/second"));
    }

    #[test]
    fn build_feed_escapes_html_in_titles() {
        let posts = [sample_post("A & B <c>", "a", "2025-01-01")];
        let refs: Vec<&Post> = posts.iter().collect();
        let xml = build_feed(&refs);
        assert!(xml.contains("A &amp; B &lt;c&gt;"));
        assert!(!xml.contains("A & B <c>"));
    }
}
