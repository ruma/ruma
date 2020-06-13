//! Implementation of the top level `Any*Event` derive macro.
//!
//! This is just a custom `Deserialize` impl made into a derive to avoid
//! code duplication.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DataEnum, DeriveInput, Ident};

/// Derive `AnyEventDeserialize` macro code generation.
pub fn expand_any_event_deserialize(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let variant_idents = if let Data::Enum(DataEnum { variants, .. }) = input.data.clone() {
        variants.into_iter().map(|v| v.ident).collect::<Vec<_>>()
    } else {
        return Err(syn::Error::new(
            Span::call_site(),
            "the `AnyEventDeserialize` derive only supports enums",
        ));
    };

    let match_block = variant_idents.iter().map(match_event_type).collect::<Vec<_>>();

    let deserialize_impl = quote! {
        impl<'de> ::serde::de::Deserialize<'de> for #ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::de::Deserializer<'de>,
            {
                use ::serde::de::Error as _;

                let json = ::serde_json::Value::deserialize(deserializer)?;
                let ev_type: String = ::ruma_events::util::get_field(&json, "type")?;

                match ev_type.as_str() {
                    #( #match_block )*
                    _ => Err(D::Error::custom(format!("event type `{}` is not a valid event", ev_type))),
                }
            }
        }
    };

    Ok(quote! {
        #deserialize_impl
    })
}

/// Match the variant name with the grouping of events and return the deserialized event
/// wrapped in the correct variant.
fn match_event_type(variant: &Ident) -> TokenStream {
    let deserialize_event = quote! {
        Ok(Self::#variant(::serde_json::from_value(json).map_err(D::Error::custom)?))
    };
    match variant.to_string().as_str() {
        "Basic" => quote! {
            "m.direct"
            | "m.dummy"
            | "m.ignored_user_list"
            | "m.push_rules"
            | "m.room_key"
            | "m.tag" => {
                #deserialize_event
            }
        },
        "Presence" => quote! {
            "m.presence" => #deserialize_event,
        },
        "Redaction" => quote! {
            "m.room.redaction" => #deserialize_event,
        },
        "Ephemeral" => quote! {
            "m.fully_read" | "m.receipt" | "m.typing" => {
                #deserialize_event
            }
        },
        "Message" => quote! {
            "m.call.answer"
            | "m.call.invite"
            | "m.call.hangup"
            | "m.call.candidates"
            | "m.room.encrypted"
            | "m.room.message"
            | "m.room.message.feedback"
            | "m.sticker" => {
                #deserialize_event
            }
        },
        "State" | "StateEvent" => quote! {
            "m.room.aliases"
            | "m.room.avatar"
            | "m.room.canonical_alias"
            | "m.room.create"
            | "m.room.encryption"
            | "m.room.guest_access"
            | "m.room.history_visibility"
            | "m.room.join_rules"
            | "m.room.member"
            | "m.room.name"
            | "m.room.pinned_events"
            | "m.room.power_levels"
            | "m.room.redaction"
            | "m.room.server_acl"
            | "m.room.third_party_invite"
            | "m.room.tombstone"
            | "m.room.topic" => {
                #deserialize_event
            }
        },
        _ => TokenStream::new(),
    }
}
