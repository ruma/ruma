use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, LitStr};

use super::event_parse::{EventEnumEntry, EventEnumInput, EventKind};

pub fn expand_event_type_enum(
    input: EventEnumInput,
    ruma_common: TokenStream,
) -> syn::Result<TokenStream> {
    let mut room: Vec<&Vec<EventEnumEntry>> = vec![];
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
                room.push(&event.events);
            }
            EventKind::State => {
                state.push(&event.events);
                room.push(&event.events);
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
        ev_type: LitStr::new("m.presence", Span::call_site()),
    }];
    let mut all = input.enums.iter().map(|e| &e.events).collect::<Vec<_>>();
    all.push(&presence);

    let mut res = TokenStream::new();

    res.extend(generate_enum("EventType", &all, &ruma_common)?);
    res.extend(generate_enum("RoomEventType", &room, &ruma_common)?);
    res.extend(generate_enum("StateEventType", &state, &ruma_common)?);
    res.extend(generate_enum("MessageLikeEventType", &message, &ruma_common)?);
    res.extend(generate_enum("EphemeralRoomEventType", &ephemeral, &ruma_common)?);
    res.extend(generate_enum("RoomAccountDataEventType", &room_account, &ruma_common)?);
    res.extend(generate_enum("GlobalAccountDataEventType", &global_account, &ruma_common)?);
    res.extend(generate_enum("ToDeviceEventType", &to_device, &ruma_common)?);

    Ok(res)
}

fn generate_enum(
    ident: &str,
    input: &[&Vec<EventEnumEntry>],
    ruma_common: &TokenStream,
) -> syn::Result<TokenStream> {
    let serde = quote! { #ruma_common::exports::serde };
    let enum_doc = format!("The type of `{}` this is.", ident.strip_suffix("Type").unwrap());

    let deprecated_attr = (ident == "EventType").then(|| {
        quote! {
            #[deprecated = "use a fine-grained event type enum like RoomEventType instead"]
        }
    });

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
                let fstr = prefix.to_owned() + "{}";
                quote! { #start(_s) => ::std::borrow::Cow::Owned(::std::format!(#fstr, _s)) }
            } else {
                quote! { #start => ::std::borrow::Cow::Borrowed(#ev_type) }
            })
        })
        .collect::<syn::Result<_>>()?;

    let from_str_match_arms: Vec<_> = deduped
        .iter()
        .map(|e| {
            let v = e.to_variant()?;
            let ctor = v.ctor(quote! { Self });

            let match_arm = if let Some(prefix) = e.ev_type.value().strip_suffix('*') {
                quote! {
                    // Use if-let guard once available
                    _s if _s.starts_with(#prefix) => {
                        #ctor(::std::convert::From::from(_s.strip_prefix(#prefix).unwrap()))
                    }
                }
            } else {
                let t = &e.ev_type;
                quote! { #t => #ctor }
            };

            let attrs = &e.attrs;
            Ok(quote! { #(#attrs)* #match_arm })
        })
        .collect::<syn::Result<_>>()?;

    Ok(quote! {
        #[doc = #enum_doc]
        ///
        /// This type can hold an arbitrary string. To check for events that are not available as a
        /// documented variant here, use its string representation, obtained through `.as_str()`.
        #deprecated_attr
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
        impl ::std::convert::From<&::std::primitive::str> for #ident {
            fn from(s: &::std::primitive::str) -> Self {
                match s {
                    #(#from_str_match_arms,)*
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
    })
}
