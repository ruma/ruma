//! Functions to generate `Any*EventContent` enums.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Ident};

use super::{EventEnumEntry, EventEnumVariant, expand_from_impl};
use crate::{
    events::enums::{EventContentTraitVariation, EventKind, EventType},
    util::RumaCommon,
};

/// Generate an `Any*EventContent` enum.
pub fn expand_content_enum(
    kind: EventKind,
    events: &[EventEnumEntry],
    docs: &[TokenStream],
    attrs: &[Attribute],
    variants: &[EventEnumVariant],
    ruma_events: &TokenStream,
) -> syn::Result<TokenStream> {
    let serde = quote! { #ruma_events::exports::serde };

    let ident = kind.to_content_enum();

    let event_type_enum = kind.to_event_type_enum();

    let content: Vec<_> = events.iter().map(|event| event.to_event_content_path(kind)).collect();

    let variant_decls = variants.iter().map(|v| v.decl()).collect::<Vec<_>>();
    let variant_arms = variants.iter().map(|v| v.match_arm(quote! { Self })).collect::<Vec<_>>();

    let event_content_kind_trait_name =
        kind.to_content_kind_trait(EventContentTraitVariation::Original);
    let state_event_content_impl = (kind == EventKind::State).then(|| {
        quote! {
            type StateKey = String;
        }
    });

    let from_impl = expand_from_impl(&ident, &content, variants);
    let json_castable_impl = expand_json_castable_impl(&ident, &content, variants);

    let serialize_custom_event_error_path =
        quote! { #ruma_events::serialize_custom_event_error }.to_string();

    // Generate an `EventContentFromType` implementation.
    let serde_json = quote! { #ruma_events::exports::serde_json };
    let event_type_match_arms: TokenStream = events
        .iter()
        .map(|event| {
            let variant = event.to_variant();
            let variant_attrs = {
                let attrs = &variant.attrs;
                quote! { #(#attrs)* }
            };
            let self_variant = variant.ctor(quote! { Self });

            let ev_types = event.types.iter().map(EventType::as_match_arm);

            let deserialize_content = if event.has_type_fragment() {
                // If the event has a type fragment, then it implements EventContentFromType itself;
                // see `generate_event_content_impl` which does that. In this case, forward to its
                // implementation.
                let content_type = event.to_event_content_path(kind);
                quote! {
                    #content_type::from_parts(event_type, json)?
                }
            } else {
                // The event doesn't have a type fragment, so it *should* implement Deserialize:
                // use that here.
                quote! {
                    #serde_json::from_str(json.get())?
                }
            };

            quote! {
                #variant_attrs #(#ev_types)|* => {
                    let content = #deserialize_content;
                    Ok(#self_variant(content))
                },
            }
        })
        .collect();

    Ok(quote! {
        #( #attrs )*
        #[derive(Clone, Debug, #serde::Serialize)]
        #[serde(untagged)]
        #[allow(clippy::large_enum_variant)]
        #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
        pub enum #ident {
            #(
                #docs
                #variant_decls(#content),
            )*
            #[doc(hidden)]
            #[serde(serialize_with = #serialize_custom_event_error_path)]
            _Custom {
                event_type: crate::PrivOwnedStr,
            },
        }

        #[automatically_derived]
        impl #ruma_events::EventContentFromType for #ident {
            fn from_parts(event_type: &str, json: &#serde_json::value::RawValue) -> serde_json::Result<Self> {
                match event_type {
                    #event_type_match_arms

                    _ => {
                        Ok(Self::_Custom {
                            event_type: crate::PrivOwnedStr(
                                ::std::convert::From::from(event_type.to_owned())
                            )
                        })
                    }
                }
            }
        }

        #[automatically_derived]
        impl #ruma_events::#event_content_kind_trait_name for #ident {
            #state_event_content_impl

            fn event_type(&self) -> #ruma_events::#event_type_enum {
                match self {
                    #( #variant_arms(content) => content.event_type(), )*
                    Self::_Custom { event_type } => ::std::convert::From::from(&event_type.0[..]),
                }
            }
        }

        #from_impl
        #json_castable_impl
    })
}

/// Generate an `AnyFull*EventContent` enum.
pub fn expand_full_content_enum(
    kind: EventKind,
    events: &[EventEnumEntry],
    docs: &[TokenStream],
    attrs: &[Attribute],
    variants: &[EventEnumVariant],
    ruma_events: &TokenStream,
) -> syn::Result<TokenStream> {
    let ident = kind.to_full_content_enum();

    let event_type_enum = kind.to_event_type_enum();

    let content: Vec<_> = events.iter().map(|event| event.to_event_content_path(kind)).collect();

    let variant_decls = variants.iter().map(|v| v.decl()).collect::<Vec<_>>();
    let variant_arms = variants.iter().map(|v| v.match_arm(quote! { Self })).collect::<Vec<_>>();

    Ok(quote! {
        #( #attrs )*
        #[derive(Clone, Debug)]
        #[allow(clippy::large_enum_variant)]
        #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
        pub enum #ident {
            #(
                #docs
                #variant_decls(#ruma_events::FullStateEventContent<#content>),
            )*
            #[doc(hidden)]
            _Custom {
                event_type: crate::PrivOwnedStr,
                redacted: bool,
            },
        }

        impl #ident {
            /// Get the event’s type, like `m.room.create`.
            pub fn event_type(&self) -> #ruma_events::#event_type_enum {
                match self {
                    #( #variant_arms(content) => content.event_type(), )*
                    Self::_Custom { event_type, .. } => ::std::convert::From::from(&event_type.0[..]),
                }
            }
        }
    })
}

/// Implement `JsonCastable<{enum}>` for all the variants of an enum and `JsonCastable<JsonObject>`
/// for the enum.
fn expand_json_castable_impl(
    ty: &Ident,
    event_ty: &[TokenStream],
    variants: &[EventEnumVariant],
) -> TokenStream {
    let ruma_common = RumaCommon::new();

    // All event content types are represented as objects in JSON.
    let mut json_castable_impls = vec![quote! {
        #[automatically_derived]
        impl #ruma_common::serde::JsonCastable<#ruma_common::serde::JsonObject> for #ty {}
    }];

    json_castable_impls.extend(event_ty.iter().zip(variants).map(|(event_ty, variant)| {
        let attrs = &variant.attrs;

        quote! {
            #[allow(unused_qualifications)]
            #[automatically_derived]
            #(#attrs)*
            impl #ruma_common::serde::JsonCastable<#ty> for #event_ty {}
        }
    }));

    quote! { #( #json_castable_impls )* }
}
