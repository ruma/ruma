#![allow(clippy::disallowed_types)]

use std::collections::{HashMap, HashSet};

pub(crate) mod clean;

use crate::HtmlSanitizerMode;

/// Configuration to sanitize HTML elements and attributes.
#[derive(Debug, Default, Clone)]
pub struct SanitizerConfig {
    /// The mode of the sanitizer, if any.
    mode: Option<HtmlSanitizerMode>,

    /// Change to the list of elements to replace.
    ///
    /// The content is a map of element name to their replacement's element name.
    replace_elements: Option<List<HashMap<&'static str, &'static str>>>,

    /// Elements to remove.
    remove_elements: Option<HashSet<&'static str>>,

    /// Whether to remove the rich reply fallback.
    remove_reply_fallback: bool,

    /// Elements to ignore.
    ignore_elements: Option<HashSet<&'static str>>,

    /// Change to the list of elements to allow.
    allow_elements: Option<List<HashSet<&'static str>>>,

    /// Change to the list of attributes to replace per element.
    ///
    /// The content is a map of element name to a map of attribute name to their replacement's
    /// attribute name.
    replace_attrs: Option<List<HashMap<&'static str, HashMap<&'static str, &'static str>>>>,

    /// Removed attributes per element.
    remove_attrs: Option<HashMap<&'static str, HashSet<&'static str>>>,

    /// Change to the list of allowed attributes per element.
    allow_attrs: Option<List<HashMap<&'static str, HashSet<&'static str>>>>,

    /// Denied URI schemes per attribute per element.
    ///
    /// The content is a map of element name to a map of attribute name to a set of schemes.
    deny_schemes: Option<HashMap<&'static str, HashMap<&'static str, HashSet<&'static str>>>>,

    /// Change to the list of allowed URI schemes per attribute per element.
    ///
    /// The content is a map of element name to a map of attribute name to a set of schemes.
    #[allow(clippy::type_complexity)]
    allow_schemes:
        Option<List<HashMap<&'static str, HashMap<&'static str, HashSet<&'static str>>>>>,

    /// Removed classes per element.
    ///
    /// The content is a map of element name to a set of classes.
    remove_classes: Option<HashMap<&'static str, HashSet<&'static str>>>,

    /// Change to the list of allowed classes per element.
    ///
    /// The content is a map of element name to a set of classes.
    allow_classes: Option<List<HashMap<&'static str, HashSet<&'static str>>>>,

    /// Maximum nesting level of the elements.
    max_depth: Option<u32>,
}

impl SanitizerConfig {
    /// Constructs an empty `SanitizerConfig` that will not filter any element or attribute.
    ///
    /// The list of allowed and replaced elements can be changed with [`Self::allow_elements()`],
    /// [`Self::replace_elements()`], [`Self::ignore_elements()`], [`Self::remove_elements()`],
    /// [`Self::remove_reply_fallback()`].
    ///
    /// The list of allowed and replaced attributes can be changed with
    /// [`Self::allow_attributes()`], [`Self::replace_attributes()`],
    /// [`Self::remove_attributes()`], [`Self::allow_schemes()`], [`Self::deny_schemes()`],
    /// [`Self::allow_classes()`], [`Self::remove_classes()`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Constructs a `SanitizerConfig` with the given mode for filtering elements and attributes.
    ///
    /// The mode defines the basic list of allowed and replaced elements and attributes and the
    /// maximum nesting level of elements.
    ///
    /// The list of allowed and replaced elements can be changed with [`Self::allow_elements()`],
    /// [`Self::replace_elements()`], [`Self::ignore_elements()`], [`Self::remove_elements()`],
    /// [`Self::remove_reply_fallback()`].
    ///
    /// The list of allowed and replaced attributes can be changed with
    /// [`Self::allow_attributes()`], [`Self::replace_attributes()`],
    /// [`Self::remove_attributes()`], [`Self::allow_schemes()`], [`Self::deny_schemes()`],
    /// [`Self::allow_classes()`], [`Self::remove_classes()`].
    pub fn with_mode(mode: HtmlSanitizerMode) -> Self {
        Self { mode: Some(mode), ..Default::default() }
    }

    /// Constructs a `SanitizerConfig` that will filter elements and attributes not [suggested in
    /// the Matrix specification].
    ///
    /// The list of allowed and replaced elements can be changed with [`Self::allow_elements()`],
    /// [`Self::replace_elements()`], [`Self::ignore_elements()`], [`Self::remove_elements()`],
    /// [`Self::remove_reply_fallback()`].
    ///
    /// The list of allowed and replaced attributes can be changed with
    /// [`Self::allow_attributes()`], [`Self::replace_attributes()`],
    /// [`Self::remove_attributes()`], [`Self::allow_schemes()`], [`Self::deny_schemes()`],
    /// [`Self::allow_classes()`], [`Self::remove_classes()`].
    ///
    /// This is the same as calling `SanitizerConfig::with_mode(HtmlSanitizerMode::Strict)`.
    ///
    /// [suggested in the Matrix specification]: https://spec.matrix.org/latest/client-server-api/#mroommessage-msgtypes
    pub fn strict() -> Self {
        Self::with_mode(HtmlSanitizerMode::Strict)
    }

    /// Constructs a `SanitizerConfig` that will filter elements and attributes not [suggested in
    /// the Matrix specification], except a few for improved compatibility:
    ///
    /// * The `matrix` scheme is allowed in links.
    ///
    /// The list of allowed elements can be changed with [`Self::allow_elements()`],
    /// [`Self::replace_elements()`], [`Self::ignore_elements()`], [`Self::remove_elements()`],
    /// [`Self::remove_reply_fallback()`].
    ///
    /// The list of allowed attributes can be changed with [`Self::allow_attributes()`],
    /// [`Self::replace_attributes()`], [`Self::remove_attributes()`], [`Self::allow_schemes()`],
    /// [`Self::deny_schemes()`], [`Self::allow_classes()`], [`Self::remove_classes()`].
    ///
    /// This is the same as calling `SanitizerConfig::with_mode(HtmlSanitizerMode::Compat)`.
    ///
    /// [listed in the Matrix specification]: https://spec.matrix.org/latest/client-server-api/#mroommessage-msgtypes
    pub fn compat() -> Self {
        Self::with_mode(HtmlSanitizerMode::Compat)
    }

    /// Change the list of replaced HTML elements.
    ///
    /// The given list is added to or replaces the list of replacements of the current mode,
    /// depending on the [`ListBehavior`].
    ///
    /// The replacement occurs before the removal, so the replaced element should not be in
    /// the allowed list of elements, but the replacement element should.
    ///
    /// # Parameters
    ///
    /// * `elements`: The list of element names replacements.
    pub fn replace_elements(
        mut self,
        elements: impl IntoIterator<Item = NameReplacement>,
        behavior: ListBehavior,
    ) -> Self {
        let content = elements.into_iter().map(|r| r.to_tuple()).collect();
        self.replace_elements = Some(List { content, behavior });
        self
    }

    /// Remove the given HTML elements.
    ///
    /// When an element is removed, the element and its children are dropped. If you want to remove
    /// an element but keep its children, use [`SanitizerConfig::ignore_elements`] or
    /// [`SanitizerConfig::allow_elements`].
    ///
    /// Removing elements has a higher priority than ignoring or allowing. So if an element is in
    /// this list, it will always be removed.
    ///
    /// # Parameters
    ///
    /// * `elements`: The list of element names to remove.
    pub fn remove_elements(mut self, elements: impl IntoIterator<Item = &'static str>) -> Self {
        self.remove_elements = Some(elements.into_iter().collect());
        self
    }

    /// Remove the [rich reply fallback].
    ///
    /// Calling this allows to remove the `mx-reply` element in addition to the list of elements to
    /// remove.
    ///
    /// Removing elements has a higher priority than ignoring or allowing. So if this settings is
    /// set, `mx-reply` will always be removed.
    ///
    /// [rich reply fallback]: https://spec.matrix.org/latest/client-server-api/#fallbacks-for-rich-replies
    pub fn remove_reply_fallback(mut self) -> Self {
        self.remove_reply_fallback = true;
        self
    }

    /// Ignore the given HTML elements.
    ///
    /// When an element is ignored, the element is dropped and replaced by its children. If you want
    /// to drop an element and its children, use [`SanitizerConfig::remove_elements`].
    ///
    /// Removing elements has a lower priority than removing but a higher priority than allowing.
    ///
    /// # Parameters
    ///
    /// * `elements`: The list of element names to ignore.
    pub fn ignore_elements(mut self, elements: impl IntoIterator<Item = &'static str>) -> Self {
        self.ignore_elements = Some(elements.into_iter().collect());
        self
    }

    /// Change the list of allowed HTML elements.
    ///
    /// The given list is added to or replaces the list of allowed elements of the current
    /// mode, depending on the [`ListBehavior`].
    ///
    /// If an element is not allowed, it is ignored. If no mode is set and no elements are
    /// explicitly allowed, all elements are allowed.
    ///
    /// # Parameters
    ///
    /// * `elements`: The list of element names.
    pub fn allow_elements(
        mut self,
        elements: impl IntoIterator<Item = &'static str>,
        behavior: ListBehavior,
    ) -> Self {
        let content = elements.into_iter().collect();
        self.allow_elements = Some(List { content, behavior });
        self
    }

    /// Change the list of replaced attributes per HTML element.
    ///
    /// The given list is added to or replaces the list of replacements of the current mode,
    /// depending on the [`ListBehavior`].
    ///
    /// The replacement occurs before the removal, so the replaced attribute should not be in the
    /// list of allowed attributes, but the replacement attribute should. Attribute replacement
    /// occurs before element replacement, so if you want to replace an attribute on an element
    /// that is set to be replaced, you must use the replaced element's name, not the name of its
    /// replacement.
    ///
    /// # Parameters
    ///
    /// * `attrs`: The list of element's attributes replacements.
    pub fn replace_attributes<'a>(
        mut self,
        attrs: impl IntoIterator<Item = ElementAttributesReplacement<'a>>,
        behavior: ListBehavior,
    ) -> Self {
        let content = attrs.into_iter().map(|r| r.to_tuple()).collect();
        self.replace_attrs = Some(List { content, behavior });
        self
    }

    /// Remove the given attributes per HTML element.
    ///
    /// Removing attributes has a higher priority than allowing. So if an attribute is in
    /// this list, it will always be removed.
    ///
    /// # Parameters
    ///
    /// * `attrs`: The list of attributes per element. The value of `parent` is the element name,
    ///   and `properties` contains attribute names.
    pub fn remove_attributes<'a>(
        mut self,
        attrs: impl IntoIterator<Item = PropertiesNames<'a>>,
    ) -> Self {
        self.remove_attrs = Some(attrs.into_iter().map(|a| a.to_tuple()).collect());
        self
    }

    /// Change the list of allowed attributes per HTML element.
    ///
    /// The given list is added to or replaces the list of allowed attributes of the current
    /// mode, depending on the [`ListBehavior`].
    ///
    /// If an attribute is not allowed, it is removed. If no mode is set and no attributes are
    /// explicitly allowed, all attributes are allowed.
    ///
    /// # Parameters
    ///
    /// * `attrs`: The list of attributes per element. The value of `parent` is the element name,
    ///   and `properties` contains attribute names.
    pub fn allow_attributes<'a>(
        mut self,
        attrs: impl IntoIterator<Item = PropertiesNames<'a>>,
        behavior: ListBehavior,
    ) -> Self {
        let content = attrs.into_iter().map(|a| a.to_tuple()).collect();
        self.allow_attrs = Some(List { content, behavior });
        self
    }

    /// Deny the given URI schemes per attribute per HTML element.
    ///
    /// Denying schemes has a higher priority than allowing. So if a scheme is in
    /// this list, it will always be denied.
    ///
    /// If a scheme is denied, its element is removed, because it is deemed that the element will
    /// not be usable without it URI.
    ///
    /// # Parameters
    ///
    /// * `schemes`: The list of schemes per attribute per element.
    pub fn deny_schemes<'a>(
        mut self,
        schemes: impl IntoIterator<Item = ElementAttributesSchemes<'a>>,
    ) -> Self {
        self.deny_schemes = Some(schemes.into_iter().map(|s| s.to_tuple()).collect());
        self
    }

    /// Change the list of allowed schemes per attribute per HTML element.
    ///
    /// The given list is added to or replaces the list of allowed schemes of the current
    /// mode, depending on the [`ListBehavior`].
    ///
    /// If a scheme is not allowed, it is denied. If a scheme is denied, its element is ignored,
    /// because it is deemed that the element will not be usable without it URI. If no mode is set
    /// and no schemes are explicitly allowed, all schemes are allowed.
    ///
    /// # Parameters
    ///
    /// * `schemes`: The list of schemes per attribute per element.
    pub fn allow_schemes<'a>(
        mut self,
        schemes: impl IntoIterator<Item = ElementAttributesSchemes<'a>>,
        behavior: ListBehavior,
    ) -> Self {
        let content = schemes.into_iter().map(|s| s.to_tuple()).collect();
        self.allow_schemes = Some(List { content, behavior });
        self
    }

    /// Deny the given classes per HTML element.
    ///
    /// Removing classes has a higher priority than allowing. So if a class is in
    /// this list, it will always be removed.
    ///
    /// If all the classes of a `class` attribute are removed, the whole attribute is removed.
    ///
    /// In the list of classes, the names must match the full class name. `*` can be used as a
    /// wildcard for any number of characters. So `language` will only match a class named
    /// `language`, and `language-*` will match any class name starting with `language-`.
    ///
    /// # Parameters
    ///
    /// * `attrs`: The list of classes per element. The value of `parent` is the element name, and
    ///   `properties` contains classes.
    pub fn remove_classes<'a>(
        mut self,
        classes: impl IntoIterator<Item = PropertiesNames<'a>>,
    ) -> Self {
        self.remove_classes = Some(classes.into_iter().map(|c| c.to_tuple()).collect());
        self
    }

    /// Change the list of allowed classes per HTML element.
    ///
    /// The given list is added, removed or replaces the list of allowed classes of the current
    /// mode, depending on the [`ListBehavior`].
    ///
    /// If a class is not allowed, it is removed. If all the classes of a `class` attribute are
    /// removed, the whole attribute is removed. If no mode is set and no classes are explicitly
    /// allowed, all classes are allowed.
    ///
    /// In the list of classes, the names must match the full class name. `*` can be used as a
    /// wildcard for any number of characters. So `language` will only match a class named
    /// `language`, and `language-*` will match any class name starting with `language-`.
    ///
    /// # Parameters
    ///
    /// * `attrs`: The list of classes per element. The value of `parent` is the element name, and
    ///   `properties` contains classes.
    pub fn allow_classes<'a>(
        mut self,
        classes: impl IntoIterator<Item = PropertiesNames<'a>>,
        behavior: ListBehavior,
    ) -> Self {
        let content = classes.into_iter().map(|c| c.to_tuple()).collect();
        self.allow_classes = Some(List { content, behavior });
        self
    }

    /// The maximum nesting level of HTML elements.
    ///
    /// This overrides the maximum depth set by the mode, if one is set.
    ///
    /// All elements that are deeper than the maximum depth will be removed. If no mode is set and
    /// no maximum depth is explicitly set, elements are not filtered by their nesting level.
    ///
    /// # Parameters
    ///
    /// * `depth`: The maximum nesting level allowed.
    pub fn max_depth(mut self, depth: u32) -> Self {
        self.max_depth = Some(depth);
        self
    }
}

/// A list with a behavior.
#[derive(Debug, Clone)]
struct List<T> {
    /// The content of this list.
    content: T,

    /// The behavior of this list.
    behavior: ListBehavior,
}

impl<T> List<T> {
    /// Whether this is `ListBehavior::Override`.
    fn is_override(&self) -> bool {
        self.behavior == ListBehavior::Override
    }
}

/// The behavior of the setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::exhaustive_enums)]
pub enum ListBehavior {
    /// The list replaces the default list of the current mode, if one is set.
    ///
    /// If no mode is set, this is the full allow list.
    Override,

    /// The list is added to the default list of the current mode, if one is set.
    ///
    /// If no mode is set, this is the full allow list.
    Add,
}

/// The replacement of a name.
#[derive(Debug, Clone, Copy)]
#[allow(clippy::exhaustive_structs)]
pub struct NameReplacement {
    /// The name to replace.
    pub old: &'static str,
    /// The name of the replacement.
    pub new: &'static str,
}

impl NameReplacement {
    fn to_tuple(self) -> (&'static str, &'static str) {
        (self.old, self.new)
    }
}

/// A list of properties names for a parent.
#[allow(clippy::exhaustive_structs)]
#[derive(Debug, Clone, Copy)]
pub struct PropertiesNames<'a> {
    /// The name of the parent.
    pub parent: &'static str,
    /// The list of properties names.
    pub properties: &'a [&'static str],
}

impl<'a> PropertiesNames<'a> {
    fn to_tuple(self) -> (&'static str, HashSet<&'static str>) {
        let set = self.properties.iter().copied().collect();
        (self.parent, set)
    }
}

/// The replacement of an element's attributes.
#[allow(clippy::exhaustive_structs)]
#[derive(Debug, Clone, Copy)]
pub struct ElementAttributesReplacement<'a> {
    /// The name of the element.
    pub element: &'static str,
    /// The list of attributes replacements.
    pub replacements: &'a [NameReplacement],
}

impl<'a> ElementAttributesReplacement<'a> {
    fn to_tuple(self) -> (&'static str, HashMap<&'static str, &'static str>) {
        let map = self.replacements.iter().map(|r| r.to_tuple()).collect();
        (self.element, map)
    }
}

/// An element's attributes' URI schemes.
#[allow(clippy::exhaustive_structs)]
#[derive(Debug, Clone, Copy)]
pub struct ElementAttributesSchemes<'a> {
    /// The name of the element.
    pub element: &'static str,
    /// The list of allowed URI schemes per attribute name.
    ///
    /// The value of the `parent` is the attribute name and the properties are schemes.
    pub attr_schemes: &'a [PropertiesNames<'a>],
}

impl<'a> ElementAttributesSchemes<'a> {
    fn to_tuple(self) -> (&'static str, HashMap<&'static str, HashSet<&'static str>>) {
        let map = self.attr_schemes.iter().map(|s| s.to_tuple()).collect();
        (self.element, map)
    }
}
