//! Types to work with HTML elements and attributes [suggested by the Matrix Specification][spec].
//!
//! [spec]: https://spec.matrix.org/latest/client-server-api/#mroommessage-msgtypes

use std::collections::BTreeSet;

use html5ever::{namespace_url, ns, tendril::StrTendril, Attribute, QualName};
use ruma_common::{
    IdParseError, MatrixToError, MatrixToUri, MatrixUri, MatrixUriError, MxcUri, OwnedMxcUri,
};

use crate::sanitizer_config::clean::{
    ALLOWED_SCHEMES_A_HREF_COMPAT, ALLOWED_SCHEMES_A_HREF_STRICT,
};

const CLASS_LANGUAGE_PREFIX: &str = "language-";

/// The data of a Matrix HTML element.
///
/// This is a helper type to work with elements [suggested by the Matrix Specification][spec].
///
/// This performs a lossless conversion from [`ElementData`]. Unsupported elements are represented
/// by [`MatrixElement::Other`] and unsupported attributes are listed in the `attrs` field.
///
/// [`ElementData`]: crate::ElementData
/// [spec]: https://spec.matrix.org/latest/client-server-api/#mroommessage-msgtypes
#[derive(Debug, Clone)]
#[allow(clippy::exhaustive_structs)]
pub struct MatrixElementData {
    /// The HTML element and its supported data.
    pub element: MatrixElement,

    /// The unsupported attributes found on the element.
    pub attrs: BTreeSet<Attribute>,
}

impl MatrixElementData {
    /// Parse a `MatrixElementData` from the given qualified name and attributes.
    #[allow(clippy::mutable_key_type)]
    pub(super) fn parse(name: &QualName, attrs: &BTreeSet<Attribute>) -> Self {
        let (element, attrs) = MatrixElement::parse(name, attrs);
        Self { element, attrs }
    }
}

/// A Matrix HTML element.
///
/// All the elements [suggested by the Matrix Specification][spec] have a variant. The others are
/// handled by the fallback `Other` variant.
///
/// Suggested attributes are represented as optional fields on the variants structs.
///
/// [spec]: https://spec.matrix.org/latest/client-server-api/#mroommessage-msgtypes
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum MatrixElement {
    /// [`<del>`], a deleted text element.
    ///
    /// [`<del>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/del
    Del,

    /// [`<h1>-<h6>`], a section heading element.
    ///
    /// [`<h1>-<h6>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/Heading_Elements
    H(HeadingData),

    /// [`<blockquote>`], a block quotation element.
    ///
    /// [`<blockquote>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/blockquote
    Blockquote,

    /// [`<p>`], a paragraph element.
    ///
    /// [`<p>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/p
    P,

    /// [`<a>`], an anchor element.
    ///
    /// [`<a>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/a
    A(AnchorData),

    /// [`<ul>`], an unordered list element.
    ///
    /// [`<ul>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ul
    Ul,

    /// [`<ol>`], an ordered list element.
    ///
    /// [`<ol>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ol
    Ol(OrderedListData),

    /// [`<sup>`], a superscript element.
    ///
    /// [`<sup>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/sup
    Sup,

    /// [`<sub>`], a subscript element.
    ///
    /// [`<sub>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/sub
    Sub,

    /// [`<li>`], a list item element.
    ///
    /// [`<li>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/li
    Li,

    /// [`<b>`], a bring attention to element.
    ///
    /// [`<b>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/b
    B,

    /// [`<i>`], an idiomatic text element.
    ///
    /// [`<i>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/i
    I,

    /// [`<u>`], an unarticulated annotation element.
    ///
    /// [`<u>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/u
    U,

    /// [`<strong>`], a strong importance element.
    ///
    /// [`<strong>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/strong
    Strong,

    /// [`<em>`], an emphasis element.
    ///
    /// [`<em>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/em
    Em,

    /// [`<s>`], a strikethrough element.
    ///
    /// [`<s>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/s
    S,

    /// [`<code>`], an inline code element.
    ///
    /// [`<code>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/code
    Code(CodeData),

    /// [`<hr>`], a thematic break element.
    ///
    /// [`<hr>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/hr
    Hr,

    /// [`<br>`], a line break element.
    ///
    /// [`<br>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/br
    Br,

    /// [`<div>`], a content division element.
    ///
    /// [`<div>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/div
    Div,

    /// [`<table>`], a table element.
    ///
    /// [`<table>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/table
    Table,

    /// [`<thead>`], a table head element.
    ///
    /// [`<thead>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/thead
    Thead,

    /// [`<tbody>`], a table body element.
    ///
    /// [`<tbody>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tbody
    Tbody,

    /// [`<tr>`], a table row element.
    ///
    /// [`<tr>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tr
    Tr,

    /// [`<th>`], a table header element.
    ///
    /// [`<th>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/th
    Th,

    /// [`<td>`], a table data cell element.
    ///
    /// [`<td>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/td
    Td,

    /// [`<caption>`], a table caption element.
    ///
    /// [`<caption>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/caption
    Caption,

    /// [`<pre>`], a preformatted text element.
    ///
    /// [`<pre>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/pre
    Pre,

    /// [`<span>`], a content span element.
    ///
    /// [`<span>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/span
    Span(SpanData),

    /// [`<img>`], an image embed element.
    ///
    /// [`<img>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/img
    Img(ImageData),

    /// [`<details>`], a details disclosure element.
    ///
    /// [`<details>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/details
    Details,

    /// [`<summary>`], a disclosure summary element.
    ///
    /// [`<summary>`]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/summary
    Summary,

    /// [`mx-reply`], a Matrix rich reply fallback element.
    ///
    /// [`mx-reply`]: https://spec.matrix.org/latest/client-server-api/#fallbacks-for-rich-replies
    MatrixReply,

    /// An HTML element that is not in the suggested list.
    Other(QualName),
}

impl MatrixElement {
    /// Parse a `MatrixElement` from the given qualified name and attributes.
    ///
    /// Returns a tuple containing the constructed `Element` and the list of remaining unsupported
    /// attributes.
    #[allow(clippy::mutable_key_type)]
    fn parse(name: &QualName, attrs: &BTreeSet<Attribute>) -> (Self, BTreeSet<Attribute>) {
        if name.ns != ns!(html) {
            return (Self::Other(name.clone()), attrs.clone());
        }

        match name.local.as_bytes() {
            b"del" => (Self::Del, attrs.clone()),
            b"h1" => (Self::H(HeadingData::new(1)), attrs.clone()),
            b"h2" => (Self::H(HeadingData::new(2)), attrs.clone()),
            b"h3" => (Self::H(HeadingData::new(3)), attrs.clone()),
            b"h4" => (Self::H(HeadingData::new(4)), attrs.clone()),
            b"h5" => (Self::H(HeadingData::new(5)), attrs.clone()),
            b"h6" => (Self::H(HeadingData::new(6)), attrs.clone()),
            b"blockquote" => (Self::Blockquote, attrs.clone()),
            b"p" => (Self::P, attrs.clone()),
            b"a" => {
                let (data, attrs) = AnchorData::parse(attrs);
                (Self::A(data), attrs)
            }
            b"ul" => (Self::Ul, attrs.clone()),
            b"ol" => {
                let (data, attrs) = OrderedListData::parse(attrs);
                (Self::Ol(data), attrs)
            }
            b"sup" => (Self::Sup, attrs.clone()),
            b"sub" => (Self::Sub, attrs.clone()),
            b"li" => (Self::Li, attrs.clone()),
            b"b" => (Self::B, attrs.clone()),
            b"i" => (Self::I, attrs.clone()),
            b"u" => (Self::U, attrs.clone()),
            b"strong" => (Self::Strong, attrs.clone()),
            b"em" => (Self::Em, attrs.clone()),
            b"s" => (Self::S, attrs.clone()),
            b"code" => {
                let (data, attrs) = CodeData::parse(attrs);
                (Self::Code(data), attrs)
            }
            b"hr" => (Self::Hr, attrs.clone()),
            b"br" => (Self::Br, attrs.clone()),
            b"div" => (Self::Div, attrs.clone()),
            b"table" => (Self::Table, attrs.clone()),
            b"thead" => (Self::Thead, attrs.clone()),
            b"tbody" => (Self::Tbody, attrs.clone()),
            b"tr" => (Self::Tr, attrs.clone()),
            b"th" => (Self::Th, attrs.clone()),
            b"td" => (Self::Td, attrs.clone()),
            b"caption" => (Self::Caption, attrs.clone()),
            b"pre" => (Self::Pre, attrs.clone()),
            b"span" => {
                let (data, attrs) = SpanData::parse(attrs);
                (Self::Span(data), attrs)
            }
            b"img" => {
                let (data, attrs) = ImageData::parse(attrs);
                (Self::Img(data), attrs)
            }
            b"details" => (Self::Details, attrs.clone()),
            b"summary" => (Self::Summary, attrs.clone()),
            b"mx-reply" => (Self::MatrixReply, attrs.clone()),
            _ => (Self::Other(name.clone()), attrs.clone()),
        }
    }
}

/// The supported data of a `<h1>-<h6>` HTML element.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct HeadingData {
    /// The level of the heading.
    pub level: HeadingLevel,
}

impl HeadingData {
    /// Constructs a new `HeadingData` with the given heading level.
    fn new(level: u8) -> Self {
        Self { level: HeadingLevel(level) }
    }
}

/// The level of a heading element.
///
/// The supported levels range from 1 (highest) to 6 (lowest). Other levels cannot construct this
/// and do not use the [`MatrixElement::H`] variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeadingLevel(u8);

impl HeadingLevel {
    /// The value of the level.
    ///
    /// Can only be a value between 1 and 6 included.
    pub fn value(&self) -> u8 {
        self.0
    }
}

impl PartialEq<u8> for HeadingLevel {
    fn eq(&self, other: &u8) -> bool {
        self.0.eq(other)
    }
}

/// The supported data of a `<a>` HTML element.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct AnchorData {
    /// The name of the anchor.
    pub name: Option<StrTendril>,

    /// Where to display the linked URL.
    pub target: Option<StrTendril>,

    /// The URL that the hyperlink points to.
    pub href: Option<AnchorUri>,
}

impl AnchorData {
    /// Construct an empty `AnchorData`.
    fn new() -> Self {
        Self { name: None, target: None, href: None }
    }

    /// Parse the given attributes to construct a new `AnchorData`.
    ///
    /// Returns a tuple containing the constructed data and the remaining unsupported attributes.
    #[allow(clippy::mutable_key_type)]
    fn parse(attrs: &BTreeSet<Attribute>) -> (Self, BTreeSet<Attribute>) {
        let mut data = Self::new();
        let mut remaining_attrs = BTreeSet::new();

        for attr in attrs {
            if attr.name.ns != ns!() {
                remaining_attrs.insert(attr.clone());
                continue;
            }

            match attr.name.local.as_bytes() {
                b"name" => {
                    data.name = Some(attr.value.clone());
                }
                b"target" => {
                    data.target = Some(attr.value.clone());
                }
                b"href" => {
                    if let Some(uri) = AnchorUri::parse(&attr.value) {
                        data.href = Some(uri);
                    } else {
                        remaining_attrs.insert(attr.clone());
                    }
                }
                _ => {
                    remaining_attrs.insert(attr.clone());
                }
            }
        }

        (data, remaining_attrs)
    }
}

/// A URI as a value for the `href` attribute of a `<a>` HTML element.
///
/// This is a helper type that recognizes `matrix:` and `https://matrix.to` URIs to detect mentions.
///
/// If the URI is an invalid Matrix URI or does not use one of the suggested schemes, the `href`
/// attribute will be in the `attrs` list of [`MatrixElementData`].
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum AnchorUri {
    /// A `matrix:` URI.
    Matrix(MatrixUri),
    /// A `https://matrix.to` URI.
    MatrixTo(MatrixToUri),
    /// An other URL using one of the suggested schemes.
    ///
    /// Those schemes are:
    ///
    /// * `https`
    /// * `http`
    /// * `ftp`
    /// * `mailto`
    /// * `magnet`
    Other(StrTendril),
}

impl AnchorUri {
    /// Parse the given string to construct a new `AnchorUri`.
    fn parse(value: &StrTendril) -> Option<Self> {
        let s = value.as_ref();

        // Check if it starts with a supported scheme.
        let mut allowed_schemes =
            ALLOWED_SCHEMES_A_HREF_STRICT.iter().chain(ALLOWED_SCHEMES_A_HREF_COMPAT.iter());
        if !allowed_schemes.any(|scheme| s.starts_with(&format!("{scheme}:"))) {
            return None;
        }

        match MatrixUri::parse(s) {
            Ok(uri) => return Some(Self::Matrix(uri)),
            // It's not a `matrix:` URI, continue.
            Err(IdParseError::InvalidMatrixUri(MatrixUriError::WrongScheme)) => {}
            // The URI is invalid.
            _ => return None,
        }

        match MatrixToUri::parse(s) {
            Ok(uri) => return Some(Self::MatrixTo(uri)),
            // It's not a `https://matrix.to` URI, continue.
            Err(IdParseError::InvalidMatrixToUri(MatrixToError::WrongBaseUrl)) => {}
            // The URI is invalid.
            _ => return None,
        }

        Some(Self::Other(value.clone()))
    }
}

/// The supported data of a `<ol>` HTML element.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct OrderedListData {
    /// An integer to start counting from for the list items.
    ///
    /// If parsing the integer from a string fails, the attribute will be in the `attrs` list of
    /// [`MatrixElementData`].
    pub start: Option<i64>,
}

impl OrderedListData {
    /// Construct an empty `OrderedListData`.
    fn new() -> Self {
        Self { start: None }
    }

    /// Parse the given attributes to construct a new `OrderedListData`.
    ///
    /// Returns a tuple containing the constructed data and the remaining unsupported attributes.
    #[allow(clippy::mutable_key_type)]
    fn parse(attrs: &BTreeSet<Attribute>) -> (Self, BTreeSet<Attribute>) {
        let mut data = Self::new();
        let mut remaining_attrs = BTreeSet::new();

        for attr in attrs {
            if attr.name.ns != ns!() {
                remaining_attrs.insert(attr.clone());
                continue;
            }

            match attr.name.local.as_bytes() {
                b"start" => {
                    if let Ok(start) = attr.value.parse() {
                        data.start = Some(start);
                    } else {
                        remaining_attrs.insert(attr.clone());
                    }
                }
                _ => {
                    remaining_attrs.insert(attr.clone());
                }
            }
        }

        (data, remaining_attrs)
    }
}

/// The supported data of a `<code>` HTML element.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct CodeData {
    /// The language of the code, for syntax highlighting.
    ///
    /// This corresponds to the `class` attribute with a value that starts with the
    /// `language-` prefix. The prefix is stripped from the value.
    ///
    /// If there are other classes in the `class` attribute, the whole attribute will be in the
    /// `attrs` list of [`MatrixElementData`].
    pub language: Option<StrTendril>,
}

impl CodeData {
    /// Construct an empty `CodeData`.
    fn new() -> Self {
        Self { language: None }
    }

    /// Parse the given attributes to construct a new `CodeData`.
    ///
    /// Returns a tuple containing the constructed data and the remaining unsupported attributes.
    #[allow(clippy::mutable_key_type)]
    fn parse(attrs: &BTreeSet<Attribute>) -> (Self, BTreeSet<Attribute>) {
        let mut data = Self::new();
        let mut remaining_attrs = BTreeSet::new();

        for attr in attrs {
            if attr.name.ns != ns!() {
                remaining_attrs.insert(attr.clone());
                continue;
            }

            match attr.name.local.as_bytes() {
                b"class" => {
                    let value_str = attr.value.as_ref();

                    // The attribute could contain several classes separated by spaces, so let's
                    // find the first class starting with `language-`.
                    for (match_start, _) in value_str.match_indices(CLASS_LANGUAGE_PREFIX) {
                        // The class name must either be at the start of the string or preceded by a
                        // space.
                        if match_start != 0
                            && !value_str.as_bytes()[match_start - 1].is_ascii_whitespace()
                        {
                            continue;
                        }

                        let language_start = match_start + CLASS_LANGUAGE_PREFIX.len();

                        let str_end = &value_str[language_start..];
                        let language_end = str_end
                            .find(|c: char| c.is_ascii_whitespace())
                            .map(|pos| language_start + pos)
                            .unwrap_or(value_str.len());

                        if language_end == language_start {
                            continue;
                        }

                        let sub_len = (language_end - language_start) as u32;
                        data.language = Some(attr.value.subtendril(language_start as u32, sub_len));

                        if match_start != 0 || language_end != value_str.len() {
                            // There are other classes, keep the whole attribute for the conversion
                            // to be lossless.
                            remaining_attrs.insert(attr.clone());
                        }

                        break;
                    }

                    if data.language.is_none() {
                        // We didn't find the class we want, keep the whole attribute.
                        remaining_attrs.insert(attr.clone());
                    }
                }
                _ => {
                    remaining_attrs.insert(attr.clone());
                }
            }
        }

        (data, remaining_attrs)
    }
}

/// The supported data of a `<span>` HTML element.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SpanData {
    /// `data-mx-bg-color`, the background color of the text.
    pub bg_color: Option<StrTendril>,

    /// `data-mx-color`, the foreground color of the text.
    pub color: Option<StrTendril>,

    /// `data-mx-spoiler`, a Matrix [spoiler message].
    ///
    /// The value is the reason of the spoiler. If the string is empty, this is a spoiler
    /// without a reason.
    ///
    /// [spoiler message]: https://spec.matrix.org/latest/client-server-api/#spoiler-messages
    pub spoiler: Option<StrTendril>,
}

impl SpanData {
    /// Construct an empty `SpanData`.
    fn new() -> Self {
        Self { bg_color: None, color: None, spoiler: None }
    }

    /// Parse the given attributes to construct a new `SpanData`.
    ///
    /// Returns a tuple containing the constructed data and the remaining unsupported attributes.
    #[allow(clippy::mutable_key_type)]
    fn parse(attrs: &BTreeSet<Attribute>) -> (Self, BTreeSet<Attribute>) {
        let mut data = Self::new();
        let mut remaining_attrs = BTreeSet::new();

        for attr in attrs {
            if attr.name.ns != ns!() {
                remaining_attrs.insert(attr.clone());
                continue;
            }

            match attr.name.local.as_bytes() {
                b"data-mx-bg-color" => {
                    data.bg_color = Some(attr.value.clone());
                }
                b"data-mx-color" => data.color = Some(attr.value.clone()),
                b"data-mx-spoiler" => {
                    data.spoiler = Some(attr.value.clone());
                }
                _ => {
                    remaining_attrs.insert(attr.clone());
                }
            }
        }

        (data, remaining_attrs)
    }
}

/// The supported data of a `<img>` HTML element.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ImageData {
    /// The intrinsic width of the image, in pixels.
    ///
    /// If parsing the integer from a string fails, the attribute will be in the `attrs` list of
    /// `MatrixElementData`.
    pub width: Option<i64>,

    /// The intrinsic height of the image, in pixels.
    ///
    /// If parsing the integer from a string fails, the attribute will be in the `attrs` list of
    /// [`MatrixElementData`].
    pub height: Option<i64>,

    /// Text that can replace the image.
    pub alt: Option<StrTendril>,

    ///  Text representing advisory information about the image.
    pub title: Option<StrTendril>,

    /// The image URL.
    ///
    /// It this is not a valid `mxc:` URI, the attribute will be in the `attrs` list of
    /// [`MatrixElementData`].
    pub src: Option<OwnedMxcUri>,
}

impl ImageData {
    /// Construct an empty `ImageData`.
    fn new() -> Self {
        Self { width: None, height: None, alt: None, title: None, src: None }
    }

    /// Parse the given attributes to construct a new `ImageData`.
    ///
    /// Returns a tuple containing the constructed data and the remaining unsupported attributes.
    #[allow(clippy::mutable_key_type)]
    fn parse(attrs: &BTreeSet<Attribute>) -> (Self, BTreeSet<Attribute>) {
        let mut data = Self::new();
        let mut remaining_attrs = BTreeSet::new();

        for attr in attrs {
            if attr.name.ns != ns!() {
                remaining_attrs.insert(attr.clone());
                continue;
            }

            match attr.name.local.as_bytes() {
                b"width" => {
                    if let Ok(width) = attr.value.parse() {
                        data.width = Some(width);
                    } else {
                        remaining_attrs.insert(attr.clone());
                    }
                }
                b"height" => {
                    if let Ok(height) = attr.value.parse() {
                        data.height = Some(height);
                    } else {
                        remaining_attrs.insert(attr.clone());
                    }
                }
                b"alt" => data.alt = Some(attr.value.clone()),
                b"title" => data.title = Some(attr.value.clone()),
                b"src" => {
                    let uri = <&MxcUri>::from(attr.value.as_ref());
                    if uri.validate().is_ok() {
                        data.src = Some(uri.to_owned());
                    } else {
                        remaining_attrs.insert(attr.clone());
                    }
                }
                _ => {
                    remaining_attrs.insert(attr.clone());
                }
            }
        }

        (data, remaining_attrs)
    }
}
