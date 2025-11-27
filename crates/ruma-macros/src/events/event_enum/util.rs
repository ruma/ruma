use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};

use super::{EventKind, EventVariation};
use crate::{events::enums::EventWithBounds, util::RumaEvents};

/// Generate `ruma_common::serde::JsonCastable` implementations for all compatible types.
pub(super) fn expand_json_castable_impl(
    ident: &syn::Ident,
    kind: EventKind,
    variation: EventVariation,
    ruma_events: &RumaEvents,
) -> syn::Result<Option<TokenStream>> {
    let ruma_common = ruma_events.ruma_common();

    // All event types are represented as objects in JSON.
    let mut json_castable_impls = vec![quote! {
        #[automatically_derived]
        impl #ruma_common::serde::JsonCastable<#ruma_common::serde::JsonObject> for #ident {}
    }];

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
                        event_kind.to_event_with_bounds(*event_variation, ruma_events)?;

                    Ok(quote! {
                        #[automatically_derived]
                        impl #impl_generics #ruma_common::serde::JsonCastable<#ident> for #type_with_generics
                        #where_clause
                        {}
                    })
                })
                .collect::<syn::Result<Vec<_>>>()?,
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
                    let other_ident = event_kind
                        .to_event_enum_ident(*event_enum_variation)
                        .expect("we only use variations that match an enum type");

                    quote! {
                        #[automatically_derived]
                        impl #ruma_common::serde::JsonCastable<#ident> for #other_ident {}
                    }
                }),
        );
    }

    Ok(Some(json_castable_impls.into_iter().collect()))
}

impl EventKind {
    /// Get the name of the `Any*Event` enum for this kind and the given variation.
    pub(super) fn to_event_enum_ident(self, var: EventVariation) -> syn::Result<syn::Ident> {
        if !self.event_enum_variations().contains(&var) {
            return Err(syn::Error::new(
                Span::call_site(),
                format!(
                    "({self:?}, {var:?}) is not a valid event enum kind / variation combination"
                ),
            ));
        }

        Ok(format_ident!("Any{var}{self}"))
    }
}
