use html5ever::{tendril::StrTendril, Attribute, LocalName};
use wildmatch::WildMatch;

use crate::{ElementData, Html, HtmlSanitizerMode, NodeData, NodeRef, SanitizerConfig};

impl SanitizerConfig {
    /// Whether the current mode uses the rules from the Matrix specification.
    fn use_spec(&self) -> bool {
        self.mode.is_some()
    }

    /// Whether the current mode uses the values of the compat mode.
    fn use_compat(&self) -> bool {
        self.mode.is_some_and(|m| m == HtmlSanitizerMode::Compat)
    }

    /// The maximum nesting level allowed by the config.
    fn max_depth_value(&self) -> Option<u32> {
        self.max_depth.or_else(|| self.use_spec().then_some(spec::MAX_DEPTH))
    }

    /// Clean the given HTML with this sanitizer.
    pub(crate) fn clean(&self, html: &Html) {
        for child in html.children() {
            self.clean_node(child, 0);
        }
    }

    fn clean_node(&self, node: NodeRef, depth: u32) {
        let node = self.apply_replacements(node);

        let action = self.node_action(&node, depth);

        if action != NodeAction::Remove {
            for child in node.children() {
                if action == NodeAction::Ignore {
                    child.insert_before_sibling(&node);
                }

                self.clean_node(child, depth + 1);
            }
        }

        if matches!(action, NodeAction::Ignore | NodeAction::Remove) {
            node.detach();
        } else if let Some(data) = node.as_element() {
            self.clean_element_attributes(data);
        }
    }

    /// Apply the attributes and element name replacements to the given node.
    ///
    /// This might return a different node than the one provided.
    fn apply_replacements(&self, node: NodeRef) -> NodeRef {
        let mut element_replacement = None;

        if let NodeData::Element(ElementData { name, attrs, .. }) = node.data() {
            let element_name = name.local.as_ref();

            // Replace attributes.
            let list_replacements =
                self.replace_attrs.as_ref().and_then(|list| list.content.get(element_name));
            let list_is_override =
                self.replace_attrs.as_ref().map(|list| list.is_override()).unwrap_or_default();

            let use_spec = !list_is_override && self.use_spec();
            let has_spec_replacements =
                use_spec && spec::has_element_deprecated_attributes(element_name);

            if list_replacements.is_some() || has_spec_replacements {
                let mut attrs = attrs.borrow_mut();
                *attrs = attrs
                    .clone()
                    .into_iter()
                    .map(|mut attr| {
                        let attr_name = attr.name.local.as_ref();

                        let attr_replacement = list_replacements
                            .and_then(|s| s.get(attr_name))
                            .copied()
                            .or_else(|| {
                                use_spec
                                    .then(|| {
                                        spec::deprecated_attribute_replacement(
                                            element_name,
                                            attr_name,
                                        )
                                    })
                                    .flatten()
                            });

                        if let Some(attr_replacement) = attr_replacement {
                            attr.name.local = LocalName::from(attr_replacement);
                        }

                        attr
                    })
                    .collect();
            }

            // Replace element.
            element_replacement = self
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
                element_replacement = (!list_is_override && self.use_spec())
                    .then(|| spec::deprecated_element_replacement(element_name))
                    .flatten();
            }
        }

        if let Some(element_replacement) = element_replacement {
            node.replace_with_element_name(LocalName::from(element_replacement))
        } else {
            node
        }
    }

    fn node_action(&self, node: &NodeRef, depth: u32) -> NodeAction {
        match node.data() {
            NodeData::Element(ElementData { name, attrs, .. }) => {
                let element_name = name.local.as_ref();
                let attrs = attrs.borrow();

                // Check if element should be removed.
                if self.remove_elements.as_ref().is_some_and(|set| set.contains(element_name)) {
                    return NodeAction::Remove;
                }
                if self.remove_reply_fallback && element_name == spec::RICH_REPLY_ELEMENT_NAME {
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
                if self.allow_elements.is_some() || self.use_spec() {
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
                        && self.use_spec()
                        && spec::is_element_allowed(element_name);

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

                if self.allow_schemes.is_none() && !self.use_spec() {
                    // All schemes are allowed.
                    return NodeAction::None;
                }

                // Check if element contains scheme that should be allowed.
                let list_element_schemes =
                    self.allow_schemes.as_ref().and_then(|list| list.content.get(element_name));
                let list_is_override =
                    self.allow_schemes.as_ref().map(|list| list.is_override()).unwrap_or_default();
                let has_spec_scheme_rules = !list_is_override
                    && self.use_spec()
                    && has_element_allowed_schemes(element_name);

                if list_element_schemes.is_none() && !has_spec_scheme_rules {
                    // We don't check schemes for this element.
                    return NodeAction::None;
                }

                for attr in attrs.iter() {
                    let value = &attr.value;
                    let attr_name = attr.name.local.as_ref();

                    let list_schemes = list_element_schemes.and_then(|map| map.get(attr_name));
                    let spec_schemes = (!list_is_override && self.use_spec())
                        .then(|| spec::allowed_schemes(element_name, attr_name))
                        .flatten();
                    let compat_schemes = (!list_is_override && self.use_compat())
                        .then(|| compat::allowed_schemes(element_name, attr_name))
                        .flatten();

                    if list_schemes.is_none() && spec_schemes.is_none() && compat_schemes.is_none()
                    {
                        // We don't check schemes for this attribute.
                        return NodeAction::None;
                    }

                    let mut allowed_schemes = list_schemes
                        .into_iter()
                        .flatten()
                        .chain(spec_schemes.into_iter().flatten())
                        .chain(compat_schemes.into_iter().flatten());

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

    fn clean_element_attributes(&self, data: &ElementData) {
        let ElementData { name, attrs } = data;
        let element_name = name.local.as_ref();
        let use_spec = self.use_spec();
        let mut attrs = attrs.borrow_mut();

        let list_remove_attrs = self.remove_attrs.as_ref().and_then(|map| map.get(element_name));

        let whitelist_attrs = self.allow_attrs.is_some() || use_spec;
        let list_allow_attrs =
            self.allow_attrs.as_ref().and_then(|list| list.content.get(element_name));
        let list_is_override =
            self.allow_attrs.as_ref().map(|list| list.is_override()).unwrap_or_default();
        let use_spec_attr_rules = !list_is_override && use_spec;

        let list_remove_classes =
            self.remove_classes.as_ref().and_then(|map| map.get(element_name));

        let whitelist_classes = self.allow_classes.is_some() || use_spec;
        let list_allow_classes =
            self.allow_classes.as_ref().and_then(|list| list.content.get(element_name));
        let list_is_override =
            self.allow_classes.as_ref().map(|list| list.is_override()).unwrap_or_default();
        let mode_allow_classes =
            (!list_is_override && use_spec).then(|| spec::allowed_classes(element_name)).flatten();

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
                    let mode_allowed =
                        use_spec_attr_rules && spec::is_attribute_allowed(element_name, attr_name);

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
                                .chain(mode_allow_classes.into_iter().flatten());

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

/// Whether the given HTML element has a list of allowed schemes according to the Matrix
/// specification or the compatibility list.
pub(super) fn has_element_allowed_schemes(element_name: &str) -> bool {
    // Keep in sync with `spec::allowed_schemes` and `compat::allowed_schemes`.
    matches!(element_name, "a" | "img")
}

/// Rules defined in the Matrix specification.
pub(crate) mod spec {
    /// Max depth of nested HTML elements allowed by the Matrix specification.
    pub(super) const MAX_DEPTH: u32 = 100;

    /// The HTML element name for a rich reply fallback.
    pub(super) const RICH_REPLY_ELEMENT_NAME: &str = "mx-reply";

    /// Whether the given HTML element is allowed in the Matrix specification.
    pub(super) fn is_element_allowed(element_name: &str) -> bool {
        matches!(
            element_name,
            "del"
                | "h1"
                | "h2"
                | "h3"
                | "h4"
                | "h5"
                | "h6"
                | "blockquote"
                | "p"
                | "a"
                | "ul"
                | "ol"
                | "sup"
                | "sub"
                | "li"
                | "b"
                | "i"
                | "u"
                | "strong"
                | "em"
                | "s"
                | "code"
                | "hr"
                | "br"
                | "div"
                | "table"
                | "thead"
                | "tbody"
                | "tr"
                | "th"
                | "td"
                | "caption"
                | "pre"
                | "span"
                | "img"
                | "details"
                | "summary"
                | "mx-reply"
        )
    }

    /// The replacement element for the given HTML element, if it was previously allowed in the
    /// Matrix specification.
    pub(super) fn deprecated_element_replacement(element_name: &str) -> Option<&'static str> {
        let replacement = match element_name {
            "font" => "span",
            "strike" => "s",
            _ => return None,
        };

        Some(replacement)
    }

    /// Whether the given attribute is in the list of allowed attributes for the given HTML element
    /// according to the Matrix specification.
    pub(super) fn is_attribute_allowed(element_name: &str, attribute_name: &str) -> bool {
        match element_name {
            "span" => match attribute_name {
                "data-mx-bg-color" | "data-mx-color" | "data-mx-spoiler" | "data-mx-maths" => true,
                #[cfg(feature = "unstable-msc4286")]
                "data-msc4286-external-payment-details" => true,
                _ => false,
            },
            "a" => matches!(attribute_name, "target" | "href",),
            "img" => matches!(attribute_name, "width" | "height" | "alt" | "title" | "src",),
            "ol" => matches!(attribute_name, "start",),
            "code" => matches!(attribute_name, "class",),
            "div" => matches!(attribute_name, "data-mx-maths",),
            _ => false,
        }
    }

    /// Whether the given HTML element has deprecated attributes in the Matrix specification.
    pub(super) fn has_element_deprecated_attributes(element_name: &str) -> bool {
        // Must be kept in sync with `deprecated_attribute_replacement`.
        matches!(element_name, "font")
    }

    /// The replacement attribute for the given attribute of the given HTML element, if it was
    /// previously allowed in the Matrix specification.
    pub(super) fn deprecated_attribute_replacement(
        element_name: &str,
        attribute_name: &str,
    ) -> Option<&'static str> {
        // Must be kept in sync with `has_element_deprecated_attributes`.
        match element_name {
            "font" => match attribute_name {
                "color" => Some("data-mx-color"),
                _ => None,
            },
            _ => None,
        }
    }

    /// The list of allowed URI schemes for the given attribute in the given HTML element in the
    /// Matrix specification.
    ///
    /// If `compat` is `true`, a few extra schemes are allowed:
    ///
    /// * The `matrix` scheme for `a` elements (see [matrix-org/matrix-spec#1108]).
    ///
    /// [matrix-org/matrix-spec#1108]: https://github.com/matrix-org/matrix-spec/issues/1108
    pub(crate) fn allowed_schemes(
        element_name: &str,
        attribute_name: &str,
    ) -> Option<&'static [&'static str]> {
        // Keep in sync with `has_element_allowed_schemes`.
        let schemes: &'static [&'static str] = match (element_name, attribute_name) {
            ("a", "href") => &["http", "https", "ftp", "mailto", "magnet"],
            ("img", "src") => &["mxc"],
            _ => return None,
        };

        Some(schemes)
    }

    /// Get the allowed CSS classes for the given HTML element in the Matrix specification.
    ///
    /// The returned classes use `*` as a wildcard for any number of any characters.
    pub(super) fn allowed_classes(element_name: &str) -> Option<&'static [&'static str]> {
        match element_name {
            "code" => Some(&["language-*"]),
            _ => None,
        }
    }
}

/// Extra rules for improved compatibility.
pub(crate) mod compat {
    /// Additional allowed URI schemes for improved compatibility.
    ///
    /// This adds schemes that can be encountered but are not listed in the Matrix specification. It
    /// consists of:
    ///
    /// * The `matrix` scheme for `a` elements (see [matrix-org/matrix-spec#1108]).
    ///
    /// [matrix-org/matrix-spec#1108]: https://github.com/matrix-org/matrix-spec/issues/1108
    pub(crate) fn allowed_schemes(
        element_name: &str,
        attribute_name: &str,
    ) -> Option<&'static [&'static str]> {
        // Keep in sync with `has_element_allowed_schemes`.
        let schemes: &'static [&'static str] = match (element_name, attribute_name) {
            ("a", "href") => &["matrix"],
            _ => return None,
        };

        Some(schemes)
    }
}
