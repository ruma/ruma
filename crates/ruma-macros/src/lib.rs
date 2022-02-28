#![doc(html_favicon_url = "https://www.ruma.io/favicon.ico")]
#![doc(html_logo_url = "https://www.ruma.io/images/logo.png")]
//! Procedural macros used by ruma crates.
//!
//! See the documentation for the individual macros for usage details.

#![warn(missing_docs)]

use proc_macro::TokenStream;
use proc_macro2 as pm2;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};
use ruma_identifiers_validation::{
    device_key_id, event_id, key_id, mxc_uri, room_alias_id, room_id, room_version_id, server_name,
    user_id,
};
use syn::{parse_macro_input, DeriveInput};

use self::{
    events::{
        event::expand_event,
        event_content::expand_event_content,
        event_enum::{expand_event_enums, expand_from_impls_derived},
        event_parse::EventEnumInput,
        event_type::expand_event_type_enum,
    },
    identifiers::IdentifierInput,
};

mod events;
mod identifiers;

/// Generates an enum to represent the various Matrix event types.
///
/// This macro also implements the necessary traits for the type to serialize and deserialize
/// itself.
///
/// # Examples
///
/// ```ignore
/// # // HACK: This is "ignore" because of cyclical dependency drama.
/// use ruma_macros::event_enum;
///
/// event_enum! {
///     enum ToDevice {
///         "m.any.event",
///         "m.other.event",
///     }
///
///     enum State {
///         "m.more.events",
///         "m.different.event",
///     }
/// }
/// ```
/// (The enum name has to be a valid identifier for `<EventKind as Parse>::parse`)
//// TODO: Change above (`<EventKind as Parse>::parse`) to [] after fully qualified syntax is
//// supported:  https://github.com/rust-lang/rust/issues/74563
#[proc_macro]
pub fn event_enum(input: TokenStream) -> TokenStream {
    let ruma_events = import_ruma_events();

    let event_enum_input = syn::parse_macro_input!(input as EventEnumInput);
    let enums = event_enum_input
        .enums
        .iter()
        .map(expand_event_enums)
        .collect::<syn::Result<pm2::TokenStream>>();
    let event_types = expand_event_type_enum(event_enum_input, ruma_events);
    event_types
        .and_then(|types| {
            enums.map(|mut enums| {
                enums.extend(types);
                enums
            })
        })
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Generates an implementation of `ruma_events::EventContent`.
#[proc_macro_derive(EventContent, attributes(ruma_event))]
pub fn derive_event_content(input: TokenStream) -> TokenStream {
    let ruma_events = import_ruma_events();
    let input = parse_macro_input!(input as DeriveInput);

    expand_event_content(&input, &ruma_events).unwrap_or_else(syn::Error::into_compile_error).into()
}

/// Generates implementations needed to serialize and deserialize Matrix events.
#[proc_macro_derive(Event, attributes(ruma_event))]
pub fn derive_event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_event(input).unwrap_or_else(syn::Error::into_compile_error).into()
}

pub(crate) fn import_ruma_events() -> pm2::TokenStream {
    if let Ok(FoundCrate::Name(name)) = crate_name("ruma-events") {
        let import = format_ident!("{}", name);
        quote! { ::#import }
    } else if let Ok(FoundCrate::Name(name)) = crate_name("ruma") {
        let import = format_ident!("{}", name);
        quote! { ::#import::events }
    } else if let Ok(FoundCrate::Name(name)) = crate_name("matrix-sdk") {
        let import = format_ident!("{}", name);
        quote! { ::#import::ruma::events }
    } else if let Ok(FoundCrate::Name(name)) = crate_name("matrix-sdk-appservice") {
        let import = format_ident!("{}", name);
        quote! { ::#import::ruma::events }
    } else {
        quote! { ::ruma_events }
    }
}

/// Generates `From` implementations for event enums.
#[proc_macro_derive(EventEnumFromEvent)]
pub fn derive_from_event_to_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_from_impls_derived(input).into()
}

/// Compile-time checked `DeviceKeyId` construction.
#[proc_macro]
pub fn device_key_id(input: TokenStream) -> TokenStream {
    let IdentifierInput { dollar_crate, id } = parse_macro_input!(input as IdentifierInput);
    assert!(device_key_id::validate(&id.value()).is_ok(), "Invalid device key id");

    let output = quote! {
        <&#dollar_crate::DeviceKeyId as ::std::convert::TryFrom<&str>>::try_from(#id).unwrap()
    };

    output.into()
}

/// Compile-time checked `EventId` construction.
#[proc_macro]
pub fn event_id(input: TokenStream) -> TokenStream {
    let IdentifierInput { dollar_crate, id } = parse_macro_input!(input as IdentifierInput);
    assert!(event_id::validate(&id.value()).is_ok(), "Invalid event id");

    let output = quote! {
        <&#dollar_crate::EventId as ::std::convert::TryFrom<&str>>::try_from(#id).unwrap()
    };

    output.into()
}

/// Compile-time checked `RoomAliasId` construction.
#[proc_macro]
pub fn room_alias_id(input: TokenStream) -> TokenStream {
    let IdentifierInput { dollar_crate, id } = parse_macro_input!(input as IdentifierInput);
    assert!(room_alias_id::validate(&id.value()).is_ok(), "Invalid room_alias_id");

    let output = quote! {
        <&#dollar_crate::RoomAliasId as ::std::convert::TryFrom<&str>>::try_from(#id).unwrap()
    };

    output.into()
}

/// Compile-time checked `RoomId` construction.
#[proc_macro]
pub fn room_id(input: TokenStream) -> TokenStream {
    let IdentifierInput { dollar_crate, id } = parse_macro_input!(input as IdentifierInput);
    assert!(room_id::validate(&id.value()).is_ok(), "Invalid room_id");

    let output = quote! {
        <&#dollar_crate::RoomId as ::std::convert::TryFrom<&str>>::try_from(#id).unwrap()
    };

    output.into()
}

/// Compile-time checked `RoomVersionId` construction.
#[proc_macro]
pub fn room_version_id(input: TokenStream) -> TokenStream {
    let IdentifierInput { dollar_crate, id } = parse_macro_input!(input as IdentifierInput);
    assert!(room_version_id::validate(&id.value()).is_ok(), "Invalid room_version_id");

    let output = quote! {
        <#dollar_crate::RoomVersionId as ::std::convert::TryFrom<&str>>::try_from(#id).unwrap()
    };

    output.into()
}

/// Compile-time checked `ServerSigningKeyId` construction.
#[proc_macro]
pub fn server_signing_key_id(input: TokenStream) -> TokenStream {
    let IdentifierInput { dollar_crate, id } = parse_macro_input!(input as IdentifierInput);
    assert!(key_id::validate(&id.value()).is_ok(), "Invalid server_signing_key_id");

    let output = quote! {
        <&#dollar_crate::ServerSigningKeyId as ::std::convert::TryFrom<&str>>::try_from(#id).unwrap()
    };

    output.into()
}

/// Compile-time checked `ServerName` construction.
#[proc_macro]
pub fn server_name(input: TokenStream) -> TokenStream {
    let IdentifierInput { dollar_crate, id } = parse_macro_input!(input as IdentifierInput);
    assert!(server_name::validate(&id.value()).is_ok(), "Invalid server_name");

    let output = quote! {
        <&#dollar_crate::ServerName as ::std::convert::TryFrom<&str>>::try_from(#id).unwrap()
    };

    output.into()
}

/// Compile-time checked `MxcUri` construction.
#[proc_macro]
pub fn mxc_uri(input: TokenStream) -> TokenStream {
    let IdentifierInput { dollar_crate, id } = parse_macro_input!(input as IdentifierInput);
    assert!(mxc_uri::validate(&id.value()).is_ok(), "Invalid mxc://");

    let output = quote! {
        <&#dollar_crate::MxcUri as ::std::convert::From<&str>>::from(#id)
    };

    output.into()
}

/// Compile-time checked `UserId` construction.
#[proc_macro]
pub fn user_id(input: TokenStream) -> TokenStream {
    let IdentifierInput { dollar_crate, id } = parse_macro_input!(input as IdentifierInput);
    assert!(user_id::validate(&id.value()).is_ok(), "Invalid user_id");

    let output = quote! {
        <&#dollar_crate::UserId as ::std::convert::TryFrom<&str>>::try_from(#id).unwrap()
    };

    output.into()
}
