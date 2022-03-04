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
use syn::{parse_macro_input, DeriveInput, ItemEnum};

use self::{
    api::{request::expand_derive_request, response::expand_derive_response, Api},
    events::{
        event::expand_event,
        event_content::expand_event_content,
        event_enum::{expand_event_enums, expand_from_impls_derived},
        event_parse::EventEnumInput,
        event_type::expand_event_type_enum,
    },
    identifiers::IdentifierInput,
    serde::{
        deserialize_from_cow_str::expand_deserialize_from_cow_str,
        display_as_ref_str::expand_display_as_ref_str,
        enum_as_ref_str::expand_enum_as_ref_str,
        enum_from_string::expand_enum_from_string,
        eq_as_ref_str::expand_partial_eq_as_ref_str,
        ord_as_ref_str::{expand_ord_as_ref_str, expand_partial_ord_as_ref_str},
        outgoing::expand_derive_outgoing,
        serialize_as_ref_str::expand_serialize_as_ref_str,
    },
};

mod api;
mod events;
mod identifiers;
mod serde;

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
    let ruma_common = import_ruma_common();

    let event_enum_input = syn::parse_macro_input!(input as EventEnumInput);
    let enums = event_enum_input
        .enums
        .iter()
        .map(expand_event_enums)
        .collect::<syn::Result<pm2::TokenStream>>();
    let event_types = expand_event_type_enum(event_enum_input, ruma_common);
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

/// Generates an implementation of `ruma_common::events::EventContent`.
#[proc_macro_derive(EventContent, attributes(ruma_event))]
pub fn derive_event_content(input: TokenStream) -> TokenStream {
    let ruma_common = import_ruma_common();
    let input = parse_macro_input!(input as DeriveInput);

    expand_event_content(&input, &ruma_common).unwrap_or_else(syn::Error::into_compile_error).into()
}

/// Generates implementations needed to serialize and deserialize Matrix events.
#[proc_macro_derive(Event, attributes(ruma_event))]
pub fn derive_event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_event(input).unwrap_or_else(syn::Error::into_compile_error).into()
}

pub(crate) fn import_ruma_common() -> pm2::TokenStream {
    if let Ok(FoundCrate::Name(name)) = crate_name("ruma-common") {
        let import = format_ident!("{}", name);
        quote! { ::#import }
    } else if let Ok(FoundCrate::Name(name)) = crate_name("ruma") {
        let import = format_ident!("{}", name);
        quote! { ::#import }
    } else if let Ok(FoundCrate::Name(name)) = crate_name("matrix-sdk") {
        let import = format_ident!("{}", name);
        quote! { ::#import::ruma }
    } else if let Ok(FoundCrate::Name(name)) = crate_name("matrix-sdk-appservice") {
        let import = format_ident!("{}", name);
        quote! { ::#import::ruma }
    } else {
        quote! { ::ruma_common }
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

/// Derive the `Outgoing` trait, possibly generating an 'Incoming' version of the struct this
/// derive macro is used on.
///
/// Specifically, if no lifetime variables are used on any of the fields of the struct, this simple
/// implementation will be generated:
///
/// ```ignore
/// # // TODO: "ignore" this doctest until it is more extensively documented. (See #541)
/// impl Outgoing for MyType {
///     type Incoming = Self;
/// }
/// ```
/*

TODO: Extend docs. Previously:

If, however, `#[wrap_incoming]` is used (which is the only reason you should ever use this
derive macro manually), a new struct `IncomingT` (where `T` is the type this derive is used on)
is generated, with all of the fields with `#[wrap_incoming]` replaced:

```ignore
#[derive(Outgoing)]
struct MyType {
    pub foo: Foo,
    #[wrap_incoming]
    pub bar: Bar,
    #[wrap_incoming(Baz)]
    pub baz: Option<Baz>,
    #[wrap_incoming(with EventResult)]
    pub x: XEvent,
    #[wrap_incoming(YEvent with EventResult)]
    pub ys: Vec<YEvent>,
}

// generated
struct IncomingMyType {
    pub foo: Foo,
    pub bar: IncomingBar,
    pub baz: Option<IncomingBaz>,
    pub x: EventResult<XEvent>,
    pub ys: Vec<EventResult<YEvent>>,
}
```

*/
#[proc_macro_derive(Outgoing, attributes(incoming_derive))]
pub fn derive_outgoing(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_derive_outgoing(input).unwrap_or_else(syn::Error::into_compile_error).into()
}

/// Derive the `AsRef<str>` trait for an enum.
#[proc_macro_derive(AsRefStr, attributes(ruma_enum))]
pub fn derive_enum_as_ref_str(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemEnum);
    expand_enum_as_ref_str(&input).unwrap_or_else(syn::Error::into_compile_error).into()
}

/// Derive the `From<T: AsRef<str> + Into<Box<str>>>` trait for an enum.
#[proc_macro_derive(FromString, attributes(ruma_enum))]
pub fn derive_enum_from_string(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemEnum);
    expand_enum_from_string(&input).unwrap_or_else(syn::Error::into_compile_error).into()
}

// FIXME: The following macros aren't actually interested in type details beyond name (and possibly
//        generics in the future). They probably shouldn't use `DeriveInput`.

/// Derive the `fmt::Display` trait using the `AsRef<str>` implementation of the type.
#[proc_macro_derive(DisplayAsRefStr)]
pub fn derive_display_as_ref_str(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_display_as_ref_str(&input.ident).unwrap_or_else(syn::Error::into_compile_error).into()
}

/// Derive the `Serialize` trait using the `AsRef<str>` implementation of the type.
#[proc_macro_derive(SerializeAsRefStr)]
pub fn derive_serialize_as_ref_str(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_serialize_as_ref_str(&input.ident).unwrap_or_else(syn::Error::into_compile_error).into()
}

/// Derive the `Deserialize` trait using the `From<Cow<str>>` implementation of the type.
#[proc_macro_derive(DeserializeFromCowStr)]
pub fn derive_deserialize_from_cow_str(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_deserialize_from_cow_str(&input.ident)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Derive the `PartialOrd` trait using the `AsRef<str>` implementation of the type.
#[proc_macro_derive(PartialOrdAsRefStr)]
pub fn derive_partial_ord_as_ref_str(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_partial_ord_as_ref_str(&input.ident)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Derive the `Ord` trait using the `AsRef<str>` implementation of the type.
#[proc_macro_derive(OrdAsRefStr)]
pub fn derive_ord_as_ref_str(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_ord_as_ref_str(&input.ident).unwrap_or_else(syn::Error::into_compile_error).into()
}

/// Derive the `PartialEq` trait using the `AsRef<str>` implementation of the type.
#[proc_macro_derive(PartialEqAsRefStr)]
pub fn derive_partial_eq_as_ref_str(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_partial_eq_as_ref_str(&input.ident).unwrap_or_else(syn::Error::into_compile_error).into()
}

/// Shorthand for the derives `AsRefStr`, `FromString`, `DisplayAsRefStr`, `SerializeAsRefStr` and
/// `DeserializeFromCowStr`.
#[proc_macro_derive(StringEnum, attributes(ruma_enum))]
pub fn derive_string_enum(input: TokenStream) -> TokenStream {
    fn expand_all(input: ItemEnum) -> syn::Result<proc_macro2::TokenStream> {
        let as_ref_str_impl = expand_enum_as_ref_str(&input)?;
        let from_string_impl = expand_enum_from_string(&input)?;
        let display_impl = expand_display_as_ref_str(&input.ident)?;
        let serialize_impl = expand_serialize_as_ref_str(&input.ident)?;
        let deserialize_impl = expand_deserialize_from_cow_str(&input.ident)?;

        Ok(quote! {
            #as_ref_str_impl
            #from_string_impl
            #display_impl
            #serialize_impl
            #deserialize_impl
        })
    }

    let input = parse_macro_input!(input as ItemEnum);
    expand_all(input).unwrap_or_else(syn::Error::into_compile_error).into()
}

/// A derive macro that generates no code, but registers the serde attribute so both `#[serde(...)]`
/// and `#[cfg_attr(..., serde(...))]` are accepted on the type, its fields and (in case the input
/// is an enum) variants fields.
#[doc(hidden)]
#[proc_macro_derive(_FakeDeriveSerde, attributes(serde))]
pub fn fake_derive_serde(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}

/// A procedural macro for easily generating API endpoints.
///
/// Note that for technical reasons, the `ruma_api!` macro is only documented in [ruma-common], not
/// here.
///
/// [ruma-common]: https://github.com/ruma/ruma/tree/main/ruma-common
#[proc_macro]
pub fn ruma_api(input: TokenStream) -> TokenStream {
    let api = parse_macro_input!(input as Api);
    api.expand_all().into()
}

/// Internal helper taking care of the request-specific parts of `ruma_api!`.
#[proc_macro_derive(Request, attributes(ruma_api))]
pub fn derive_request(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_derive_request(input).unwrap_or_else(syn::Error::into_compile_error).into()
}

/// Internal helper taking care of the response-specific parts of `ruma_api!`.
#[proc_macro_derive(Response, attributes(ruma_api))]
pub fn derive_response(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_derive_response(input).unwrap_or_else(syn::Error::into_compile_error).into()
}

/// A derive macro that generates no code, but registers the ruma_api attribute so both
/// `#[ruma_api(...)]` and `#[cfg_attr(..., ruma_api(...))]` are accepted on the type, its fields
/// and (in case the input is an enum) variants fields.
#[doc(hidden)]
#[proc_macro_derive(_FakeDeriveRumaApi, attributes(ruma_api))]
pub fn fake_derive_ruma_api(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
