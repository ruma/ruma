//! Parsing helpers for the `EventContent` derive macro.

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    Field, Ident, LitStr, Token, Type,
};

use crate::events::enums::{EventContentVariation, EventKind, EventVariation};

mod kw {
    // This `content` field is kept when the event is redacted.
    syn::custom_keyword!(skip_redaction);
    // Do not emit any redacted event code.
    syn::custom_keyword!(custom_redacted);
    // Do not emit any possibly redacted event code.
    syn::custom_keyword!(custom_possibly_redacted);
    // The kind of event content this is.
    syn::custom_keyword!(kind);
    syn::custom_keyword!(type_fragment);
    // The type to use for a state events' `state_key` field.
    syn::custom_keyword!(state_key_type);
    // The type to use for a state events' `unsigned` field.
    syn::custom_keyword!(unsigned_type);
    // Another type string accepted for deserialization.
    syn::custom_keyword!(alias);
    // The content has a form without relation.
    syn::custom_keyword!(without_relation);
}

/// Parses field attributes for `*EventContent` derives.
///
/// `#[ruma_event(skip_redaction)]`
pub enum EventFieldMeta {
    /// Fields marked with `#[ruma_event(skip_redaction)]` are kept when the event is
    /// redacted.
    SkipRedaction,

    /// The given field holds a part of the event type (replaces the `*` in a `m.foo.*` event
    /// type).
    TypeFragment,
}

impl Parse for EventFieldMeta {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::skip_redaction) {
            let _: kw::skip_redaction = input.parse()?;
            Ok(EventFieldMeta::SkipRedaction)
        } else if lookahead.peek(kw::type_fragment) {
            let _: kw::type_fragment = input.parse()?;
            Ok(EventFieldMeta::TypeFragment)
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Default)]
pub struct ContentMeta {
    event_type: Option<LitStr>,
    kind: Option<EventContentKind>,
    custom_redacted: Option<kw::custom_redacted>,
    custom_possibly_redacted: Option<kw::custom_possibly_redacted>,
    state_key_type: Option<Box<Type>>,
    unsigned_type: Option<Box<Type>>,
    aliases: Vec<LitStr>,
    without_relation: Option<kw::without_relation>,
}

impl ContentMeta {
    pub fn merge(self, other: ContentMeta) -> syn::Result<Self> {
        fn either_spanned<T: ToTokens>(a: Option<T>, b: Option<T>) -> syn::Result<Option<T>> {
            match (a, b) {
                (None, None) => Ok(None),
                (Some(val), None) | (None, Some(val)) => Ok(Some(val)),
                (Some(a), Some(b)) => {
                    let mut error = syn::Error::new_spanned(a, "redundant attribute argument");
                    error.combine(syn::Error::new_spanned(b, "note: first one here"));
                    Err(error)
                }
            }
        }

        fn either_named<T>(name: &str, a: Option<T>, b: Option<T>) -> syn::Result<Option<T>> {
            match (a, b) {
                (None, None) => Ok(None),
                (Some(val), None) | (None, Some(val)) => Ok(Some(val)),
                (Some(_), Some(_)) => Err(syn::Error::new(
                    Span::call_site(),
                    format!("multiple {name} attributes found, there can only be one"),
                )),
            }
        }

        Ok(Self {
            event_type: either_spanned(self.event_type, other.event_type)?,
            kind: either_named("kind", self.kind, other.kind)?,
            custom_redacted: either_spanned(self.custom_redacted, other.custom_redacted)?,
            custom_possibly_redacted: either_spanned(
                self.custom_possibly_redacted,
                other.custom_possibly_redacted,
            )?,
            state_key_type: either_spanned(self.state_key_type, other.state_key_type)?,
            unsigned_type: either_spanned(self.unsigned_type, other.unsigned_type)?,
            aliases: [self.aliases, other.aliases].concat(),
            without_relation: either_spanned(self.without_relation, other.without_relation)?,
        })
    }
}

impl Parse for ContentMeta {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![type]) {
            let _: Token![type] = input.parse()?;
            let _: Token![=] = input.parse()?;
            let event_type = input.parse()?;

            Ok(Self { event_type: Some(event_type), ..Default::default() })
        } else if lookahead.peek(kw::kind) {
            let kind = input.parse()?;

            Ok(Self { kind: Some(kind), ..Default::default() })
        } else if lookahead.peek(kw::custom_redacted) {
            let custom_redacted: kw::custom_redacted = input.parse()?;

            Ok(Self { custom_redacted: Some(custom_redacted), ..Default::default() })
        } else if lookahead.peek(kw::custom_possibly_redacted) {
            let custom_possibly_redacted: kw::custom_possibly_redacted = input.parse()?;

            Ok(Self {
                custom_possibly_redacted: Some(custom_possibly_redacted),
                ..Default::default()
            })
        } else if lookahead.peek(kw::state_key_type) {
            let _: kw::state_key_type = input.parse()?;
            let _: Token![=] = input.parse()?;
            let state_key_type = input.parse()?;

            Ok(Self { state_key_type: Some(state_key_type), ..Default::default() })
        } else if lookahead.peek(kw::unsigned_type) {
            let _: kw::unsigned_type = input.parse()?;
            let _: Token![=] = input.parse()?;
            let unsigned_type = input.parse()?;

            Ok(Self { unsigned_type: Some(unsigned_type), ..Default::default() })
        } else if lookahead.peek(kw::alias) {
            let _: kw::alias = input.parse()?;
            let _: Token![=] = input.parse()?;
            let alias = input.parse()?;

            Ok(Self { aliases: vec![alias], ..Default::default() })
        } else if lookahead.peek(kw::without_relation) {
            let without_relation: kw::without_relation = input.parse()?;

            Ok(Self { without_relation: Some(without_relation), ..Default::default() })
        } else {
            Err(lookahead.error())
        }
    }
}

pub struct ContentAttrs {
    pub event_type: LitStr,
    pub kind: EventContentKind,
    pub state_key_type: Option<TokenStream>,
    pub unsigned_type: Option<TokenStream>,
    pub aliases: Vec<LitStr>,
    pub is_custom_redacted: bool,
    pub is_custom_possibly_redacted: bool,
    pub has_without_relation: bool,
}

impl TryFrom<ContentMeta> for ContentAttrs {
    type Error = syn::Error;

    fn try_from(value: ContentMeta) -> Result<Self, Self::Error> {
        let ContentMeta {
            event_type,
            kind,
            custom_redacted,
            custom_possibly_redacted,
            state_key_type,
            unsigned_type,
            aliases,
            without_relation,
        } = value;

        let event_type = event_type.ok_or_else(|| {
            syn::Error::new(
                Span::call_site(),
                "no event type attribute found, \
                add `#[ruma_event(type = \"any.room.event\", kind = Kind)]` \
                below the event content derive",
            )
        })?;

        let kind = kind.ok_or_else(|| {
            syn::Error::new(
                Span::call_site(),
                "no event kind attribute found, \
                add `#[ruma_event(type = \"any.room.event\", kind = Kind)]` \
                below the event content derive",
            )
        })?;

        let state_key_type = match (kind.is_state(), state_key_type) {
            (true, None) => {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "no state_key_type attribute found, please specify one",
                ));
            }
            (true, Some(ty)) => Some(quote! { #ty }),
            (false, None) => None,
            (false, Some(ty)) => {
                return Err(syn::Error::new_spanned(
                    ty,
                    "state_key_type attribute is not valid for non-state event kinds",
                ));
            }
        };

        let is_custom_redacted = custom_redacted.is_some();
        let is_custom_possibly_redacted = custom_possibly_redacted.is_some();

        let unsigned_type = unsigned_type.map(|ty| quote! { #ty });

        let event_type_s = event_type.value();
        let prefix = event_type_s.strip_suffix(".*");

        if prefix.unwrap_or(&event_type_s).contains('*') {
            return Err(syn::Error::new_spanned(
                event_type,
                "event type may only contain `*` as part of a `.*` suffix",
            ));
        }

        if prefix.is_some() && !kind.is_account_data() {
            return Err(syn::Error::new_spanned(
                event_type,
                "only account data events may contain a `.*` suffix",
            ));
        }

        for alias in &aliases {
            if alias.value().ends_with(".*") != prefix.is_some() {
                return Err(syn::Error::new_spanned(
                    alias,
                    "aliases should have the same `.*` suffix, or lack thereof, as the main event type",
                ));
            }
        }

        let has_without_relation = without_relation.is_some();

        Ok(Self {
            event_type,
            kind,
            state_key_type,
            unsigned_type,
            aliases,
            is_custom_redacted,
            is_custom_possibly_redacted,
            has_without_relation,
        })
    }
}

/// Data about the type fragment of an event content with a type that ends with `.*`.
pub struct EventTypeFragment<'a> {
    pub prefix: String,
    pub field: &'a Ident,
}

impl<'a> EventTypeFragment<'a> {
    /// Try to construct an `EventTypeFragment` from the given data.
    ///
    /// Returns `Ok(None)` if the event type doesn't contain a `*` suffix, `Ok(Some(_))` if the
    /// event type contains a `*` suffix and the type fragment field was found, and `Err(_)` if
    /// the event type contains a `*` suffix and the type fragment field was NOT found.
    pub fn try_from_parts(
        event_type: &LitStr,
        mut fields: Option<impl Iterator<Item = &'a Field>>,
    ) -> syn::Result<Option<Self>> {
        let event_type_s = event_type.value();

        let Some(prefix) = event_type_s.strip_suffix('*') else {
            return Ok(None);
        };

        let Some(fields) = &mut fields else {
            return Err(syn::Error::new_spanned(
                event_type,
                "event type with a `.*` suffix is required to be a struct",
            ));
        };

        let field = fields
            .find_map(|f| {
                f.attrs.iter().filter(|a| a.path().is_ident("ruma_event")).find_map(|attr| {
                    match attr.parse_args() {
                        Ok(EventFieldMeta::TypeFragment) => Some(Ok(f)),
                        Ok(_) => None,
                        Err(e) => Some(Err(e)),
                    }
                })
            })
            .transpose()?
            .ok_or_else(|| {
                syn::Error::new_spanned(
                    event_type,
                    "event type with a `.*` suffix requires there to be a \
                     `#[ruma_event(type_fragment)]` field",
                )
            })?
            .ident
            .as_ref()
            .expect("type fragment field needs to have a name");

        Ok(Some(Self { prefix: prefix.to_owned(), field }))
    }
}

/// The possible values of the `kind` of event content.
///
/// This is a wrapper around `EventKind` that allows to provide two kinds for the same event
/// content.
#[derive(Clone, Copy)]
pub enum EventContentKind {
    /// The event content has a single kind.
    Single(EventKind),
    /// The event content is of the two account data kinds.
    DoubleAccountData,
}

impl EventContentKind {
    /// Whether this kind contains an `EventKind::State`.
    pub fn is_state(self) -> bool {
        matches!(self, Self::Single(EventKind::State))
    }

    /// Whether this kind contains only account data kinds.
    fn is_account_data(self) -> bool {
        match self {
            Self::Single(event_kind) => event_kind.is_account_data(),
            Self::DoubleAccountData => true,
        }
    }

    /// Whether we need to generate the redacted content type for this kind.
    pub fn generate_redacted(self, is_custom_redacted: bool) -> bool {
        // `is_custom_redacted` means that the content struct does not need a generated
        // redacted struct.
        !is_custom_redacted
            && matches!(self, Self::Single(EventKind::MessageLike | EventKind::State))
    }

    /// Whether we need to generate the possibly redacted content type for this kind.
    pub fn generate_possibly_redacted(self, is_custom_possibly_redacted: bool) -> bool {
        // `is_custom_possibly_redacted` means that the content struct does not need a generated
        // possibly redacted struct.
        !is_custom_possibly_redacted && matches!(self, Self::Single(EventKind::State))
    }

    /// Get the idents of the event struct for these kinds and the given variation.
    ///
    /// Returns a list of `(type_prefix, event_ident)` if the variation is supported for these
    /// kinds.
    pub fn to_event_idents(self, variation: EventVariation) -> Option<Vec<(&'static str, Ident)>> {
        match self {
            Self::Single(event_kind) => {
                event_kind.to_event_ident(variation).ok().map(|event_ident| vec![("", event_ident)])
            }
            Self::DoubleAccountData => {
                let first_event_ident = EventKind::GlobalAccountData
                    .to_event_ident(variation)
                    .ok()
                    .map(|event_ident| ("Global", event_ident));
                let second_event_ident = EventKind::RoomAccountData
                    .to_event_ident(variation)
                    .ok()
                    .map(|event_ident| ("Room", event_ident));

                if first_event_ident.is_none() && second_event_ident.is_none() {
                    None
                } else {
                    Some(first_event_ident.into_iter().chain(second_event_ident).collect())
                }
            }
        }
    }

    /// Get the idents of the event struct for these kinds and the given variation.
    ///
    /// Returns a list of `(type_enum, event_content_trait)`.
    pub fn to_content_kind_enums_and_traits(
        self,
        variation: EventContentVariation,
    ) -> Vec<(Ident, Ident)> {
        match self {
            Self::Single(event_kind) => {
                vec![(event_kind.to_event_type_enum(), event_kind.to_content_kind_trait(variation))]
            }
            Self::DoubleAccountData => [EventKind::GlobalAccountData, EventKind::RoomAccountData]
                .iter()
                .map(|event_kind| {
                    (event_kind.to_event_type_enum(), event_kind.to_content_kind_trait(variation))
                })
                .collect(),
        }
    }
}

impl From<EventKind> for EventContentKind {
    fn from(value: EventKind) -> Self {
        Self::Single(value)
    }
}

impl Parse for EventContentKind {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _: kw::kind = input.parse()?;
        let _: Token![=] = input.parse()?;
        let first_event_kind: EventKind = input.parse()?;

        let second_event_kind = input
            .peek(Token![+])
            .then(|| {
                let _: Token![+] = input.parse()?;
                input.parse::<EventKind>()
            })
            .transpose()?;

        match (first_event_kind, second_event_kind) {
            (event_kind, None) => Ok(Self::Single(event_kind)),
            (EventKind::GlobalAccountData, Some(EventKind::RoomAccountData))
            | (EventKind::RoomAccountData, Some(EventKind::GlobalAccountData)) => {
                Ok(Self::DoubleAccountData)
            }
            _ => Err(syn::Error::new(Span::call_site(), "only account data can have two kinds")),
        }
    }
}
