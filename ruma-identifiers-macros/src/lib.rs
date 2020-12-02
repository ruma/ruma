use proc_macro::TokenStream;

use quote::quote;
use ruma_identifiers_validation::{
    device_key_id, event_id, key_id, room_alias_id, room_id, room_version_id, server_name, user_id,
};
use syn::{parse::Parse, parse_macro_input, LitStr, Path, Token};

struct Input {
    dollar_crate: Path,
    id: LitStr,
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let dollar_crate = input.parse()?;
        input.parse::<Token![,]>()?;
        let id = input.parse()?;

        Ok(Self { dollar_crate, id })
    }
}

#[proc_macro]
pub fn device_key_id(input: TokenStream) -> TokenStream {
    let Input { dollar_crate, id } = parse_macro_input!(input as Input);
    assert!(device_key_id::validate(&id.value()).is_ok(), "Invalid device key id");

    let output = quote! {
        <#dollar_crate::DeviceKeyId as ::std::convert::TryFrom<&str>>::try_from(#id).unwrap()
    };

    output.into()
}

#[proc_macro]
pub fn event_id(input: TokenStream) -> TokenStream {
    let Input { dollar_crate, id } = parse_macro_input!(input as Input);
    assert!(event_id::validate(&id.value()).is_ok(), "Invalid event id");

    let output = quote! {
        <#dollar_crate::EventId as ::std::convert::TryFrom<&str>>::try_from(#id).unwrap()
    };

    output.into()
}

#[proc_macro]
pub fn room_alias_id(input: TokenStream) -> TokenStream {
    let Input { dollar_crate, id } = parse_macro_input!(input as Input);
    assert!(room_alias_id::validate(&id.value()).is_ok(), "Invalid room_alias_id");

    let output = quote! {
        <#dollar_crate::RoomAliasId as ::std::convert::TryFrom<&str>>::try_from(#id).unwrap()
    };

    output.into()
}

#[proc_macro]
pub fn room_id(input: TokenStream) -> TokenStream {
    let Input { dollar_crate, id } = parse_macro_input!(input as Input);
    assert!(room_id::validate(&id.value()).is_ok(), "Invalid room_id");

    let output = quote! {
        <#dollar_crate::RoomId as ::std::convert::TryFrom<&str>>::try_from(#id).unwrap()
    };

    output.into()
}

#[proc_macro]
pub fn room_version_id(input: TokenStream) -> TokenStream {
    let Input { dollar_crate, id } = parse_macro_input!(input as Input);
    assert!(room_version_id::validate(&id.value()).is_ok(), "Invalid room_version_id");

    let output = quote! {
        <#dollar_crate::RoomVersionId as ::std::convert::TryFrom<&str>>::try_from(#id).unwrap()
    };

    output.into()
}

#[proc_macro]
pub fn server_signing_key_id(input: TokenStream) -> TokenStream {
    let Input { dollar_crate, id } = parse_macro_input!(input as Input);
    assert!(key_id::validate(&id.value()).is_ok(), "Invalid server_signing_key_id");

    let output = quote! {
        <#dollar_crate::ServerSigningKeyId as ::std::convert::TryFrom<&str>>::try_from(#id).unwrap()
    };

    output.into()
}

#[proc_macro]
pub fn server_name(input: TokenStream) -> TokenStream {
    let Input { dollar_crate, id } = parse_macro_input!(input as Input);
    assert!(server_name::validate(&id.value()).is_ok(), "Invalid server_name");

    let output = quote! {
        <::std::boxed::Box::<#dollar_crate::ServerName> as ::std::convert::TryFrom<&str>>::try_from(
            #id,
        ).unwrap()
    };

    output.into()
}

#[proc_macro]
pub fn user_id(input: TokenStream) -> TokenStream {
    let Input { dollar_crate, id } = parse_macro_input!(input as Input);
    assert!(user_id::validate(&id.value()).is_ok(), "Invalid user_id");

    let output = quote! {
        <#dollar_crate::UserId as ::std::convert::TryFrom<&str>>::try_from(#id).unwrap()
    };

    output.into()
}
