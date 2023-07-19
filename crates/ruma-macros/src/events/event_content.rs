//! Implementations of the EventContent derive macro.
#![allow(clippy::too_many_arguments)] // FIXME

use std::{borrow::Cow, fmt};

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    DeriveInput, Field, Ident, LitStr, Meta, Token, Type,
};

use crate::util::{m_prefix_name_to_type_name, PrivateField};

use super::event_parse::{EventKind, EventKindVariation};

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
enum EventFieldMeta {
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
struct ContentMeta {
    event_type: Option<LitStr>,
    event_kind: Option<EventKind>,
    custom_redacted: Option<kw::custom_redacted>,
    custom_possibly_redacted: Option<kw::custom_possibly_redacted>,
    state_key_type: Option<Box<Type>>,
    unsigned_type: Option<Box<Type>>,
    aliases: Vec<LitStr>,
    without_relation: Option<kw::without_relation>,
}

impl ContentMeta {
    fn merge(self, other: ContentMeta) -> syn::Result<Self> {
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
            event_kind: either_named("event_kind", self.event_kind, other.event_kind)?,
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
            let _: kw::kind = input.parse()?;
            let _: Token![=] = input.parse()?;
            let event_kind = input.parse()?;

            Ok(Self { event_kind: Some(event_kind), ..Default::default() })
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

struct ContentAttrs {
    event_type: LitStr,
    event_kind: Option<EventKind>,
    state_key_type: Option<TokenStream>,
    unsigned_type: Option<TokenStream>,
    aliases: Vec<LitStr>,
    is_custom_redacted: bool,
    is_custom_possibly_redacted: bool,
    has_without_relation: bool,
}

impl TryFrom<ContentMeta> for ContentAttrs {
    type Error = syn::Error;

    fn try_from(value: ContentMeta) -> Result<Self, Self::Error> {
        let ContentMeta {
            event_type,
            event_kind,
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

        let state_key_type = match (event_kind, state_key_type) {
            (Some(EventKind::State), None) => {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "no state_key_type attribute found, please specify one",
                ));
            }
            (Some(EventKind::State), Some(ty)) => Some(quote! { #ty }),
            (_, None) => None,
            (_, Some(ty)) => {
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

        if prefix.is_some() && !event_kind.map_or(false, |k| k.is_account_data()) {
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
            event_kind,
            state_key_type,
            unsigned_type,
            aliases,
            is_custom_redacted,
            is_custom_possibly_redacted,
            has_without_relation,
        })
    }
}

/// Create an `EventContent` implementation for a struct.
pub fn expand_event_content(
    input: &DeriveInput,
    ruma_common: &TokenStream,
) -> syn::Result<TokenStream> {
    let content_meta = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("ruma_event"))
        .try_fold(ContentMeta::default(), |meta, attr| {
            let list: Punctuated<ContentMeta, Token![,]> =
                attr.parse_args_with(Punctuated::parse_terminated)?;

            list.into_iter().try_fold(meta, ContentMeta::merge)
        })?;

    let ContentAttrs {
        event_type,
        event_kind,
        state_key_type,
        unsigned_type,
        aliases,
        is_custom_redacted,
        is_custom_possibly_redacted,
        has_without_relation,
    } = content_meta.try_into()?;

    let ident = &input.ident;
    let fields = match &input.data {
        syn::Data::Struct(syn::DataStruct { fields, .. }) => fields.iter(),
        _ => {
            return Err(syn::Error::new(
                Span::call_site(),
                "event content types need to be structs",
            ));
        }
    };

    // We only generate redacted content structs for state and message-like events
    let redacted_event_content =
        event_kind.filter(|kind| needs_redacted(is_custom_redacted, *kind)).map(|kind| {
            generate_redacted_event_content(
                ident,
                &input.vis,
                fields.clone(),
                &event_type,
                kind,
                state_key_type.as_ref(),
                unsigned_type.clone(),
                &aliases,
                ruma_common,
            )
            .unwrap_or_else(syn::Error::into_compile_error)
        });

    // We only generate possibly redacted content structs for state events.
    let possibly_redacted_event_content = event_kind
        .filter(|kind| needs_possibly_redacted(is_custom_possibly_redacted, *kind))
        .map(|_| {
            generate_possibly_redacted_event_content(
                ident,
                &input.vis,
                fields.clone(),
                &event_type,
                state_key_type.as_ref(),
                unsigned_type.clone(),
                &aliases,
                ruma_common,
            )
            .unwrap_or_else(syn::Error::into_compile_error)
        });

    let event_content_without_relation = has_without_relation.then(|| {
        generate_event_content_without_relation(ident, &input.vis, fields.clone(), ruma_common)
            .unwrap_or_else(syn::Error::into_compile_error)
    });

    let event_content_impl = generate_event_content_impl(
        ident,
        &input.vis,
        fields,
        &event_type,
        event_kind,
        EventKindContentVariation::Original,
        state_key_type.as_ref(),
        unsigned_type,
        &aliases,
        ruma_common,
    )
    .unwrap_or_else(syn::Error::into_compile_error);
    let static_event_content_impl =
        generate_static_event_content_impl(ident, &event_type, ruma_common);
    let type_aliases = event_kind.map(|k| {
        generate_event_type_aliases(k, ident, &input.vis, &event_type.value(), ruma_common)
            .unwrap_or_else(syn::Error::into_compile_error)
    });

    Ok(quote! {
        #redacted_event_content
        #possibly_redacted_event_content
        #event_content_without_relation
        #event_content_impl
        #static_event_content_impl
        #type_aliases
    })
}

fn generate_redacted_event_content<'a>(
    ident: &Ident,
    vis: &syn::Visibility,
    fields: impl Iterator<Item = &'a Field>,
    event_type: &LitStr,
    event_kind: EventKind,
    state_key_type: Option<&TokenStream>,
    unsigned_type: Option<TokenStream>,
    aliases: &[LitStr],
    ruma_common: &TokenStream,
) -> syn::Result<TokenStream> {
    assert!(
        !event_type.value().contains('*'),
        "Event type shouldn't contain a `*`, this should have been checked previously"
    );

    let serde = quote! { #ruma_common::exports::serde };

    let doc = format!("Redacted form of [`{ident}`]");
    let redacted_ident = format_ident!("Redacted{ident}");

    let kept_redacted_fields: Vec<_> = fields
        .map(|f| {
            let mut keep_field = false;
            let attrs = f
                .attrs
                .iter()
                .map(|a| -> syn::Result<_> {
                    if a.path().is_ident("ruma_event") {
                        if let EventFieldMeta::SkipRedaction = a.parse_args()? {
                            keep_field = true;
                        }

                        // don't re-emit our `ruma_event` attributes
                        Ok(None)
                    } else {
                        Ok(Some(a.clone()))
                    }
                })
                .filter_map(Result::transpose)
                .collect::<syn::Result<_>>()?;

            if keep_field {
                Ok(Some(Field { attrs, ..f.clone() }))
            } else {
                Ok(None)
            }
        })
        .filter_map(Result::transpose)
        .collect::<syn::Result<_>>()?;

    let redaction_struct_fields = kept_redacted_fields.iter().flat_map(|f| &f.ident);

    let constructor = kept_redacted_fields.is_empty().then(|| {
        let doc = format!("Creates an empty {redacted_ident}.");
        quote! {
            impl #redacted_ident {
                #[doc = #doc]
                #vis fn new() -> Self {
                    Self {}
                }
            }
        }
    });

    let redacted_event_content = generate_event_content_impl(
        &redacted_ident,
        vis,
        kept_redacted_fields.iter(),
        event_type,
        Some(event_kind),
        EventKindContentVariation::Redacted,
        state_key_type,
        unsigned_type,
        aliases,
        ruma_common,
    )
    .unwrap_or_else(syn::Error::into_compile_error);

    let static_event_content_impl =
        generate_static_event_content_impl(&redacted_ident, event_type, ruma_common);

    Ok(quote! {
        // this is the non redacted event content's impl
        #[automatically_derived]
        impl #ruma_common::events::RedactContent for #ident {
            type Redacted = #redacted_ident;

            fn redact(self, version: &#ruma_common::RoomVersionId) -> #redacted_ident {
                #redacted_ident {
                    #( #redaction_struct_fields: self.#redaction_struct_fields, )*
                }
            }
        }

        #[doc = #doc]
        #[derive(Clone, Debug, #serde::Deserialize, #serde::Serialize)]
        #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
        #vis struct #redacted_ident {
            #( #kept_redacted_fields, )*
        }

        #constructor

        #redacted_event_content

        #static_event_content_impl
    })
}

fn generate_possibly_redacted_event_content<'a>(
    ident: &Ident,
    vis: &syn::Visibility,
    fields: impl Iterator<Item = &'a Field>,
    event_type: &LitStr,
    state_key_type: Option<&TokenStream>,
    unsigned_type: Option<TokenStream>,
    aliases: &[LitStr],
    ruma_common: &TokenStream,
) -> syn::Result<TokenStream> {
    assert!(
        !event_type.value().contains('*'),
        "Event type shouldn't contain a `*`, this should have been checked previously"
    );

    let serde = quote! { #ruma_common::exports::serde };

    let doc = format!(
        "The possibly redacted form of [`{ident}`].\n\n\
        This type is used when it's not obvious whether the content is redacted or not."
    );
    let possibly_redacted_ident = format_ident!("PossiblyRedacted{ident}");

    let mut field_changed = false;
    let possibly_redacted_fields: Vec<_> = fields
        .map(|f| {
            let mut keep_field = false;
            let mut unsupported_serde_attribute = None;

            if let Type::Path(type_path) = &f.ty {
                if type_path.path.segments.first().filter(|s| s.ident == "Option").is_some() {
                    // Keep the field if it's an `Option`.
                    keep_field = true;
                }
            }

            let mut attrs = f
                .attrs
                .iter()
                .map(|a| -> syn::Result<_> {
                    if a.path().is_ident("ruma_event") {
                        // Keep the field if it is not redacted.
                        if let EventFieldMeta::SkipRedaction = a.parse_args()? {
                            keep_field = true;
                        }

                        // Don't re-emit our `ruma_event` attributes.
                        Ok(None)
                    } else {
                        if a.path().is_ident("serde") {
                            if let Meta::List(list) = &a.meta {
                                let nested: Punctuated<Meta, Token![,]> =
                                    list.parse_args_with(Punctuated::parse_terminated)?;
                                for meta in &nested {
                                    if meta.path().is_ident("default") {
                                        // Keep the field if it deserializes to its default value.
                                        keep_field = true;
                                    } else if !meta.path().is_ident("rename")
                                        && !meta.path().is_ident("alias")
                                        && unsupported_serde_attribute.is_none()
                                    {
                                        // Error if the field is not kept and uses an unsupported
                                        // serde attribute.
                                        unsupported_serde_attribute =
                                            Some(syn::Error::new_spanned(
                                                meta,
                                                "Can't generate PossiblyRedacted struct with \
                                                 unsupported serde attribute\n\
                                                 Expected one of `default`, `rename` or `alias`\n\
                                                 Use the `custom_possibly_redacted` attribute \
                                                 and create the struct manually",
                                            ));
                                    }
                                }
                            }
                        }

                        Ok(Some(a.clone()))
                    }
                })
                .filter_map(Result::transpose)
                .collect::<syn::Result<_>>()?;

            if keep_field {
                Ok(Field { attrs, ..f.clone() })
            } else if let Some(err) = unsupported_serde_attribute {
                Err(err)
            } else if f.ident.is_none() {
                // If the field has no `ident`, it's a tuple struct. Since `content` is an object,
                // it will need a custom struct to deserialize from an empty object.
                Err(syn::Error::new(
                    Span::call_site(),
                    "Can't generate PossiblyRedacted struct for tuple structs\n\
                    Use the `custom_possibly_redacted` attribute and create the struct manually",
                ))
            } else {
                // Change the field to an `Option`.
                field_changed = true;

                let old_type = &f.ty;
                let ty = parse_quote! { Option<#old_type> };
                attrs.push(parse_quote! { #[serde(skip_serializing_if = "Option::is_none")] });

                Ok(Field { attrs, ty, ..f.clone() })
            }
        })
        .collect::<syn::Result<_>>()?;

    // If at least one field needs to change, generate a new struct, else use a type alias.
    if field_changed {
        let possibly_redacted_event_content = generate_event_content_impl(
            &possibly_redacted_ident,
            vis,
            possibly_redacted_fields.iter(),
            event_type,
            Some(EventKind::State),
            EventKindContentVariation::PossiblyRedacted,
            state_key_type,
            unsigned_type,
            aliases,
            ruma_common,
        )
        .unwrap_or_else(syn::Error::into_compile_error);

        let static_event_content_impl =
            generate_static_event_content_impl(&possibly_redacted_ident, event_type, ruma_common);

        Ok(quote! {
            #[doc = #doc]
            #[derive(Clone, Debug, #serde::Deserialize, #serde::Serialize)]
            #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
            #vis struct #possibly_redacted_ident {
                #( #possibly_redacted_fields, )*
            }

            #possibly_redacted_event_content

            #static_event_content_impl
        })
    } else {
        Ok(quote! {
            #[doc = #doc]
            #vis type #possibly_redacted_ident = #ident;

            #[automatically_derived]
            impl #ruma_common::events::PossiblyRedactedStateEventContent for #ident {
                type StateKey = #state_key_type;
            }
        })
    }
}

fn generate_event_content_without_relation<'a>(
    ident: &Ident,
    vis: &syn::Visibility,
    fields: impl Iterator<Item = &'a Field>,
    ruma_common: &TokenStream,
) -> syn::Result<TokenStream> {
    let serde = quote! { #ruma_common::exports::serde };

    let type_doc = format!(
        "Form of [`{ident}`] without relation.\n\n\
        To construct this type, construct a [`{ident}`] and then use one of its `::from()` / `.into()` methods."
    );
    let without_relation_ident = format_ident!("{ident}WithoutRelation");

    let with_relation_fn_doc =
        format!("Transform `self` into a [`{ident}`] with the given relation.");

    let (relates_to, other_fields) = fields.partition::<Vec<_>, _>(|f| {
        f.ident.as_ref().filter(|ident| *ident == "relates_to").is_some()
    });

    let relates_to_type = relates_to.into_iter().next().map(|f| &f.ty).ok_or_else(|| {
        syn::Error::new(
            Span::call_site(),
            "`without_relation` can only be used on events with a `relates_to` field",
        )
    })?;

    let without_relation_fields = other_fields.iter().flat_map(|f| &f.ident).collect::<Vec<_>>();
    let without_relation_struct = if other_fields.is_empty() {
        quote! { ; }
    } else {
        quote! {
            { #( #other_fields, )* }
        }
    };

    Ok(quote! {
        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl ::std::convert::From<#ident> for #without_relation_ident {
            fn from(c: #ident) -> Self {
                Self {
                    #( #without_relation_fields: c.#without_relation_fields, )*
                }
            }
        }

        #[doc = #type_doc]
        #[derive(Clone, Debug, #serde::Deserialize, #serde::Serialize)]
        #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
        #vis struct #without_relation_ident #without_relation_struct

        impl #without_relation_ident {
            #[doc = #with_relation_fn_doc]
            #vis fn with_relation(self, relates_to: #relates_to_type) -> #ident {
                #ident {
                    #( #without_relation_fields: self.#without_relation_fields, )*
                    relates_to,
                }
            }
        }
    })
}

fn generate_event_type_aliases(
    event_kind: EventKind,
    ident: &Ident,
    vis: &syn::Visibility,
    event_type: &str,
    ruma_common: &TokenStream,
) -> syn::Result<TokenStream> {
    // The redaction module has its own event types.
    if ident == "RoomRedactionEventContent" {
        return Ok(quote! {});
    }

    let ident_s = ident.to_string();
    let ev_type_s = ident_s.strip_suffix("Content").ok_or_else(|| {
        syn::Error::new_spanned(ident, "Expected content struct name ending in `Content`")
    })?;

    let type_aliases = [
        EventKindVariation::None,
        EventKindVariation::Sync,
        EventKindVariation::Original,
        EventKindVariation::OriginalSync,
        EventKindVariation::Stripped,
        EventKindVariation::Initial,
        EventKindVariation::Redacted,
        EventKindVariation::RedactedSync,
    ]
    .iter()
    .filter_map(|&var| Some((var, event_kind.to_event_ident(var).ok()?)))
    .map(|(var, ev_struct)| {
        let ev_type = format_ident!("{var}{ev_type_s}");

        let doc_text = match var {
            EventKindVariation::None | EventKindVariation::Original => "",
            EventKindVariation::Sync | EventKindVariation::OriginalSync => {
                " from a `sync_events` response"
            }
            EventKindVariation::Stripped => " from an invited room preview",
            EventKindVariation::Redacted => " that has been redacted",
            EventKindVariation::RedactedSync => {
                " from a `sync_events` response that has been redacted"
            }
            EventKindVariation::Initial => " for creating a room",
        };
        let ev_type_doc = format!("An `{event_type}` event{doc_text}.");

        let content_struct = if var.is_redacted() {
            Cow::Owned(format_ident!("Redacted{ident}"))
        } else if let EventKindVariation::Stripped = var {
            Cow::Owned(format_ident!("PossiblyRedacted{ident}"))
        } else {
            Cow::Borrowed(ident)
        };

        quote! {
            #[doc = #ev_type_doc]
            #vis type #ev_type = #ruma_common::events::#ev_struct<#content_struct>;
        }
    })
    .collect();

    Ok(type_aliases)
}

#[derive(PartialEq)]
enum EventKindContentVariation {
    Original,
    Redacted,
    PossiblyRedacted,
}

impl fmt::Display for EventKindContentVariation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventKindContentVariation::Original => Ok(()),
            EventKindContentVariation::Redacted => write!(f, "Redacted"),
            EventKindContentVariation::PossiblyRedacted => write!(f, "PossiblyRedacted"),
        }
    }
}

fn generate_event_content_impl<'a>(
    ident: &Ident,
    vis: &syn::Visibility,
    mut fields: impl Iterator<Item = &'a Field>,
    event_type: &LitStr,
    event_kind: Option<EventKind>,
    variation: EventKindContentVariation,
    state_key_type: Option<&TokenStream>,
    unsigned_type: Option<TokenStream>,
    aliases: &[LitStr],
    ruma_common: &TokenStream,
) -> syn::Result<TokenStream> {
    let serde = quote! { #ruma_common::exports::serde };
    let serde_json = quote! { #ruma_common::exports::serde_json };

    let (event_type_ty_decl, event_type_ty, event_type_fn_impl);

    let type_suffix_data = event_type
        .value()
        .strip_suffix('*')
        .map(|type_prefix| {
            let type_fragment_field = fields
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

            <syn::Result<_>>::Ok((type_prefix.to_owned(), type_fragment_field))
        })
        .transpose()?;

    match event_kind {
        Some(kind) => {
            let i = kind.to_event_type_enum();
            event_type_ty_decl = None;
            event_type_ty = quote! { #ruma_common::events::#i };
            event_type_fn_impl = match &type_suffix_data {
                Some((type_prefix, type_fragment_field)) => {
                    let format = type_prefix.to_owned() + "{}";

                    quote! {
                        ::std::convert::From::from(::std::format!(#format, self.#type_fragment_field))
                    }
                }
                None => quote! { ::std::convert::From::from(#event_type) },
            };
        }
        None => {
            let camel_case_type_name = m_prefix_name_to_type_name(event_type)?;
            let i = format_ident!("{}EventType", camel_case_type_name);
            event_type_ty_decl = Some(quote! {
                /// Implementation detail, you don't need to care about this.
                #[doc(hidden)]
                #vis struct #i {
                    // Set to None for intended type, Some for a different one
                    ty: ::std::option::Option<crate::PrivOwnedStr>,
                }

                impl #serde::Serialize for #i {
                    fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
                    where
                        S: #serde::Serializer,
                    {
                        let s = self.ty.as_ref().map(|t| &t.0[..]).unwrap_or(#event_type);
                        serializer.serialize_str(s)
                    }
                }
            });
            event_type_ty = quote! { #i };
            event_type_fn_impl = quote! { #event_type_ty { ty: ::std::option::Option::None } };
        }
    }

    let sub_trait_impl = event_kind.map(|kind| {
        let trait_name = format_ident!("{variation}{kind}Content");

        let state_key = (kind == EventKind::State).then(|| {
            assert!(state_key_type.is_some());

            quote! {
                type StateKey = #state_key_type;
            }
        });

        quote! {
            #[automatically_derived]
            impl #ruma_common::events::#trait_name for #ident {
                #state_key
            }
        }
    });

    let static_state_event_content_impl = (event_kind == Some(EventKind::State)
        && variation == EventKindContentVariation::Original)
        .then(|| {
            let possibly_redacted_ident = format_ident!("PossiblyRedacted{ident}");

            let unsigned_type = unsigned_type.unwrap_or_else(
                || quote! { #ruma_common::events::StateUnsigned<Self::PossiblyRedacted> },
            );

            quote! {
                #[automatically_derived]
                impl #ruma_common::events::StaticStateEventContent for #ident {
                    type PossiblyRedacted = #possibly_redacted_ident;
                    type Unsigned = #unsigned_type;
                }
            }
        });

    let event_types = aliases.iter().chain([event_type]);

    let event_content_from_type_impl = type_suffix_data.map(|(_, type_fragment_field)| {
        let type_prefixes = event_types.map(|ev_type| {
            ev_type
                .value()
                .strip_suffix('*')
                .expect("aliases have already been checked to have the same suffix")
                .to_owned()
        });
        let type_prefixes = quote! {
            [#(#type_prefixes,)*]
        };
        let fields_without_type_fragment = fields
            .filter(|f| {
                !f.attrs.iter().any(|a| {
                    a.path().is_ident("ruma_event")
                        && matches!(a.parse_args(), Ok(EventFieldMeta::TypeFragment))
                })
            })
            .map(PrivateField)
            .collect::<Vec<_>>();
        let fields_ident_without_type_fragment =
            fields_without_type_fragment.iter().filter_map(|f| f.0.ident.as_ref());

        quote! {
            impl #ruma_common::events::EventContentFromType for #ident {
                fn from_parts(
                    ev_type: &::std::primitive::str,
                    content: &#serde_json::value::RawValue,
                ) -> #serde_json::Result<Self> {
                    #[derive(#serde::Deserialize)]
                    struct WithoutTypeFragment {
                        #( #fields_without_type_fragment, )*
                    }

                    if let ::std::option::Option::Some(type_fragment) =
                        #type_prefixes.iter().find_map(|prefix| ev_type.strip_prefix(prefix))
                    {
                        let c: WithoutTypeFragment = #serde_json::from_str(content.get())?;

                        ::std::result::Result::Ok(Self {
                            #(
                                #fields_ident_without_type_fragment:
                                    c.#fields_ident_without_type_fragment,
                            )*
                            #type_fragment_field: type_fragment.to_owned(),
                        })
                    } else {
                        ::std::result::Result::Err(#serde::de::Error::custom(
                            ::std::format!(
                                "expected event type starting with one of `{:?}`, found `{}`",
                                #type_prefixes, ev_type,
                            )
                        ))
                    }
                }
            }
        }
    });

    Ok(quote! {
        #event_type_ty_decl

        #[automatically_derived]
        impl #ruma_common::events::EventContent for #ident {
            type EventType = #event_type_ty;

            fn event_type(&self) -> Self::EventType {
                #event_type_fn_impl
            }
        }

        #event_content_from_type_impl
        #sub_trait_impl
        #static_state_event_content_impl
    })
}

fn generate_static_event_content_impl(
    ident: &Ident,
    event_type: &LitStr,
    ruma_common: &TokenStream,
) -> TokenStream {
    quote! {
        impl #ruma_common::events::StaticEventContent for #ident {
            const TYPE: &'static ::std::primitive::str = #event_type;
        }
    }
}

fn needs_redacted(is_custom_redacted: bool, event_kind: EventKind) -> bool {
    // `is_custom` means that the content struct does not need a generated
    // redacted struct also. If no `custom_redacted` attrs are found the content
    // needs a redacted struct generated.
    !is_custom_redacted && matches!(event_kind, EventKind::MessageLike | EventKind::State)
}

fn needs_possibly_redacted(is_custom_possibly_redacted: bool, event_kind: EventKind) -> bool {
    // `is_custom_possibly_redacted` means that the content struct does not need
    // a generated possibly redacted struct also. If no `custom_possibly_redacted`
    // attrs are found the content needs a possibly redacted struct generated.
    !is_custom_possibly_redacted && event_kind == EventKind::State
}
