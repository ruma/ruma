//! Functions to generate `Any*EventContent` enums.

use std::ops::Deref;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};

use crate::{
    events::{
        common::EventContentTraitVariation,
        event_enum::{EventEnum, EventEnumKind},
    },
    util::RumaEventsReexport,
};

/// A list of `Any*EventContent` enums.
pub(super) struct EventContentEnums<'a> {
    inner: &'a EventEnum<'a>,

    /// The `Any*EventContent` enum, if any.
    any_enum: Option<EventContentEnum<'a>>,

    /// The special `AnyFullStateEventContent` enum.
    full: Option<FullEventContentEnum<'a>>,
}

impl<'a> EventContentEnums<'a> {
    /// Construct an `EventContentEnums` with the given event enum data.
    pub(super) fn new(inner: &'a EventEnum<'a>) -> Self {
        Self { inner, any_enum: None, full: None }
    }

    /// Get the [`EventContentEnum`].
    ///
    /// If it doesn't exist in this list, it is created.
    pub(super) fn any_enum(&mut self) -> Option<&EventContentEnum<'a>> {
        Some(self.any_enum.get_or_insert_with(|| EventContentEnum::new(self.inner)))
    }

    /// Get the [`FullEventContentEnum`].
    ///
    /// If it doesn't exist in this list, it is created.
    pub(super) fn full_event_content_enum(&mut self) -> &FullEventContentEnum<'a> {
        self.full.get_or_insert_with(|| FullEventContentEnum::new(self.inner))
    }

    /// Expand the event content enums in this list.
    pub(super) fn expand(&self) -> TokenStream {
        self.any_enum
            .iter()
            .map(EventContentEnum::expand)
            .chain(self.full.iter().map(FullEventContentEnum::expand))
            .collect()
    }
}

/// The data for an `Any*EventContent` enum.
pub(super) struct EventContentEnum<'a> {
    /// The event enum data.
    inner: &'a EventEnum<'a>,

    /// The name of this enum.
    ident: syn::Ident,

    /// The paths to the `*EventContent` types of the entries.
    event_content_types: Vec<syn::Path>,
}

impl<'a> EventContentEnum<'a> {
    fn new(inner: &'a EventEnum<'a>) -> Self {
        let kind = inner.kind;
        let event_content_types =
            inner.events.iter().map(|event| event.to_event_content_path(kind)).collect();

        Self { inner, ident: format_ident!("Any{kind}Content"), event_content_types }
    }
}

impl EventContentEnum<'_> {
    /// Generate this `Any*EventContent` enum.
    fn expand(&self) -> TokenStream {
        let ruma_events = self.ruma_events;
        let serde = ruma_events.reexported(RumaEventsReexport::Serde);

        let attrs = &self.attrs;
        let ident = &self.ident;
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

        let ident = &self.ident;
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

        let ident = &self.ident;
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

    /// Implement `JsonCastable<{enum}>` for all the variants and `JsonCastable<JsonObject>` for the
    /// given event content enum.
    fn expand_content_enum_json_castable_impl(&self) -> TokenStream {
        let ruma_common = self.ruma_events.ruma_common();
        let ident = &self.ident;

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

    /// Generate the accessors on an event enum to get the event content.
    pub(super) fn expand_content_accessors(&self, maybe_redacted: bool) -> TokenStream {
        let ruma_events = self.ruma_events;

        let ident = &self.ident;
        let variants = &self.variants;
        let variant_attrs = &self.variant_attrs;

        let event_content_kind_trait =
            self.kind.to_content_kind_trait(EventContentTraitVariation::Original);

        if maybe_redacted {
            quote! {
                /// Returns the content for this event if it is not redacted, or `None` if it is.
                pub fn original_content(&self) -> Option<#ident> {
                    match self {
                        #(
                            #( #variant_attrs )*
                            Self::#variants(event) => {
                                event.as_original().map(|ev| #ident::#variants(ev.content.clone()))
                            }
                        )*
                        Self::_Custom(event) => event.as_original().map(|ev| {
                            #ident::_Custom {
                                event_type: crate::PrivOwnedStr(
                                    ::std::convert::From::from(
                                        ::std::string::ToString::to_string(
                                            &#ruma_events::#event_content_kind_trait::event_type(
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
            }
        } else {
            quote! {
                /// Returns the content for this event.
                pub fn content(&self) -> #ident {
                    match self {
                        #(
                            #( #variant_attrs )*
                            Self::#variants(event) => #ident::#variants(event.content.clone()),
                        )*
                        Self::_Custom(event) => #ident::_Custom {
                            event_type: crate::PrivOwnedStr(
                                ::std::convert::From::from(
                                    ::std::string::ToString::to_string(
                                        &#ruma_events::#event_content_kind_trait::event_type(&event.content)
                                    )
                                ),
                            ),
                        },
                    }
                }
            }
        }
    }
}

impl<'a> Deref for EventContentEnum<'a> {
    type Target = EventEnum<'a>;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

/// The data for the `AnyFullStateEventContent` enum.
pub(super) struct FullEventContentEnum<'a> {
    /// The event enum data.
    inner: &'a EventEnum<'a>,

    /// The name of the enum.
    ident: syn::Ident,

    /// The paths to the `*EventContent` types of the entries.
    event_content_types: Vec<syn::Path>,
}

impl<'a> FullEventContentEnum<'a> {
    /// Construct a `FullEventContentEnum` with the given event enum data.
    fn new(inner: &'a EventEnum<'a>) -> Self {
        let ident = syn::Ident::new("AnyFullStateEventContent", Span::call_site());
        let kind = inner.kind;
        let event_content_types =
            inner.events.iter().map(|event| event.to_event_content_path(kind)).collect();

        Self { inner, ident, event_content_types }
    }
}

impl FullEventContentEnum<'_> {
    /// Generate the `AnyFullStateEventContent` enum.
    fn expand(&self) -> TokenStream {
        let ruma_events = self.ruma_events;

        let attrs = &self.attrs;
        let ident = &self.ident;
        let event_type_enum = &self.event_type_enum;
        let variants = &self.variants;
        let variant_attrs = &self.variant_attrs;
        let variant_docs = &self.variant_docs;
        let event_content_types = &self.event_content_types;

        let event_content_kind_trait =
            self.kind.to_content_kind_trait(EventContentTraitVariation::Original);

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
                },
            }

            impl #ident {
                /// Get the event’s type, like `m.room.create`.
                pub fn event_type(&self) -> #ruma_events::#event_type_enum {
                    match self {
                        #(
                            #( #variant_attrs )*
                            Self::#variants(event) => #ruma_events::#event_content_kind_trait::event_type(&event.content),
                        )*
                        Self::_Custom { event_type } => ::std::convert::From::from(&event_type.0[..]),
                    }
                }
            }
        }
    }

    /// Generate the accessors on an event enum to get the event content.
    pub(super) fn expand_content_accessors(&self) -> TokenStream {
        let ruma_events = self.ruma_events;

        let ident = &self.ident;
        let variants = &self.variants;
        let variant_attrs = &self.variant_attrs;

        let event_content_kind_trait =
            self.kind.to_content_kind_trait(EventContentTraitVariation::Original);

        quote! {
            /// Returns the content and optional previous content of this state event.
            pub fn full_content(&self) -> #ident {
                match self {
                    #(
                        #( #variant_attrs )*
                        Self::#variants(event) => {
                            #ident::#variants(
                                #ruma_events::FullStateEventContent {
                                    content: event.content.clone(),
                                    prev_content: event.unsigned.prev_content.clone()
                                }
                            )
                        }
                    )*
                    Self::_Custom(event) =>  {
                        #ident::_Custom {
                            event_type: crate::PrivOwnedStr(
                                ::std::string::ToString::to_string(
                                    &#ruma_events::#event_content_kind_trait::event_type(
                                        &event.content,
                                    ),
                                ).into_boxed_str(),
                            ),
                        }
                    }
                }
            }
        }
    }
}

impl<'a> Deref for FullEventContentEnum<'a> {
    type Target = EventEnum<'a>;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}
