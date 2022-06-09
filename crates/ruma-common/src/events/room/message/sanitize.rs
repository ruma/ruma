use std::collections::{HashMap, HashSet};

use maplit::{hashmap, hashset};
use scraper::Html;

/// A sanitizer to filter [HTML tags and attributes] according to the Matrix specification.
///
/// [HTML tags and attributes]: https://spec.matrix.org/v1.2/client-server-api/#mroommessage-msgtypes
#[derive(Debug, Clone)]
pub struct HtmlSanitizer<'a> {
    /// HTML tags that will be left in the output of the sanitizer.
    ///
    /// If this is `None`, all the tags are allowed.
    ///
    /// If this is `Some`, tags that are not present in this list will be removed, but their
    /// children will still be present in the output.
    ///
    /// To remove all tags, set this to `Some(Vec::new())`.
    pub allowed_tags: Option<HashSet<&'a str>>,

    /// HTML tags whose content will be removed.
    ///
    /// These tags will be removed from the output with their children.
    ///
    /// If a tag is both in `allowed_tags` and `remove_content_tags`, the sanitizer will panic.
    pub remove_content_tags: HashSet<&'a str>,

    /// HTML attributes per tag that will be left in the output of the sanitizer.
    ///
    /// If this is `None`, all the attributes are allowed on all tags.
    ///
    /// If this is `Some`, attributes that are not present in this list will be removed, even on
    /// tags that are not listed.
    ///
    /// To remove all attributes, set this to `Some(HashMap::new())`.
    pub allowed_attributes: Option<HashMap<&'a str, HashSet<&'a str>>>,

    /// URI scheme per tag and attribute tuple that will be left in the output of the sanitizer.
    ///
    /// This only checks URIs in `href` and `src` attributes.
    ///
    /// If this is `None`, all the URIs are allowed.
    ///
    /// If this is `Some`, URIs that are not present in this list will be removed, even on
    /// tags and attributes that are not listed.
    ///
    /// If no attribute that allows a URI is allowed, this will have no effect.
    pub allowed_schemes: Option<HashMap<(&'a str, &'a str), HashSet<&'a str>>>,

    /// Class name per tag that will be left in the output of the sanitizer.
    ///
    /// The class names to match allow the following wildcards:
    ///
    /// * `?` matches exactly one occurrence of any character.
    /// * `*` matches arbitrary many (including zero) occurrences of any character.
    ///
    /// If this is `None`, all the class names are allowed.
    ///
    /// If this is `Some`, class names that are not present in this list will be removed, even on
    /// tags that are not listed.
    ///
    /// If no `class` attribute is allowed, this will have no effect.
    pub allowed_classes: Option<HashMap<&'a str, HashSet<&'a str>>>,

    /// The maximum depth at which tags can be nested.
    ///
    /// If this is `None`, any depth is allowed.
    ///
    /// If this is `Some`, all tags deeper than this will be removed.
    pub max_depth: Option<u32>,
}

impl<'a> HtmlSanitizer<'a> {
    /// List of tags allowed in the Matrix specification.
    pub const ALLOWED_TAGS_STRICT_WITH_REPLY: HashSet<&'static str> = hashset! {
        "font",
        "del",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "blockquote",
        "p",
        "a",
        "ul",
        "ol",
        "sup",
        "sub",
        "li",
        "b",
        "i",
        "u",
        "strong",
        "em",
        "strike",
        "code",
        "hr",
        "br",
        "div",
        "table",
        "thead",
        "tbody",
        "tr",
        "th",
        "td",
        "caption",
        "pre",
        "span",
        "img",
        "details",
        "summary",
        "mx-reply",
    };

    pub const ALLOWED_TAGS_STRICT_WITHOUT_REPLY: HashSet<&'static str> = hashset! {
        "font",
        "del",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "blockquote",
        "p",
        "a",
        "ul",
        "ol",
        "sup",
        "sub",
        "li",
        "b",
        "i",
        "u",
        "strong",
        "em",
        "strike",
        "code",
        "hr",
        "br",
        "div",
        "table",
        "thead",
        "tbody",
        "tr",
        "th",
        "td",
        "caption",
        "pre",
        "span",
        "img",
        "details",
        "summary",
    };

    /// The tag name for a rich reply.
    pub const RICH_REPLY_TAG: &'static str = "mx-reply";

    /// Allowed attributes per tag according to the Matrix specification.
    pub const ALLOWED_ATTRIBUTES_STRICT: HashMap<&'static str, HashSet<&'static str>> = hashmap! {
        "font" => hashset!{"data-mx-bg-color", "data-mx-color", "color"},
        "span" => hashset!{"data-mx-bg-color", "data-mx-color", "data-mx-spoiler"},
        "a" => hashset!{"name", "target", "href"},
        "img" => hashset!{"width", "height", "alt", "title", "src"},
        "ol" => hashset!{"start"},
        "code" => hashset!{"class"},
    };

    /// Allowed scheme of URLs per tag according to the Matrix specification.
    pub const ALLOWED_SCHEMES_STRICT: HashMap<(&'static str, &'static str), HashSet<&'static str>> = hashmap! {
        ("a", "href") => hashset!{"http", "https", "ftp", "mailto", "magnet"},
        ("img", "src") => hashset!{"mxc"},
    };

    /// Allowed scheme of URIs per tag, `STRICT_ALLOWED_SCHEME` plus the `matrix:` URI scheme for
    /// links.
    pub const ALLOWED_SCHEMES_COMPAT: HashMap<(&'static str, &'static str), HashSet<&'static str>> = hashmap! {
        ("a", "href") => hashset!{"http", "https", "ftp", "mailto", "magnet", "matrix"},
        ("img", "src") => hashset!{"mxc"},
    };

    /// Allowed classes per tag according to the Matrix specification.
    pub const ALLOWED_CLASSES_STRICT: HashMap<&'static str, HashSet<&'static str>> = hashmap! {
        "code" => hashset!{"language-*"},
    };

    /// Max depth of nested tags allowed by the Matrix specification.
    pub const MAX_DEPTH_STRICT: u32 = 100;

    /// Constructs a `Sanitizer` instance configured with the strict lists.
    ///
    /// It can also optionally remove the [rich reply fallback].
    ///
    /// [rich reply fallback]: https://spec.matrix.org/v1.2/client-server-api/#fallbacks-for-rich-replies
    pub fn new(remove_reply_fallback: RemoveReplyFallback) -> Self {
        let (allowed_tags, remove_content_tags) =
            if remove_reply_fallback == RemoveReplyFallback::Yes {
                (Self::ALLOWED_TAGS_STRICT_WITHOUT_REPLY, hashset! {Self::RICH_REPLY_TAG})
            } else {
                (Self::ALLOWED_TAGS_STRICT_WITH_REPLY, HashSet::new())
            };
        Self {
            allowed_tags: Some(allowed_tags),
            remove_content_tags,
            allowed_attributes: Some(Self::ALLOWED_ATTRIBUTES_STRICT),
            allowed_schemes: Some(Self::ALLOWED_SCHEMES_STRICT),
            allowed_classes: Some(Self::ALLOWED_CLASSES_STRICT),
            max_depth: Some(Self::MAX_DEPTH_STRICT),
        }
    }

    /// Constructs a `Sanitizer` instance configured with the compat lists that are available.
    ///
    /// Defaults to using the strict lists for the parameters without compat lists.
    ///
    /// It can also optionally remove the [rich reply fallback].
    ///
    /// [rich reply fallback]: https://spec.matrix.org/v1.2/client-server-api/#fallbacks-for-rich-replies
    pub fn compat(remove_reply_fallback: RemoveReplyFallback) -> Self {
        let mut sanitizer = Self::new(remove_reply_fallback);
        sanitizer.allowed_schemes = Some(Self::ALLOWED_SCHEMES_COMPAT);
        sanitizer
    }

    /// Constructs a `Sanitizer` that does nothing.
    pub fn empty() -> Self {
        Self {
            allowed_tags: None,
            remove_content_tags: HashSet::new(),
            allowed_attributes: None,
            allowed_schemes: None,
            allowed_classes: None,
            max_depth: None,
        }
    }

    /// Constructs a `Sanitizer` instance that only removes the [rich reply fallback].
    ///
    /// [rich reply fallback]: https://spec.matrix.org/v1.2/client-server-api/#fallbacks-for-rich-replies
    pub fn remove_reply_fallback() -> Self {
        let mut sanitizer = Self::empty();
        sanitizer.remove_content_tags = hashset!(Self::RICH_REPLY_TAG);
        sanitizer
    }

    /// Clean the given HTML string with this sanitizer.
    pub fn clean(&self, html: &str) -> String {
        let html = Html::parse_fragment(html);
        let root = html.root_element();
        for child in root.children() {
            self.clean_node(child, 1)
        }
        root.html()
    }

    fn clean_node(&self, node: NodeRef<Node>, depth: u32) -> NodeRef<Node> {
        node
    }
}

/// Sanitize the given HTML string.
///
/// This removes the [tags and attributes] that are not listed in the Matrix specification.
///
/// It can also optionally remove the [rich reply fallback].
///
/// [tags and attributes]: https://spec.matrix.org/v1.2/client-server-api/#mroommessage-msgtypes
/// [rich reply fallback]: https://spec.matrix.org/v1.2/client-server-api/#fallbacks-for-rich-replies
#[cfg(feature = "sanitize")]
pub fn sanitize_html(s: &str, remove_reply_fallback: RemoveReplyFallback) -> String {
    let mut builder = ammonia::Builder::empty();
    builder.add_url_schemes(allowed_urls).add_tags(allowed_tags).link_rel(Some("noopener"));

    for (tag, attr) in allowed_attributes {
        builder.add_tag_attributes(tag, attr);
    }

    if remove_reply_fallback == RemoveReplyFallback::Yes {
        builder.add_clean_content_tags(["mx-reply"]);
    } else {
        builder.add_tags(["mx-reply"]);
    }

    builder.clean(s).to_string()
}

/// Whether to remove the [rich reply fallback] while sanitizing.
///
/// [rich reply fallback]: https://spec.matrix.org/v1.2/client-server-api/#fallbacks-for-rich-replies
#[cfg(feature = "sanitize")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::exhaustive_enums)]
pub enum RemoveReplyFallback {
    /// Remove the rich reply fallback.
    Yes,

    /// Don't remove the rich reply fallback.
    No,
}

/// Remove the [rich reply fallback] of the given plain text string.
///
/// [rich reply fallback]: https://spec.matrix.org/v1.2/client-server-api/#fallbacks-for-rich-replies
pub fn remove_plain_reply_fallback(s: &str) -> &str {
    if !s.starts_with("> ") {
        s
    } else {
        let mut start = 0;
        for (pos, _) in s.match_indices('\n') {
            if !&s[pos + 1..].starts_with("> ") {
                start = pos + 1;
                break;
            }
        }
        &s[start..]
    }
}

#[cfg(test)]
mod tests {
    use super::remove_plain_reply_fallback;
    #[cfg(feature = "sanitize")]
    use super::{sanitize_html, RemoveReplyFallback};

    #[test]
    #[cfg(feature = "sanitize")]
    fn sanitize() {
        let sanitized = sanitize_html(
            "\
            <mx-reply>\
                <blockquote>\
                    <a href=\"https://matrix.to/#/!n8f893n9:example.com/$1598361704261elfgc:localhost\">In reply to</a> \
                    <a href=\"https://matrix.to/#/@alice:example.com\">@alice:example.com</a>\
                    <br>\
                    Previous message\
                </blockquote>\
            </mx-reply>\
            <removed>This has no tag</removed>\
            <p>But this is inside a tag</p>\
            ",
            RemoveReplyFallback::No
        );

        assert_eq!(
            sanitized,
            "\
            <mx-reply>\
                <blockquote>\
                    <a href=\"https://matrix.to/#/!n8f893n9:example.com/$1598361704261elfgc:localhost\" rel=\"noopener\">In reply to</a> \
                    <a href=\"https://matrix.to/#/@alice:example.com\" rel=\"noopener\">@alice:example.com</a>\
                    <br>\
                    Previous message\
                </blockquote>\
            </mx-reply>\
            This has no tag\
            <p>But this is inside a tag</p>\
            "
        );
    }

    #[test]
    #[cfg(feature = "sanitize")]
    fn sanitize_without_reply() {
        let sanitized = sanitize_html(
            "\
            <mx-reply>\
                <blockquote>\
                    <a href=\"https://matrix.to/#/!n8f893n9:example.com/$1598361704261elfgc:localhost\">In reply to</a> \
                    <a href=\"https://matrix.to/#/@alice:example.com\">@alice:example.com</a>\
                    <br>\
                    Previous message\
                </blockquote>\
            </mx-reply>\
            <removed>This has no tag</removed>\
            <p>But this is inside a tag</p>\
            ",
            RemoveReplyFallback::Yes
        );

        assert_eq!(
            sanitized,
            "\
            This has no tag\
            <p>But this is inside a tag</p>\
            "
        );
    }

    #[test]
    fn remove_plain_reply() {
        assert_eq!(
            remove_plain_reply_fallback("No reply here\nJust a simple message"),
            "No reply here\nJust a simple message"
        );

        assert_eq!(
            remove_plain_reply_fallback(
                "> <@user:notareal.hs> Replied to on\n> two lines\nThis is my reply"
            ),
            "This is my reply"
        );

        assert_eq!(remove_plain_reply_fallback("\n> Not on first line"), "\n> Not on first line");

        assert_eq!(
            remove_plain_reply_fallback("> <@user:notareal.hs> Previous message\n\n> New quote"),
            "\n> New quote"
        );
    }
}
