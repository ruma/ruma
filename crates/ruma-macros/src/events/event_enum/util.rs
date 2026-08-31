use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::EventEnumKind;
use crate::{
    events::common::{EventContentTraitVariation, EventVariation},
    util::RumaEvents,
};

/// Generate `ruma_common::serde::JsonCastable` implementations for all compatible types.
pub(super) fn expand_json_castable_impl(
    ident: &syn::Ident,
    kind: EventEnumKind,
    variation: EventVariation,
    ruma_events: &RumaEvents,
) -> TokenStream {
    let ruma_common = ruma_events.ruma_common();

    // All event types are represented as objects in JSON.
    let mut json_castable_impls = quote! {
        #[automatically_derived]
        impl #ruma_common::serde::JsonCastable<#ruma_common::serde::JsonObject> for #ident {}
    };

    // The event type kinds in this enum.
    let mut event_kinds = vec![kind];
    event_kinds.extend(kind.extra_enum_kinds());

    for event_kind in event_kinds {
        let event_variations = event_kind.event_variations();

        // Matching event types (structs or enums) can be cast to this event enum.
        json_castable_impls.extend(
            event_variations
                .iter()
                // Filter variations that can't be cast from.
                .filter(|event_variation| event_variation.is_json_castable_to(variation))
                // All enum variations can also be cast from event structs from the same variation.
                .chain(event_variations.contains(&variation).then_some(&variation))
                .map(|event_variation| {
                    let EventWithBounds { type_with_generics, impl_generics, where_clause } =
                        EventWithBounds::new(event_kind, *event_variation, ruma_events);

                    quote! {
                        #[automatically_derived]
                        impl #impl_generics #ruma_common::serde::JsonCastable<#ident> for #type_with_generics
                        #where_clause
                        {}
                    }
                }),
        );

        // Matching event enums can be cast to this one, e.g. `AnyMessageLikeEvent` can be cast to
        // `AnyTimelineEvent`.
        let event_enum_variations = event_kind.event_enum_variations();

        json_castable_impls.extend(
            event_enum_variations
                .iter()
                // Filter variations that can't be cast from.
                .filter(|event_enum_variation| event_enum_variation.is_json_castable_to(variation))
                // All enum variations can also be cast from other event enums from the same
                // variation.
                .chain(
                    (event_kind != kind && event_enum_variations.contains(&variation))
                        .then_some(&variation),
                )
                .map(|event_enum_variation| {
                    let other_ident = event_kind.to_event_enum_ident(*event_enum_variation);

                    quote! {
                        #[automatically_derived]
                        impl #ruma_common::serde::JsonCastable<#ident> for #other_ident {}
                    }
                }),
        );
    }

    json_castable_impls
}

impl EventEnumKind {
    /// Get the name of the `Any*Event` enum for this kind and the given variation.
    pub(super) fn to_event_enum_ident(self, variation: EventVariation) -> syn::Ident {
        format_ident!("Any{variation}{self}")
    }

    /// Get the list of extra event kinds that are part of the event enum for this kind.
    fn extra_enum_kinds(self) -> &'static [Self] {
        match self {
            Self::Timeline => &[Self::MessageLike, Self::State],
            Self::GlobalAccountData
            | Self::RoomAccountData
            | Self::EphemeralRoom
            | Self::MessageLike
            | Self::State
            | Self::ToDevice => &[],
        }
    }
}

/// An event type (struct or enum) with its bounds.
struct EventWithBounds {
    /// The type name with its generics.
    type_with_generics: TokenStream,

    /// The generics declaration.
    impl_generics: Option<TokenStream>,

    /// The `where` clause with the event bounds.
    where_clause: Option<TokenStream>,
}

impl EventWithBounds {
    fn new(kind: EventEnumKind, variation: EventVariation, ruma_events: &RumaEvents) -> Self {
        let ident = kind.to_event_ident(variation);

        let event_content_trait = match variation {
            EventVariation::None
            | EventVariation::Sync
            | EventVariation::Original
            | EventVariation::OriginalSync
            | EventVariation::Initial
            | EventVariation::Stripped => {
                // `State` event structs have a `StaticStateEventContent` bound.
                if kind == EventEnumKind::State {
                    kind.to_content_kind_trait(EventContentTraitVariation::Static)
                } else {
                    kind.to_content_kind_trait(EventContentTraitVariation::Original)
                }
            }
            EventVariation::Redacted | EventVariation::RedactedSync => {
                kind.to_content_kind_trait(EventContentTraitVariation::Redacted)
            }
        };

        let (type_with_generics, impl_generics, where_clause) = match kind {
            EventEnumKind::MessageLike
                if matches!(variation, EventVariation::None | EventVariation::Sync) =>
            {
                // `MessageLike` event kind has an extra `RedactContent` bound with a `where` clause
                // on the variations that match enum types.
                let redacted_trait =
                    kind.to_content_kind_trait(EventContentTraitVariation::Redacted);

                (
                    quote! { #ruma_events::#ident<C> },
                    Some(
                        quote! { <C: #ruma_events::#event_content_trait + #ruma_events::RedactContent> },
                    ),
                    Some(quote! {
                        where
                            C::Redacted: #ruma_events::#redacted_trait,
                    }),
                )
            }
            EventEnumKind::GlobalAccountData
            | EventEnumKind::RoomAccountData
            | EventEnumKind::EphemeralRoom
            | EventEnumKind::MessageLike
            | EventEnumKind::State
            | EventEnumKind::ToDevice => (
                quote! { #ruma_events::#ident<C> },
                Some(quote! { <C: #ruma_events::#event_content_trait> }),
                None,
            ),
            EventEnumKind::Timeline => {
                panic!("Timeline kind should not generate JsonCastable implementations")
            }
        };

        Self { impl_generics, type_with_generics, where_clause }
    }
}
