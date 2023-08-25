use html5ever::{tendril::StrTendril, Attribute};
use phf::{phf_map, phf_set, Map, Set};
use wildmatch::WildMatch;

use super::{HtmlSanitizerMode, RemoveReplyFallback};
use crate::html::{ElementData, Html, NodeData};

/// A sanitizer to filter [HTML tags and attributes] according to the Matrix specification.
///
/// [HTML tags and attributes]: https://spec.matrix.org/latest/client-server-api/#mroommessage-msgtypes
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
    /// [rich reply fallback]: https://spec.matrix.org/latest/client-server-api/#fallbacks-for-rich-replies
    pub fn new(mode: HtmlSanitizerMode, remove_reply_fallback: RemoveReplyFallback) -> Self {
        Self {
            mode,
            filter_tags_attributes: true,
            remove_replies: remove_reply_fallback == RemoveReplyFallback::Yes,
        }
    }

    /// Constructs a `HTMLSanitizer` instance that only removes the [rich reply fallback].
    ///
    /// [rich reply fallback]: https://spec.matrix.org/latest/client-server-api/#fallbacks-for-rich-replies
    pub fn reply_fallback_remover() -> Self {
        Self {
            mode: HtmlSanitizerMode::Strict,
            filter_tags_attributes: false,
            remove_replies: true,
        }
    }

    /// Clean the given HTML string with this sanitizer.
    pub fn clean(&self, html: &str) -> Html {
        let mut fragment = Html::parse(html);

        let root = fragment.nodes[0].first_child.unwrap();
        let mut next_child = fragment.nodes[root].first_child;
        while let Some(child) = next_child {
            next_child = fragment.nodes[child].next_sibling;
            self.clean_node(&mut fragment, child, 0);
        }

        fragment
    }

    fn clean_node(&self, fragment: &mut Html, node_id: usize, depth: u32) {
        let action = self.node_action(fragment, node_id, depth);

        if action != NodeAction::Remove {
            let mut next_child = fragment.nodes[node_id].first_child;
            while let Some(child) = next_child {
                next_child = fragment.nodes[child].next_sibling;

                if action == NodeAction::Ignore {
                    fragment.insert_before(node_id, child);
                }

                self.clean_node(fragment, child, depth + 1);
            }
        }

        if matches!(action, NodeAction::Ignore | NodeAction::Remove) {
            fragment.detach(node_id);
        } else if self.filter_tags_attributes {
            if let Some(data) = fragment.nodes[node_id].as_element_mut() {
                self.clean_element_attributes(data);
            }
        }
    }

    fn node_action(&self, fragment: &Html, node_id: usize, depth: u32) -> NodeAction {
        match &fragment.nodes[node_id].data {
            NodeData::Element(ElementData { name, attrs, .. }) => {
                let tag: &str = &name.local;

                if (self.remove_replies && tag == RICH_REPLY_TAG)
                    || (self.filter_tags_attributes && depth >= MAX_DEPTH_STRICT)
                {
                    NodeAction::Remove
                } else if self.filter_tags_attributes
                    && (!ALLOWED_TAGS_WITHOUT_REPLY_STRICT.contains(tag) && tag != RICH_REPLY_TAG)
                {
                    NodeAction::Ignore
                } else if self.filter_tags_attributes {
                    let allowed_schemes = if self.mode == HtmlSanitizerMode::Strict {
                        &ALLOWED_SCHEMES_STRICT
                    } else {
                        &ALLOWED_SCHEMES_COMPAT
                    };
                    for attr in attrs.iter() {
                        let value = &attr.value;
                        let attr: &str = &attr.name.local;

                        // Check if there is a (tag, attr) tuple entry.
                        if let Some(schemes) = allowed_schemes.get(&*format!("{tag}:{attr}")) {
                            // Check if the scheme is allowed.
                            if !schemes
                                .iter()
                                .any(|scheme| value.starts_with(&format!("{scheme}:")))
                            {
                                return NodeAction::Ignore;
                            }
                        }
                    }
                    NodeAction::None
                } else {
                    NodeAction::None
                }
            }
            NodeData::Text(_) => NodeAction::None,
            _ => NodeAction::Remove,
        }
    }

    fn clean_element_attributes(&self, data: &mut ElementData) {
        let ElementData { name, attrs } = data;
        let tag: &str = &name.local;

        let actions: Vec<_> = attrs
            .iter()
            .filter_map(|attr| {
                let value = &attr.value;
                let name: &str = &attr.name.local;

                if ALLOWED_ATTRIBUTES_STRICT.get(tag).filter(|attrs| attrs.contains(name)).is_none()
                {
                    return Some(AttributeAction::Remove(attr.to_owned()));
                }

                if name == "class" {
                    if let Some(classes) = ALLOWED_CLASSES_STRICT.get(tag) {
                        let mut changed = false;
                        let attr_classes = value.split_whitespace().filter(|attr_class| {
                            for class in classes.iter() {
                                if WildMatch::new(class).matches(attr_class) {
                                    return true;
                                }
                            }
                            changed = true;
                            false
                        });

                        let folded_classes = attr_classes.fold(String::new(), |mut a, b| {
                            a.reserve(b.len() + 1);
                            a.push_str(b);
                            a.push('\n');
                            a
                        });
                        let final_classes = folded_classes.trim_end();

                        if changed {
                            if final_classes.is_empty() {
                                return Some(AttributeAction::Remove(attr.to_owned()));
                            } else {
                                return Some(AttributeAction::ReplaceValue(
                                    attr.to_owned(),
                                    final_classes.to_owned().into(),
                                ));
                            }
                        }
                    }
                }

                None
            })
            .collect();

        for action in actions {
            match action {
                AttributeAction::ReplaceValue(attr, value) => {
                    if let Some(mut attr) = attrs.take(&attr) {
                        attr.value = value;
                        attrs.insert(attr);
                    }
                }
                AttributeAction::Remove(attr) => {
                    attrs.remove(&attr);
                }
            }
        }
    }
}

/// The possible actions to apply to an element node.
#[derive(Debug, PartialEq, Eq)]
enum NodeAction {
    /// Don't do anything.
    None,

    /// Remove the element but keep its children.
    Ignore,

    /// Remove the element and its children.
    Remove,
}

/// The possible actions to apply to an element node.
#[derive(Debug)]
enum AttributeAction {
    /// Replace the value of the attribute.
    ReplaceValue(Attribute, StrTendril),

    /// Remove the element and its children.
    Remove(Attribute),
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
            sanitized.to_string(),
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
            sanitized.to_string(),
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
            sanitized.to_string(),
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
            sanitized.to_string(),
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
            sanitized.to_string(),
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
            sanitized.to_string(),
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
            sanitized.to_string(),
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
            sanitized.to_string(),
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
            sanitized.to_string(),
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
            sanitized.to_string(),
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

        let sanitized = sanitizer.clean(&deeply_nested_html).to_string();

        assert!(sanitized.contains("I should be fine."));
        assert!(!sanitized.contains("I am in too deep!"));
    }
}
