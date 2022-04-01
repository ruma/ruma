//! Implementation of event enum and event content enum macros.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Attribute, Data, DataEnum, DeriveInput, Ident, LitStr};

use super::{
    event_parse::{EventEnumDecl, EventEnumEntry, EventKind, EventKindVariation},
    util::{has_prev_content, is_non_stripped_room_event},
};
use crate::util::m_prefix_name_to_type_name;

type EventKindFn = fn(EventKind, EventKindVariation) -> bool;

/// This const is used to generate the accessor methods for the `Any*Event` enums.
///
/// DO NOT alter the field names unless the structs in `ruma_common::events::event_kinds` have
/// changed.
const EVENT_FIELDS: &[(&str, EventKindFn)] = &[
    ("origin_server_ts", is_non_stripped_room_event),
    ("room_id", |kind, var| {
        matches!(kind, EventKind::MessageLike | EventKind::State | EventKind::Ephemeral)
            && matches!(var, EventKindVariation::Full | EventKindVariation::Redacted)
    }),
    ("event_id", is_non_stripped_room_event),
    ("sender", |kind, var| {
        matches!(kind, EventKind::MessageLike | EventKind::State | EventKind::ToDevice)
            && var != EventKindVariation::Initial
    }),
    ("state_key", |kind, _| matches!(kind, EventKind::State)),
];

/// Create a content enum from `EventEnumInput`.
pub fn expand_event_enums(input: &EventEnumDecl) -> syn::Result<TokenStream> {
    use EventKindVariation as V;

    let ruma_common = crate::import_ruma_common();

    let mut res = TokenStream::new();

    let kind = input.kind;
    let attrs = &input.attrs;
    let events: Vec<_> = input.events.iter().map(|entry| entry.ev_type.clone()).collect();
    let variants: Vec<_> =
        input.events.iter().map(EventEnumEntry::to_variant).collect::<syn::Result<_>>()?;

    let events = &events;
    let variants = &variants;
    let ruma_common = &ruma_common;

    res.extend(
        expand_event_enum(kind, V::Full, events, attrs, variants, ruma_common)
            .unwrap_or_else(syn::Error::into_compile_error),
    );
    res.extend(expand_content_enum(kind, events, attrs, variants, ruma_common));

    if matches!(kind, EventKind::Ephemeral | EventKind::MessageLike | EventKind::State) {
        res.extend(
            expand_event_enum(kind, V::Sync, events, attrs, variants, ruma_common)
                .unwrap_or_else(syn::Error::into_compile_error),
        );
        res.extend(
            expand_from_full_event(kind, V::Full, variants)
                .unwrap_or_else(syn::Error::into_compile_error),
        );
        res.extend(
            expand_into_full_event(kind, V::Sync, variants, ruma_common)
                .unwrap_or_else(syn::Error::into_compile_error),
        );
    }

    if matches!(kind, EventKind::State) {
        res.extend(
            expand_event_enum(kind, V::Stripped, events, attrs, variants, ruma_common)
                .unwrap_or_else(syn::Error::into_compile_error),
        );
        res.extend(
            expand_event_enum(kind, V::Initial, events, attrs, variants, ruma_common)
                .unwrap_or_else(syn::Error::into_compile_error),
        );
    }

    if matches!(kind, EventKind::MessageLike | EventKind::State) {
        res.extend(
            expand_event_enum(kind, V::Redacted, events, attrs, variants, ruma_common)
                .unwrap_or_else(syn::Error::into_compile_error),
        );
        res.extend(
            expand_event_enum(kind, V::RedactedSync, events, attrs, variants, ruma_common)
                .unwrap_or_else(syn::Error::into_compile_error),
        );
        res.extend(
            expand_redact(kind, V::Full, variants, ruma_common)
                .unwrap_or_else(syn::Error::into_compile_error),
        );
        res.extend(
            expand_redact(kind, V::Sync, variants, ruma_common)
                .unwrap_or_else(syn::Error::into_compile_error),
        );
        res.extend(
            expand_possibly_redacted_enum(kind, V::Full, ruma_common)
                .unwrap_or_else(syn::Error::into_compile_error),
        );
        res.extend(
            expand_possibly_redacted_enum(kind, V::Sync, ruma_common)
                .unwrap_or_else(syn::Error::into_compile_error),
        );
        res.extend(
            expand_from_full_event(kind, V::Redacted, variants)
                .unwrap_or_else(syn::Error::into_compile_error),
        );
        res.extend(
            expand_into_full_event(kind, V::RedactedSync, variants, ruma_common)
                .unwrap_or_else(syn::Error::into_compile_error),
        );
    }

    Ok(res)
}

fn expand_event_enum(
    kind: EventKind,
    var: EventKindVariation,
    events: &[LitStr],
    attrs: &[Attribute],
    variants: &[EventEnumVariant],
    ruma_common: &TokenStream,
) -> syn::Result<TokenStream> {
    let event_struct = kind.to_event_ident(var)?;
    let ident = kind.to_event_enum_ident(var)?;

    let variant_decls = variants.iter().map(|v| v.decl());
    let content: Vec<_> =
        events.iter().map(|event| to_event_path(event, kind, var, ruma_common)).collect();

    let custom_ty = format_ident!("Custom{}Content", kind);

    let deserialize_impl = expand_deserialize_impl(kind, var, events, variants, ruma_common)?;
    let field_accessor_impl = expand_accessor_methods(kind, var, variants, ruma_common)?;
    let from_impl = expand_from_impl(&ident, &content, variants);

    Ok(quote! {
        #( #attrs )*
        #[derive(Clone, Debug)]
        #[allow(clippy::large_enum_variant)]
        #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
        pub enum #ident {
            #(
                #[doc = #events]
                #variant_decls(#content),
            )*
            /// An event not defined by the Matrix specification
            #[doc(hidden)]
            _Custom(
                #ruma_common::events::#event_struct<#ruma_common::events::_custom::#custom_ty>,
            ),
        }

        #deserialize_impl
        #field_accessor_impl
        #from_impl
    })
}

fn expand_deserialize_impl(
    kind: EventKind,
    var: EventKindVariation,
    events: &[LitStr],
    variants: &[EventEnumVariant],
    ruma_common: &TokenStream,
) -> syn::Result<TokenStream> {
    let serde = quote! { #ruma_common::exports::serde };
    let serde_json = quote! { #ruma_common::exports::serde_json };

    let ident = kind.to_event_enum_ident(var)?;

    let variant_attrs = variants.iter().map(|v| {
        let attrs = &v.attrs;
        quote! { #(#attrs)* }
    });
    let self_variants = variants.iter().map(|v| v.ctor(quote! { Self }));
    let content = events.iter().map(|event| to_event_path(event, kind, var, ruma_common));

    Ok(quote! {
        impl<'de> #serde::de::Deserialize<'de> for #ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: #serde::de::Deserializer<'de>,
            {
                use #serde::de::Error as _;

                let json = Box::<#serde_json::value::RawValue>::deserialize(deserializer)?;
                let #ruma_common::events::EventTypeDeHelper { ev_type, .. } =
                    #ruma_common::serde::from_raw_json_value(&json)?;

                match &*ev_type {
                    #(
                        #variant_attrs #events => {
                            let event = #serde_json::from_str::<#content>(json.get())
                                .map_err(D::Error::custom)?;
                            Ok(#self_variants(event))
                        },
                    )*
                    _ => {
                        let event = #serde_json::from_str(json.get()).map_err(D::Error::custom)?;
                        Ok(Self::_Custom(event))
                    },
                }
            }
        }
    })
}

fn expand_from_impl(
    ty: &Ident,
    content: &[TokenStream],
    variants: &[EventEnumVariant],
) -> TokenStream {
    let from_impls = content.iter().zip(variants).map(|(content, variant)| {
        let ident = &variant.ident;
        let attrs = &variant.attrs;

        quote! {
            #[automatically_derived]
            #(#attrs)*
            impl ::std::convert::From<#content> for #ty {
                fn from(c: #content) -> Self {
                    Self::#ident(c)
                }
            }
        }
    });

    quote! { #( #from_impls )* }
}

fn expand_from_full_event(
    kind: EventKind,
    var: EventKindVariation,
    variants: &[EventEnumVariant],
) -> syn::Result<TokenStream> {
    let ident = kind.to_event_enum_ident(var)?;
    let sync = kind.to_event_enum_ident(var.to_sync())?;

    let ident_variants = variants.iter().map(|v| v.match_arm(&ident));
    let self_variants = variants.iter().map(|v| v.ctor(quote! { Self }));

    Ok(quote! {
        #[automatically_derived]
        impl ::std::convert::From<#ident> for #sync {
            fn from(event: #ident) -> Self {
                match event {
                    #(
                        #ident_variants(event) => {
                            #self_variants(::std::convert::From::from(event))
                        },
                    )*
                    #ident::_Custom(event) => {
                        Self::_Custom(::std::convert::From::from(event))
                    },
                }
            }
        }
    })
}

fn expand_into_full_event(
    kind: EventKind,
    var: EventKindVariation,
    variants: &[EventEnumVariant],
    ruma_common: &TokenStream,
) -> syn::Result<TokenStream> {
    let ident = kind.to_event_enum_ident(var)?;
    let full = kind.to_event_enum_ident(var.to_full())?;

    let self_variants = variants.iter().map(|v| v.match_arm(quote! { Self }));
    let full_variants = variants.iter().map(|v| v.ctor(&full));

    Ok(quote! {
        #[automatically_derived]
        impl #ident {
            /// Convert this sync event into a full event (one with a `room_id` field).
            pub fn into_full_event(
                self,
                room_id: ::std::boxed::Box<#ruma_common::RoomId>,
            ) -> #full {
                match self {
                    #(
                        #self_variants(event) => {
                            #full_variants(event.into_full_event(room_id))
                        },
                    )*
                    Self::_Custom(event) => {
                        #full::_Custom(event.into_full_event(room_id))
                    },
                }
            }
        }
    })
}

/// Create a content enum from `EventEnumInput`.
fn expand_content_enum(
    kind: EventKind,
    event_types: &[LitStr],
    attrs: &[Attribute],
    variants: &[EventEnumVariant],
    ruma_common: &TokenStream,
) -> TokenStream {
    let serde = quote! { #ruma_common::exports::serde };
    let serde_json = quote! { #ruma_common::exports::serde_json };

    let ident = kind.to_content_enum();

    let event_type_enum = kind.to_event_type_enum();

    let content: Vec<_> =
        event_types.iter().map(|ev| to_event_content_path(kind, ev, None, ruma_common)).collect();
    let event_type_match_arms = event_types.iter().map(|s| {
        if let Some(prefix) = s.value().strip_suffix(".*") {
            quote! { _s if _s.starts_with(#prefix) }
        } else {
            quote! { #s }
        }
    });

    let variant_decls = variants.iter().map(|v| v.decl()).collect::<Vec<_>>();
    let variant_attrs = variants.iter().map(|v| {
        let attrs = &v.attrs;
        quote! { #(#attrs)* }
    });
    let variant_arms = variants.iter().map(|v| v.match_arm(quote! { Self })).collect::<Vec<_>>();
    let variant_ctors = variants.iter().map(|v| v.ctor(quote! { Self }));

    let from_impl = expand_from_impl(&ident, &content, variants);

    let serialize_custom_event_error_path =
        quote! { #ruma_common::events::serialize_custom_event_error }.to_string();

    quote! {
        #( #attrs )*
        #[derive(Clone, Debug, #serde::Serialize)]
        #[serde(untagged)]
        #[allow(clippy::large_enum_variant)]
        #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
        pub enum #ident {
            #(
                #[doc = #event_types]
                #variant_decls(#content),
            )*
            #[doc(hidden)]
            #[serde(serialize_with = #serialize_custom_event_error_path)]
            _Custom {
                event_type: crate::PrivOwnedStr,
            },
        }

        #[automatically_derived]
        impl #ruma_common::events::EventContent for #ident {
            type EventType = #ruma_common::events::#event_type_enum;

            fn event_type(&self) -> Self::EventType {
                match self {
                    #( #variant_arms(content) => content.event_type(), )*
                    Self::_Custom { event_type } => ::std::convert::From::from(&event_type.0[..]),
                }
            }

            fn from_parts(
                event_type: &::std::primitive::str,
                input: &#serde_json::value::RawValue,
            ) -> #serde_json::Result<Self> {
                match event_type {
                    #(
                        #variant_attrs #event_type_match_arms => {
                            let content = #content::from_parts(event_type, input)?;
                            ::std::result::Result::Ok(#variant_ctors(content))
                        }
                    )*
                    ty => {
                        ::std::result::Result::Ok(Self::_Custom {
                            event_type: crate::PrivOwnedStr(ty.into()),
                        })
                    }
                }
            }
        }

        #from_impl
    }
}

fn expand_redact(
    kind: EventKind,
    var: EventKindVariation,
    variants: &[EventEnumVariant],
    ruma_common: &TokenStream,
) -> syn::Result<TokenStream> {
    let ident = kind.to_event_enum_ident(var)?;
    let redacted_enum = kind.to_event_enum_ident(var.to_redacted())?;

    let self_variants = variants.iter().map(|v| v.match_arm(quote! { Self }));
    let redacted_variants = variants.iter().map(|v| v.ctor(&redacted_enum));

    Ok(quote! {
        #[automatically_derived]
        impl #ruma_common::events::Redact for #ident {
            type Redacted = #redacted_enum;

            fn redact(
                self,
                redaction: #ruma_common::events::room::redaction::SyncRoomRedactionEvent,
                version: &#ruma_common::RoomVersionId,
            ) -> #redacted_enum {
                match self {
                    #(
                        #self_variants(event) => #redacted_variants(
                            #ruma_common::events::Redact::redact(event, redaction, version),
                        ),
                    )*
                    Self::_Custom(event) => #redacted_enum::_Custom(
                        #ruma_common::events::Redact::redact(event, redaction, version),
                    )
                }
            }
        }
    })
}

fn expand_possibly_redacted_enum(
    kind: EventKind,
    var: EventKindVariation,
    ruma_common: &TokenStream,
) -> syn::Result<TokenStream> {
    let serde = quote! { #ruma_common::exports::serde };
    let serde_json = quote! { #ruma_common::exports::serde_json };

    let ident = format_ident!("AnyPossiblyRedacted{}", kind.to_event_ident(var)?);
    let regular_enum_ident = kind.to_event_enum_ident(var)?;
    let redacted_enum_ident = kind.to_event_enum_ident(var.to_redacted())?;

    Ok(quote! {
        /// An enum that holds either regular un-redacted events or redacted events.
        #[derive(Clone, Debug)]
        #[allow(clippy::exhaustive_enums)]
        pub enum #ident {
            /// An un-redacted event.
            Regular(#regular_enum_ident),

            /// A redacted event.
            Redacted(#redacted_enum_ident),
        }

        impl<'de> #serde::de::Deserialize<'de> for #ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: #serde::de::Deserializer<'de>,
            {
                let json = Box::<#serde_json::value::RawValue>::deserialize(deserializer)?;
                let #ruma_common::events::RedactionDeHelper { unsigned } =
                    #ruma_common::serde::from_raw_json_value(&json)?;

                Ok(match unsigned {
                    Some(unsigned) if unsigned.redacted_because.is_some() => {
                        Self::Redacted(#ruma_common::serde::from_raw_json_value(&json)?)
                    }
                    _ => Self::Regular(#ruma_common::serde::from_raw_json_value(&json)?),
                })
            }
        }
    })
}

fn expand_accessor_methods(
    kind: EventKind,
    var: EventKindVariation,
    variants: &[EventEnumVariant],
    ruma_common: &TokenStream,
) -> syn::Result<TokenStream> {
    let ident = kind.to_event_enum_ident(var)?;
    let event_type_enum = format_ident!("{}Type", kind);
    let self_variants: Vec<_> = variants.iter().map(|v| v.match_arm(quote! { Self })).collect();

    let content_accessors = (!var.is_redacted()).then(|| {
        let content_enum = kind.to_content_enum();
        let content_variants: Vec<_> = variants.iter().map(|v| v.ctor(&content_enum)).collect();

        let unsigned = if has_prev_content(kind, var) {
            quote! {
                /// Returns this event's unsigned field.
                pub fn unsigned(&self) -> #ruma_common::events::StateUnsigned<#content_enum> {
                    match self {
                        #(
                            #self_variants(event) => {
                                event.unsigned._map_prev_unsigned(|c| {
                                    #content_variants(c.clone())
                                })
                            },
                        )*
                        Self::_Custom(event) => {
                            event.unsigned._map_prev_unsigned(|c| #content_enum::_Custom {
                                event_type: crate::PrivOwnedStr(
                                    ::std::convert::From::from(
                                        ::std::string::ToString::to_string(
                                            &#ruma_common::events::EventContent::event_type(c)
                                        )
                                    ),
                                ),
                            })
                        },
                    }
                }
            }
        } else if is_non_stripped_room_event(kind, var) {
            let field_type = field_return_type("unsigned", var, ruma_common);
            let variants = variants.iter().map(|v| v.match_arm(quote! { Self }));

            quote! {
                /// Returns this event's unsigned field.
                pub fn unsigned(&self) -> &#field_type {
                    match self {
                        #( #variants(event) => &event.unsigned, )*
                        Self::_Custom(event) => &event.unsigned,
                    }
                }
            }
        } else {
            quote! {}
        };

        quote! {
            /// Returns the content for this event.
            pub fn content(&self) -> #content_enum {
                match self {
                    #( #self_variants(event) => #content_variants(event.content.clone()), )*
                    Self::_Custom(event) => #content_enum::_Custom {
                        event_type: crate::PrivOwnedStr(
                            ::std::convert::From::from(
                                ::std::string::ToString::to_string(
                                    &#ruma_common::events::EventContent::event_type(&event.content)
                                )
                            ),
                        ),
                    },
                }
            }

            #unsigned
        }
    });

    let methods = EVENT_FIELDS.iter().map(|(name, has_field)| {
        has_field(kind, var).then(|| {
            let docs = format!("Returns this event's {} field.", name);
            let ident = Ident::new(name, Span::call_site());
            let field_type = field_return_type(name, var, ruma_common);
            let variants = variants.iter().map(|v| v.match_arm(quote! { Self }));

            quote! {
                #[doc = #docs]
                pub fn #ident(&self) -> &#field_type {
                    match self {
                        #( #variants(event) => &event.#ident, )*
                        Self::_Custom(event) => &event.#ident,
                    }
                }
            }
        })
    });

    Ok(quote! {
        #[automatically_derived]
        impl #ident {
            /// Returns the `type` of this event.
            pub fn event_type(&self) -> #ruma_common::events::#event_type_enum {
                match self {
                    #( #self_variants(event) =>
                        #ruma_common::events::EventContent::event_type(&event.content), )*
                    Self::_Custom(event) => ::std::convert::From::from(
                        #ruma_common::events::EventContent::event_type(&event.content),
                    ),
                }
            }

            #content_accessors

            #( #methods )*
        }
    })
}

fn to_event_path(
    name: &LitStr,
    kind: EventKind,
    var: EventKindVariation,
    ruma_common: &TokenStream,
) -> TokenStream {
    let path = event_module_path(name);
    let event = m_prefix_name_to_type_name(name).unwrap();
    let event_name = if kind == EventKind::ToDevice {
        assert_eq!(var, EventKindVariation::Full);
        format_ident!("ToDevice{}Event", event)
    } else {
        format_ident!("{}{}Event", var, event)
    };
    quote! { #ruma_common::events::#( #path )::*::#event_name }
}

fn to_event_content_path(
    kind: EventKind,
    name: &LitStr,
    prefix: Option<&str>,
    ruma_common: &TokenStream,
) -> TokenStream {
    let path = event_module_path(name);
    let event = m_prefix_name_to_type_name(name).unwrap();
    let content_str = match kind {
        EventKind::ToDevice => {
            format_ident!("ToDevice{}{}EventContent", prefix.unwrap_or(""), event)
        }
        _ => format_ident!("{}{}EventContent", prefix.unwrap_or(""), event),
    };

    quote! {
        #ruma_common::events::#( #path )::*::#content_str
    }
}

fn event_module_path(name: &LitStr) -> Vec<Ident> {
    let value = name.value();
    let value = value.strip_prefix("m.").unwrap();
    value
        .strip_suffix(".*")
        .unwrap_or(value)
        .split('.')
        .map(|s| Ident::new(s, name.span()))
        .collect()
}

fn field_return_type(
    name: &str,
    var: EventKindVariation,
    ruma_common: &TokenStream,
) -> TokenStream {
    match name {
        "origin_server_ts" => quote! { #ruma_common::MilliSecondsSinceUnixEpoch },
        "room_id" => quote! { #ruma_common::RoomId },
        "event_id" => quote! { #ruma_common::EventId },
        "sender" => quote! { #ruma_common::UserId },
        "state_key" => quote! { ::std::primitive::str },
        "unsigned" => {
            if var.is_redacted() {
                quote! { #ruma_common::events::RedactedUnsigned }
            } else {
                quote! { #ruma_common::events::MessageLikeUnsigned }
            }
        }
        _ => panic!("the `ruma_macros::event_enum::EVENT_FIELD` const was changed"),
    }
}

pub(crate) struct EventEnumVariant {
    pub attrs: Vec<Attribute>,
    pub ident: Ident,
}

impl EventEnumVariant {
    pub(crate) fn to_tokens<T>(&self, prefix: Option<T>, with_attrs: bool) -> TokenStream
    where
        T: ToTokens,
    {
        let mut tokens = TokenStream::new();
        if with_attrs {
            for attr in &self.attrs {
                attr.to_tokens(&mut tokens);
            }
        }
        if let Some(p) = prefix {
            tokens.extend(quote! { #p :: })
        }
        self.ident.to_tokens(&mut tokens);

        tokens
    }

    pub(crate) fn decl(&self) -> TokenStream {
        self.to_tokens::<TokenStream>(None, true)
    }

    pub(crate) fn match_arm(&self, prefix: impl ToTokens) -> TokenStream {
        self.to_tokens(Some(prefix), true)
    }

    pub(crate) fn ctor(&self, prefix: impl ToTokens) -> TokenStream {
        self.to_tokens(Some(prefix), false)
    }
}

impl EventEnumEntry {
    pub(crate) fn has_type_fragment(&self) -> bool {
        self.ev_type.value().ends_with(".*")
    }

    pub(crate) fn to_variant(&self) -> syn::Result<EventEnumVariant> {
        let attrs = self.attrs.clone();
        let ident = m_prefix_name_to_type_name(&self.ev_type)?;

        Ok(EventEnumVariant { attrs, ident })
    }
}

pub(crate) fn expand_from_impls_derived(input: DeriveInput) -> TokenStream {
    let variants = match &input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => panic!("this derive macro only works with enums"),
    };

    let from_impls = variants.iter().map(|variant| match &variant.fields {
        syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            let inner_struct = &fields.unnamed.first().unwrap().ty;
            let var_ident = &variant.ident;
            let id = &input.ident;
            quote! {
                #[automatically_derived]
                impl ::std::convert::From<#inner_struct> for #id {
                    fn from(c: #inner_struct) -> Self {
                        Self::#var_ident(c)
                    }
                }
            }
        }
        _ => {
            panic!("this derive macro only works with enum variants with a single unnamed field")
        }
    });

    quote! {
        #( #from_impls )*
    }
}
