use html5ever::{tendril::StrTendril, Attribute, LocalName};
use phf::{phf_map, phf_set, Map, Set};
use wildmatch::WildMatch;

use crate::html::{ElementData, Html, NodeData};

/// Configuration to sanitize HTML tags and attributes.
#[derive(Debug, Default, Clone)]
pub struct SanitizerConfig {
    /// The allowed HTML tags.
    ///
    /// If this is `None`, all tags are allowed.
    allowed_tags: Option<&'static Set<&'static str>>,

    /// The allowed deprecated HTML tags.
    ///
    /// This is a map of allowed deprecated tag to their replacement tag.
    deprecated_tags: Option<&'static Map<&'static str, &'static str>>,

    /// The allowed attributes per tag.
    ///
    /// If this is `None`, all attributes are allowed.
    allowed_attrs: Option<&'static Map<&'static str, &'static Set<&'static str>>>,

    /// The allowed deprecated attributes per tag.
    ///
    /// This is a map of tag to a map of allowed deprecated attribute to their replacement
    /// attribute.
    deprecated_attrs: Option<&'static Map<&'static str, &'static Map<&'static str, &'static str>>>,

    /// The allowed URI schemes per tag.
    ///
    /// If this is `None`, all schemes are allowed.
    allowed_schemes: Option<&'static Map<&'static str, &'static Set<&'static str>>>,

    /// The allowed classes per tag.
    ///
    /// If this is `None`, all classes are allowed.
    allowed_classes: Option<&'static Map<&'static str, &'static Set<&'static str>>>,

    /// The maximum nesting level of the tags.
    max_depth: Option<u32>,

    /// Whether to remove rich reply fallback.
    remove_reply_fallback: bool,
}

impl SanitizerConfig {
    /// Constructs an empty `SanitizerConfig` that will not filter any tag or attribute.
    pub fn new() -> Self {
        Self::default()
    }

    /// Constructs a `SanitizerConfig` that will filter tags or attributes not [listed in the
    /// Matrix specification].
    ///
    /// Deprecated tags will be replaced with their non-deprecated equivalent.
    ///
    /// It will not remove the reply fallback by default.
    ///
    /// [listed in the Matrix specification]: https://spec.matrix.org/latest/client-server-api/#mroommessage-msgtypes
    pub fn strict() -> Self {
        Self {
            allowed_tags: Some(&ALLOWED_TAGS_WITHOUT_REPLY_STRICT),
            deprecated_tags: Some(&DEPRECATED_TAGS),
            allowed_attrs: Some(&ALLOWED_ATTRIBUTES_STRICT),
            deprecated_attrs: Some(&DEPRECATED_ATTRS),
            allowed_schemes: Some(&ALLOWED_SCHEMES_STRICT),
            allowed_classes: Some(&ALLOWED_CLASSES_STRICT),
            max_depth: Some(MAX_DEPTH_STRICT),
            remove_reply_fallback: false,
        }
    }

    /// Constructs a `SanitizerConfig` that will filter tags or attributes not [listed in the
    /// Matrix specification], except a few for improved compatibility:
    ///
    /// - The `matrix` scheme is allowed in links.
    ///
    /// Deprecated tags will be replaced with their non-deprecated equivalent.
    ///
    /// It will not remove the reply fallback by default.
    ///
    /// [listed in the Matrix specification]: https://spec.matrix.org/latest/client-server-api/#mroommessage-msgtypes
    pub fn compat() -> Self {
        Self { allowed_schemes: Some(&ALLOWED_SCHEMES_COMPAT), ..Self::strict() }
    }

    /// Remove the [rich reply fallback].
    ///
    /// [rich reply fallback]: https://spec.matrix.org/latest/client-server-api/#fallbacks-for-rich-replies
    pub fn remove_reply_fallback(mut self) -> Self {
        self.remove_reply_fallback = true;
        self
    }

    /// Clean the given HTML with this sanitizer.
    pub(crate) fn clean(self, html: &mut Html) {
        let root = html.nodes[0].first_child.unwrap();
        let mut next_child = html.nodes[root].first_child;

        while let Some(child) = next_child {
            next_child = html.nodes[child].next_sibling;
            self.clean_node(html, child, 0);
        }
    }

    fn clean_node(&self, html: &mut Html, node_id: usize, depth: u32) {
        self.apply_deprecations(html, node_id);

        let action = self.node_action(html, node_id, depth);

        if action != NodeAction::Remove {
            let mut next_child = html.nodes[node_id].first_child;
            while let Some(child) = next_child {
                next_child = html.nodes[child].next_sibling;

                if action == NodeAction::Ignore {
                    html.insert_before(node_id, child);
                }

                self.clean_node(html, child, depth + 1);
            }
        }

        if matches!(action, NodeAction::Ignore | NodeAction::Remove) {
            html.detach(node_id);
        } else if let Some(data) = html.nodes[node_id].as_element_mut() {
            self.clean_element_attributes(data);
        }
    }

    fn apply_deprecations(&self, html: &mut Html, node_id: usize) {
        if let NodeData::Element(ElementData { name, attrs, .. }) = &mut html.nodes[node_id].data {
            let tag: &str = &name.local;

            if let Some(deprecated_attrs) =
                self.deprecated_attrs.and_then(|deprecated_attrs| deprecated_attrs.get(tag))
            {
                *attrs = attrs
                    .clone()
                    .into_iter()
                    .map(|mut attr| {
                        let attr_name: &str = &attr.name.local;

                        let attr_replacement =
                            deprecated_attrs.get(attr_name).map(|s| LocalName::from(*s));

                        if let Some(attr_replacement) = attr_replacement {
                            attr.name.local = attr_replacement;
                        }

                        attr
                    })
                    .collect();
            }

            let tag_replacement = self
                .deprecated_tags
                .and_then(|deprecated_tags| deprecated_tags.get(tag))
                .map(|s| LocalName::from(*s));

            if let Some(tag_replacement) = tag_replacement {
                name.local = tag_replacement;
            }
        }
    }

    fn node_action(&self, html: &Html, node_id: usize, depth: u32) -> NodeAction {
        match &html.nodes[node_id].data {
            NodeData::Element(ElementData { name, attrs, .. }) => {
                let tag: &str = &name.local;

                if (self.remove_reply_fallback && tag == RICH_REPLY_TAG)
                    || self.max_depth.is_some_and(|max| depth >= max)
                {
                    NodeAction::Remove
                } else if self
                    .allowed_tags
                    .is_some_and(|allowed| tag != RICH_REPLY_TAG && !allowed.contains(tag))
                {
                    NodeAction::Ignore
                } else if let Some(allowed_schemes) = self.allowed_schemes {
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

                if self
                    .allowed_attrs
                    .is_some_and(|m| !m.get(tag).is_some_and(|attrs| attrs.contains(name)))
                {
                    return Some(AttributeAction::Remove(attr.to_owned()));
                }

                if name == "class" {
                    if let Some(classes) = self.allowed_classes.and_then(|m| m.get(tag)) {
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

/// The possible actions to apply to an attribute.
#[derive(Debug)]
enum AttributeAction {
    /// Replace the value of the attribute.
    ReplaceValue(Attribute, StrTendril),

    /// Remove the attribute.
    Remove(Attribute),
}

/// List of HTML tags allowed in the Matrix specification, without the rich reply fallback tag.
static ALLOWED_TAGS_WITHOUT_REPLY_STRICT: Set<&str> = phf_set! {
    "del", "h1", "h2", "h3", "h4", "h5", "h6", "blockquote", "p", "a",
    "ul", "ol", "sup", "sub", "li", "b", "i", "u", "strong", "em", "s",
    "code", "hr", "br", "div", "table", "thead", "tbody", "tr", "th", "td",
    "caption", "pre", "span", "img", "details", "summary",
};

/// The HTML tag name for a rich reply fallback.
const RICH_REPLY_TAG: &str = "mx-reply";

/// HTML tags that were allowed in the Matrix specification, with their replacement.
static DEPRECATED_TAGS: Map<&str, &str> = phf_map! {
    "font" => "span",
    "strike" => "s",
};

/// Allowed attributes per HTML tag according to the Matrix specification.
static ALLOWED_ATTRIBUTES_STRICT: Map<&str, &Set<&str>> = phf_map! {
    "span" => &ALLOWED_ATTRIBUTES_SPAN_STRICT,
    "a" => &ALLOWED_ATTRIBUTES_A_STRICT,
    "img" => &ALLOWED_ATTRIBUTES_IMG_STRICT,
    "ol" => &ALLOWED_ATTRIBUTES_OL_STRICT,
    "code" => &ALLOWED_ATTRIBUTES_CODE_STRICT,
};
static ALLOWED_ATTRIBUTES_SPAN_STRICT: Set<&str> =
    phf_set! { "data-mx-bg-color", "data-mx-color", "data-mx-spoiler" };
static ALLOWED_ATTRIBUTES_A_STRICT: Set<&str> = phf_set! { "name", "target", "href" };
static ALLOWED_ATTRIBUTES_IMG_STRICT: Set<&str> =
    phf_set! { "width", "height", "alt", "title", "src" };
static ALLOWED_ATTRIBUTES_OL_STRICT: Set<&str> = phf_set! { "start" };
static ALLOWED_ATTRIBUTES_CODE_STRICT: Set<&str> = phf_set! { "class" };

/// Attributes that were allowed on HTML tags according to the Matrix specification, with their
/// replacement.
static DEPRECATED_ATTRS: Map<&str, &Map<&str, &str>> = phf_map! {
    "font" => &DEPRECATED_ATTRIBUTES_FONT,
};
static DEPRECATED_ATTRIBUTES_FONT: Map<&str, &str> = phf_map! { "color" => "data-mx-color" };

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
