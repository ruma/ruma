use std::ops::Deref;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};

mod content;

use super::{EventEnumData, EventEnumEntry, EventEnumKind, util::expand_json_castable_impl};
use crate::{
    events::common::{CommonEventField, EventContentTraitVariation, EventType, EventVariation},
    util::{RumaEvents, RumaEventsReexport},
};

/// Cache for [`EventEnum`] data that is used in several places.
pub(super) struct EventEnum<'a> {
    /// The data for the enum.
    data: &'a EventEnumData,

    /// The import path for the ruma-events crate.
    ruma_events: &'a RumaEvents,

    /// The attributes of the event entries.
    variant_attrs: Vec<&'a [syn::Attribute]>,

    /// The names of the variants of the entries.
    variants: Vec<&'a syn::Ident>,

    /// The docs for the variants.
    variant_docs: Vec<TokenStream>,

    /// The match arms for the entries' `type` string.
    event_type_string_match_arms: Vec<Vec<TokenStream>>,

    /// The paths to the `*EventContent` types of the entries.
    event_content_types: Vec<syn::Path>,

    /// The name of the enum that contains the content of the events for this kind.
    content_enum: syn::Ident,

    /// The name of the enum that contains the "full" content of the events for this kind.
    full_content_enum: syn::Ident,

    /// The name of the `*EventType` enum for this kind.
    event_type_enum: syn::Ident,
}

impl<'a> EventEnum<'a> {
    /// Construct a new `EventEnum` with the given data and ruma-events import.
    pub(super) fn new(data: &'a EventEnumData, ruma_events: &'a RumaEvents) -> Self {
        // Compute data that is used in several places.
        let variant_attrs = data.events.iter().map(|event| event.attrs.as_slice()).collect();
        let variants = data.events.iter().map(|event| &event.ident).collect();
        let variant_docs = data.events.iter().map(EventEnumEntry::docs).collect();
        let event_content_types =
            data.events.iter().map(|event| event.to_event_content_path(data.kind)).collect();
        let event_type_string_match_arms = data
            .events
            .iter()
            .map(|event| event.types.iter().map(EventType::as_match_arm).collect())
            .collect();

        let kind = data.kind;
        let content_enum = format_ident!("Any{kind}Content");
        let full_content_enum = format_ident!("AnyFull{kind}Content");
        let event_type_enum = kind.to_event_type_enum();

        Self {
            data,
            ruma_events,
            variant_attrs,
            variants,
            variant_docs,
            event_type_string_match_arms,
            event_content_types,
            content_enum,
            full_content_enum,
            event_type_enum,
        }
    }
}

impl EventEnum<'_> {
    /// Generate the `Any*Event(Content)` enums and their implementations.
    pub(super) fn expand(&self) -> syn::Result<TokenStream> {
        let variations = self.kind.event_enum_variations();

        if variations.is_empty() {
            return Err(syn::Error::new(
                Span::call_site(),
                format!("The {:?} kind is not supported", self.kind),
            ));
        }

        // Generate the `Any*EventContent` enum.
        let mut tokens = self.expand_content_enum()?;
        let has_full = variations.contains(&EventVariation::None);

        // Generate the `Any*Event` enums for all the variations.
        for variation in variations {
            tokens.extend(
                EventEnumVariation::new(self, *variation)?
                    .expand_event_kind_enum()
                    .unwrap_or_else(syn::Error::into_compile_error),
            );

            if variation.is_sync() && has_full {
                tokens.extend(
                    self.expand_sync_from_into_full()
                        .unwrap_or_else(syn::Error::into_compile_error),
                );
            }
        }

        // Generate the `AnyFull*EventContent` enum.
        if matches!(self.kind, EventEnumKind::State) {
            tokens.extend(self.expand_full_content_enum());
        }

        Ok(tokens)
    }

    /// Implement `From<Any*Event>` and `.into_full_event()` for an `AnySync*Event` enum.
    fn expand_sync_from_into_full(&self) -> syn::Result<TokenStream> {
        let ruma_common = self.ruma_events.ruma_common();

        let sync = self.kind.to_event_enum_ident(EventVariation::Sync)?;
        let full = self.kind.to_event_enum_ident(EventVariation::None)?;

        let variants = &self.variants;
        let variant_attrs = &self.variant_attrs;

        Ok(quote! {
            #[automatically_derived]
            impl ::std::convert::From<#full> for #sync {
                fn from(event: #full) -> Self {
                    match event {
                        #(
                            #( #variant_attrs )*
                            #full::#variants(event) => {
                                Self::#variants(::std::convert::From::from(event))
                            }
                        )*
                        #full::_Custom(event) => {
                            Self::_Custom(::std::convert::From::from(event))
                        },
                    }
                }
            }

            #[automatically_derived]
            impl #sync {
                /// Convert this sync event into a full event (one with a `room_id` field).
                pub fn into_full_event(self, room_id: #ruma_common::OwnedRoomId) -> #full {
                    match self {
                        #(
                            #( #variant_attrs )*
                            Self::#variants(event) => {
                                #full::#variants(event.into_full_event(room_id))
                            }
                        )*
                        Self::_Custom(event) => {
                            #full::_Custom(event.into_full_event(room_id))
                        },
                    }
                }
            }
        })
    }

    /// Implement `From<{event_type}>` for all the variants of the given enum.
    fn expand_from_impl(&self, ident: &syn::Ident, event_types: &[syn::Path]) -> TokenStream {
        let variants = &self.variants;
        let variant_attrs = &self.variant_attrs;

        quote! {
            #(
                #[allow(unused_qualifications)]
                #[automatically_derived]
                #( #variant_attrs )*
                impl ::std::convert::From<#event_types> for #ident {
                    fn from(c: #event_types) -> Self {
                        Self::#variants(c)
                    }
                }
            )*
        }
    }
}

impl Deref for EventEnum<'_> {
    type Target = EventEnumData;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

/// A variation of an event enum.
struct EventEnumVariation<'a> {
    /// The event enum data.
    inner: &'a EventEnum<'a>,

    /// The variation of this enum.
    variation: EventVariation,

    /// The name of this enum.
    ident: syn::Ident,

    /// The name of the struct used for the events for this variation.
    event_struct: syn::Ident,

    /// The paths to the event types of the variants of this enum.
    event_types: Vec<syn::Path>,
}

impl<'a> EventEnumVariation<'a> {
    /// Construct an `EventEnumVariation` for the given data and variation.
    fn new(inner: &'a EventEnum<'a>, variation: EventVariation) -> syn::Result<Self> {
        let ident = inner.kind.to_event_enum_ident(variation)?;
        let event_struct = inner.kind.to_event_ident(variation);
        let event_types =
            inner.events.iter().map(|event| event.to_event_path(inner.kind, variation)).collect();

        Ok(Self { inner, variation, ident, event_struct, event_types })
    }
}

impl EventEnumVariation<'_> {
    /// Whether the content in the variants of this enum can be redacted.
    fn maybe_redacted(&self) -> bool {
        self.kind.is_timeline()
            && matches!(self.variation, EventVariation::None | EventVariation::Sync)
    }

    /// Generate this `Any*Event` enum.
    fn expand_event_kind_enum(&self) -> syn::Result<TokenStream> {
        let ruma_events = self.ruma_events;

        let ident = &self.ident;
        let event_struct = &self.event_struct;
        let attrs = &self.attrs;
        let variants = &self.variants;
        let variant_attrs = &self.variant_attrs;
        let variant_docs = &self.variant_docs;
        let event_types = &self.event_types;

        let kind = self.kind;
        let custom_content_ty = format_ident!("Custom{kind}Content");

        let deserialize_impl = self.expand_deserialize_impl();
        let field_accessor_impl = self.expand_accessor_methods()?;
        let from_impl = self.expand_from_impl(ident, event_types);
        let json_castable_impl =
            expand_json_castable_impl(ident, kind, self.variation, ruma_events);

        Ok(quote! {
            #( #attrs )*
            #[derive(Clone, Debug)]
            #[allow(clippy::large_enum_variant, unused_qualifications)]
            #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
            pub enum #ident {
                #(
                    #variant_docs
                    #( #variant_attrs )*
                    #variants(#event_types),
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
            #json_castable_impl
        })
    }

    /// Generate the `serde::de::Deserialize` implementation for this enum.
    fn expand_deserialize_impl(&self) -> TokenStream {
        let ruma_events = self.ruma_events;
        let ruma_common = ruma_events.ruma_common();
        let serde = ruma_events.reexported(RumaEventsReexport::Serde);
        let serde_json = ruma_events.reexported(RumaEventsReexport::SerdeJson);

        let ident = &self.ident;
        let variants = &self.variants;
        let variant_attrs = &self.variant_attrs;
        let event_type_string_match_arms = &self.event_type_string_match_arms;
        let event_types = &self.event_types;

        quote! {
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
                        #(
                            #( #variant_attrs )*
                            #( #event_type_string_match_arms )|* => {
                                let event = #serde_json::from_str::<#event_types>(json.get())
                                    .map_err(D::Error::custom)?;
                                Ok(Self::#variants(event))
                            },
                        )*
                        _ => {
                            let event = #serde_json::from_str(json.get()).map_err(D::Error::custom)?;
                            Ok(Self::_Custom(event))
                        },
                    }
                }
            }
        }
    }

    /// Implement accessors for the common fields of an `Any*Event` enum.
    fn expand_accessor_methods(&self) -> syn::Result<TokenStream> {
        let ruma_events = self.ruma_events;

        let ident = &self.ident;
        let event_type_enum = &self.event_type_enum;
        let variants = &self.variants;
        let variant_attrs = &self.variant_attrs;

        let event_type_match_arms = if self.maybe_redacted() {
            quote! {
                #(
                    #( #variant_attrs )*
                    Self::#variants(event) => event.event_type(),
                )*
                Self::_Custom(event) => event.event_type(),
            }
        } else if self.variation == EventVariation::Stripped {
            let possibly_redacted_event_content_kind_trait =
                self.kind.to_content_kind_trait(EventContentTraitVariation::PossiblyRedacted);

            quote! {
                #(
                    #( #variant_attrs )*
                    Self::#variants(event) =>
                        #ruma_events::#possibly_redacted_event_content_kind_trait::event_type(&event.content),
                )*
                Self::_Custom(event) => ::std::convert::From::from(
                    #ruma_events::#possibly_redacted_event_content_kind_trait::event_type(&event.content),
                ),
            }
        } else {
            let original_event_content_kind_trait =
                self.kind.to_content_kind_trait(EventContentTraitVariation::Original);

            quote! {
                #(
                    #( #variant_attrs )*
                    Self::#variants(event) =>
                        #ruma_events::#original_event_content_kind_trait::event_type(&event.content),
                )*
                Self::_Custom(event) => ::std::convert::From::from(
                    #ruma_events::#original_event_content_kind_trait::event_type(&event.content),
                ),
            }
        };

        let content_accessor = self.expand_content_accessors();
        let field_accessors = self.expand_event_field_accessors();
        let state_key_accessor = self.expand_state_key_accessor();
        let relations_accessor = self.expand_relations_accessor();
        let transaction_id_accessor = self.expand_transaction_id_accessor();

        Ok(quote! {
            #[automatically_derived]
            impl #ident {
                /// Returns the `type` of this event.
                pub fn event_type(&self) -> #ruma_events::#event_type_enum {
                    match self { #event_type_match_arms }
                }

                #content_accessor
                #( #field_accessors )*
                #relations_accessor
                #state_key_accessor
                #transaction_id_accessor
            }
        })
    }

    /// Generate accessors for the `content` field for this enum.
    ///
    /// The code that is generated depends on the (kind, variation) tuple:
    ///
    /// * `pub fn original_content(&self) -> Option<ContentEnum>` and `pub fn is_redacted(&self)` ->
    ///   bool` for kinds and variations that return `true` in
    ///   [`maybe_redacted()`](Self::maybe_redacted). It also generates `pub fn content(&self) ->
    ///   FullContentEnum` for state events.
    /// * An empty `TokenStream` for the stripped variation.
    /// * `pub fn content(&self) -> ContentEnum` for the others.
    fn expand_content_accessors(&self) -> TokenStream {
        let ruma_events = self.ruma_events;

        let content_enum = &self.content_enum;
        let event_struct = &self.event_struct;
        let variants = &self.variants;
        let variant_attrs = &self.variant_attrs;

        let original_event_content_kind_trait =
            self.kind.to_content_kind_trait(EventContentTraitVariation::Original);

        if self.maybe_redacted() {
            let mut accessors = quote! {
                /// Returns the content for this event if it is not redacted, or `None` if it is.
                pub fn original_content(&self) -> Option<#content_enum> {
                    match self {
                        #(
                            #( #variant_attrs )*
                            Self::#variants(event) => {
                                event.as_original().map(|ev| #content_enum::#variants(ev.content.clone()))
                            }
                        )*
                        Self::_Custom(event) => event.as_original().map(|ev| {
                            #content_enum::_Custom {
                                event_type: crate::PrivOwnedStr(
                                    ::std::convert::From::from(
                                        ::std::string::ToString::to_string(
                                            &#ruma_events::#original_event_content_kind_trait::event_type(
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
                            #( #variant_attrs )*
                            Self::#variants(event) => {
                                event.as_original().is_none()
                            }
                        )*
                        Self::_Custom(event) => event.as_original().is_none(),
                    }
                }
            };

            if self.kind == EventEnumKind::State {
                let full_content_enum = &self.full_content_enum;
                let redacted_event_content_kind_trait_name =
                    self.kind.to_content_kind_trait(EventContentTraitVariation::Redacted);

                accessors.extend(quote! {
                    /// Returns the content of this state event.
                    pub fn content(&self) -> #full_content_enum {
                        match self {
                            #(
                                #( #variant_attrs )*
                                Self::#variants(event) => match event {
                                    #ruma_events::#event_struct::Original(ev) => #full_content_enum::#variants(
                                        #ruma_events::FullStateEventContent::Original {
                                            content: ev.content.clone(),
                                            prev_content: ev.unsigned.prev_content.clone()
                                        }
                                    ),
                                    #ruma_events::#event_struct::Redacted(ev) => #full_content_enum::#variants(
                                        #ruma_events::FullStateEventContent::Redacted(
                                            ev.content.clone()
                                        )
                                    ),
                                },
                            )*
                            Self::_Custom(event) => match event {
                                #ruma_events::#event_struct::Original(ev) => {
                                    #full_content_enum::_Custom {
                                        event_type: crate::PrivOwnedStr(
                                            ::std::string::ToString::to_string(
                                                &#ruma_events::#original_event_content_kind_trait::event_type(
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
                });
            }

            accessors
        } else if self.variation == EventVariation::Stripped {
            // There is no content enum for possibly-redacted content types (yet)
            TokenStream::new()
        } else {
            quote! {
                /// Returns the content for this event.
                pub fn content(&self) -> #content_enum {
                    match self {
                        #(
                            #( #variant_attrs )*
                            Self::#variants(event) => #content_enum::#variants(event.content.clone()),
                        )*
                        Self::_Custom(event) => #content_enum::_Custom {
                            event_type: crate::PrivOwnedStr(
                                ::std::convert::From::from(
                                    ::std::string::ToString::to_string(
                                        &#ruma_events::#original_event_content_kind_trait::event_type(&event.content)
                                    )
                                ),
                            ),
                        },
                    }
                }
            }
        }
    }

    /// Generate accessors for the [`EventField`]s that are present for this enum.
    fn expand_event_field_accessors(&self) -> impl Iterator<Item = TokenStream> {
        CommonEventField::ALL
            .iter()
            .filter(|field| self.kind.field_is_present(**field, self.variation))
            .map(|field| {
                let variants = &self.variants;
                let variant_attrs = &self.variant_attrs;

                let docs = format!("Returns this event's `{field}` field.");
                let ident = field.ident();

                // Field types that don't implement `Copy` must be accessedd via a reference.
                let (field_type, is_ref) = field.ty(self.ruma_events);
                let ampersand = is_ref.then(|| quote! { & });

                // If this content might be redacted, the field is available through an accessor on
                // the inner content enum.
                let call_parens = self.maybe_redacted().then(|| quote! { () });

                quote! {
                    #[doc = #docs]
                    pub fn #ident(&self) -> #field_type {
                        match self {
                            #(
                                #( #variant_attrs )*
                                Self::#variants(event) => #ampersand event.#ident #call_parens,
                            )*
                            Self::_Custom(event) => #ampersand event.#ident #call_parens,
                        }
                    }
                }
            })
    }

    /// Generate an accessor for the `state_key` field for this enum, if present.
    fn expand_state_key_accessor(&self) -> Option<TokenStream> {
        if self.kind != EventEnumKind::State {
            return None;
        }

        let variants = &self.variants;
        let variant_attrs = &self.variant_attrs;

        // If this content might be redacted, the field is available through an accessor on
        // the inner content enum.
        let call_parens = self.maybe_redacted().then(|| quote! { () });

        Some(quote! {
            /// Returns this event's `state_key` field.
            pub fn state_key(&self) -> &::std::primitive::str {
                match self {
                    #(
                        #( #variant_attrs )*
                        Self::#variants(event) => event.state_key #call_parens .as_ref(),
                    )*
                    Self::_Custom(event) => event.state_key #call_parens .as_ref(),
                }
            }
        })
    }

    /// Generate an accessor for the `unsigned.relations` field for this enum, if present.
    fn expand_relations_accessor(&self) -> Option<TokenStream> {
        if self.kind != EventEnumKind::MessageLike {
            return None;
        }

        let ruma_events = self.ruma_events;
        let variants = &self.variants;
        let variant_attrs = &self.variant_attrs;

        Some(quote! {
            /// Returns this event's `relations` from inside `unsigned`.
            pub fn relations(
                &self,
            ) -> #ruma_events::BundledMessageLikeRelations<AnySyncMessageLikeEvent> {
                match self {
                    #(
                        #( #variant_attrs )*
                        Self::#variants(event) => event.as_original().map_or_else(
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
        })
    }

    /// Generate an accessor for the `unsigned.transaction_id` field for this enum, if present.
    fn expand_transaction_id_accessor(&self) -> Option<TokenStream> {
        if !self.maybe_redacted() {
            return None;
        }

        let ruma_common = self.ruma_events.ruma_common();
        let variants = &self.variants;
        let variant_attrs = &self.variant_attrs;

        Some(quote! {
            /// Returns this event's `transaction_id` from inside `unsigned`, if there is one.
            pub fn transaction_id(&self) -> Option<&#ruma_common::TransactionId> {
                match self {
                    #(
                        #( #variant_attrs )*
                        Self::#variants(event) => {
                            event.as_original().and_then(|ev| ev.unsigned.transaction_id.as_deref())
                        }
                    )*
                    Self::_Custom(event) => {
                        event.as_original().and_then(|ev| ev.unsigned.transaction_id.as_deref())
                    }
                }
            }
        })
    }
}

impl<'a> Deref for EventEnumVariation<'a> {
    type Target = EventEnum<'a>;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}
