use std::collections::{BTreeMap, BTreeSet};

use html5ever::{local_name, namespace_url, ns, QualName};
use kuchiki::{parse_fragment, traits::TendrilSink, Attributes, ElementData, NodeData, NodeRef};
use wildmatch::WildMatch;

use super::{
    RemoveReplyFallback, ALLOWED_ATTRIBUTES_STRICT, ALLOWED_CLASSES_STRICT, ALLOWED_SCHEMES_COMPAT,
    ALLOWED_SCHEMES_STRICT, ALLOWED_TAGS_STRICT_WITHOUT_REPLY, MAX_DEPTH_STRICT, RICH_REPLY_TAG,
};

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
    pub allowed_tags: Option<BTreeSet<&'a str>>,

    /// HTML tags whose content will be removed.
    ///
    /// These tags will be removed from the output with their children.
    ///
    /// If a tag is both in `allowed_tags` and `remove_content_tags`, the behavior is unexpected.
    pub remove_content_tags: BTreeSet<&'a str>,

    /// HTML attributes per tag that will be left in the output of the sanitizer.
    ///
    /// If this is `None`, all the attributes are allowed on all tags.
    ///
    /// If this is `Some`, attributes that are not present in this list will be removed, even on
    /// tags that are not listed.
    ///
    /// To remove all attributes, set this to `Some(BTreeMap::new())`.
    pub allowed_attributes: Option<BTreeMap<&'a str, BTreeSet<&'a str>>>,

    /// URI scheme per tag and attribute tuple that will be left in the output of the sanitizer.
    ///
    /// If this is `None`, all the URIs are allowed.
    ///
    /// If this is `Some`, URIs that are not present in this list will be removed. Tags and
    /// attributes tuples that are not in this list are ignored.
    ///
    /// If no attribute that allows a URI is allowed, this will have no effect.
    pub allowed_schemes: Option<BTreeMap<(&'a str, &'a str), BTreeSet<&'a str>>>,

    /// Class name per tag that will be left in the output of the sanitizer.
    ///
    /// The class names to match allow the following wildcards:
    ///
    /// * `?` matches exactly one occurrence of any character.
    /// * `*` matches arbitrary many (including zero) occurrences of any character.
    ///
    /// If this is `None`, all the class names are allowed.
    ///
    /// If this is `Some`, class names that are not present in this list will be removed. Tags that
    /// are not in this list are ignored.
    ///
    /// If no `class` attribute is allowed, this will have no effect.
    pub allowed_classes: Option<BTreeMap<&'a str, BTreeSet<&'a str>>>,

    /// The maximum depth at which tags can be nested.
    ///
    /// If this is `None`, any depth is allowed.
    ///
    /// If this is `Some`, all tags deeper than this will be removed.
    pub max_depth: Option<u32>,
}

impl<'a> HtmlSanitizer<'a> {
    /// Constructs a `HTMLSanitizer` configured with the strict lists.
    ///
    /// It can also optionally remove the [rich reply fallback].
    ///
    /// [rich reply fallback]: https://spec.matrix.org/v1.2/client-server-api/#fallbacks-for-rich-replies
    pub fn new(remove_reply_fallback: RemoveReplyFallback) -> Self {
        let (allowed_tags, remove_content_tags) =
            if remove_reply_fallback == RemoveReplyFallback::Yes {
                let allowed_tags = BTreeSet::from(ALLOWED_TAGS_STRICT_WITHOUT_REPLY);
                let remove_content_tags = BTreeSet::from([RICH_REPLY_TAG]);
                (allowed_tags, remove_content_tags)
            } else {
                let allowed_tags = BTreeSet::from_iter(
                    ALLOWED_TAGS_STRICT_WITHOUT_REPLY.into_iter().chain([RICH_REPLY_TAG]),
                );
                let remove_content_tags = BTreeSet::new();
                (allowed_tags, remove_content_tags)
            };

        let allowed_attributes = BTreeMap::from_iter(
            ALLOWED_ATTRIBUTES_STRICT
                .into_iter()
                .map(|(s, attrs)| (s, BTreeSet::from_iter(attrs.iter().copied()))),
        );
        let allowed_schemes = BTreeMap::from_iter(
            ALLOWED_SCHEMES_STRICT
                .into_iter()
                .map(|(s, attrs)| (s, BTreeSet::from_iter(attrs.iter().copied()))),
        );
        let allowed_classes = BTreeMap::from_iter(
            ALLOWED_CLASSES_STRICT
                .into_iter()
                .map(|(s, attrs)| (s, BTreeSet::from_iter(attrs.iter().copied()))),
        );
        Self {
            allowed_tags: Some(allowed_tags),
            remove_content_tags,
            allowed_attributes: Some(allowed_attributes),
            allowed_schemes: Some(allowed_schemes),
            allowed_classes: Some(allowed_classes),
            max_depth: Some(MAX_DEPTH_STRICT),
        }
    }

    /// Constructs a `HTMLSanitizer` configured with the compat lists.
    ///
    /// Defaults to using the strict lists for the fields without compat lists.
    ///
    /// It can also optionally remove the [rich reply fallback].
    ///
    /// [rich reply fallback]: https://spec.matrix.org/v1.2/client-server-api/#fallbacks-for-rich-replies
    pub fn compat(remove_reply_fallback: RemoveReplyFallback) -> Self {
        let mut sanitizer = Self::new(remove_reply_fallback);
        if let Some(allowed_schemes) = sanitizer.allowed_schemes.as_mut() {
            for (key, schemes) in ALLOWED_SCHEMES_COMPAT {
                allowed_schemes.entry(key).or_default().extend(schemes.iter().copied());
            }
        }
        sanitizer
    }

    /// Constructs a `HTMLSanitizer` that does nothing.
    pub fn empty() -> Self {
        Self {
            allowed_tags: None,
            remove_content_tags: BTreeSet::new(),
            allowed_attributes: None,
            allowed_schemes: None,
            allowed_classes: None,
            max_depth: None,
        }
    }

    /// Constructs a `HTMLSanitizer` instance that only removes the [rich reply fallback].
    ///
    /// [rich reply fallback]: https://spec.matrix.org/v1.2/client-server-api/#fallbacks-for-rich-replies
    pub fn reply_fallback_remover() -> Self {
        let mut sanitizer = Self::empty();
        sanitizer.remove_content_tags = BTreeSet::from([RICH_REPLY_TAG]);
        sanitizer
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
                } else {
                    self.clean_attributes(&mut attributes.borrow_mut(), tag);
                }
            }
            NodeData::Text(_) => {}
            _ => node.detach(),
        }
    }

    fn element_action(&self, tag: &str, attributes: &Attributes, depth: u32) -> ElementAction {
        if self.remove_content_tags.contains(tag)
            || self.max_depth.filter(|d| *d <= depth).is_some()
        {
            ElementAction::Remove
        } else if self.allowed_tags.as_ref().filter(|tags| !tags.contains(tag)).is_some() {
            ElementAction::Ignore
        } else if let Some(allowed_schemes) = &self.allowed_schemes {
            for (name, val) in &attributes.map {
                let attr: &str = &name.local;

                // Check if there is a (tag, attr) tuple entry.
                if let Some(schemes) = allowed_schemes.get(&(tag, attr)) {
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

                if let Some(allowed_attributes) = &self.allowed_attributes {
                    if allowed_attributes.get(tag).filter(|attrs| attrs.contains(attr)).is_none() {
                        return Some(attr.to_owned());
                    }
                }

                if let Some(allowed_classes) =
                    self.allowed_classes.as_ref().filter(|_| attr == "class")
                {
                    if let Some(classes) = allowed_classes.get(tag) {
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

#[cfg(test)]
mod tests {
    use super::{HtmlSanitizer, RemoveReplyFallback};

    #[test]
    fn valid_input() {
        let sanitizer = HtmlSanitizer::new(RemoveReplyFallback::Yes);
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
        let sanitizer = HtmlSanitizer::new(RemoveReplyFallback::No);
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
        let sanitizer = HtmlSanitizer::new(RemoveReplyFallback::Yes);
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
        let sanitizer = HtmlSanitizer::new(RemoveReplyFallback::No);
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
        let sanitizer = HtmlSanitizer::new(RemoveReplyFallback::No);
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
        let sanitizer = HtmlSanitizer::new(RemoveReplyFallback::No);
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
        let sanitizer = HtmlSanitizer::new(RemoveReplyFallback::No);
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

        let sanitizer = HtmlSanitizer::compat(RemoveReplyFallback::No);
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
        let sanitizer = HtmlSanitizer::new(RemoveReplyFallback::No);
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
        let mut sanitizer = HtmlSanitizer::new(RemoveReplyFallback::No);
        sanitizer.max_depth = Some(2);
        let sanitized = sanitizer.clean(
            "\
            <div>\
                <p>\
                    <span>I am in too deep!</span>\
                    I should be fine.\
                </p>\
            </div>\
            ",
        );

        assert_eq!(
            sanitized,
            "\
            <div>\
                <p>\
                    I should be fine.\
                </p>\
            </div>\
            "
        );
    }
}
