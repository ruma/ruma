use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse_quote, Ident, LitStr};

use super::event_parse::{EventEnumEntry, EventEnumInput, EventKind};

pub fn expand_event_type_enum(
    input: EventEnumInput,
    ruma_common: TokenStream,
) -> syn::Result<TokenStream> {
    let mut timeline: Vec<&Vec<EventEnumEntry>> = vec![];
    let mut state: Vec<&Vec<EventEnumEntry>> = vec![];
    let mut message: Vec<&Vec<EventEnumEntry>> = vec![];
    let mut ephemeral: Vec<&Vec<EventEnumEntry>> = vec![];
    let mut room_account: Vec<&Vec<EventEnumEntry>> = vec![];
    let mut global_account: Vec<&Vec<EventEnumEntry>> = vec![];
    let mut to_device: Vec<&Vec<EventEnumEntry>> = vec![];
    for event in &input.enums {
        match event.kind {
            EventKind::GlobalAccountData => global_account.push(&event.events),
            EventKind::RoomAccountData => room_account.push(&event.events),
            EventKind::Ephemeral => ephemeral.push(&event.events),
            EventKind::MessageLike => {
                message.push(&event.events);
                timeline.push(&event.events);
            }
            EventKind::State => {
                state.push(&event.events);
                timeline.push(&event.events);
            }
            EventKind::ToDevice => to_device.push(&event.events),
            EventKind::RoomRedaction
            | EventKind::Presence
            | EventKind::Decrypted
            | EventKind::HierarchySpaceChild => {}
        }
    }
    let presence = vec![EventEnumEntry {
        attrs: vec![],
        aliases: vec![],
        ev_type: LitStr::new("m.presence", Span::call_site()),
        ev_path: parse_quote! { #ruma_common::events::presence },
        ident: None,
    }];
    let mut all = input.enums.iter().map(|e| &e.events).collect::<Vec<_>>();
    all.push(&presence);

    let mut res = TokenStream::new();

    res.extend(
        generate_enum("TimelineEventType", &timeline, &ruma_common)
            .unwrap_or_else(syn::Error::into_compile_error),
    );
    res.extend(
        generate_enum("StateEventType", &state, &ruma_common)
            .unwrap_or_else(syn::Error::into_compile_error),
    );
    res.extend(
        generate_enum("MessageLikeEventType", &message, &ruma_common)
            .unwrap_or_else(syn::Error::into_compile_error),
    );
    res.extend(
        generate_enum("EphemeralRoomEventType", &ephemeral, &ruma_common)
            .unwrap_or_else(syn::Error::into_compile_error),
    );
    res.extend(
        generate_enum("RoomAccountDataEventType", &room_account, &ruma_common)
            .unwrap_or_else(syn::Error::into_compile_error),
    );
    res.extend(
        generate_enum("GlobalAccountDataEventType", &global_account, &ruma_common)
            .unwrap_or_else(syn::Error::into_compile_error),
    );
    res.extend(
        generate_enum("ToDeviceEventType", &to_device, &ruma_common)
            .unwrap_or_else(syn::Error::into_compile_error),
    );

    Ok(res)
}

fn generate_enum(
    ident: &str,
    input: &[&Vec<EventEnumEntry>],
    ruma_common: &TokenStream,
) -> syn::Result<TokenStream> {
    let serde = quote! { #ruma_common::exports::serde };
    let enum_doc = format!("The type of `{}` this is.", ident.strip_suffix("Type").unwrap());

    let ident = Ident::new(ident, Span::call_site());

    let mut deduped: Vec<&EventEnumEntry> = vec![];
    for item in input.iter().copied().flatten() {
        if let Some(idx) = deduped.iter().position(|e| e.ev_type == item.ev_type) {
            // If there is a variant without config attributes use that
            if deduped[idx].attrs != item.attrs && item.attrs.is_empty() {
                deduped[idx] = item;
            }
        } else {
            deduped.push(item);
        }
    }

    let event_types = deduped.iter().map(|e| &e.ev_type);

    let variants: Vec<_> = deduped
        .iter()
        .map(|e| {
            let start = e.to_variant()?.decl();
            let data = e.has_type_fragment().then(|| quote! { (::std::string::String) });

            Ok(quote! {
                #start #data
            })
        })
        .collect::<syn::Result<_>>()?;

    let to_cow_str_match_arms: Vec<_> = deduped
        .iter()
        .map(|e| {
            let v = e.to_variant()?;
            let start = v.match_arm(quote! { Self });
            let ev_type = &e.ev_type;

            Ok(if let Some(prefix) = ev_type.value().strip_suffix(".*") {
                let fstr = prefix.to_owned() + ".{}";
                quote! { #start(_s) => ::std::borrow::Cow::Owned(::std::format!(#fstr, _s)) }
            } else {
                quote! { #start => ::std::borrow::Cow::Borrowed(#ev_type) }
            })
        })
        .collect::<syn::Result<_>>()?;

    let mut from_str_match_arms = TokenStream::new();
    for event in &deduped {
        let v = event.to_variant()?;
        let ctor = v.ctor(quote! { Self });
        let ev_types = event.aliases.iter().chain([&event.ev_type]);
        let attrs = &event.attrs;

        if event.ev_type.value().ends_with(".*") {
            for ev_type in ev_types {
                let name = ev_type.value();
                let prefix = name
                    .strip_suffix('*')
                    .expect("aliases have already been checked to have the same suffix");

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

    let from_ident_for_timeline = if ident == "StateEventType" || ident == "MessageLikeEventType" {
        let match_arms: Vec<_> = deduped
            .iter()
            .map(|e| {
                let v = e.to_variant()?;
                let ident_var = v.match_arm(quote! { #ident });
                let timeline_var = v.ctor(quote! { Self });

                Ok(if e.has_type_fragment() {
                    quote! { #ident_var (_s) => #timeline_var (_s) }
                } else {
                    quote! { #ident_var => #timeline_var }
                })
            })
            .collect::<syn::Result<_>>()?;

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
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
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
