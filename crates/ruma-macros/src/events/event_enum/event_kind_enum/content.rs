//! Functions to generate `Any*EventContent` enums.

use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    events::{
        common::EventContentTraitVariation,
        event_enum::{EventEnum, EventEnumKind},
    },
    util::RumaEventsReexport,
};

impl EventEnum<'_> {
    /// Generate the `Any*EventContent` enum for this kind.
    pub(super) fn expand_content_enum(&self) -> TokenStream {
        let ruma_events = self.ruma_events;
        let serde = ruma_events.reexported(RumaEventsReexport::Serde);

        let attrs = &self.attrs;
        let ident = &self.content_enum;
        let variants = &self.variants;
        let variant_attrs = &self.variant_attrs;
        let variant_docs = &self.variant_docs;
        let event_content_types = &self.event_content_types;

        let event_content_from_type_impl = self.expand_content_enum_event_content_from_type_impl();
        let event_content_kind_trait_impl =
            self.expand_content_enum_event_content_kind_trait_impl();
        let from_impl = self.expand_from_impl(ident, event_content_types);
        let json_castable_impl = self.expand_content_enum_json_castable_impl();

        // We need this path as a string.
        let serialize_custom_event_error_path =
            quote! { #ruma_events::serialize_custom_event_error }.to_string();

        quote! {
            #( #attrs )*
            #[derive(Clone, Debug, #serde::Serialize)]
            #[serde(untagged)]
            #[allow(clippy::large_enum_variant, unused_qualifications)]
            #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
            pub enum #ident {
                #(
                    #variant_docs
                    #( #variant_attrs )*
                    #variants(#event_content_types),
                )*
                #[doc(hidden)]
                #[serde(serialize_with = #serialize_custom_event_error_path)]
                _Custom {
                    event_type: crate::PrivOwnedStr,
                },
            }

            #event_content_from_type_impl
            #event_content_kind_trait_impl
            #from_impl
            #json_castable_impl
        }
    }

    /// Generate the `ruma_events::EventContentFromType` implementation for the
    /// `Any*EventContent` enum.
    fn expand_content_enum_event_content_from_type_impl(&self) -> TokenStream {
        let ruma_events = self.ruma_events;
        let serde_json = ruma_events.reexported(RumaEventsReexport::SerdeJson);

        let ident = &self.content_enum;
        let variants = &self.variants;
        let event_attrs = &self.variant_attrs;
        let event_content_types = &self.event_content_types;
        let event_type_string_match_arms = &self.event_type_string_match_arms;

        let deserialize_event_contents = self.events.iter().zip(event_content_types.iter()).map(
            |(event, event_content_type)| {
                if event.has_type_fragment() {
                    // If the event has a type fragment, then it implements EventContentFromType
                    // itself; see `generate_event_content_impl` which does that. In this case,
                    // forward to its implementation.
                    quote! {
                        #event_content_type::from_parts(event_type, json)?
                    }
                } else {
                    // The event doesn't have a type fragment, so it *should* implement
                    // Deserialize: use that here.
                    quote! {
                        #serde_json::from_str(json.get())?
                    }
                }
            },
        );

        quote! {
            #[automatically_derived]
            impl #ruma_events::EventContentFromType for #ident {
                fn from_parts(event_type: &str, json: &#serde_json::value::RawValue) -> serde_json::Result<Self> {
                    match event_type {
                        #(
                            #( #event_attrs )*
                            #( #event_type_string_match_arms )|* => {
                                let content = #deserialize_event_contents;
                                Ok(Self::#variants(content))
                            },
                        )*
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
        }
    }

    /// Generate the `ruma_events::{kind}EventContent` trait implementation for the
    /// `Any*EventContent` enum.
    fn expand_content_enum_event_content_kind_trait_impl(&self) -> TokenStream {
        let ruma_events = self.ruma_events;

        let ident = &self.content_enum;
        let event_type_enum = &self.event_type_enum;
        let variants = &self.variants;
        let variant_attrs = &self.variant_attrs;

        let event_content_kind_trait =
            self.kind.to_content_kind_trait(EventContentTraitVariation::Original);
        let extra_event_content_impl = (self.kind == EventEnumKind::State).then(|| {
            quote! {
                type StateKey = String;
            }
        });

        quote! {
            #[automatically_derived]
            impl #ruma_events::#event_content_kind_trait for #ident {
                #extra_event_content_impl

                fn event_type(&self) -> #ruma_events::#event_type_enum {
                    match self {
                        #(
                            #( #variant_attrs )*
                            Self::#variants(content) => content.event_type(),
                        )*
                        Self::_Custom { event_type } => ::std::convert::From::from(&event_type.0[..]),
                    }
                }
            }
        }
    }

    /// Generate an `AnyFull*EventContent` enum.
    pub(super) fn expand_full_content_enum(&self) -> TokenStream {
        let ruma_events = self.ruma_events;

        let attrs = &self.attrs;
        let ident = &self.full_content_enum;
        let event_type_enum = &self.event_type_enum;
        let variants = &self.variants;
        let variant_attrs = &self.variant_attrs;
        let variant_docs = &self.variant_docs;
        let event_content_types = &self.event_content_types;

        quote! {
            #( #attrs )*
            #[derive(Clone, Debug)]
            #[allow(clippy::large_enum_variant, unused_qualifications)]
            #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
            pub enum #ident {
                #(
                    #variant_docs
                    #( #variant_attrs )*
                    #variants(#ruma_events::FullStateEventContent<#event_content_types>),
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
                        #(
                            #( #variant_attrs )*
                            Self::#variants(content) => content.event_type(),
                        )*
                        Self::_Custom { event_type, .. } => ::std::convert::From::from(&event_type.0[..]),
                    }
                }
            }
        }
    }

    /// Implement `JsonCastable<{enum}>` for all the variants and `JsonCastable<JsonObject>` for the
    /// given event content enum.
    fn expand_content_enum_json_castable_impl(&self) -> TokenStream {
        let ruma_common = self.ruma_events.ruma_common();
        let ident = &self.content_enum;

        // All event content types are represented as objects in JSON.
        let mut json_castable_impls = quote! {
            #[automatically_derived]
            impl #ruma_common::serde::JsonCastable<#ruma_common::serde::JsonObject> for #ident {}
        };

        json_castable_impls.extend(
            self.event_content_types.iter().zip(self.variant_attrs.iter()).map(
                |(event_content_type, variant_attrs)| {
                    quote! {
                        #[allow(unused_qualifications)]
                        #[automatically_derived]
                        #( #variant_attrs )*
                        impl #ruma_common::serde::JsonCastable<#ident> for #event_content_type {}
                    }
                },
            ),
        );

        json_castable_impls
    }
}
