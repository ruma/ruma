//! Implementation of event enum and event content enum macros.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Attribute, Ident, LitStr};

use crate::event_parse::{EventEnumDecl, EventEnumEntry, EventKind, EventKindVariation};

fn is_non_stripped_room_event(kind: &EventKind, var: &EventKindVariation) -> bool {
    matches!(kind, EventKind::Message | EventKind::State)
        && matches!(
            var,
            EventKindVariation::Full
                | EventKindVariation::Sync
                | EventKindVariation::Redacted
                | EventKindVariation::RedactedSync
        )
}

fn has_prev_content_field(kind: &EventKind, var: &EventKindVariation) -> bool {
    matches!(kind, EventKind::State)
        && matches!(var, EventKindVariation::Full | EventKindVariation::Sync)
}

type EventKindFn = fn(&EventKind, &EventKindVariation) -> bool;

/// This const is used to generate the accessor methods for the `Any*Event` enums.
///
/// DO NOT alter the field names unless the structs in `ruma_events::event_kinds` have changed.
const EVENT_FIELDS: &[(&str, EventKindFn)] = &[
    ("origin_server_ts", is_non_stripped_room_event),
    ("room_id", |kind, var| {
        matches!(kind, EventKind::Message | EventKind::State | EventKind::Ephemeral)
            && matches!(var, EventKindVariation::Full | EventKindVariation::Redacted)
    }),
    ("event_id", is_non_stripped_room_event),
    ("sender", |kind, &var| {
        matches!(kind, EventKind::Message | EventKind::State | EventKind::ToDevice)
            && var != EventKindVariation::Initial
    }),
    ("state_key", |kind, _| matches!(kind, EventKind::State)),
    ("unsigned", is_non_stripped_room_event),
];

/// Create a content enum from `EventEnumInput`.
pub fn expand_event_enum(input: &EventEnumDecl) -> syn::Result<TokenStream> {
    let ruma_events = crate::import_ruma_events();

    let name = &input.name;
    let attrs = &input.attrs;
    let events: Vec<_> = input.events.iter().map(|entry| entry.ev_type.clone()).collect();
    let variants: Vec<_> =
        input.events.iter().map(EventEnumEntry::to_variant).collect::<syn::Result<_>>()?;

    let event_enum = expand_any_with_deser(
        name,
        &events,
        attrs,
        &variants,
        &EventKindVariation::Full,
        &ruma_events,
    );

    let sync_event_enum = expand_any_with_deser(
        name,
        &events,
        attrs,
        &variants,
        &EventKindVariation::Sync,
        &ruma_events,
    );

    let stripped_event_enum = expand_any_with_deser(
        name,
        &events,
        attrs,
        &variants,
        &EventKindVariation::Stripped,
        &ruma_events,
    );

    let initial_event_enum = expand_any_with_deser(
        name,
        &events,
        attrs,
        &variants,
        &EventKindVariation::Initial,
        &ruma_events,
    );

    let redacted_event_enums = expand_any_redacted(name, &events, attrs, &variants, &ruma_events);

    let event_content_enum = expand_content_enum(name, &events, attrs, &variants, &ruma_events);

    Ok(quote! {
        #event_enum

        #sync_event_enum

        #stripped_event_enum

        #initial_event_enum

        #redacted_event_enums

        #event_content_enum
    })
}

fn expand_any_with_deser(
    kind: &EventKind,
    events: &[LitStr],
    attrs: &[Attribute],
    variants: &[EventEnumVariant],
    var: &EventKindVariation,
    ruma_events: &TokenStream,
) -> Option<TokenStream> {
    let serde = quote! { #ruma_events::exports::serde };
    let serde_json = quote! { #ruma_events::exports::serde_json };

    // If the event cannot be generated this bails out returning None which is rendered the same
    // as an empty `TokenStream`. This is effectively the check if the given input generates
    // a valid event enum.
    let (event_struct, ident) = generate_event_idents(kind, var)?;

    let content: Vec<_> =
        events.iter().map(|event| to_event_path(event, &event_struct, ruma_events)).collect();

    let variant_decls = variants.iter().map(|v| v.decl());
    let self_variants = variants.iter().map(|v| v.ctor(quote!(Self)));

    let (custom_variant, custom_deserialize) =
        generate_custom_variant(&event_struct, var, ruma_events);

    let any_enum = quote! {
        #( #attrs )*
        #[derive(Clone, Debug, #serde::Serialize)]
        #[serde(untagged)]
        #[allow(clippy::large_enum_variant)]
        #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
        pub enum #ident {
            #(
                #[doc = #events]
                #variant_decls(#content),
            )*
            #custom_variant
        }
    };

    let variant_attrs = variants.iter().map(|v| {
        let attrs = &v.attrs;
        quote! { #(#attrs)* }
    });

    let event_deserialize_impl = quote! {
        impl<'de> #serde::de::Deserialize<'de> for #ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: #serde::de::Deserializer<'de>,
            {
                use #serde::de::Error as _;

                let json = Box::<#serde_json::value::RawValue>::deserialize(deserializer)?;
                let #ruma_events::EventDeHelper { ev_type, .. } =
                    #ruma_events::from_raw_json_value(&json)?;

                match ev_type.as_str() {
                    #(
                        #variant_attrs #events => {
                            let event = #serde_json::from_str::<#content>(json.get())
                                .map_err(D::Error::custom)?;
                            Ok(#self_variants(event))
                        },
                    )*
                    #custom_deserialize
                }
            }
        }
    };

    let event_enum_to_from_sync = expand_conversion_impl(kind, var, variants, ruma_events);

    let redacted_enum = expand_redacted_enum(kind, var, ruma_events);

    let field_accessor_impl = accessor_methods(kind, var, variants, ruma_events);

    let redact_impl = expand_redact(&ident, kind, var, variants, ruma_events);

    Some(quote! {
        #any_enum

        #event_enum_to_from_sync

        #field_accessor_impl

        #redact_impl

        #event_deserialize_impl

        #redacted_enum
    })
}

fn expand_conversion_impl(
    kind: &EventKind,
    var: &EventKindVariation,
    variants: &[EventEnumVariant],
    ruma_events: &TokenStream,
) -> Option<TokenStream> {
    let ruma_identifiers = quote! { #ruma_events::exports::ruma_identifiers };

    let ident = kind.to_event_enum_ident(var)?;
    let variants: Vec<_> = variants
        .iter()
        .filter(|v| {
            // We filter this variant out only for non redacted events.
            // The type of the struct held in the enum variant is different in this case
            // so we construct the variant manually.
            !(v.ident == "RoomRedaction"
                && matches!(var, EventKindVariation::Full | EventKindVariation::Sync))
        })
        .collect();

    match var {
        EventKindVariation::Full | EventKindVariation::Redacted => {
            // the opposite event variation full -> sync, redacted -> redacted sync
            let variation = if var == &EventKindVariation::Full {
                EventKindVariation::Sync
            } else {
                EventKindVariation::RedactedSync
            };

            let sync = kind.to_event_enum_ident(&variation)?;
            let sync_struct = kind.to_event_ident(&variation)?;

            let ident_variants = variants.iter().map(|v| v.match_arm(&ident));
            let self_variants = variants.iter().map(|v| v.ctor(quote!(Self)));

            let redaction =
                (*kind == EventKind::Message && *var == EventKindVariation::Full).then(|| {
                    quote! {
                        #ident::RoomRedaction(event) => Self::RoomRedaction(
                            #ruma_events::room::redaction::SyncRedactionEvent::from(event),
                        ),
                    }
                });

            Some(quote! {
                impl From<#ident> for #sync {
                    fn from(event: #ident) -> Self {
                        match event {
                            #(
                                #ident_variants(event) => {
                                    #self_variants(#ruma_events::#sync_struct::from(event))
                                },
                            )*
                            #redaction
                            #ident::_Custom(event) => {
                                Self::_Custom(#ruma_events::#sync_struct::from(event))
                            },
                        }
                    }
                }
            })
        }
        EventKindVariation::Sync | EventKindVariation::RedactedSync => {
            let variation = if var == &EventKindVariation::Sync {
                EventKindVariation::Full
            } else {
                EventKindVariation::Redacted
            };
            let full = kind.to_event_enum_ident(&variation)?;

            let self_variants = variants.iter().map(|v| v.match_arm(quote!(Self)));
            let full_variants = variants.iter().map(|v| v.ctor(&full));

            let redaction =
                (*kind == EventKind::Message && *var == EventKindVariation::Sync).then(|| {
                    quote! {
                        Self::RoomRedaction(event) => {
                            #full::RoomRedaction(event.into_full_event(room_id))
                        },
                    }
                });

            Some(quote! {
                #[automatically_derived]
                impl #ident {
                    /// Convert this sync event into a full event, one with a room_id field.
                    pub fn into_full_event(
                        self,
                        room_id: #ruma_identifiers::RoomId
                    ) -> #full {
                        match self {
                            #(
                                #self_variants(event) => {
                                    #full_variants(event.into_full_event(room_id))
                                },
                            )*
                            #redaction
                            Self::_Custom(event) => {
                                #full::_Custom(event.into_full_event(room_id))
                            },
                        }
                    }
                }
            })
        }
        _ => None,
    }
}

/// Generates the 3 redacted state enums, 2 redacted message enums,
/// and `Deserialize` implementations.
///
/// No content enums are generated since no part of the API deals with
/// redacted event's content. There are only five state variants that contain content.
fn expand_any_redacted(
    kind: &EventKind,
    events: &[LitStr],
    attrs: &[Attribute],
    variants: &[EventEnumVariant],
    ruma_events: &TokenStream,
) -> TokenStream {
    use EventKindVariation as V;

    if kind.is_state() {
        let full_state =
            expand_any_with_deser(kind, events, attrs, variants, &V::Redacted, ruma_events);
        let sync_state =
            expand_any_with_deser(kind, events, attrs, variants, &V::RedactedSync, ruma_events);
        let stripped_state =
            expand_any_with_deser(kind, events, attrs, variants, &V::RedactedStripped, ruma_events);

        quote! {
            #full_state

            #sync_state

            #stripped_state
        }
    } else if kind.is_message() {
        let full_message =
            expand_any_with_deser(kind, events, attrs, variants, &V::Redacted, ruma_events);
        let sync_message =
            expand_any_with_deser(kind, events, attrs, variants, &V::RedactedSync, ruma_events);

        quote! {
            #full_message

            #sync_message
        }
    } else {
        TokenStream::new()
    }
}

/// Create a content enum from `EventEnumInput`.
fn expand_content_enum(
    kind: &EventKind,
    events: &[LitStr],
    attrs: &[Attribute],
    variants: &[EventEnumVariant],
    ruma_events: &TokenStream,
) -> TokenStream {
    let ruma_identifiers = quote! { #ruma_events::exports::ruma_identifiers };
    let serde = quote! { #ruma_events::exports::serde };
    let serde_json = quote! { #ruma_events::exports::serde_json };

    let ident = kind.to_content_enum();

    let event_type_str = events;

    let content: Vec<_> =
        events.iter().map(|ev| to_event_content_path(kind, ev, None, ruma_events)).collect();

    let variant_decls = variants.iter().map(|v| v.decl()).collect::<Vec<_>>();

    let content_enum = quote! {
        #( #attrs )*
        #[derive(Clone, Debug, #serde::Serialize)]
        #[serde(untagged)]
        #[allow(clippy::large_enum_variant)]
        #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
        pub enum #ident {
            #(
                #[doc = #event_type_str]
                #variant_decls(#content),
            )*
            /// Content of an event not defined by the Matrix specification.
            #[doc(hidden)]
            _Custom(#ruma_events::custom::CustomEventContent),
        }
    };

    let variant_attrs = variants.iter().map(|v| {
        let attrs = &v.attrs;
        quote! { #(#attrs)* }
    });
    let variant_arms = variants.iter().map(|v| v.match_arm(quote!(Self))).collect::<Vec<_>>();
    let variant_ctors = variants.iter().map(|v| v.ctor(quote!(Self)));

    let event_content_impl = quote! {
        #[automatically_derived]
        impl #ruma_events::EventContent for #ident {
            fn event_type(&self) -> &::std::primitive::str {
                match self {
                    #( #variant_arms(content) => content.event_type(), )*
                    Self::_Custom(content) => content.event_type(),
                }
            }

            fn from_parts(
                event_type: &::std::primitive::str,
                input: &#serde_json::value::RawValue,
            ) -> #serde_json::Result<Self> {
                match event_type {
                    #(
                        #variant_attrs #event_type_str => {
                            let content = #content::from_parts(event_type, input)?;
                            ::std::result::Result::Ok(#variant_ctors(content))
                        },
                    )*
                    ev_type => {
                        let content =
                            #ruma_events::custom::CustomEventContent::from_parts(ev_type, input)?;
                        ::std::result::Result::Ok(Self::_Custom(content))
                    },
                }
            }
        }
    };

    let marker_trait_impl = marker_trait(kind, ruma_events);

    let redacted_content_enum = if kind.is_state() || kind.is_message() {
        let redacted_ident = kind.to_redacted_content_enum();
        let redaction_variants = variants.iter().map(|v| v.ctor(&redacted_ident));
        let redacted_content: Vec<_> = events
            .iter()
            .map(|ev| to_event_content_path(kind, ev, Some("Redacted"), ruma_events))
            .collect();

        quote! {
            #( #attrs )*
            #[derive(Clone, Debug, #serde::Serialize)]
            #[serde(untagged)]
            #[allow(clippy::large_enum_variant)]
            #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
            pub enum #redacted_ident {
                #(
                    #[doc = #event_type_str]
                    #variant_decls(#redacted_content),
                )*
                /// Content of a redacted event not defined by the Matrix specification.
                #[doc(hidden)]
                _Custom(#ruma_events::custom::RedactedCustomEventContent),
            }

            impl #ruma_events::RedactContent for #ident {
                type Redacted = #redacted_ident;

                /// Redacts `Self` given a `RoomVersionId`.
                fn redact(
                    self,
                    version: &#ruma_identifiers::RoomVersionId,
                ) -> #redacted_ident {
                    match self {
                        #(
                            #variant_arms(content) => {
                                #redaction_variants(
                                    #ruma_events::RedactContent::redact(content, version)
                                )
                            },
                        )*
                        Self::_Custom(content) => {
                            #redacted_ident::_Custom(
                                #ruma_events::RedactContent::redact(content, version)
                            )
                        },
                    }
                }
            }
        }
    } else {
        TokenStream::new()
    };

    quote! {
        #content_enum

        #event_content_impl

        #marker_trait_impl

        #redacted_content_enum
    }
}

fn expand_redact(
    ident: &Ident,
    kind: &EventKind,
    var: &EventKindVariation,
    variants: &[EventEnumVariant],
    ruma_events: &TokenStream,
) -> Option<TokenStream> {
    let ruma_identifiers = quote! { #ruma_events::exports::ruma_identifiers };

    if let EventKindVariation::Full | EventKindVariation::Sync | EventKindVariation::Stripped = var
    {
        let (redacted_type, redacted_enum) = match var {
            EventKindVariation::Full => {
                let struct_id = kind.to_event_ident(&EventKindVariation::Redacted)?;
                (
                    quote! { #ruma_events::#struct_id },
                    kind.to_event_enum_ident(&EventKindVariation::Redacted)?,
                )
            }
            EventKindVariation::Sync => {
                let struct_id = kind.to_event_ident(&EventKindVariation::RedactedSync)?;
                (
                    quote! { #ruma_events::#struct_id },
                    kind.to_event_enum_ident(&EventKindVariation::RedactedSync)?,
                )
            }
            EventKindVariation::Stripped => {
                let struct_id = kind.to_event_ident(&EventKindVariation::RedactedStripped)?;
                (
                    quote! { #ruma_events::#struct_id },
                    kind.to_event_enum_ident(&EventKindVariation::RedactedStripped)?,
                )
            }
            _ => return None,
        };

        let self_variants = variants.iter().map(|v| v.match_arm(quote!(Self)));
        let redaction_variants = variants.iter().map(|v| v.ctor(&redacted_enum));

        let fields = EVENT_FIELDS.iter().map(|(name, has_field)| {
            generate_redacted_fields(name, kind, var, *has_field, ruma_events)
        });

        let fields = quote! { #( #fields )* };

        Some(quote! {
            #[automatically_derived]
            impl #ruma_events::Redact for #ident {
                type Redacted = #redacted_enum;

                fn redact(
                    self,
                    redaction: #ruma_events::room::redaction::SyncRedactionEvent,
                    version: &#ruma_identifiers::RoomVersionId,
                ) -> #redacted_enum {
                    match self {
                        #(
                            #self_variants(event) => {
                                let content = #ruma_events::RedactContent::redact(event.content, version);
                                #redaction_variants(#redacted_type {
                                    content,
                                    #fields
                                })
                            }
                        )*
                        Self::_Custom(event) => {
                            let content = #ruma_events::RedactContent::redact(event.content, version);
                            #redacted_enum::_Custom(#redacted_type {
                                content,
                                #fields
                            })
                        }
                    }
                }
            }
        })
    } else {
        None
    }
}

fn expand_redacted_enum(
    kind: &EventKind,
    var: &EventKindVariation,
    ruma_events: &TokenStream,
) -> Option<TokenStream> {
    let serde = quote! { #ruma_events::exports::serde };
    let serde_json = quote! { #ruma_events::exports::serde_json };

    if let EventKind::State | EventKind::Message = kind {
        let ident = format_ident!("AnyPossiblyRedacted{}", kind.to_event_ident(var)?);
        let (regular_enum_ident, redacted_enum_ident) = inner_enum_idents(kind, var)?;

        Some(quote! {
            /// An enum that holds either regular un-redacted events or redacted events.
            #[derive(Clone, Debug, #serde::Serialize)]
            #[serde(untagged)]
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
                    let #ruma_events::EventDeHelper { unsigned, .. } =
                        #ruma_events::from_raw_json_value(&json)?;

                    Ok(match unsigned {
                        Some(unsigned) if unsigned.redacted_because.is_some() => {
                            Self::Redacted(#ruma_events::from_raw_json_value(&json)?)
                        }
                        _ => Self::Regular(#ruma_events::from_raw_json_value(&json)?),
                    })
                }
            }
        })
    } else {
        None
    }
}

fn generate_event_idents(kind: &EventKind, var: &EventKindVariation) -> Option<(Ident, Ident)> {
    kind.to_event_ident(var).zip(kind.to_event_enum_ident(var))
}

fn generate_redacted_fields(
    name: &str,
    kind: &EventKind,
    var: &EventKindVariation,
    is_event_kind: EventKindFn,
    ruma_events: &TokenStream,
) -> Option<TokenStream> {
    is_event_kind(kind, var).then(|| {
        let name = Ident::new(name, Span::call_site());

        if name == "unsigned" {
            quote! {
                unsigned: #ruma_events::RedactedUnsigned::new_because(::std::boxed::Box::new(redaction)),
            }
        } else {
            quote! {
                #name: event.#name,
            }
        }
    })
}

fn generate_custom_variant(
    event_struct: &Ident,
    var: &EventKindVariation,
    ruma_events: &TokenStream,
) -> (TokenStream, TokenStream) {
    use EventKindVariation as V;

    let serde_json = quote! { #ruma_events::exports::serde_json };

    if matches!(var, V::Redacted | V::RedactedSync | V::RedactedStripped) {
        (
            quote! {
                /// A redacted event not defined by the Matrix specification
                #[doc(hidden)]
                _Custom(
                    #ruma_events::#event_struct<#ruma_events::custom::RedactedCustomEventContent>,
                ),
            },
            quote! {
                event => {
                    let event = #serde_json::from_str::<#ruma_events::#event_struct<
                        #ruma_events::custom::RedactedCustomEventContent,
                    >>(json.get())
                    .map_err(D::Error::custom)?;

                    Ok(Self::_Custom(event))
                },
            },
        )
    } else {
        (
            quote! {
                /// An event not defined by the Matrix specification
                #[doc(hidden)]
                _Custom(#ruma_events::#event_struct<#ruma_events::custom::CustomEventContent>),
            },
            quote! {
                event => {
                    let event =
                        #serde_json::from_str::<
                            #ruma_events::#event_struct<#ruma_events::custom::CustomEventContent>
                        >(json.get())
                        .map_err(D::Error::custom)?;

                    Ok(Self::_Custom(event))
                },
            },
        )
    }
}

fn marker_trait(kind: &EventKind, ruma_events: &TokenStream) -> TokenStream {
    let marker_trait = match kind {
        EventKind::State => quote! { StateEventContent },
        EventKind::Message => quote! { MessageEventContent },
        EventKind::Ephemeral => quote! { EphemeralRoomEventContent },
        EventKind::GlobalAccountData => quote! { GlobalAccountDataEventContent },
        EventKind::RoomAccountData => quote! { RoomAccountDataEventContent },
        EventKind::ToDevice => quote! { ToDeviceEventContent },
        _ => return TokenStream::new(),
    };

    let ident = kind.to_content_enum();
    quote! {
        #[automatically_derived]
        impl #ruma_events::#marker_trait for #ident {}
    }
}

fn accessor_methods(
    kind: &EventKind,
    var: &EventKindVariation,
    variants: &[EventEnumVariant],
    ruma_events: &TokenStream,
) -> Option<TokenStream> {
    use EventKindVariation as V;

    let ident = kind.to_event_enum_ident(var)?;

    // matching `EventKindVariation`s
    if let V::Redacted | V::RedactedSync | V::RedactedStripped = var {
        return redacted_accessor_methods(kind, var, variants, ruma_events);
    }

    let methods = EVENT_FIELDS.iter().map(|(name, has_field)| {
        generate_accessor(name, kind, var, *has_field, variants, ruma_events)
    });

    let content_enum = kind.to_content_enum();

    let self_variants: Vec<_> = variants.iter().map(|v| v.match_arm(quote!(Self))).collect();
    let content_variants: Vec<_> = variants.iter().map(|v| v.ctor(&content_enum)).collect();

    let event_type = quote! {
        /// Returns the `type` of this event.
        pub fn event_type(&self) -> &::std::primitive::str {
            match self {
                #( #self_variants(event) =>
                    #ruma_events::EventContent::event_type(&event.content), )*
                Self::_Custom(event) =>
                    #ruma_events::EventContent::event_type(&event.content),
            }
        }
    };

    let content = quote! {
        /// Returns the any content enum for this event.
        pub fn content(&self) -> #content_enum {
            match self {
                #( #self_variants(event) => #content_variants(event.content.clone()), )*
                Self::_Custom(event) => #content_enum::_Custom(event.content.clone()),
            }
        }
    };

    let prev_content = has_prev_content_field(kind, var).then(|| {
        quote! {
            /// Returns the any content enum for this events prev_content.
            pub fn prev_content(&self) -> Option<#content_enum> {
                match self {
                    #(
                        #self_variants(event) => {
                            event.prev_content.as_ref().map(|c| #content_variants(c.clone()))
                        },
                    )*
                    Self::_Custom(event) => {
                        event.prev_content.as_ref().map(|c| #content_enum::_Custom(c.clone()))
                    },
                }
            }
        }
    });

    Some(quote! {
        #[automatically_derived]
        impl #ident {
            #content

            #event_type

            #prev_content

            #( #methods )*
        }
    })
}

fn inner_enum_idents(kind: &EventKind, var: &EventKindVariation) -> Option<(Ident, Ident)> {
    Some(match var {
        EventKindVariation::Full => (
            kind.to_event_enum_ident(var)?,
            kind.to_event_enum_ident(&EventKindVariation::Redacted)?,
        ),
        EventKindVariation::Sync => (
            kind.to_event_enum_ident(var)?,
            kind.to_event_enum_ident(&EventKindVariation::RedactedSync)?,
        ),
        EventKindVariation::Stripped => (
            kind.to_event_enum_ident(var)?,
            kind.to_event_enum_ident(&EventKindVariation::RedactedStripped)?,
        ),
        _ => return None,
    })
}

/// Redacted events do NOT generate `content` or `prev_content` methods like
/// un-redacted events; otherwise, they are the same.
fn redacted_accessor_methods(
    kind: &EventKind,
    var: &EventKindVariation,
    variants: &[EventEnumVariant],
    ruma_events: &TokenStream,
) -> Option<TokenStream> {
    let ident = kind.to_event_enum_ident(var)?;
    let methods = EVENT_FIELDS.iter().map(|(name, has_field)| {
        generate_accessor(name, kind, var, *has_field, variants, ruma_events)
    });

    Some(quote! {
        #[automatically_derived]
        impl #ident {
            #( #methods )*
        }
    })
}

fn to_event_path(name: &LitStr, struct_name: &Ident, ruma_events: &TokenStream) -> TokenStream {
    let span = name.span();
    let name = name.value();

    // There is no need to give a good compiler error as `to_camel_case` is called first.
    assert_eq!(&name[..2], "m.");

    let path: Vec<_> = name[2..].split('.').collect();

    let event_str = path.last().unwrap();
    let event: String = event_str
        .split('_')
        .map(|s| s.chars().next().unwrap().to_uppercase().to_string() + &s[1..])
        .collect();

    let path = path.iter().map(|s| Ident::new(s, span));

    match struct_name.to_string().as_str() {
        "MessageEvent" | "SyncMessageEvent" if name == "m.room.redaction" => {
            let redaction = if struct_name == "MessageEvent" {
                quote! { RedactionEvent }
            } else {
                quote! { SyncRedactionEvent }
            };
            quote! { #ruma_events::room::redaction::#redaction }
        }
        "SyncStateEvent"
        | "StrippedStateEvent"
        | "InitialStateEvent"
        | "SyncMessageEvent"
        | "SyncEphemeralRoomEvent" => {
            let content = format_ident!("{}EventContent", event);
            quote! { #ruma_events::#struct_name<#ruma_events::#( #path )::*::#content> }
        }
        "ToDeviceEvent" => {
            let content = format_ident!("{}ToDeviceEventContent", event);
            quote! { #ruma_events::#struct_name<#ruma_events::#( #path )::*::#content> }
        }
        struct_str if struct_str.contains("Redacted") => {
            let content = format_ident!("Redacted{}EventContent", event);
            quote! { #ruma_events::#struct_name<#ruma_events::#( #path )::*::#content> }
        }
        _ => {
            let event_name = format_ident!("{}Event", event);
            quote! { #ruma_events::#( #path )::*::#event_name }
        }
    }
}

fn to_event_content_path(
    kind: &EventKind,
    name: &LitStr,
    prefix: Option<&str>,
    ruma_events: &TokenStream,
) -> TokenStream {
    let span = name.span();
    let name = name.value();

    // There is no need to give a good compiler error as `to_camel_case` is called first.
    assert_eq!(&name[..2], "m.");

    let path: Vec<_> = name[2..].split('.').collect();

    let event_str = path.last().unwrap();
    let event: String = event_str
        .split('_')
        .map(|s| s.chars().next().unwrap().to_uppercase().to_string() + &s[1..])
        .collect();

    let content_str = match kind {
        EventKind::ToDevice => {
            format_ident!("{}{}ToDeviceEventContent", prefix.unwrap_or(""), event)
        }
        _ => format_ident!("{}{}EventContent", prefix.unwrap_or(""), event),
    };

    let path = path.iter().map(|s| Ident::new(s, span));

    quote! {
        #ruma_events::#( #path )::*::#content_str
    }
}

/// Splits the given `event_type` string on `.` and `_` removing the `m.room.` then
/// camel casing to give the `Event` struct name.
fn to_camel_case(name: &LitStr) -> syn::Result<Ident> {
    let span = name.span();
    let name = name.value();

    if &name[..2] != "m." {
        return Err(syn::Error::new(
            span,
            format!("well-known matrix events have to start with `m.` found `{}`", name),
        ));
    }

    let s: String = name[2..]
        .split(&['.', '_'] as &[char])
        .map(|s| s.chars().next().unwrap().to_uppercase().to_string() + &s[1..])
        .collect();

    Ok(Ident::new(&s, span))
}

fn generate_accessor(
    name: &str,
    kind: &EventKind,
    var: &EventKindVariation,
    is_event_kind: EventKindFn,
    variants: &[EventEnumVariant],
    ruma_events: &TokenStream,
) -> Option<TokenStream> {
    is_event_kind(kind, var).then(|| {
        let docs = format!("Returns this event's {} field.", name);
        let ident = Ident::new(name, Span::call_site());
        let field_type = field_return_type(name, var, ruma_events);
        let variants = variants.iter().map(|v| v.match_arm(quote!(Self)));

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
}

fn field_return_type(
    name: &str,
    var: &EventKindVariation,
    ruma_events: &TokenStream,
) -> TokenStream {
    let ruma_common = quote! { #ruma_events::exports::ruma_common };
    let ruma_identifiers = quote! { #ruma_events::exports::ruma_identifiers };

    match name {
        "origin_server_ts" => quote! { #ruma_common::MilliSecondsSinceUnixEpoch },
        "room_id" => quote! { #ruma_identifiers::RoomId },
        "event_id" => quote! { #ruma_identifiers::EventId },
        "sender" => quote! { #ruma_identifiers::UserId },
        "state_key" => quote! { ::std::primitive::str },
        "unsigned" => {
            if var.is_redacted() {
                quote! { #ruma_events::RedactedUnsigned }
            } else {
                quote! { #ruma_events::Unsigned }
            }
        }
        _ => panic!("the `ruma_events_macros::event_enum::EVENT_FIELD` const was changed"),
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
    pub(crate) fn to_variant(&self) -> syn::Result<EventEnumVariant> {
        let attrs = self.attrs.clone();
        let ident = to_camel_case(&self.ev_type)?;
        Ok(EventEnumVariant { attrs, ident })
    }
}
