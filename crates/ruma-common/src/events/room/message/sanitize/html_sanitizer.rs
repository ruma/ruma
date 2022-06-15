use html5ever::{local_name, namespace_url, ns, QualName};
use kuchiki::{parse_fragment, traits::TendrilSink, Attributes, ElementData, NodeData, NodeRef};
use phf::{phf_map, phf_set, Map, Set};
use wildmatch::WildMatch;

use super::{HtmlSanitizerMode, RemoveReplyFallback};

/// A sanitizer to filter [HTML tags and attributes] according to the Matrix specification.
///
/// [HTML tags and attributes]: https://spec.matrix.org/v1.2/client-server-api/#mroommessage-msgtypes
#[derive(Debug, Clone)]
pub struct HtmlSanitizer {
    /// The mode of the HTML sanitizer.
    mode: HtmlSanitizerMode,

    /// Whether to filter HTML tags and attributes.
    ///
    /// If this is `true`, tags and attributes that do not match the lists will be removed, but
    /// the tags' children will still be present in the output.
    ///
    /// If this is `false`, all the tags and attributes are allowed.
    filter_tags_attributes: bool,

    /// Whether to remove replies.
    ///
    /// If this is `true`, the rich reply fallback will be removed.
    ///
    /// If this is `false`, the rich reply tag will be allowed.
    remove_replies: bool,
}

impl HtmlSanitizer {
    /// Constructs a `HTMLSanitizer` that will filter the tags and attributes according to the given
    /// mode.
    ///
    /// It can also optionally remove the [rich reply fallback].
    ///
    /// [rich reply fallback]: https://spec.matrix.org/v1.2/client-server-api/#fallbacks-for-rich-replies
    pub fn new(mode: HtmlSanitizerMode, remove_reply_fallback: RemoveReplyFallback) -> Self {
        Self {
            mode,
            filter_tags_attributes: true,
            remove_replies: remove_reply_fallback == RemoveReplyFallback::Yes,
        }
    }

    /// Constructs a `HTMLSanitizer` instance that only removes the [rich reply fallback].
    ///
    /// [rich reply fallback]: https://spec.matrix.org/v1.2/client-server-api/#fallbacks-for-rich-replies
    pub fn reply_fallback_remover() -> Self {
        Self {
            mode: HtmlSanitizerMode::Strict,
            filter_tags_attributes: false,
            remove_replies: true,
        }
    }

    /// Clean the given HTML string with this sanitizer.
    pub fn clean(&self, html: &str) -> String {
        let mut parser =
            parse_fragment(QualName::new(None, ns!(html), local_name!("div")), Vec::new());
        parser.process(html.into());
        let dom = parser.finish();
        let root = dom.first_child().unwrap();

        for child in root.children() {
            self.clean_node(child, 0);
        }

        let mut buf: Vec<u8> = Vec::new();
        for child in root.children() {
            child.serialize(&mut buf).unwrap();
        }

        String::from_utf8(buf).unwrap()
    }

    fn clean_node(&self, node: NodeRef, depth: u32) {
        match node.data() {
            NodeData::Element(ElementData { name, attributes, .. }) => {
                let tag: &str = &name.local;
                let action = self.element_action(tag, &attributes.borrow(), depth);

                if action != ElementAction::Remove {
                    for child in node.children() {
                        if action == ElementAction::Ignore {
                            node.insert_before(child.clone());
                        }
                        self.clean_node(child, depth + 1);
                    }
                }

                if matches!(action, ElementAction::Ignore | ElementAction::Remove) {
                    node.detach();
                } else if self.filter_tags_attributes {
                    self.clean_attributes(&mut attributes.borrow_mut(), tag);
                }
            }
            NodeData::Text(_) => {}
            _ => node.detach(),
        }
    }

    fn element_action(&self, tag: &str, attributes: &Attributes, depth: u32) -> ElementAction {
        if (self.remove_replies && tag == RICH_REPLY_TAG)
            || (self.filter_tags_attributes && depth >= MAX_DEPTH_STRICT)
        {
            ElementAction::Remove
        } else if self.filter_tags_attributes
            && (!ALLOWED_TAGS_WITHOUT_REPLY_STRICT.contains(tag) && tag != RICH_REPLY_TAG)
        {
            ElementAction::Ignore
        } else if self.filter_tags_attributes {
            let allowed_schemes = if self.mode == HtmlSanitizerMode::Strict {
                &ALLOWED_SCHEMES_STRICT
            } else {
                &ALLOWED_SCHEMES_COMPAT
            };
            for (name, val) in &attributes.map {
                let attr: &str = &name.local;

                // Check if there is a (tag, attr) tuple entry.
                if let Some(schemes) = allowed_schemes.get(&*format!("{tag}:{attr}")) {
                    // Check if the scheme is allowed.
                    if !schemes.iter().any(|scheme| val.value.starts_with(&format!("{scheme}:"))) {
                        return ElementAction::Ignore;
                    }
                }
            }
            ElementAction::None
        } else {
            ElementAction::None
        }
    }

    fn clean_attributes(&self, attributes: &mut Attributes, tag: &str) {
        let removed_attributes: Vec<_> = attributes
            .map
            .iter_mut()
            .filter_map(|(name, mut val)| {
                let attr: &str = &name.local;

                if ALLOWED_ATTRIBUTES_STRICT.get(tag).filter(|attrs| attrs.contains(attr)).is_none()
                {
                    return Some(attr.to_owned());
                }

                if attr == "class" {
                    if let Some(classes) = ALLOWED_CLASSES_STRICT.get(tag) {
                        let attr_classes = val.value.split_whitespace().filter(|attr_class| {
                            for class in classes.iter() {
                                if WildMatch::new(class).matches(attr_class) {
                                    return true;
                                }
                            }
                            false
                        });
                        let folded_classes = attr_classes.fold(String::new(), |mut a, b| {
                            a.reserve(b.len() + 1);
                            a.push_str(b);
                            a.push('\n');
                            a
                        });
                        let final_classes = folded_classes.trim_end();

                        if final_classes.is_empty() {
                            return Some(attr.to_owned());
                        } else if val.value != final_classes {
                            val.value = final_classes.to_owned();
                        }
                    }
                }

                None
            })
            .collect();

        for name in removed_attributes {
            attributes.remove(name);
        }
    }
}

/// The possible actions to apply to an element node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ElementAction {
    /// Don't do anything.
    None,

    /// Remove the element but keep its children.
    Ignore,

    /// Remove the element and its children.
    Remove,
}

/// List of HTML tags allowed in the Matrix specification, without the rich reply fallback tag.
static ALLOWED_TAGS_WITHOUT_REPLY_STRICT: Set<&str> = phf_set! {
    "font", "del", "h1", "h2", "h3", "h4", "h5", "h6", "blockquote", "p", "a",
    "ul", "ol", "sup", "sub", "li", "b", "i", "u", "strong", "em", "strike",
    "code", "hr", "br", "div", "table", "thead", "tbody", "tr", "th", "td",
    "caption", "pre", "span", "img", "details", "summary",
};

/// The HTML tag name for a rich reply fallback.
const RICH_REPLY_TAG: &str = "mx-reply";

/// Allowed attributes per HTML tag according to the Matrix specification.
static ALLOWED_ATTRIBUTES_STRICT: Map<&str, &Set<&str>> = phf_map! {
    "font" => &ALLOWED_ATTRIBUTES_FONT_STRICT,
    "span" => &ALLOWED_ATTRIBUTES_SPAN_STRICT,
    "a" => &ALLOWED_ATTRIBUTES_A_STRICT,
    "img" => &ALLOWED_ATTRIBUTES_IMG_STRICT,
    "ol" => &ALLOWED_ATTRIBUTES_OL_STRICT,
    "code" => &ALLOWED_ATTRIBUTES_CODE_STRICT,
};
static ALLOWED_ATTRIBUTES_FONT_STRICT: Set<&str> =
    phf_set! { "data-mx-bg-color", "data-mx-color", "color" };
static ALLOWED_ATTRIBUTES_SPAN_STRICT: Set<&str> =
    phf_set! { "data-mx-bg-color", "data-mx-color", "data-mx-spoiler" };
static ALLOWED_ATTRIBUTES_A_STRICT: Set<&str> = phf_set! { "name", "target", "href" };
static ALLOWED_ATTRIBUTES_IMG_STRICT: Set<&str> =
    phf_set! { "width", "height", "alt", "title", "src" };
static ALLOWED_ATTRIBUTES_OL_STRICT: Set<&str> = phf_set! { "start" };
static ALLOWED_ATTRIBUTES_CODE_STRICT: Set<&str> = phf_set! { "class" };

/// Allowed schemes of URIs per HTML tag and attribute tuple according to the Matrix specification.
static ALLOWED_SCHEMES_STRICT: Map<&str, &Set<&str>> = phf_map! {
    "a:href" => &ALLOWED_SCHEMES_A_HREF_STRICT,
    "img:src" => &ALLOWED_SCHEMES_IMG_SRC_STRICT,
};
static ALLOWED_SCHEMES_A_HREF_STRICT: Set<&str> =
    phf_set! { "http", "https", "ftp", "mailto", "magnet" };
static ALLOWED_SCHEMES_IMG_SRC_STRICT: Set<&str> = phf_set! { "mxc" };

/// Extra allowed schemes of URIs per HTML tag and attribute tuple.
///
/// This is a convenience list to add schemes that can be encountered but are not listed in the
/// Matrix specification. It consists of:
///
/// * The `matrix` scheme for `a` tags (see [matrix-org/matrix-spec#1108]).
///
/// To get a complete list, add these to `ALLOWED_SCHEMES_STRICT`.
///
/// [matrix-org/matrix-spec#1108]: https://github.com/matrix-org/matrix-spec/issues/1108
static ALLOWED_SCHEMES_COMPAT: Map<&str, &Set<&str>> = phf_map! {
    "a:href" => &ALLOWED_SCHEMES_A_HREF_COMPAT,
    "img:src" => &ALLOWED_SCHEMES_IMG_SRC_STRICT,
};
static ALLOWED_SCHEMES_A_HREF_COMPAT: Set<&str> =
    phf_set! { "http", "https", "ftp", "mailto", "magnet", "matrix" };

/// Allowed classes per HTML tag according to the Matrix specification.
static ALLOWED_CLASSES_STRICT: Map<&str, &Set<&str>> =
    phf_map! { "code" => &ALLOWED_CLASSES_CODE_STRICT };
static ALLOWED_CLASSES_CODE_STRICT: Set<&str> = phf_set! { "language-*" };

/// Max depth of nested HTML tags allowed by the Matrix specification.
const MAX_DEPTH_STRICT: u32 = 100;

#[cfg(test)]
mod tests {
    use super::{HtmlSanitizer, HtmlSanitizerMode, RemoveReplyFallback};

    #[test]
    fn valid_input() {
        let sanitizer = HtmlSanitizer::new(HtmlSanitizerMode::Strict, RemoveReplyFallback::Yes);
        let sanitized = sanitizer.clean(
            "\
            <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
            <p>This is a paragraph <span data-mx-color=\"green\">with some color</span></p>\
            <img src=\"mxc://notareal.hs/abcdef\">\
            <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
            ",
        );

        assert_eq!(
            sanitized,
            "\
            <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
            <p>This is a paragraph <span data-mx-color=\"green\">with some color</span></p>\
            <img src=\"mxc://notareal.hs/abcdef\">\
            <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
            "
        );
    }

    #[test]
    fn tags_remove() {
        let sanitizer = HtmlSanitizer::new(HtmlSanitizerMode::Strict, RemoveReplyFallback::No);
        let sanitized = sanitizer.clean(
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
        );

        assert_eq!(
            sanitized,
            "\
            <mx-reply>\
                <blockquote>\
                    <a href=\"https://matrix.to/#/!n8f893n9:example.com/$1598361704261elfgc:localhost\">In reply to</a> \
                    <a href=\"https://matrix.to/#/@alice:example.com\">@alice:example.com</a>\
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
    fn tags_remove_without_reply() {
        let sanitizer = HtmlSanitizer::new(HtmlSanitizerMode::Strict, RemoveReplyFallback::Yes);
        let sanitized = sanitizer.clean(
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
    fn tags_remove_only_reply_fallback() {
        let sanitizer = HtmlSanitizer::reply_fallback_remover();
        let sanitized = sanitizer.clean(
            "\
            <mx-reply>\
                <blockquote>\
                    <a href=\"https://matrix.to/#/!n8f893n9:example.com/$1598361704261elfgc:localhost\">In reply to</a> \
                    <a href=\"https://matrix.to/#/@alice:example.com\">@alice:example.com</a>\
                    <br>\
                    Previous message\
                </blockquote>\
            </mx-reply>\
            <keep-me>This keeps its tag</keep-me>\
            <p>But this is inside a tag</p>\
            ",
        );

        assert_eq!(
            sanitized,
            "\
            <keep-me>This keeps its tag</keep-me>\
            <p>But this is inside a tag</p>\
            "
        );
    }

    #[test]
    fn attrs_remove() {
        let sanitizer = HtmlSanitizer::new(HtmlSanitizerMode::Strict, RemoveReplyFallback::No);
        let sanitized = sanitizer.clean(
            "\
            <h1 id=\"anchor1\">Title for important stuff</h1>\
            <p class=\"important\">Look at <font color=\"blue\" size=20>me!</font></p>\
            ",
        );

        assert_eq!(
            sanitized,
            "\
            <h1>Title for important stuff</h1>\
            <p>Look at <font color=\"blue\">me!</font></p>\
            "
        );
    }

    #[test]
    fn img_remove_scheme() {
        let sanitizer = HtmlSanitizer::new(HtmlSanitizerMode::Strict, RemoveReplyFallback::No);
        let sanitized = sanitizer.clean(
            "\
            <p>Look at that picture:</p>\
            <img src=\"https://notareal.hs/abcdef\">\
            ",
        );

        assert_eq!(
            sanitized,
            "\
            <p>Look at that picture:</p>\
            "
        );
    }

    #[test]
    fn link_remove_scheme() {
        let sanitizer = HtmlSanitizer::new(HtmlSanitizerMode::Strict, RemoveReplyFallback::No);
        let sanitized = sanitizer.clean(
            "\
            <p>Go see <a href=\"file://local/file.html\">my local website</a></p>\
            ",
        );

        assert_eq!(
            sanitized,
            "\
            <p>Go see my local website</p>\
            "
        );
    }

    #[test]
    fn link_compat_scheme() {
        let sanitizer = HtmlSanitizer::new(HtmlSanitizerMode::Strict, RemoveReplyFallback::No);
        let sanitized = sanitizer.clean(
            "\
            <p>Join <a href=\"matrix:r/myroom:notareal.hs\">my room</a></p>\
            <p>To talk about <a href=\"https://mycat.org\">my cat</a></p>\
            ",
        );
        assert_eq!(
            sanitized,
            "\
            <p>Join my room</p>\
            <p>To talk about <a href=\"https://mycat.org\">my cat</a></p>\
            "
        );

        let sanitizer = HtmlSanitizer::new(HtmlSanitizerMode::Compat, RemoveReplyFallback::No);
        let sanitized = sanitizer.clean(
            "\
            <p>Join <a href=\"matrix:r/myroom:notareal.hs\">my room</a></p>\
            <p>To talk about <a href=\"https://mycat.org\">my cat</a></p>\
            ",
        );
        assert_eq!(
            sanitized,
            "\
            <p>Join <a href=\"matrix:r/myroom:notareal.hs\">my room</a></p>\
            <p>To talk about <a href=\"https://mycat.org\">my cat</a></p>\
            "
        );
    }

    #[test]
    fn class_remove() {
        let sanitizer = HtmlSanitizer::new(HtmlSanitizerMode::Strict, RemoveReplyFallback::No);
        let sanitized = sanitizer.clean(
            "\
            <pre><code class=\"language-rust custom-class\">
                type StringList = Vec&lt;String&gt;;
            </code></pre>\
            <p>What do you think of the name <code class=\"fake-language-rust\">StringList</code>?</p>\
            ",
        );

        assert_eq!(
            sanitized,
            "\
            <pre><code class=\"language-rust\">
                type StringList = Vec&lt;String&gt;;
            </code></pre>\
            <p>What do you think of the name <code>StringList</code>?</p>\
            "
        );
    }

    #[test]
    fn depth_remove() {
        let sanitizer = HtmlSanitizer::new(HtmlSanitizerMode::Strict, RemoveReplyFallback::No);
        let deeply_nested_html: String = std::iter::repeat("<div>")
            .take(100)
            .chain(Some(
                "<span>I am in too deep!</span>\
                I should be fine.",
            ))
            .chain(std::iter::repeat("</div>").take(100))
            .collect();
        println!("{deeply_nested_html}");

        let sanitized = sanitizer.clean(&deeply_nested_html);

        assert!(sanitized.contains("I should be fine."));
        assert!(!sanitized.contains("I am in too deep!"));
    }
}
