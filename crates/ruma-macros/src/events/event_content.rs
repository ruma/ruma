//! Implementations of the EventContent derive macro.

use std::borrow::Cow;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    DeriveInput, Field, Ident, LitStr, Token,
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
}

/// Parses attributes for `*EventContent` derives.
///
/// `#[ruma_event(type = "m.room.alias")]`
enum EventMeta {
    /// Variant holds the "m.whatever" event type.
    Type(LitStr),

    Kind(EventKind),

    /// Fields marked with `#[ruma_event(skip_redaction)]` are kept when the event is
    /// redacted.
    SkipRedaction,

    /// This attribute signals that the events redacted form is manually implemented and should not
    /// be generated.
    CustomRedacted,

    /// The given field holds a part of the event type (replaces the `*` in a `m.foo.*` event
    /// type).
    TypeFragment,
}

impl EventMeta {
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
}

impl Parse for EventMeta {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![type]) {
            let _: Token![type] = input.parse()?;
            let _: Token![=] = input.parse()?;
            input.parse().map(EventMeta::Type)
        } else if lookahead.peek(kw::kind) {
            let _: kw::kind = input.parse()?;
            let _: Token![=] = input.parse()?;
            EventKind::parse(input).map(EventMeta::Kind)
        } else if lookahead.peek(kw::skip_redaction) {
            let _: kw::skip_redaction = input.parse()?;
            Ok(EventMeta::SkipRedaction)
        } else if lookahead.peek(kw::custom_redacted) {
            let _: kw::custom_redacted = input.parse()?;
            Ok(EventMeta::CustomRedacted)
        } else if lookahead.peek(kw::type_fragment) {
            let _: kw::type_fragment = input.parse()?;
            Ok(EventMeta::TypeFragment)
        } else {
            Err(lookahead.error())
        }
    }
}

struct MetaAttrs(Vec<EventMeta>);

impl MetaAttrs {
    fn is_custom(&self) -> bool {
        self.0.iter().any(|a| matches!(a, &EventMeta::CustomRedacted))
    }

    fn get_event_type(&self) -> Option<&LitStr> {
        self.0.iter().find_map(|a| a.get_event_type())
    }

    fn get_event_kind(&self) -> Option<EventKind> {
        self.0.iter().find_map(|a| a.get_event_kind())
    }
}

impl Parse for MetaAttrs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let attrs = syn::punctuated::Punctuated::<EventMeta, Token![,]>::parse_terminated(input)?;
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

    // We only generate redacted content structs for state and message-like events
    let redacted_event_content = needs_redacted(&content_attr, event_kind)
        .then(|| {
            generate_redacted_event_content(
                ident,
                fields.clone(),
                event_type,
                event_kind,
                ruma_common,
            )
        })
        .transpose()?;

    let event_content_impl =
        generate_event_content_impl(ident, fields, event_type, event_kind, ruma_common)?;
    let static_event_content_impl = event_kind
        .map(|k| generate_static_event_content_impl(ident, k, false, event_type, ruma_common));
    let type_aliases = event_kind
        .map(|k| generate_event_type_aliases(k, ident, &event_type.value(), ruma_common))
        .transpose()?;

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
                        if let EventMeta::SkipRedaction = a.parse_args()? {
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
        ruma_common,
    )?;

    let static_event_content_impl = event_kind.map(|k| {
        generate_static_event_content_impl(&redacted_ident, k, true, event_type, ruma_common)
    });

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
                if ev_type != #event_type {
                    return Err(#serde::de::Error::custom(
                        format!("expected event type `{}`, found `{}`", #event_type, ev_type)
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
        EventKindVariation::Full,
        EventKindVariation::Sync,
        EventKindVariation::Stripped,
        EventKindVariation::Initial,
        EventKindVariation::Redacted,
        EventKindVariation::RedactedSync,
    ]
    .iter()
    .filter_map(|&var| Some((var, event_kind.try_to_event_ident(var)?)))
    .map(|(var, ev_struct)| {
        let ev_type = format_ident!("{}{}", var, ev_type_s);

        let doc_text = match var {
            EventKindVariation::Full => "",
            EventKindVariation::Sync => " from a `sync_events` response",
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
    ruma_common: &TokenStream,
) -> syn::Result<TokenStream> {
    let serde = quote! { #ruma_common::exports::serde };
    let serde_json = quote! { #ruma_common::exports::serde_json };

    let (event_type_ty_decl, event_type_ty, event_type_fn_impl);

    match event_kind {
        Some(kind) => {
            let i = kind.to_event_type_enum();
            event_type_ty_decl = None;
            event_type_ty = quote! { #ruma_common::events::#i };
            event_type_fn_impl = match event_type.value().strip_suffix(".*") {
                Some(type_prefix) => {
                    let type_fragment_field = fields
                        .find_map(|f| {
                            f.attrs.iter().filter(|a| a.path.is_ident("ruma_event")).find_map(|a| {
                                match a.parse_args() {
                                    Ok(EventMeta::TypeFragment) => Some(Ok(f)),
                                    Ok(_) => None,
                                    Err(e) => Some(Err(e)),
                                }
                            })
                        })
                        .transpose()?;

                    let f = type_fragment_field
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

                    let format = type_prefix.to_owned() + ".{}";

                    quote! {
                        ::std::convert::From::from(::std::format!(#format, self.#f))
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
                if ev_type != #event_type {
                    return ::std::result::Result::Err(#serde::de::Error::custom(
                        ::std::format!("expected event type `{}`, found `{}`", #event_type, ev_type)
                    ));
                }

                #serde_json::from_str(content.get())
            }
        }
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
