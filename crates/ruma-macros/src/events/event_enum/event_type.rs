//! Functions to generate `*EventType` enums.

use std::collections::BTreeMap;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::{EventEnumEntry, EventEnumInput};
use crate::{
    events::enums::EventKind,
    util::{RumaCommon, RumaCommonReexport},
};

/// Generate the `*EventType` enums.
pub fn expand_event_type_enums(
    input: EventEnumInput,
    ruma_common: &RumaCommon,
) -> syn::Result<TokenStream> {
    let mut entries_map: BTreeMap<EventKind, Vec<&Vec<EventEnumEntry>>> = BTreeMap::new();

    for event in &input.enums {
        if event.events.is_empty() {
            continue;
        }

        entries_map.entry(event.kind).or_default().push(&event.events);

        if event.kind.is_timeline() {
            entries_map.entry(EventKind::Timeline).or_default().push(&event.events);
        }
    }

    let mut res = TokenStream::new();

    for (kind, entries) in entries_map {
        res.extend(
            generate_enum(kind, &entries, ruma_common)
                .unwrap_or_else(syn::Error::into_compile_error),
        );
    }

    Ok(res)
}

/// Generate an `*EventType` enum.
fn generate_enum(
    kind: EventKind,
    entries: &[&Vec<EventEnumEntry>],
    ruma_common: &RumaCommon,
) -> syn::Result<TokenStream> {
    let serde = ruma_common.reexported(RumaCommonReexport::Serde);
    let enum_doc = format!("The type of `{kind}` this is.");

    let ident = format_ident!("{kind}Type");

    let mut deduped: Vec<&EventEnumEntry> = vec![];
    for item in entries.iter().copied().flatten() {
        if let Some(idx) = deduped.iter().position(|e| e.types.ev_type == item.types.ev_type) {
            // If there is a variant without config attributes use that
            if deduped[idx].attrs != item.attrs && item.attrs.is_empty() {
                deduped[idx] = item;
            }
        } else {
            deduped.push(item);
        }
    }

    let event_types = deduped.iter().map(|e| &e.types.ev_type);

    let variants: Vec<_> = deduped
        .iter()
        .map(|e| {
            let start = e.to_variant().decl();
            let data = e.has_type_fragment().then(|| quote! { (::std::string::String) });

            quote! {
                #start #data
            }
        })
        .collect();

    let event_type_str_match_arms: Vec<_> = deduped
        .iter()
        .map(|e| {
            let v = e.to_variant();
            let start = v.match_arm(quote! { Self });
            let ev_type = &e.types.ev_type;

            if ev_type.is_prefix() {
                let ev_type = ev_type.without_wildcard();
                quote! { #start(_s) => #ev_type }
            } else {
                quote! { #start => #ev_type }
            }
        })
        .collect();

    let cmp_type_fragment_match_arms: Vec<_> = deduped
        .iter()
        // We only need to compare types with fragment, others will be equal.
        .filter(|e| e.has_type_fragment())
        .map(|e| {
            let v = e.to_variant();
            let start = v.match_arm(quote! { Self });

            quote! { (#start(this), #start(other)) => this.cmp(other) }
        })
        .collect();

    let cmp_type_fragment_impl = if cmp_type_fragment_match_arms.is_empty() {
        quote! { ::std::cmp::Ordering::Equal }
    } else {
        quote! {
            match (self, other) {
                #(#cmp_type_fragment_match_arms,)*
                _ => ::std::cmp::Ordering::Equal,
            }
        }
    };

    let to_cow_str_match_arms: Vec<_> = deduped
        .iter()
        .map(|e| {
            let v = e.to_variant();
            let start = v.match_arm(quote! { Self });
            let ev_type = &e.types.ev_type;

            if ev_type.is_prefix() {
                let fstr = ev_type.without_wildcard().to_owned() + "{}";
                quote! { #start(_s) => ::std::borrow::Cow::Owned(::std::format!(#fstr, _s)) }
            } else {
                quote! { #start => ::std::borrow::Cow::Borrowed(#ev_type) }
            }
        })
        .collect();

    let mut from_str_match_arms = TokenStream::new();
    for event in &deduped {
        let v = event.to_variant();
        let ctor = v.ctor(quote! { Self });
        let ev_types = event.types.iter();
        let attrs = &event.attrs;

        if event.has_type_fragment() {
            for ev_type in ev_types {
                let prefix = ev_type.without_wildcard();

                from_str_match_arms.extend(quote! {
                    #(#attrs)*
                    // Use if-let guard once available
                    _s if _s.starts_with(#prefix) => {
                        #ctor(::std::convert::From::from(_s.strip_prefix(#prefix).unwrap()))
                    }
                });
            }
        } else {
            from_str_match_arms.extend(quote! { #(#attrs)* #(#ev_types)|* => #ctor, });
        }
    }

    let from_ident_for_timeline = if kind.is_timeline() && !matches!(kind, EventKind::Timeline) {
        let match_arms = deduped.iter().map(|e| {
            let v = e.to_variant();
            let ident_var = v.match_arm(quote! { #ident });
            let timeline_var = v.ctor(quote! { Self });

            if e.has_type_fragment() {
                quote! { #ident_var (_s) => #timeline_var (_s) }
            } else {
                quote! { #ident_var => #timeline_var }
            }
        });

        Some(quote! {
            #[allow(deprecated)]
            impl ::std::convert::From<#ident> for TimelineEventType {
                fn from(s: #ident) -> Self {
                    match s {
                        #(#match_arms,)*
                        #ident ::_Custom(_s) => Self::_Custom(_s),
                    }
                }
            }
        })
    } else {
        None
    };

    Ok(quote! {
        #[doc = #enum_doc]
        ///
        /// This type can hold an arbitrary string. To build events with a custom type, convert it
        /// from a string with `::from()` / `.into()`. To check for events that are not available as a
        /// documented variant here, use its string representation, obtained through `.to_string()`.
        #[derive(Clone, PartialEq, Eq, Hash)]
        #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
        pub enum #ident {
            #(
                #[doc = #event_types]
                #variants,
            )*
            #[doc(hidden)]
            _Custom(crate::PrivOwnedStr),
        }

        #[allow(deprecated)]
        impl #ident {
            fn event_type_str(&self) -> &::std::primitive::str {
                match self {
                    #(#event_type_str_match_arms,)*
                    Self::_Custom(crate::PrivOwnedStr(s)) => s,
                }
            }

            fn cmp_type_fragment(&self, other: &Self) -> ::std::cmp::Ordering {
                #cmp_type_fragment_impl
            }

            fn to_cow_str(&self) -> ::std::borrow::Cow<'_, ::std::primitive::str> {
                match self {
                    #(#to_cow_str_match_arms,)*
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
        impl ::std::convert::From<&::std::primitive::str> for #ident {
            fn from(s: &::std::primitive::str) -> Self {
                match s {
                    #from_str_match_arms
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

        #[allow(deprecated)]
        impl #serde::Serialize for #ident {
            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
            where
                S: #serde::Serializer,
            {
                self.to_cow_str().serialize(serializer)
            }
        }

        #from_ident_for_timeline
    })
}
