use proc_macro::TokenStream;

use quote::quote;
use ruma_identifiers_validation::{
    device_key_id, event_id, room_alias_id, room_id, room_version_id, server_key_id, server_name,
    user_id,
};
use syn::{parse_macro_input, LitStr};

#[proc_macro]
pub fn device_id(input: TokenStream) -> TokenStream {
    let id = parse_macro_input!(input as LitStr);
    let output = quote! {
        ::std::boxed::Box<::ruma::identifiers::DeviceId>::from(#id)
    };

    output.into()
}

#[proc_macro]
pub fn device_key_id(input: TokenStream) -> TokenStream {
    let id = parse_macro_input!(input as LitStr);
    assert!(device_key_id::validate(&id.value()).is_ok(), "Invalid device key id");

    let output = quote! {
        <::ruma::identifiers::DeviceKeyId as ::std::convert::TryFrom<&str>>::try_from(#id).unwrap()
    };

    output.into()
}

#[proc_macro]
pub fn event_id(input: TokenStream) -> TokenStream {
    let id = parse_macro_input!(input as LitStr);
    assert!(event_id::validate(&id.value()).is_ok(), "Invalid event id");

    let output = quote! {
        <::ruma::identifiers::EventId as ::std::convert::TryFrom<&str>>::try_from(#id).unwrap()
    };

    output.into()
}

#[proc_macro]
pub fn room_alias_id(input: TokenStream) -> TokenStream {
    let id = parse_macro_input!(input as LitStr);
    assert!(room_alias_id::validate(&id.value()).is_ok(), "Invalid room_alias_id");

    let output = quote! {
        <::ruma::identifiers::RoomAliasId as ::std::convert::TryFrom<&str>>::try_from(#id).unwrap()
    };

    output.into()
}

#[proc_macro]
pub fn room_id(input: TokenStream) -> TokenStream {
    let id = parse_macro_input!(input as LitStr);
    assert!(room_id::validate(&id.value()).is_ok(), "Invalid room_id");

    let output = quote! {
        <::ruma::identifiers::RoomId as ::std::convert::TryFrom<&str>>::try_from(#id).unwrap()
    };

    output.into()
}

#[proc_macro]
pub fn room_version_id(input: TokenStream) -> TokenStream {
    let id = parse_macro_input!(input as LitStr);
    assert!(room_version_id::validate(&id.value()).is_ok(), "Invalid room_version_id");

    let output = quote! {
        <::ruma::identifiers::RoomVersionId as ::std::convert::TryFrom<&str>>::try_from(#id).unwrap()
    };

    output.into()
}

#[proc_macro]
pub fn server_key_id(input: TokenStream) -> TokenStream {
    let id = parse_macro_input!(input as LitStr);
    assert!(server_key_id::validate(&id.value()).is_ok(), "Invalid server_key_id");

    let output = quote! {
        <::ruma::identifiers::ServerKeyId as ::std::convert::TryFrom<&str>>::try_from(#id).unwrap()
    };

    output.into()
}

#[proc_macro]
pub fn server_name(input: TokenStream) -> TokenStream {
    let id = parse_macro_input!(input as LitStr);
    assert!(server_name::validate(&id.value()).is_ok(), "Invalid server_name");

    let output = quote! {
        <::std::boxed::Box::<::ruma::identifiers::ServerName> as ::std::convert::TryFrom<&str>>::try_from(#id).unwrap()
    };

    output.into()
}

#[proc_macro]
pub fn user_id(input: TokenStream) -> TokenStream {
    let id = parse_macro_input!(input as LitStr);
    assert!(user_id::validate(&id.value()).is_ok(), "Invalid user_id");

    let output = quote! {
        <::ruma::identifiers::UserId as ::std::convert::TryFrom<&str>>::try_from(#id).unwrap()
    };

    output.into()
}
