//! Functions to generate the `*EventType` enums.

use std::ops::Deref;

use proc_macro2::TokenStream;
use quote::quote;

use super::{EventEnumData, EventEnumKind};
use crate::util::{RumaEvents, RumaEventsReexport};

/// Data to generate an `*EventType` enum.
pub(super) struct EventTypeEnum<'a> {
    /// The data for the enum.
    data: &'a EventEnumData,

    /// The import path for the ruma-events crate.
    ruma_events: &'a RumaEvents,

    /// The import path for the serde crate.
    serde: TokenStream,

    /// The name of the event type enum
    ident: syn::Ident,
}

impl<'a> EventTypeEnum<'a> {
    /// Create an `EventTypeEnum` with the given data.
    pub(super) fn new(data: &'a EventEnumData, ruma_events: &'a RumaEvents) -> Self {
        let serde = ruma_events.reexported(RumaEventsReexport::Serde);

        let ident = data.kind.to_event_type_enum();

        Self { data, ruma_events, ident, serde }
    }
}

impl EventTypeEnum<'_> {
    /// Generate the `*EventType` enum and its implementations.
    pub(super) fn expand(&self) -> syn::Result<TokenStream> {
        let ident = &self.ident;
        let enum_doc = format!("The type of `{}` this is.", self.kind);

        let variants = self.events.iter().map(|event| {
            let variant = &event.ident;
            let variant_attrs = &event.attrs;
            let variant_docs = event.docs();

            if event.has_type_fragment() {
                quote! {
                    #variant_docs
                    #( #variant_attrs )*
                    #variant(::std::string::String),
                }
            } else {
                quote! {
                    #variant_docs
                    #( #variant_attrs )*
                    #variant,
                }
            }
        });

        let ord_impl = self.expand_ord_impl();
        let to_string_impl = self.expand_to_string_impl();
        let from_string_impl = self.expand_from_string_impl();
        let into_timeline_event_type_impl = self.expand_into_timeline_event_type_impl();

        Ok(quote! {
            #[doc = #enum_doc]
            ///
            /// This type can hold an arbitrary string. To build events with a custom type, convert it
            /// from a string with `::from()` / `.into()`. To check for events that are not available as a
            /// documented variant here, use its string representation, obtained through `.to_string()`.
            #[derive(Clone, PartialEq, Eq, Hash)]
            #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
            pub enum #ident {
                #( #variants )*
                #[doc(hidden)]
                _Custom(crate::PrivOwnedStr),
            }

            #ord_impl
            #to_string_impl
            #from_string_impl
            #into_timeline_event_type_impl
        })
    }

    /// Generate the `Ord` and `PartialOrd` implementations for the event type enum.
    ///
    /// To compare event types we need to compare the static event type first, and then the "type
    /// fragment" if there is one.
    fn expand_ord_impl(&self) -> TokenStream {
        let ident = &self.ident;

        let event_type_str_match_arms = self.events.iter().map(|event| {
            let variant = &event.ident;
            let variant_attrs = &event.attrs;
            let ev_type = &event.types.ev_type;

            if ev_type.is_prefix() {
                let ev_type = ev_type.without_wildcard();
                quote! {
                    #( #variant_attrs )*
                    Self::#variant(_s) => #ev_type,
                }
            } else {
                quote! {
                    #( #variant_attrs )*
                    Self::#variant => #ev_type,
                }
            }
        });

        let mut type_fragment_match_arms = self
            .events
            .iter()
            // We only need to compare types with fragment, others will be equal.
            .filter(|event| event.has_type_fragment())
            .map(|event| {
                let variant = &event.ident;
                let variant_attrs = &event.attrs;

                quote! {
                    #( #variant_attrs )*
                    (Self::#variant(this), Self::#variant(other)) => this.cmp(other),
                }
            })
            .peekable();

        let cmp_type_fragment_impl = if type_fragment_match_arms.peek().is_none() {
            // If there are no type fragments, all variants are equal.
            quote! { ::std::cmp::Ordering::Equal }
        } else {
            quote! {
                match (self, other) {
                    #( #type_fragment_match_arms )*
                    _ => ::std::cmp::Ordering::Equal,
                }
            }
        };

        quote! {
            #[allow(deprecated)]
            impl #ident {
                fn event_type_str(&self) -> &::std::primitive::str {
                    match self {
                        #( #event_type_str_match_arms )*
                        Self::_Custom(crate::PrivOwnedStr(s)) => s,
                    }
                }

                fn cmp_type_fragment(&self, other: &Self) -> ::std::cmp::Ordering {
                    #cmp_type_fragment_impl
                }
            }

            impl ::std::cmp::Ord for #ident {
                fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                    let event_type_cmp = self.event_type_str().cmp(&other.event_type_str());

                    if event_type_cmp.is_eq() {
                        self.cmp_type_fragment(other)
                    } else {
                        event_type_cmp
                    }
                }
            }

            impl ::std::cmp::PartialOrd for #ident {
                fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
                    Some(self.cmp(other))
                }
            }
        }
    }

    /// Generate the `std::fmt::Display`, `std::fmt::Debug` and `serde::Serialize` implementations
    /// for the event type enum.
    fn expand_to_string_impl(&self) -> TokenStream {
        let ident = &self.ident;
        let serde = &self.serde;

        let match_arms = self.events
            .iter()
            .map(|event| {
                let variant = &event.ident;
                let variant_attrs = &event.attrs;
                let ev_type = &event.types.ev_type;

                if ev_type.is_prefix() {
                    let format_str = ev_type.without_wildcard().to_owned() + "{}";
                    quote! {
                        #( #variant_attrs )*
                        Self::#variant(_s) => ::std::borrow::Cow::Owned(::std::format!(#format_str, _s)),
                    }
                } else {
                    quote! {
                        #( #variant_attrs )*
                        Self::#variant => ::std::borrow::Cow::Borrowed(#ev_type),
                    }
                }
            });

        quote! {
            #[allow(deprecated)]
            impl #ident {
                fn to_cow_str(&self) -> ::std::borrow::Cow<'_, ::std::primitive::str> {
                    match self {
                        #( #match_arms )*
                        Self::_Custom(crate::PrivOwnedStr(s)) => ::std::borrow::Cow::Borrowed(s),
                    }
                }
            }

            #[allow(deprecated)]
            impl ::std::fmt::Display for #ident {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    self.to_cow_str().fmt(f)
                }
            }

            #[allow(deprecated)]
            impl ::std::fmt::Debug for #ident {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    <str as ::std::fmt::Debug>::fmt(&self.to_cow_str(), f)
                }
            }

            #[allow(deprecated)]
            impl #serde::Serialize for #ident {
                fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
                where
                    S: #serde::Serializer,
                {
                    self.to_cow_str().serialize(serializer)
                }
            }
        }
    }

    /// Generate the `From<&str>`, `From<String>` and `serde::Deserialize` implementations for the
    /// event type enum.
    fn expand_from_string_impl(&self) -> TokenStream {
        let ident = &self.ident;
        let ruma_common = self.ruma_events.ruma_common();
        let serde = &self.serde;

        let from_str_match_arms = self.events.iter().map(|event| {
            let variant = &event.ident;
            let variant_attrs = &event.attrs;
            let ev_types = event.types.iter();

            if event.has_type_fragment() {
                ev_types.map(|ev_type| {
                    let prefix = ev_type.without_wildcard();

                    quote! {
                        #( #variant_attrs )*
                        // Use if-let guard once available
                        s if s.starts_with(#prefix) => {
                            Self::#variant(::std::convert::From::from(s.strip_prefix(#prefix).unwrap()))
                        }
                    }
                }).collect()
            } else {
                quote! {
                    #( #variant_attrs )*
                    #( #ev_types )|* => Self::#variant,
                }
            }
        });

        quote! {
            #[allow(deprecated)]
            impl ::std::convert::From<&::std::primitive::str> for #ident {
                fn from(s: &::std::primitive::str) -> Self {
                    match s {
                        #( #from_str_match_arms )*
                        _ => Self::_Custom(crate::PrivOwnedStr(::std::convert::From::from(s))),
                    }
                }
            }

            #[allow(deprecated)]
            impl ::std::convert::From<::std::string::String> for #ident {
                fn from(s: ::std::string::String) -> Self {
                    ::std::convert::From::from(s.as_str())
                }
            }

            #[allow(deprecated)]
            impl<'de> #serde::Deserialize<'de> for #ident {
                fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
                where
                    D: #serde::Deserializer<'de>
                {
                    let s = #ruma_common::serde::deserialize_cow_str(deserializer)?;
                    Ok(::std::convert::From::from(&s[..]))
                }
            }
        }
    }

    /// Generate the `From<{ident}> for TimelineEventType` implementation for the timeline kinds.
    fn expand_into_timeline_event_type_impl(&self) -> Option<TokenStream> {
        if !self.kind.is_timeline() || self.kind == EventEnumKind::Timeline {
            return None;
        }

        let ident = &self.ident;

        let match_arms = self.events.iter().map(|event| {
            let variant = &event.ident;
            let variant_attrs = &event.attrs;

            if event.has_type_fragment() {
                quote! {
                    #( #variant_attrs )*
                    #ident::#variant(s) => Self::#variant(s),
                }
            } else {
                quote! {
                    #( #variant_attrs )*
                    #ident::#variant => Self::#variant,
                }
            }
        });

        Some(quote! {
            #[allow(deprecated)]
            impl ::std::convert::From<#ident> for TimelineEventType {
                fn from(s: #ident) -> Self {
                    match s {
                        #( #match_arms )*
                        #ident ::_Custom(_s) => Self::_Custom(_s),
                    }
                }
            }
        })
    }
}

impl Deref for EventTypeEnum<'_> {
    type Target = EventEnumData;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}
