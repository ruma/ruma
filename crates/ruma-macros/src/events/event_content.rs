//! Implementations of the EventContent derive macro.

use std::borrow::Cow;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    DeriveInput, Field, Ident, LitStr, Token, Type,
};

use crate::util::m_prefix_name_to_type_name;

use super::event_parse::{EventKind, EventKindVariation};

mod kw {
    // This `content` field is kept when the event is redacted.
    syn::custom_keyword!(skip_redaction);
    // Do not emit any redacted event code.
    syn::custom_keyword!(custom_redacted);
    // The kind of event content this is.
    syn::custom_keyword!(kind);
    syn::custom_keyword!(type_fragment);
    // The type to use for a state events' `state_key` field.
    syn::custom_keyword!(state_key_type);
    // Another type string accepted for deserialization.
    syn::custom_keyword!(alias);
}

/// Parses struct attributes for `*EventContent` derives.
///
/// `#[ruma_event(type = "m.room.alias")]`
enum EventStructMeta {
    /// Variant holds the "m.whatever" event type.
    Type(LitStr),

    Kind(EventKind),

    /// This attribute signals that the events redacted form is manually implemented and should not
    /// be generated.
    CustomRedacted,

    StateKeyType(Box<Type>),

    /// Variant that holds alternate event type accepted for deserialization.
    Alias(LitStr),
}

impl EventStructMeta {
    fn get_event_type(&self) -> Option<&LitStr> {
        match self {
            Self::Type(t) => Some(t),
            _ => None,
        }
    }

    fn get_event_kind(&self) -> Option<EventKind> {
        match self {
            Self::Kind(k) => Some(*k),
            _ => None,
        }
    }

    fn get_state_key_type(&self) -> Option<&Type> {
        match self {
            Self::StateKeyType(ty) => Some(ty),
            _ => None,
        }
    }

    fn get_alias(&self) -> Option<&LitStr> {
        match self {
            Self::Alias(t) => Some(t),
            _ => None,
        }
    }
}

impl Parse for EventStructMeta {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![type]) {
            let _: Token![type] = input.parse()?;
            let _: Token![=] = input.parse()?;
            input.parse().map(EventStructMeta::Type)
        } else if lookahead.peek(kw::kind) {
            let _: kw::kind = input.parse()?;
            let _: Token![=] = input.parse()?;
            input.parse().map(EventStructMeta::Kind)
        } else if lookahead.peek(kw::custom_redacted) {
            let _: kw::custom_redacted = input.parse()?;
            Ok(EventStructMeta::CustomRedacted)
        } else if lookahead.peek(kw::state_key_type) {
            let _: kw::state_key_type = input.parse()?;
            let _: Token![=] = input.parse()?;
            input.parse().map(EventStructMeta::StateKeyType)
        } else if lookahead.peek(kw::alias) {
            let _: kw::alias = input.parse()?;
            let _: Token![=] = input.parse()?;
            input.parse().map(EventStructMeta::Alias)
        } else {
            Err(lookahead.error())
        }
    }
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

struct MetaAttrs(Vec<EventStructMeta>);

impl MetaAttrs {
    fn is_custom(&self) -> bool {
        self.0.iter().any(|a| matches!(a, &EventStructMeta::CustomRedacted))
    }

    fn get_event_type(&self) -> Option<&LitStr> {
        self.0.iter().find_map(|a| a.get_event_type())
    }

    fn get_event_kind(&self) -> Option<EventKind> {
        self.0.iter().find_map(|a| a.get_event_kind())
    }

    fn get_state_key_type(&self) -> Option<&Type> {
        self.0.iter().find_map(|a| a.get_state_key_type())
    }

    fn get_aliases(&self) -> impl Iterator<Item = &LitStr> {
        self.0.iter().filter_map(|a| a.get_alias())
    }
}

impl Parse for MetaAttrs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let attrs =
            syn::punctuated::Punctuated::<EventStructMeta, Token![,]>::parse_terminated(input)?;
        Ok(Self(attrs.into_iter().collect()))
    }
}

/// Create an `EventContent` implementation for a struct.
pub fn expand_event_content(
    input: &DeriveInput,
    ruma_common: &TokenStream,
) -> syn::Result<TokenStream> {
    let content_attr = input
        .attrs
        .iter()
        .filter(|attr| attr.path.is_ident("ruma_event"))
        .map(|attr| attr.parse_args::<MetaAttrs>())
        .collect::<syn::Result<Vec<_>>>()?;

    let mut event_types: Vec<_> =
        content_attr.iter().filter_map(|attrs| attrs.get_event_type()).collect();
    let event_type = match event_types.as_slice() {
        [] => {
            return Err(syn::Error::new(
                Span::call_site(),
                "no event type attribute found, \
                 add `#[ruma_event(type = \"any.room.event\", kind = Kind)]` \
                 below the event content derive",
            ));
        }
        [_] => event_types.pop().unwrap(),
        _ => {
            return Err(syn::Error::new(
                Span::call_site(),
                "multiple event type attribute found, there can only be one",
            ));
        }
    };

    let mut event_kinds: Vec<_> =
        content_attr.iter().filter_map(|attrs| attrs.get_event_kind()).collect();
    let event_kind = match event_kinds.as_slice() {
        [] => None,
        [_] => Some(event_kinds.pop().unwrap()),
        _ => {
            return Err(syn::Error::new(
                Span::call_site(),
                "multiple event kind attribute found, there can only be one",
            ));
        }
    };

    let state_key_types: Vec<_> =
        content_attr.iter().filter_map(|attrs| attrs.get_state_key_type()).collect();
    let state_key_type = match (event_kind, state_key_types.as_slice()) {
        (Some(EventKind::State), []) => {
            return Err(syn::Error::new(
                Span::call_site(),
                "no state_key_type attribute found, please specify one",
            ));
        }
        (Some(EventKind::State), [ty]) => Some(quote! { #ty }),
        (Some(EventKind::State), _) => {
            return Err(syn::Error::new(
                Span::call_site(),
                "multiple state_key_type attribute found, there can only be one",
            ));
        }
        (_, []) => None,
        (_, [ty, ..]) => {
            return Err(syn::Error::new_spanned(
                ty,
                "state_key_type attribute is not valid for non-state event kinds",
            ));
        }
    };

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

    let aliases: Vec<_> = content_attr.iter().flat_map(|attrs| attrs.get_aliases()).collect();
    for alias in &aliases {
        if alias.value().ends_with(".*") != prefix.is_some() {
            return Err(syn::Error::new_spanned(
                event_type,
                "aliases should have the same `.*` suffix, or lack thereof, as the main event type",
            ));
        }
    }

    // We only generate redacted content structs for state and message-like events
    let redacted_event_content = needs_redacted(&content_attr, event_kind).then(|| {
        generate_redacted_event_content(
            ident,
            fields.clone(),
            event_type,
            event_kind,
            state_key_type.as_ref(),
            &aliases,
            ruma_common,
        )
        .unwrap_or_else(syn::Error::into_compile_error)
    });

    let event_content_impl = generate_event_content_impl(
        ident,
        fields,
        event_type,
        event_kind,
        state_key_type.as_ref(),
        &aliases,
        ruma_common,
    )
    .unwrap_or_else(syn::Error::into_compile_error);
    let static_event_content_impl = event_kind
        .map(|k| generate_static_event_content_impl(ident, k, false, event_type, ruma_common));
    let type_aliases = event_kind.map(|k| {
        generate_event_type_aliases(k, ident, &event_type.value(), ruma_common)
            .unwrap_or_else(syn::Error::into_compile_error)
    });

    Ok(quote! {
        #redacted_event_content
        #event_content_impl
        #static_event_content_impl
        #type_aliases
    })
}

fn generate_redacted_event_content<'a>(
    ident: &Ident,
    fields: impl Iterator<Item = &'a Field>,
    event_type: &LitStr,
    event_kind: Option<EventKind>,
    state_key_type: Option<&TokenStream>,
    aliases: &[&LitStr],
    ruma_common: &TokenStream,
) -> syn::Result<TokenStream> {
    assert!(
        !event_type.value().contains('*'),
        "Event type shouldn't contain a `*`, this should have been checked previously"
    );

    let serde = quote! { #ruma_common::exports::serde };
    let serde_json = quote! { #ruma_common::exports::serde_json };

    let doc = format!("Redacted form of [`{}`]", ident);
    let redacted_ident = format_ident!("Redacted{}", ident);

    let kept_redacted_fields: Vec<_> = fields
        .map(|f| {
            let mut keep_field = false;
            let attrs = f
                .attrs
                .iter()
                .map(|a| -> syn::Result<_> {
                    if a.path.is_ident("ruma_event") {
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

    let (redacted_fields, redacted_return) = if kept_redacted_fields.is_empty() {
        (quote! { ; }, quote! { Ok(#redacted_ident {}) })
    } else {
        (
            quote! {
                { #( #kept_redacted_fields, )* }
            },
            quote! {
                Err(#serde::de::Error::custom(
                    format!("this redacted event has fields that cannot be constructed")
                ))
            },
        )
    };

    let (has_deserialize_fields, has_serialize_fields) = if kept_redacted_fields.is_empty() {
        (quote! { #ruma_common::events::HasDeserializeFields::False }, quote! { false })
    } else {
        (quote! { #ruma_common::events::HasDeserializeFields::True }, quote! { true })
    };

    let constructor = kept_redacted_fields.is_empty().then(|| {
        let doc = format!("Creates an empty {}.", redacted_ident);
        quote! {
            impl #redacted_ident {
                #[doc = #doc]
                pub fn new() -> Self {
                    Self
                }
            }
        }
    });

    let redacted_event_content = generate_event_content_impl(
        &redacted_ident,
        kept_redacted_fields.iter(),
        event_type,
        event_kind,
        state_key_type,
        aliases,
        ruma_common,
    )
    .unwrap_or_else(syn::Error::into_compile_error);

    let static_event_content_impl = event_kind.map(|k| {
        generate_static_event_content_impl(&redacted_ident, k, true, event_type, ruma_common)
    });

    let mut event_types = aliases.to_owned();
    event_types.push(event_type);
    let event_types = quote! {
        [#(#event_types,)*]
    };

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
        pub struct #redacted_ident #redacted_fields

        #constructor

        #redacted_event_content

        #[automatically_derived]
        impl #ruma_common::events::RedactedEventContent for #redacted_ident {
            fn empty(ev_type: &str) -> #serde_json::Result<Self> {
                if !#event_types.contains(&ev_type) {
                    return Err(#serde::de::Error::custom(
                        format!("expected event type as one of `{:?}`, found `{}`", #event_types, ev_type)
                    ));
                }

                #redacted_return
            }

            fn has_serialize_fields(&self) -> bool {
                #has_serialize_fields
            }

            fn has_deserialize_fields() -> #ruma_common::events::HasDeserializeFields {
                #has_deserialize_fields
            }
        }

        #static_event_content_impl
    })
}

fn generate_event_type_aliases(
    event_kind: EventKind,
    ident: &Ident,
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
        let ev_type = format_ident!("{}{}", var, ev_type_s);

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
        let ev_type_doc = format!("An `{}` event{}.", event_type, doc_text);

        let content_struct = if var.is_redacted() {
            Cow::Owned(format_ident!("Redacted{}", ident))
        } else {
            Cow::Borrowed(ident)
        };

        quote! {
            #[doc = #ev_type_doc]
            pub type #ev_type = #ruma_common::events::#ev_struct<#content_struct>;
        }
    })
    .collect();

    Ok(type_aliases)
}

fn generate_event_content_impl<'a>(
    ident: &Ident,
    mut fields: impl Iterator<Item = &'a Field>,
    event_type: &LitStr,
    event_kind: Option<EventKind>,
    state_key_type: Option<&TokenStream>,
    aliases: &[&'a LitStr],
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
                    f.attrs.iter().filter(|a| a.path.is_ident("ruma_event")).find_map(|a| {
                        match a.parse_args() {
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
                pub struct #i {
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

    let state_event_content_impl = (event_kind == Some(EventKind::State)).then(|| {
        assert!(state_key_type.is_some());
        quote! {
            #[automatically_derived]
            impl #ruma_common::events::StateEventContent for #ident {
                type StateKey = #state_key_type;
            }
        }
    });

    let event_types = aliases.iter().chain([&event_type]);

    let from_parts_fn_impl = if let Some((_, type_fragment_field)) = &type_suffix_data {
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

        quote! {
            if let Some(type_fragment) = #type_prefixes.iter().find_map(|prefix| ev_type.strip_prefix(prefix)) {
                let mut content: Self = #serde_json::from_str(content.get())?;
                content.#type_fragment_field = type_fragment.to_owned();

                ::std::result::Result::Ok(content)
            } else {
                ::std::result::Result::Err(#serde::de::Error::custom(
                    ::std::format!("expected event type starting with one of `{:?}`, found `{}`", #type_prefixes, ev_type)
                ))
            }
        }
    } else {
        let event_types = quote! {
            [#(#event_types,)*]
        };

        quote! {
            if !#event_types.contains(&ev_type) {
                return ::std::result::Result::Err(#serde::de::Error::custom(
                    ::std::format!("expected event type as one of `{:?}`, found `{}`", #event_types, ev_type)
                ));
            }

            #serde_json::from_str(content.get())
        }
    };

    Ok(quote! {
        #event_type_ty_decl

        #[automatically_derived]
        impl #ruma_common::events::EventContent for #ident {
            type EventType = #event_type_ty;

            fn event_type(&self) -> Self::EventType {
                #event_type_fn_impl
            }

            fn from_parts(
                ev_type: &::std::primitive::str,
                content: &#serde_json::value::RawValue,
            ) -> #serde_json::Result<Self> {
                #from_parts_fn_impl
            }
        }

        #state_event_content_impl
    })
}

fn generate_static_event_content_impl(
    ident: &Ident,
    event_kind: EventKind,
    redacted: bool,
    event_type: &LitStr,
    ruma_common: &TokenStream,
) -> TokenStream {
    let event_kind = match event_kind {
        EventKind::GlobalAccountData => quote! { GlobalAccountData },
        EventKind::RoomAccountData => quote! { RoomAccountData },
        EventKind::Ephemeral => quote! { EphemeralRoomData },
        EventKind::MessageLike => quote! { MessageLike { redacted: #redacted } },
        EventKind::State => quote! { State { redacted: #redacted } },
        EventKind::ToDevice => quote! { ToDevice },
        EventKind::RoomRedaction
        | EventKind::Presence
        | EventKind::Decrypted
        | EventKind::HierarchySpaceChild => {
            unreachable!("not a valid event content kind")
        }
    };

    quote! {
        impl #ruma_common::events::StaticEventContent for #ident {
            const KIND: #ruma_common::events::EventKind =
                #ruma_common::events::EventKind::#event_kind;
            const TYPE: &'static ::std::primitive::str = #event_type;
        }
    }
}

fn needs_redacted(input: &[MetaAttrs], event_kind: Option<EventKind>) -> bool {
    // `is_custom` means that the content struct does not need a generated
    // redacted struct also. If no `custom_redacted` attrs are found the content
    // needs a redacted struct generated.
    !input.iter().any(|a| a.is_custom())
        && matches!(event_kind, Some(EventKind::MessageLike) | Some(EventKind::State))
}
