//! Implementation of the `EventContent` derive macro.

use std::borrow::Cow;

use as_variant::as_variant;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::parse_quote;

mod parse;

use super::common::{EventContentTraitVariation, EventKind, EventType, EventTypes, EventVariation};
use crate::util::{
    PrivateField, RumaCommon, RumaEvents, RumaEventsReexport, SerdeMetaItem, StructFieldExt,
    TypeExt,
};

/// `EventContent` derive macro code generation.
pub(crate) fn expand_event_content(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let event_content = EventContent::parse(input)?;

    // Generate alternate variations.
    let redacted_event_content = event_content.expand_redacted_event_content();
    let possibly_redacted_event_content = event_content.expand_possibly_redacted_event_content();
    let event_content_without_relation = event_content.expand_event_content_without_relation();

    // Generate trait implementations of the original variation.
    let event_content_impl = event_content.expand_event_content_impl(
        EventContentVariation::Original,
        &event_content.ident,
        event_content.fields.as_ref(),
    );
    let static_event_content_impl =
        event_content.expand_static_event_content_impl(&event_content.ident);
    let json_castable_impl = generate_json_castable_impl(&event_content.ident, &[]);

    // Generate type aliases.
    let event_type_aliases = event_content.expand_event_type_aliases();

    Ok(quote! {
        #redacted_event_content
        #possibly_redacted_event_content
        #event_content_without_relation
        #event_content_impl
        #static_event_content_impl
        #json_castable_impl
        #event_type_aliases
    })
}

/// Parsed `EventContent` container data.
struct EventContent {
    /// The name of the event content type.
    ident: syn::Ident,

    /// The visibility of the event content type.
    vis: syn::Visibility,

    /// The fields of the event content type, if it is a struct.
    fields: Option<Vec<EventContentField>>,

    /// The event types.
    types: EventTypes,

    /// The event kind.
    kind: EventContentKind,

    /// Whether this macro should generate an `*EventContentWithoutRelation` type.
    has_without_relation: bool,

    /// The path for imports from the ruma-events crate.
    ruma_events: RumaEvents,
}

impl EventContent {
    /// The name of the field that contains the type fragment of the struct, if any.
    fn type_fragment_field(&self) -> Option<&syn::Ident> {
        self.fields
            .as_ref()?
            .iter()
            .find(|field| field.is_type_fragment)
            .and_then(|field| field.inner.ident.as_ref())
    }

    /// Generate the `Redacted*EventContent` variation of this struct, if it needs one.
    fn expand_redacted_event_content(&self) -> Option<TokenStream> {
        if !self.kind.should_generate_redacted() {
            return None;
        }

        let ruma_events = &self.ruma_events;
        let ruma_common = ruma_events.ruma_common();
        let serde = ruma_events.reexported(RumaEventsReexport::Serde);

        let ident = &self.ident;
        let vis = &self.vis;

        let redacted_doc = format!("Redacted form of [`{ident}`]");
        let redacted_ident = EventContentVariation::Redacted.variation_ident(ident);

        let redacted_fields =
            self.fields.iter().flatten().filter(|field| field.skip_redaction).collect::<Vec<_>>();
        let redacted_fields_idents = redacted_fields.iter().flat_map(|field| &field.inner.ident);

        let constructor = redacted_fields.is_empty().then(|| {
            let constructor_doc = format!("Creates an empty {redacted_ident}.");
            quote! {
                impl #redacted_ident {
                    #[doc = #constructor_doc]
                    #vis fn new() -> Self {
                        Self {}
                    }
                }
            }
        });

        let redacted_event_content = self.expand_event_content_impl(
            EventContentVariation::Redacted,
            &redacted_ident,
            Some(redacted_fields.iter().copied()),
        );
        let static_event_content_impl = self.expand_static_event_content_impl(&redacted_ident);
        let json_castable_impl = generate_json_castable_impl(&redacted_ident, &[ident]);

        Some(quote! {
            // this is the non redacted event content's impl
            #[automatically_derived]
            impl #ruma_events::RedactContent for #ident {
                type Redacted = #redacted_ident;

                fn redact(self, _rules: &#ruma_common::room_version_rules::RedactionRules) -> #redacted_ident {
                    #redacted_ident {
                        #( #redacted_fields_idents: self.#redacted_fields_idents, )*
                    }
                }
            }

            #[doc = #redacted_doc]
            #[derive(Clone, Debug, #serde::Deserialize, #serde::Serialize)]
            #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
            #vis struct #redacted_ident {
                #( #redacted_fields, )*
            }

            #constructor
            #redacted_event_content
            #static_event_content_impl
            #json_castable_impl
        })
    }

    /// Generate the `PossiblyRedacted*EventContent` variation of this struct, if it needs one.
    fn expand_possibly_redacted_event_content(&self) -> Option<TokenStream> {
        if !self.kind.should_generate_possibly_redacted() {
            return None;
        }

        let serde = self.ruma_events.reexported(RumaEventsReexport::Serde);

        let ident = &self.ident;
        let vis = &self.vis;

        let possibly_redacted_doc = format!(
            "The possibly redacted form of [`{ident}`].\n\n\
             This type is used when it's not obvious whether the content is redacted or not."
        );
        let possibly_redacted_ident =
            EventContentVariation::PossiblyRedacted.variation_ident(ident);

        let mut field_changed = false;

        let possibly_redacted_fields = self
            .fields
            .iter()
            .flatten()
            .map(|field| {
                if field.keep_in_possibly_redacted() {
                    return Cow::Borrowed(field);
                }

                // Otherwise, change the field to an `Option`.
                field_changed = true;

                let mut field = field.clone();
                let wrapped_type = &field.inner.ty;
                field.inner.ty = parse_quote! { Option<#wrapped_type> };
                field
                    .inner
                    .attrs
                    .push(parse_quote! { #[serde(skip_serializing_if = "Option::is_none")] });

                Cow::Owned(field)
            })
            .collect::<Vec<_>>();

        // If at least one field needs to change, generate a new struct, else use a type alias.
        if field_changed {
            let possibly_redacted_event_content = self.expand_event_content_impl(
                EventContentVariation::PossiblyRedacted,
                &possibly_redacted_ident,
                Some(possibly_redacted_fields.iter().map(|field| field.as_ref())),
            );
            let static_event_content_impl =
                self.expand_static_event_content_impl(&possibly_redacted_ident);

            let json_castable_impl = if self.kind.should_generate_redacted() {
                let redacted_ident = EventContentVariation::PossiblyRedacted.variation_ident(ident);
                generate_json_castable_impl(&possibly_redacted_ident, &[ident, &redacted_ident])
            } else {
                generate_json_castable_impl(&possibly_redacted_ident, &[ident])
            };

            Some(quote! {
                #[doc = #possibly_redacted_doc]
                #[derive(Clone, Debug, #serde::Deserialize, #serde::Serialize)]
                #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
                #vis struct #possibly_redacted_ident {
                    #( #possibly_redacted_fields, )*
                }

                #possibly_redacted_event_content
                #static_event_content_impl
                #json_castable_impl
            })
        } else {
            let event_content_kind_trait_impl = self.expand_event_content_kind_trait_impl(
                EventContentTraitVariation::PossiblyRedacted,
                ident,
            );

            Some(quote! {
                #[doc = #possibly_redacted_doc]
                #vis type #possibly_redacted_ident = #ident;

                #event_content_kind_trait_impl
            })
        }
    }

    /// Generate the `*EventContentWithoutRelation` variation of the type.
    fn expand_event_content_without_relation(&self) -> Option<TokenStream> {
        if !self.has_without_relation {
            return None;
        }

        let serde = self.ruma_events.reexported(RumaEventsReexport::Serde);

        let ident = &self.ident;
        let vis = &self.vis;

        let without_relation_doc = format!(
            "Form of [`{ident}`] without relation.\n\n\
             To construct this type, construct a [`{ident}`] and then use one of its `::from()` / `.into()` methods."
        );
        let without_relation_ident = format_ident!("{ident}WithoutRelation");
        let with_relation_fn_doc =
            format!("Convert `self` into a [`{ident}`] with the given relation.");

        let (relates_to_field, without_relation_fields) =
            self.fields.iter().flatten().partition::<Vec<_>, _>(|field| {
                field.inner.ident.as_ref().is_some_and(|ident| *ident == "relates_to")
            });

        let relates_to_type = relates_to_field.first().map(|field| &field.inner.ty).expect(
            "event content type without relation should have a `relates_to` field; \
             this should have been checked during parsing",
        );

        let without_relation_fields_idents =
            without_relation_fields.iter().flat_map(|field| &field.inner.ident).collect::<Vec<_>>();
        let without_relation_struct_definition = if without_relation_fields.is_empty() {
            quote! { ; }
        } else {
            quote! {
                { #( #without_relation_fields, )* }
            }
        };

        let json_castable_impl = generate_json_castable_impl(&without_relation_ident, &[ident]);

        Some(quote! {
            #[allow(unused_qualifications)]
            #[automatically_derived]
            impl ::std::convert::From<#ident> for #without_relation_ident {
                fn from(c: #ident) -> Self {
                    Self {
                        #( #without_relation_fields_idents: c.#without_relation_fields_idents, )*
                    }
                }
            }

            #[doc = #without_relation_doc]
            #[derive(Clone, Debug, #serde::Deserialize, #serde::Serialize)]
            #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
            #vis struct #without_relation_ident #without_relation_struct_definition

            impl #without_relation_ident {
                #[doc = #with_relation_fn_doc]
                #vis fn with_relation(self, relates_to: #relates_to_type) -> #ident {
                    #ident {
                        #( #without_relation_fields_idents: self.#without_relation_fields_idents, )*
                        relates_to,
                    }
                }
            }

            #json_castable_impl
        })
    }

    /// Generate the `ruma_events::*EventContent` trait implementations for this kind and the given
    /// event content variation with the given ident and fields.
    fn expand_event_content_impl<'a>(
        &self,
        variation: EventContentVariation,
        ident: &syn::Ident,
        fields: Option<impl IntoIterator<Item = &'a EventContentField>>,
    ) -> TokenStream {
        let event_content_kind_trait_impl =
            self.expand_event_content_kind_trait_impl(variation.into(), ident);
        let static_state_event_content_impl =
            self.expand_static_state_event_content_impl(variation, ident);
        let event_content_from_type_impl = self.expand_event_content_from_type_impl(ident, fields);

        quote! {
            #event_content_from_type_impl
            #event_content_kind_trait_impl
            #static_state_event_content_impl
        }
    }

    /// Generate the `ruma_events::*EventContent` trait implementations for this kind and the given
    /// variation with the given ident.
    fn expand_event_content_kind_trait_impl(
        &self,
        variation: EventContentTraitVariation,
        ident: &syn::Ident,
    ) -> TokenStream {
        let ruma_events = &self.ruma_events;

        let event_type = self.types.ev_type.without_wildcard();
        let event_type_fn_impl = if let Some(field) = self.type_fragment_field() {
            let format = event_type.to_owned() + "{}";

            quote! {
                ::std::convert::From::from(::std::format!(#format, self.#field))
            }
        } else {
            quote! { ::std::convert::From::from(#event_type) }
        };

        let state_key =
            as_variant!(&self.kind, EventContentKind::State { state_key_type, .. } => state_key_type).map(|state_key_type| {
                quote! {
                    type StateKey = #state_key_type;
                }
            });

        self.kind
            .as_event_type_enums_and_content_kind_traits(variation)
            .into_iter()
            .map(|(event_type_enum, event_content_kind_trait)| {
                quote! {
                    #[automatically_derived]
                    impl #ruma_events::#event_content_kind_trait for #ident {
                        #state_key

                        fn event_type(&self) -> #ruma_events::#event_type_enum {
                            #event_type_fn_impl
                        }
                    }
                }
            })
            .collect()
    }

    /// Generate the `ruma_events::StaticStateEventContent` trait implementation for this kind and
    /// the given variation with the given ident, if it needs one.
    fn expand_static_state_event_content_impl(
        &self,
        variation: EventContentVariation,
        ident: &syn::Ident,
    ) -> Option<TokenStream> {
        let EventContentKind::State { unsigned_type, .. } = &self.kind else {
            // Only the `State` kind can implement this trait.
            return None;
        };

        if variation != EventContentVariation::Original {
            // Only the original variation can implement this trait.
            return None;
        }

        let ruma_events = &self.ruma_events;
        let possibly_redacted_ident =
            EventContentVariation::PossiblyRedacted.variation_ident(ident);

        Some(quote! {
            #[automatically_derived]
            impl #ruma_events::StaticStateEventContent for #ident {
                type PossiblyRedacted = #possibly_redacted_ident;
                type Unsigned = #unsigned_type;
            }
        })
    }

    /// Generate the `StaticEventContent` trait implementation for the given ident.
    fn expand_static_event_content_impl(&self, ident: &syn::Ident) -> TokenStream {
        let ruma_events = &self.ruma_events;
        let static_event_type = self.types.ev_type.without_wildcard();

        let is_prefix = if self.types.is_prefix() {
            quote! { #ruma_events::True }
        } else {
            quote! { #ruma_events::False }
        };

        quote! {
            impl #ruma_events::StaticEventContent for #ident {
                const TYPE: &'static ::std::primitive::str = #static_event_type;
                type IsPrefix = #is_prefix;
            }
        }
    }

    /// Generate the `ruma_events::EventContentFromType` trait implementation for the given ident
    /// with the given fields, if this event type has a type fragment.
    fn expand_event_content_from_type_impl<'a>(
        &self,
        ident: &syn::Ident,
        fields: Option<impl IntoIterator<Item = &'a EventContentField>>,
    ) -> Option<TokenStream> {
        let type_fragment_field = self.type_fragment_field()?;
        let fields = fields.expect(
            "event content with `.*` type suffix should be a struct; \
             this should have been checked during parsing",
        );

        let ruma_events = &self.ruma_events;
        let serde = ruma_events.reexported(RumaEventsReexport::Serde);
        let serde_json = ruma_events.reexported(RumaEventsReexport::SerdeJson);

        let type_prefixes = self.types.iter().map(EventType::without_wildcard);
        let type_prefixes = quote! {
            [#( #type_prefixes, )*]
        };

        let fields_without_type_fragment = fields
            .into_iter()
            .filter(|field| !field.is_type_fragment)
            .map(|field| PrivateField(&field.inner))
            .collect::<Vec<_>>();
        let fields_ident_without_type_fragment =
            fields_without_type_fragment.iter().filter_map(|f| f.0.ident.as_ref());

        Some(quote! {
            impl #ruma_events::EventContentFromType for #ident {
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
        })
    }

    /// Generate the type aliases for the event.
    fn expand_event_type_aliases(&self) -> Option<TokenStream> {
        // The redaction module has its own event types.
        if self.ident == "RoomRedactionEventContent" {
            return None;
        }

        let ruma_events = &self.ruma_events;
        let event_type = &self.types.ev_type;
        let ident = &self.ident;
        let ident_s = ident.to_string();
        let ev_type_s = ident_s.strip_suffix("Content").expect(
            "event content struct name should end with `Content`; \
             this should have been checked during parsing",
        );
        let vis = &self.vis;

        Some(
            self.kind
                .event_variations()
                .iter()
                .flat_map(|&variation| {
                    std::iter::repeat(variation)
                        .zip(self.kind.as_event_idents(variation).into_iter().flatten())
                })
                .map(|(variation, (type_kind_prefix, event_ident))| {
                    let type_alias_ident =
                        format_ident!("{variation}{type_kind_prefix}{ev_type_s}");

                    // Details about the variation added at the end of the sentence.
                    let doc_suffix = match variation {
                        EventVariation::None | EventVariation::Original => "",
                        EventVariation::Sync | EventVariation::OriginalSync => {
                            " from a `sync_events` response"
                        }
                        EventVariation::Stripped => " from an invited room preview",
                        EventVariation::Redacted => " that has been redacted",
                        EventVariation::RedactedSync => {
                            " from a `sync_events` response that has been redacted"
                        }
                        EventVariation::Initial => " for creating a room",
                    };

                    let type_alias_doc = if type_kind_prefix.is_empty() {
                        format!("An `{event_type}` event{doc_suffix}.")
                    } else {
                        format!(
                            "A {} `{event_type}` event{doc_suffix}.",
                            type_kind_prefix.to_lowercase()
                        )
                    };

                    let content_ident = if variation.is_redacted() {
                        EventContentVariation::Redacted.variation_ident(ident)
                    } else if let EventVariation::Stripped = variation {
                        EventContentVariation::PossiblyRedacted.variation_ident(ident)
                    } else {
                        EventContentVariation::Original.variation_ident(ident)
                    };

                    quote! {
                        #[doc = #type_alias_doc]
                        #vis type #type_alias_ident = #ruma_events::#event_ident<#content_ident>;
                    }
                })
                .collect(),
        )
    }
}

/// A parsed field of an event content struct.
#[derive(Clone)]
struct EventContentField {
    /// The inner field, with the `ruma_enum` attributes stripped.
    inner: syn::Field,

    /// Whether this field should be kept during redaction.
    skip_redaction: bool,

    /// Whether this field represents the suffix of the event type.
    is_type_fragment: bool,
}

impl EventContentField {
    /// Whether to keep this field as-is when generating the `PossiblyRedacted*EventContent`
    /// variation.
    ///
    /// Returns `true` if the field has the `skip_redaction` attribute, if its type is wrapped in an
    /// `Option`, or if it has the serde `default` attribute.
    fn keep_in_possibly_redacted(&self) -> bool {
        self.skip_redaction
            || self.inner.ty.option_inner_type().is_some()
            || self.inner.has_serde_meta_item(SerdeMetaItem::Default)
    }
}

impl ToTokens for EventContentField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.inner.to_tokens(tokens);
    }
}

/// The possible kinds of event content an their settings.
#[derive(Clone)]
#[allow(clippy::large_enum_variant)]
enum EventContentKind {
    /// Global account data.
    ///
    /// This is user data for the whole account.
    GlobalAccountData,

    /// Room account data.
    ///
    /// This is user data specific to a room.
    RoomAccountData,

    /// Both account data kinds.
    ///
    /// This is data usable as both global and room account data.
    BothAccountData,

    /// Ephemeral room data.
    ///
    /// This is data associated to a room and that is not persisted.
    EphemeralRoom,

    /// Message-like event.
    ///
    /// This is an event that can occur in the timeline and that doesn't have a state key.
    MessageLike {
        /// Whether the `Redacted*EventContent` type is implemented manually rather than generated
        /// by this macro.
        has_custom_redacted: bool,
    },

    /// State event.
    ///
    /// This is an event that can occur in the timeline and that has a state key.
    State {
        /// The type of the state key.
        state_key_type: syn::Type,

        /// The type of the unsigned data.
        unsigned_type: syn::Type,

        /// Whether the `Redacted*EventContent` type is implemented manually rather than generated
        /// by this macro.
        has_custom_redacted: bool,

        /// Whether the `PossiblyRedacted*EventContent` type is implemented manually rather than
        /// generated by this macro.
        has_custom_possibly_redacted: bool,
    },

    /// A to-device event.
    ///
    /// This is an event that is sent directly to another device.
    ToDevice,
}

impl EventContentKind {
    /// The [`EventKind`] matching this event content kind, if there is a single one.
    ///
    /// Returns `None` for [`EventContentKind::BothAccountData`].
    fn event_kind(&self) -> Option<EventKind> {
        Some(match self {
            Self::GlobalAccountData => EventKind::GlobalAccountData,
            Self::RoomAccountData => EventKind::RoomAccountData,
            Self::BothAccountData => return None,
            Self::EphemeralRoom => EventKind::EphemeralRoom,
            Self::MessageLike { .. } => EventKind::MessageLike,
            Self::State { .. } => EventKind::State,
            Self::ToDevice => EventKind::ToDevice,
        })
    }

    /// Whether this matches an account data kind.
    fn is_account_data(&self) -> bool {
        matches!(self, Self::BothAccountData)
            || self.event_kind().is_some_and(|event_kind| event_kind.is_account_data())
    }

    /// Whether we should generate a `Redacted*EventContent` variation for this kind.
    fn should_generate_redacted(&self) -> bool {
        // We only generate redacted content structs for state and message-like events.
        matches!(self, Self::MessageLike { has_custom_redacted, .. } | Self::State { has_custom_redacted, .. } if !*has_custom_redacted)
    }

    /// Whether we should generate a `Redacted*EventContent` variation for this kind.
    fn should_generate_possibly_redacted(&self) -> bool {
        // We only generate possibly redacted content structs for state events.
        matches!(self, Self::State { has_custom_possibly_redacted, .. } if !*has_custom_possibly_redacted)
    }

    /// Get the list of variations for an event type (struct or enum) for this kind.
    fn event_variations(&self) -> &'static [EventVariation] {
        if let Some(event_kind) = self.event_kind() {
            event_kind.event_variations()
        } else {
            // Both account data types have the same variations.
            EventKind::GlobalAccountData.event_variations()
        }
    }

    /// Get the idents of the event struct for these kinds and the given variation.
    ///
    /// Returns a list of `(type_prefix, event_ident)` if the variation is supported for these
    /// kinds.
    fn as_event_idents(
        &self,
        variation: EventVariation,
    ) -> Option<Vec<(&'static str, syn::Ident)>> {
        if let Some(event_kind) = self.event_kind() {
            event_kind.to_event_ident(variation).ok().map(|event_ident| vec![("", event_ident)])
        } else {
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

    /// Get the idents of the `*EventType` enums and `*EventContent` traits for this kind and the
    /// given variation.
    ///
    /// Returns a list of `(event_type_enum, event_content_trait)`.
    fn as_event_type_enums_and_content_kind_traits(
        &self,
        variation: EventContentTraitVariation,
    ) -> Vec<(syn::Ident, syn::Ident)> {
        if let Some(event_kind) = self.event_kind() {
            vec![(event_kind.to_event_type_enum(), event_kind.to_content_kind_trait(variation))]
        } else {
            [EventKind::GlobalAccountData, EventKind::RoomAccountData]
                .iter()
                .map(|event_kind| {
                    (event_kind.to_event_type_enum(), event_kind.to_content_kind_trait(variation))
                })
                .collect()
        }
    }
}

/// Implement `JsonCastable<JsonObject> for {ident}` and `JsonCastable<{ident}> for {other}`.
fn generate_json_castable_impl(ident: &syn::Ident, others: &[&syn::Ident]) -> TokenStream {
    let ruma_common = RumaCommon::new();

    let mut json_castable_impls = quote! {
        #[automatically_derived]
        impl #ruma_common::serde::JsonCastable<#ruma_common::serde::JsonObject> for #ident {}
    };

    json_castable_impls.extend(others.iter().map(|other| {
        quote! {
            #[automatically_derived]
            impl #ruma_common::serde::JsonCastable<#ident> for #other {}
        }
    }));

    json_castable_impls
}

/// The possible variations of an event content type.
#[derive(Clone, Copy, PartialEq)]
enum EventContentVariation {
    /// The original, non-redacted, event content.
    Original,

    /// The redacted event content.
    Redacted,

    /// Event content that might be redacted or not.
    PossiblyRedacted,
}

impl EventContentVariation {
    /// Get the ident for this variation, based on the given ident.
    fn variation_ident(self, ident: &syn::Ident) -> Cow<'_, syn::Ident> {
        match self {
            Self::Original => Cow::Borrowed(ident),
            Self::Redacted => Cow::Owned(format_ident!("Redacted{ident}")),
            Self::PossiblyRedacted => Cow::Owned(format_ident!("PossiblyRedacted{ident}")),
        }
    }
}

impl From<EventContentVariation> for EventContentTraitVariation {
    fn from(value: EventContentVariation) -> Self {
        match value {
            EventContentVariation::Original => Self::Original,
            EventContentVariation::Redacted => Self::Redacted,
            EventContentVariation::PossiblyRedacted => Self::PossiblyRedacted,
        }
    }
}
