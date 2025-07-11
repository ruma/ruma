//! Implementation of the `event_enum!` macro.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{Attribute, Ident};

mod content;
mod event_type;
mod parse;

pub use self::parse::EventEnumInput;
use self::{
    content::{expand_content_enum, expand_full_content_enum},
    event_type::expand_event_type_enums,
    parse::{EventEnumDecl, EventEnumEntry, EventEnumVariant, EventEnumVariation},
};
use super::enums::{EventContentVariation, EventKind};
use crate::import_ruma_common;

pub(crate) fn is_non_stripped_room_event(kind: EventKind, var: EventEnumVariation) -> bool {
    matches!(kind, EventKind::MessageLike | EventKind::State)
        && matches!(var, EventEnumVariation::None | EventEnumVariation::Sync)
}

type EventKindFn = fn(EventKind, EventEnumVariation) -> bool;

/// This const is used to generate the accessor methods for the `Any*Event` enums.
///
/// DO NOT alter the field names unless the structs in `ruma_events::event_kinds` have
/// changed.
const EVENT_FIELDS: &[(&str, EventKindFn)] = &[
    ("origin_server_ts", is_non_stripped_room_event),
    ("room_id", |kind, var| {
        matches!(kind, EventKind::MessageLike | EventKind::State | EventKind::EphemeralRoom)
            && matches!(var, EventEnumVariation::None)
    }),
    ("event_id", is_non_stripped_room_event),
    ("sender", |kind, var| {
        matches!(kind, EventKind::MessageLike | EventKind::State | EventKind::ToDevice)
            && var != EventEnumVariation::Initial
    }),
];

/// `event_enum!` macro code generation.
pub fn expand_event_enum(input: EventEnumInput) -> syn::Result<TokenStream> {
    let ruma_common = import_ruma_common();

    let enums = input
        .enums
        .iter()
        .map(|e| expand_event_kind_enums(e).unwrap_or_else(syn::Error::into_compile_error))
        .collect::<TokenStream>();

    let event_types =
        expand_event_type_enums(input, ruma_common).unwrap_or_else(syn::Error::into_compile_error);

    Ok(quote! {
        #enums
        #event_types
    })
}

/// Generate `Any*Event(Content)` enums from `EventEnumDecl`.
pub fn expand_event_kind_enums(input: &EventEnumDecl) -> syn::Result<TokenStream> {
    use EventEnumVariation as V;

    let ruma_events = crate::import_ruma_events();

    let mut res = TokenStream::new();

    let kind = input.kind;
    let attrs = &input.attrs;
    let docs: Vec<_> = input.events.iter().map(EventEnumEntry::docs).collect::<syn::Result<_>>()?;
    let variants: Vec<_> =
        input.events.iter().map(EventEnumEntry::to_variant).collect::<syn::Result<_>>()?;

    let events = &input.events;
    let docs = &docs;
    let variants = &variants;
    let ruma_events = &ruma_events;

    res.extend(expand_content_enum(kind, events, docs, attrs, variants, ruma_events));

    if !matches!(kind, EventKind::EphemeralRoom) {
        res.extend(
            expand_event_kind_enum(kind, V::None, events, docs, attrs, variants, ruma_events)
                .unwrap_or_else(syn::Error::into_compile_error),
        );
    }

    if matches!(kind, EventKind::MessageLike | EventKind::State) {
        res.extend(
            expand_event_kind_enum(kind, V::Sync, events, docs, attrs, variants, ruma_events)
                .unwrap_or_else(syn::Error::into_compile_error),
        );
        res.extend(
            expand_from_full_event(kind, V::None, variants)
                .unwrap_or_else(syn::Error::into_compile_error),
        );
        res.extend(
            expand_into_full_event(kind, V::Sync, variants, ruma_events)
                .unwrap_or_else(syn::Error::into_compile_error),
        );
    }

    if matches!(kind, EventKind::EphemeralRoom) {
        res.extend(
            expand_event_kind_enum(kind, V::Sync, events, docs, attrs, variants, ruma_events)
                .unwrap_or_else(syn::Error::into_compile_error),
        );
    }

    if matches!(kind, EventKind::State) {
        res.extend(expand_full_content_enum(kind, events, docs, attrs, variants, ruma_events));
        res.extend(
            expand_event_kind_enum(kind, V::Stripped, events, docs, attrs, variants, ruma_events)
                .unwrap_or_else(syn::Error::into_compile_error),
        );
        res.extend(
            expand_event_kind_enum(kind, V::Initial, events, docs, attrs, variants, ruma_events)
                .unwrap_or_else(syn::Error::into_compile_error),
        );
    }

    Ok(res)
}

/// Generate an `Any*Event` enum.
fn expand_event_kind_enum(
    kind: EventKind,
    var: EventEnumVariation,
    events: &[EventEnumEntry],
    docs: &[TokenStream],
    attrs: &[Attribute],
    variants: &[EventEnumVariant],
    ruma_events: &TokenStream,
) -> syn::Result<TokenStream> {
    let event_struct = kind.to_event_ident(var.into())?;
    let ident = kind.to_event_enum_ident(var.into())?;

    let variant_decls = variants.iter().map(|v| v.decl());
    let event_ty: Vec<_> = events.iter().map(|event| event.to_event_path(kind, var)).collect();

    let custom_content_ty = format_ident!("Custom{}Content", kind);

    let deserialize_impl = expand_deserialize_impl(kind, var, events, ruma_events)?;
    let field_accessor_impl =
        expand_accessor_methods(kind, var, variants, &event_struct, ruma_events)?;
    let from_impl = expand_from_impl(&ident, &event_ty, variants);

    Ok(quote! {
        #( #attrs )*
        #[derive(Clone, Debug)]
        #[allow(clippy::large_enum_variant, unused_qualifications)]
        #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
        pub enum #ident {
            #(
                #docs
                #variant_decls(#event_ty),
            )*
            /// An event not defined by the Matrix specification
            #[doc(hidden)]
            _Custom(
                #ruma_events::#event_struct<
                    #ruma_events::_custom::#custom_content_ty
                >,
            ),
        }

        #deserialize_impl
        #field_accessor_impl
        #from_impl
    })
}

/// Implement `Deserialize` for an enum.
fn expand_deserialize_impl(
    kind: EventKind,
    var: EventEnumVariation,
    events: &[EventEnumEntry],
    ruma_events: &TokenStream,
) -> syn::Result<TokenStream> {
    let ruma_common = quote! { #ruma_events::exports::ruma_common };
    let serde = quote! { #ruma_events::exports::serde };
    let serde_json = quote! { #ruma_events::exports::serde_json };

    let ident = kind.to_event_enum_ident(var.into())?;

    let match_arms: TokenStream = events
        .iter()
        .map(|event| {
            let variant = event.to_variant()?;
            let variant_attrs = {
                let attrs = &variant.attrs;
                quote! { #(#attrs)* }
            };
            let self_variant = variant.ctor(quote! { Self });
            let content = event.to_event_path(kind, var);
            let ev_types = event.aliases.iter().chain([&event.ev_type]).map(|ev_type| {
                if event.has_type_fragment() {
                    let ev_type = ev_type.value();
                    let prefix = ev_type
                        .strip_suffix('*')
                        .expect("event type with type fragment must end with *");
                    quote! { t if t.starts_with(#prefix) }
                } else {
                    quote! { #ev_type }
                }
            });

            Ok(quote! {
                #variant_attrs #(#ev_types)|* => {
                    let event = #serde_json::from_str::<#content>(json.get())
                        .map_err(D::Error::custom)?;
                    Ok(#self_variant(event))
                },
            })
        })
        .collect::<syn::Result<_>>()?;

    Ok(quote! {
        #[allow(unused_qualifications)]
        impl<'de> #serde::de::Deserialize<'de> for #ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: #serde::de::Deserializer<'de>,
            {
                use #serde::de::Error as _;

                let json = Box::<#serde_json::value::RawValue>::deserialize(deserializer)?;
                let #ruma_events::EventTypeDeHelper { ev_type, .. } =
                    #ruma_common::serde::from_raw_json_value(&json)?;

                match &*ev_type {
                    #match_arms
                    _ => {
                        let event = #serde_json::from_str(json.get()).map_err(D::Error::custom)?;
                        Ok(Self::_Custom(event))
                    },
                }
            }
        }
    })
}

/// Implement `From<{event_struct}>` for all the variants of an enum.
fn expand_from_impl(
    ty: &Ident,
    event_ty: &[TokenStream],
    variants: &[EventEnumVariant],
) -> TokenStream {
    let from_impls = event_ty.iter().zip(variants).map(|(event_ty, variant)| {
        let ident = &variant.ident;
        let attrs = &variant.attrs;

        quote! {
            #[allow(unused_qualifications)]
            #[automatically_derived]
            #(#attrs)*
            impl ::std::convert::From<#event_ty> for #ty {
                fn from(c: #event_ty) -> Self {
                    Self::#ident(c)
                }
            }
        }
    });

    quote! { #( #from_impls )* }
}

/// Implement `From<Any*Event>` for an `AnySync*Event` enum.
fn expand_from_full_event(
    kind: EventKind,
    var: EventEnumVariation,
    variants: &[EventEnumVariant],
) -> syn::Result<TokenStream> {
    let ident = kind.to_event_enum_ident(var.into())?;
    let sync = kind.to_event_enum_ident(var.to_sync().into())?;

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

/// Implement `.into_full_event()` for an `AnySync*Event` enum.
fn expand_into_full_event(
    kind: EventKind,
    var: EventEnumVariation,
    variants: &[EventEnumVariant],
    ruma_events: &TokenStream,
) -> syn::Result<TokenStream> {
    let ruma_common = quote! { #ruma_events::exports::ruma_common };

    let ident = kind.to_event_enum_ident(var.into())?;
    let full = kind.to_event_enum_ident(var.to_full().into())?;

    let self_variants = variants.iter().map(|v| v.match_arm(quote! { Self }));
    let full_variants = variants.iter().map(|v| v.ctor(&full));

    Ok(quote! {
        #[automatically_derived]
        impl #ident {
            /// Convert this sync event into a full event (one with a `room_id` field).
            pub fn into_full_event(self, room_id: #ruma_common::OwnedRoomId) -> #full {
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

/// Implement accessors for the common fields of an `Any*Event` enum.
fn expand_accessor_methods(
    kind: EventKind,
    var: EventEnumVariation,
    variants: &[EventEnumVariant],
    event_struct: &Ident,
    ruma_events: &TokenStream,
) -> syn::Result<TokenStream> {
    let ruma_common = quote! { #ruma_events::exports::ruma_common };

    let ident = kind.to_event_enum_ident(var.into())?;
    let event_type_enum = format_ident!("{}Type", kind);
    let self_variants: Vec<_> = variants.iter().map(|v| v.match_arm(quote! { Self })).collect();
    let original_event_content_kind_trait_name =
        kind.to_content_kind_trait(EventContentVariation::Original);

    let maybe_redacted =
        kind.is_timeline() && matches!(var, EventEnumVariation::None | EventEnumVariation::Sync);

    let event_type_match_arms = if maybe_redacted {
        quote! {
            #( #self_variants(event) => event.event_type(), )*
            Self::_Custom(event) => event.event_type(),
        }
    } else if var == EventEnumVariation::Stripped {
        let possibly_redacted_event_content_kind_trait_name =
            kind.to_content_kind_trait(EventContentVariation::PossiblyRedacted);
        quote! {
            #( #self_variants(event) =>
                #ruma_events::#possibly_redacted_event_content_kind_trait_name::event_type(&event.content), )*
            Self::_Custom(event) => ::std::convert::From::from(
                #ruma_events::#possibly_redacted_event_content_kind_trait_name::event_type(&event.content),
            ),
        }
    } else {
        quote! {
            #( #self_variants(event) =>
                #ruma_events::#original_event_content_kind_trait_name::event_type(&event.content), )*
            Self::_Custom(event) => ::std::convert::From::from(
                #ruma_events::#original_event_content_kind_trait_name::event_type(&event.content),
            ),
        }
    };

    let content_enum = kind.to_content_enum();
    let content_variants: Vec<_> = variants.iter().map(|v| v.ctor(&content_enum)).collect();
    let content_accessor = if maybe_redacted {
        let mut accessors = quote! {
            /// Returns the content for this event if it is not redacted, or `None` if it is.
            pub fn original_content(&self) -> Option<#content_enum> {
                match self {
                    #(
                        #self_variants(event) => {
                            event.as_original().map(|ev| #content_variants(ev.content.clone()))
                        }
                    )*
                    Self::_Custom(event) => event.as_original().map(|ev| {
                        #content_enum::_Custom {
                            event_type: crate::PrivOwnedStr(
                                ::std::convert::From::from(
                                    ::std::string::ToString::to_string(
                                        &#ruma_events::#original_event_content_kind_trait_name::event_type(
                                            &ev.content,
                                        ),
                                    ),
                                ),
                            ),
                        }
                    }),
                }
            }

            /// Returns whether this event is redacted.
            pub fn is_redacted(&self) -> bool {
                match self {
                    #(
                        #self_variants(event) => {
                            event.as_original().is_none()
                        }
                    )*
                    Self::_Custom(event) => event.as_original().is_none(),
                }
            }
        };

        if kind == EventKind::State {
            let full_content_enum = kind.to_full_content_enum();
            let full_content_variants: Vec<_> =
                variants.iter().map(|v| v.ctor(&full_content_enum)).collect();
            let redacted_event_content_kind_trait_name =
                kind.to_content_kind_trait(EventContentVariation::Redacted);

            accessors = quote! {
                #accessors

                /// Returns the content of this state event.
                pub fn content(&self) -> #full_content_enum {
                    match self {
                        #(
                            #self_variants(event) => match event {
                                #ruma_events::#event_struct::Original(ev) => #full_content_variants(
                                    #ruma_events::FullStateEventContent::Original {
                                        content: ev.content.clone(),
                                        prev_content: ev.unsigned.prev_content.clone()
                                    }
                                ),
                                #ruma_events::#event_struct::Redacted(ev) => #full_content_variants(
                                    #ruma_events::FullStateEventContent::Redacted(
                                        ev.content.clone()
                                    )
                                ),
                            }
                        )*
                        Self::_Custom(event) => match event {
                            #ruma_events::#event_struct::Original(ev) => {
                                #full_content_enum::_Custom {
                                    event_type: crate::PrivOwnedStr(
                                        ::std::string::ToString::to_string(
                                            &#ruma_events::#original_event_content_kind_trait_name::event_type(
                                                &ev.content,
                                            ),
                                        ).into_boxed_str(),
                                    ),
                                    redacted: false,
                                }
                            }
                            #ruma_events::#event_struct::Redacted(ev) => {
                                #full_content_enum::_Custom {
                                    event_type: crate::PrivOwnedStr(
                                        ::std::string::ToString::to_string(
                                            &#ruma_events::#redacted_event_content_kind_trait_name::event_type(
                                                &ev.content,
                                            ),
                                        ).into_boxed_str(),
                                    ),
                                    redacted: true,
                                }
                            }
                        },
                    }
                }
            };
        }

        accessors
    } else if var == EventEnumVariation::Stripped {
        // There is no content enum for possibly-redacted content types (yet)
        TokenStream::new()
    } else {
        quote! {
            /// Returns the content for this event.
            pub fn content(&self) -> #content_enum {
                match self {
                    #( #self_variants(event) => #content_variants(event.content.clone()), )*
                    Self::_Custom(event) => #content_enum::_Custom {
                        event_type: crate::PrivOwnedStr(
                            ::std::convert::From::from(
                                ::std::string::ToString::to_string(
                                    &#ruma_events::#original_event_content_kind_trait_name::event_type(&event.content)
                                )
                            ),
                        ),
                    },
                }
            }
        }
    };

    let methods = EVENT_FIELDS.iter().map(|(name, has_field)| {
        has_field(kind, var).then(|| {
            let docs = format!("Returns this event's `{name}` field.");
            let ident = Ident::new(name, Span::call_site());
            let field_type = field_return_type(name, ruma_events);
            let variants = variants.iter().map(|v| v.match_arm(quote! { Self }));
            let call_parens = maybe_redacted.then(|| quote! { () });
            let ampersand = (*name != "origin_server_ts").then(|| quote! { & });

            quote! {
                #[doc = #docs]
                pub fn #ident(&self) -> #field_type {
                    match self {
                        #( #variants(event) => #ampersand event.#ident #call_parens, )*
                        Self::_Custom(event) => #ampersand event.#ident #call_parens,
                    }
                }
            }
        })
    });

    let state_key_accessor = (kind == EventKind::State).then(|| {
        let variants = variants.iter().map(|v| v.match_arm(quote! { Self }));
        let call_parens = maybe_redacted.then(|| quote! { () });

        quote! {
            /// Returns this event's `state_key` field.
            pub fn state_key(&self) -> &::std::primitive::str {
                match self {
                    #( #variants(event) => &event.state_key #call_parens .as_ref(), )*
                    Self::_Custom(event) => &event.state_key #call_parens .as_ref(),
                }
            }
        }
    });

    let relations_accessor = (kind == EventKind::MessageLike).then(|| {
        let variants = variants.iter().map(|v| v.match_arm(quote! { Self }));

        quote! {
            /// Returns this event's `relations` from inside `unsigned`.
            pub fn relations(
                &self,
            ) -> #ruma_events::BundledMessageLikeRelations<AnySyncMessageLikeEvent> {
                match self {
                    #(
                        #variants(event) => event.as_original().map_or_else(
                            ::std::default::Default::default,
                            |ev| ev.unsigned.relations.clone().map_replace(|r| {
                                ::std::convert::From::from(r.into_maybe_redacted())
                            }),
                        ),
                    )*
                    Self::_Custom(event) => event.as_original().map_or_else(
                        ::std::default::Default::default,
                        |ev| ev.unsigned.relations.clone().map_replace(|r| {
                            AnySyncMessageLikeEvent::_Custom(r.into_maybe_redacted())
                        }),
                    ),
                }
            }
        }
    });

    let maybe_redacted_accessors = maybe_redacted.then(|| {
        let variants = variants.iter().map(|v| v.match_arm(quote! { Self }));

        quote! {
            /// Returns this event's `transaction_id` from inside `unsigned`, if there is one.
            pub fn transaction_id(&self) -> Option<&#ruma_common::TransactionId> {
                match self {
                    #(
                        #variants(event) => {
                            event.as_original().and_then(|ev| ev.unsigned.transaction_id.as_deref())
                        }
                    )*
                    Self::_Custom(event) => {
                        event.as_original().and_then(|ev| ev.unsigned.transaction_id.as_deref())
                    }
                }
            }
        }
    });

    Ok(quote! {
        #[automatically_derived]
        impl #ident {
            /// Returns the `type` of this event.
            pub fn event_type(&self) -> #ruma_events::#event_type_enum {
                match self { #event_type_match_arms }
            }

            #content_accessor
            #( #methods )*
            #relations_accessor
            #state_key_accessor
            #maybe_redacted_accessors
        }
    })
}

/// Get the return type of the given field.
fn field_return_type(name: &str, ruma_events: &TokenStream) -> TokenStream {
    let ruma_common = quote! { #ruma_events::exports::ruma_common };
    match name {
        "origin_server_ts" => quote! { #ruma_common::MilliSecondsSinceUnixEpoch },
        "room_id" => quote! { &#ruma_common::RoomId },
        "event_id" => quote! { &#ruma_common::EventId },
        "sender" => quote! { &#ruma_common::UserId },
        _ => panic!("the `ruma_macros::event_enum::EVENT_FIELDS` const was changed"),
    }
}
