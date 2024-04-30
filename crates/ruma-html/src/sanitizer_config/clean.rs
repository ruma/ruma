use html5ever::{tendril::StrTendril, Attribute, LocalName};
use phf::{phf_map, phf_set, Map, Set};
use wildmatch::WildMatch;

use crate::{ElementData, Html, HtmlSanitizerMode, NodeData, SanitizerConfig};

/// HTML elements allowed in the Matrix specification.
static ALLOWED_ELEMENTS_STRICT: Set<&str> = phf_set! {
    "del", "h1", "h2", "h3", "h4", "h5", "h6", "blockquote", "p", "a",
    "ul", "ol", "sup", "sub", "li", "b", "i", "u", "strong", "em", "s",
    "code", "hr", "br", "div", "table", "thead", "tbody", "tr", "th", "td",
    "caption", "pre", "span", "img", "details", "summary", "mx-reply",
};

/// The HTML element name for a rich reply fallback.
const RICH_REPLY_ELEMENT_NAME: &str = "mx-reply";

/// HTML elements that were previously allowed in the Matrix specification, with their replacement.
static DEPRECATED_ELEMENTS: Map<&str, &str> = phf_map! {
    "font" => "span",
    "strike" => "s",
};

/// Allowed attributes per HTML element according to the Matrix specification.
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

/// Attributes that were previously allowed on HTML elements according to the Matrix specification,
/// with their replacement.
static DEPRECATED_ATTRS: Map<&str, &Map<&str, &str>> = phf_map! {
    "font" => &DEPRECATED_ATTRIBUTES_FONT,
};
static DEPRECATED_ATTRIBUTES_FONT: Map<&str, &str> = phf_map! { "color" => "data-mx-color" };

/// Allowed schemes of URIs per attribute per HTML element according to the Matrix specification.
static ALLOWED_SCHEMES_STRICT: Map<&str, &Map<&str, &Set<&str>>> = phf_map! {
    "a" => &ALLOWED_SCHEMES_A_STRICT,
    "img" => &ALLOWED_SCHEMES_IMG_STRICT,
};
static ALLOWED_SCHEMES_A_STRICT: Map<&str, &Set<&str>> = phf_map! {
    "href" => &ALLOWED_SCHEMES_A_HREF_STRICT,
};
pub(crate) static ALLOWED_SCHEMES_A_HREF_STRICT: Set<&str> =
    phf_set! { "http", "https", "ftp", "mailto", "magnet" };
static ALLOWED_SCHEMES_IMG_STRICT: Map<&str, &Set<&str>> = phf_map! {
    "src" => &ALLOWED_SCHEMES_IMG_SRC_STRICT,
};
static ALLOWED_SCHEMES_IMG_SRC_STRICT: Set<&str> = phf_set! { "mxc" };

/// Extra allowed schemes of URIs per attribute per HTML element.
///
/// This is a convenience list to add schemes that can be encountered but are not listed in the
/// Matrix specification. It consists of:
///
/// * The `matrix` scheme for `a` elements (see [matrix-org/matrix-spec#1108]).
///
/// To get a complete list, add these to `ALLOWED_SCHEMES_STRICT`.
///
/// [matrix-org/matrix-spec#1108]: https://github.com/matrix-org/matrix-spec/issues/1108
static ALLOWED_SCHEMES_COMPAT: Map<&str, &Map<&str, &Set<&str>>> = phf_map! {
    "a" => &ALLOWED_SCHEMES_A_COMPAT,
};
static ALLOWED_SCHEMES_A_COMPAT: Map<&str, &Set<&str>> = phf_map! {
    "href" => &ALLOWED_SCHEMES_A_HREF_COMPAT,
};
pub(crate) static ALLOWED_SCHEMES_A_HREF_COMPAT: Set<&str> = phf_set! { "matrix" };

/// Allowed classes per HTML element according to the Matrix specification.
static ALLOWED_CLASSES_STRICT: Map<&str, &Set<&str>> =
    phf_map! { "code" => &ALLOWED_CLASSES_CODE_STRICT };
static ALLOWED_CLASSES_CODE_STRICT: Set<&str> = phf_set! { "language-*" };

/// Max depth of nested HTML elements allowed by the Matrix specification.
const MAX_DEPTH_STRICT: u32 = 100;

impl SanitizerConfig {
    /// Whether the current mode uses the values of the strict mode.
    fn use_strict(&self) -> bool {
        self.mode.is_some()
    }

    /// Whether the current mode uses the values of the compat mode.
    fn use_compat(&self) -> bool {
        self.mode.is_some_and(|m| m == HtmlSanitizerMode::Compat)
    }

    /// The maximum nesting level allowed by the config.
    fn max_depth_value(&self) -> Option<u32> {
        self.max_depth.or_else(|| self.use_strict().then_some(MAX_DEPTH_STRICT))
    }

    /// Clean the given HTML with this sanitizer.
    pub(crate) fn clean(&self, html: &mut Html) {
        let root = html.root();
        let mut next_child = root.first_child;

        while let Some(child) = next_child {
            next_child = html.nodes[child].next_sibling;
            self.clean_node(html, child, 0);
        }
    }

    fn clean_node(&self, html: &mut Html, node_id: usize, depth: u32) {
        self.apply_replacements(html, node_id);

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

    fn apply_replacements(&self, html: &mut Html, node_id: usize) {
        if let NodeData::Element(ElementData { name, attrs, .. }) = &mut html.nodes[node_id].data {
            let element_name = name.local.as_ref();

            // Replace attributes.
            let list_replacements =
                self.replace_attrs.as_ref().and_then(|list| list.content.get(element_name));
            let list_is_override =
                self.replace_attrs.as_ref().map(|list| list.is_override()).unwrap_or_default();
            let mode_replacements = (!list_is_override && self.use_strict())
                .then(|| DEPRECATED_ATTRS.get(element_name))
                .flatten();

            if list_replacements.is_some() || mode_replacements.is_some() {
                *attrs = attrs
                    .clone()
                    .into_iter()
                    .map(|mut attr| {
                        let attr_name = attr.name.local.as_ref();

                        let attr_replacement = list_replacements
                            .and_then(|s| s.get(attr_name))
                            .or_else(|| mode_replacements.and_then(|s| s.get(attr_name)))
                            .copied();

                        if let Some(attr_replacement) = attr_replacement {
                            attr.name.local = LocalName::from(attr_replacement);
                        }

                        attr
                    })
                    .collect();
            }

            // Replace element.
            let mut element_replacement = self
                .replace_elements
                .as_ref()
                .and_then(|list| list.content.get(element_name))
                .copied();

            if element_replacement.is_none() {
                let list_is_override = self
                    .replace_elements
                    .as_ref()
                    .map(|list| list.is_override())
                    .unwrap_or_default();
                element_replacement = (!list_is_override && self.use_strict())
                    .then(|| DEPRECATED_ELEMENTS.get(element_name))
                    .flatten()
                    .copied();
            }

            if let Some(element_replacement) = element_replacement {
                name.local = LocalName::from(element_replacement);
            }
        }
    }

    fn node_action(&self, html: &Html, node_id: usize, depth: u32) -> NodeAction {
        match &html.nodes[node_id].data {
            NodeData::Element(ElementData { name, attrs, .. }) => {
                let element_name = name.local.as_ref();

                // Check if element should be removed.
                if self.remove_elements.as_ref().is_some_and(|set| set.contains(element_name)) {
                    return NodeAction::Remove;
                }
                if self.remove_reply_fallback && element_name == RICH_REPLY_ELEMENT_NAME {
                    return NodeAction::Remove;
                }
                if self.max_depth_value().is_some_and(|max| depth >= max) {
                    return NodeAction::Remove;
                }

                // Check if element should be ignored.
                if self.ignore_elements.as_ref().is_some_and(|set| set.contains(element_name)) {
                    return NodeAction::Ignore;
                }

                // Check if element should be allowed.
                if self.allow_elements.is_some() || self.use_strict() {
                    let list_allowed = self
                        .allow_elements
                        .as_ref()
                        .is_some_and(|list| list.content.contains(element_name));
                    let list_is_override = self
                        .allow_elements
                        .as_ref()
                        .map(|list| list.is_override())
                        .unwrap_or_default();
                    let mode_allowed = !list_is_override
                        && self.use_strict()
                        && ALLOWED_ELEMENTS_STRICT.contains(element_name);

                    if !list_allowed && !mode_allowed {
                        return NodeAction::Ignore;
                    }
                }

                // Check if element contains scheme that should be denied.
                if let Some(deny_schemes) =
                    self.deny_schemes.as_ref().and_then(|map| map.get(element_name))
                {
                    for attr in attrs.iter() {
                        let value = &attr.value;
                        let attr_name = attr.name.local.as_ref();

                        if let Some(schemes) = deny_schemes.get(attr_name) {
                            // Check if the scheme is denied.
                            if schemes.iter().any(|scheme| value.starts_with(&format!("{scheme}:")))
                            {
                                return NodeAction::Ignore;
                            }
                        }
                    }
                }

                if self.allow_schemes.is_none() && !self.use_strict() {
                    // All schemes are allowed.
                    return NodeAction::None;
                }

                // Check if element contains scheme that should be allowed.
                let list_element_schemes =
                    self.allow_schemes.as_ref().and_then(|list| list.content.get(element_name));
                let list_is_override =
                    self.allow_schemes.as_ref().map(|list| list.is_override()).unwrap_or_default();
                let strict_mode_element_schemes = (!list_is_override && self.use_strict())
                    .then(|| ALLOWED_SCHEMES_STRICT.get(element_name))
                    .flatten();
                let compat_mode_element_schemes = (!list_is_override && self.use_compat())
                    .then(|| ALLOWED_SCHEMES_COMPAT.get(element_name))
                    .flatten();

                if list_element_schemes.is_none()
                    && strict_mode_element_schemes.is_none()
                    && compat_mode_element_schemes.is_none()
                {
                    // We don't check schemes for this element.
                    return NodeAction::None;
                }

                for attr in attrs.iter() {
                    let value = &attr.value;
                    let attr_name = attr.name.local.as_ref();

                    let list_attr_schemes = list_element_schemes.and_then(|map| map.get(attr_name));
                    let strict_mode_attr_schemes =
                        strict_mode_element_schemes.and_then(|map| map.get(attr_name));
                    let compat_mode_attr_schemes =
                        compat_mode_element_schemes.and_then(|map| map.get(attr_name));

                    if list_attr_schemes.is_none()
                        && strict_mode_attr_schemes.is_none()
                        && compat_mode_attr_schemes.is_none()
                    {
                        // We don't check schemes for this attribute.
                        return NodeAction::None;
                    }

                    let mut allowed_schemes = list_attr_schemes
                        .into_iter()
                        .flatten()
                        .chain(strict_mode_attr_schemes.map(|set| set.iter()).into_iter().flatten())
                        .chain(
                            compat_mode_attr_schemes.map(|set| set.iter()).into_iter().flatten(),
                        );

                    // Check if the scheme is allowed.
                    if !allowed_schemes.any(|scheme| value.starts_with(&format!("{scheme}:"))) {
                        return NodeAction::Ignore;
                    }
                }

                NodeAction::None
            }
            NodeData::Text(_) => NodeAction::None,
            _ => NodeAction::Remove,
        }
    }

    fn clean_element_attributes(&self, data: &mut ElementData) {
        let ElementData { name, attrs } = data;
        let element_name = name.local.as_ref();

        let list_remove_attrs = self.remove_attrs.as_ref().and_then(|map| map.get(element_name));

        let whitelist_attrs = self.allow_attrs.is_some() || self.use_strict();
        let list_allow_attrs =
            self.allow_attrs.as_ref().and_then(|list| list.content.get(element_name));
        let list_is_override =
            self.allow_attrs.as_ref().map(|list| list.is_override()).unwrap_or_default();
        let mode_allow_attrs = (!list_is_override && self.use_strict())
            .then(|| ALLOWED_ATTRIBUTES_STRICT.get(element_name))
            .flatten();

        let list_remove_classes =
            self.remove_classes.as_ref().and_then(|map| map.get(element_name));

        let whitelist_classes = self.allow_classes.is_some() || self.use_strict();
        let list_allow_classes =
            self.allow_classes.as_ref().and_then(|list| list.content.get(element_name));
        let list_is_override =
            self.allow_classes.as_ref().map(|list| list.is_override()).unwrap_or_default();
        let mode_allow_classes = (!list_is_override && self.use_strict())
            .then(|| ALLOWED_CLASSES_STRICT.get(element_name))
            .flatten();

        let actions: Vec<_> = attrs
            .iter()
            .filter_map(|attr| {
                let value = &attr.value;
                let attr_name = attr.name.local.as_ref();

                // Check if the attribute should be removed.
                if list_remove_attrs.is_some_and(|set| set.contains(attr_name)) {
                    return Some(AttributeAction::Remove(attr.to_owned()));
                }

                // Check if the attribute is allowed.
                if whitelist_attrs {
                    let list_allowed = list_allow_attrs.is_some_and(|set| set.contains(attr_name));
                    let mode_allowed = mode_allow_attrs.is_some_and(|set| set.contains(attr_name));

                    if !list_allowed && !mode_allowed {
                        return Some(AttributeAction::Remove(attr.to_owned()));
                    }
                }

                // Filter classes.
                if attr_name == "class" {
                    let mut classes = value.split_whitespace().collect::<Vec<_>>();
                    let initial_len = classes.len();

                    // Process classes to remove.
                    if let Some(remove_classes) = list_remove_classes {
                        classes.retain(|class| {
                            for remove_class in remove_classes {
                                if WildMatch::new(remove_class).matches(class) {
                                    return false;
                                }
                            }

                            true
                        });
                    }

                    // Process classes to allow.
                    if whitelist_classes {
                        classes.retain(|class| {
                            let allow_classes = list_allow_classes
                                .map(|set| set.iter())
                                .into_iter()
                                .flatten()
                                .chain(
                                    mode_allow_classes.map(|set| set.iter()).into_iter().flatten(),
                                );

                            for allow_class in allow_classes {
                                if WildMatch::new(allow_class).matches(class) {
                                    return true;
                                }
                            }

                            false
                        });
                    }

                    if classes.len() == initial_len {
                        // The list has not changed, no action necessary.
                        return None;
                    }

                    if classes.is_empty() {
                        return Some(AttributeAction::Remove(attr.to_owned()));
                    } else {
                        let new_class = classes.join(" ");
                        return Some(AttributeAction::ReplaceValue(
                            attr.to_owned(),
                            new_class.into(),
                        ));
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
