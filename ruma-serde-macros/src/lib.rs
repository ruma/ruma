use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, ItemEnum};

use deserialize_from_cow_str::expand_deserialize_from_cow_str;
use display_as_ref_str::expand_display_as_ref_str;
use enum_as_ref_str::expand_enum_as_ref_str;
use enum_from_string::expand_enum_from_string;
use outgoing::expand_derive_outgoing;
use serialize_as_ref_str::expand_serialize_as_ref_str;

mod attr;
mod case;
mod deserialize_from_cow_str;
mod display_as_ref_str;
mod enum_as_ref_str;
mod enum_from_string;
mod outgoing;
mod serialize_as_ref_str;
mod util;

/// Derive the `Outgoing` trait, possibly generating an 'Incoming' version of the struct this
/// derive macro is used on. Specifically, if no lifetime variables are used on any of the fields
/// of the struct, this simple implementation will be generated:
///
/// ```ignore
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
    expand_derive_outgoing(input).unwrap_or_else(|err| err.to_compile_error()).into()
}

#[proc_macro_derive(AsRefStr, attributes(ruma_enum))]
pub fn derive_enum_as_ref_str(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemEnum);
    expand_enum_as_ref_str(&input).unwrap_or_else(|err| err.to_compile_error()).into()
}

#[proc_macro_derive(FromString, attributes(ruma_enum))]
pub fn derive_enum_from_string(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemEnum);
    expand_enum_from_string(&input).unwrap_or_else(|err| err.to_compile_error()).into()
}

// FIXME: The following macros aren't actually interested in type details beyond name (and possibly
//        generics in the future). They probably shouldn't use `DeriveInput`.

#[proc_macro_derive(DisplayAsRefStr)]
pub fn derive_display_as_ref_str(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_display_as_ref_str(&input.ident).unwrap_or_else(|err| err.to_compile_error()).into()
}

#[proc_macro_derive(SerializeAsRefStr)]
pub fn derive_serialize_as_ref_str(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_serialize_as_ref_str(&input.ident).unwrap_or_else(|err| err.to_compile_error()).into()
}

#[proc_macro_derive(DeserializeFromCowStr)]
pub fn derive_deserialize_from_cow_str(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_deserialize_from_cow_str(&input.ident)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
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
    expand_all(input).unwrap_or_else(|err| err.to_compile_error()).into()
}

/// A derive macro that generates no code, but registers the serde attribute so both `#[serde(...)]`
/// and `#[cfg_attr(..., serde(...))]` are accepted on the type, its fields and (in case the input
/// is an enum) variants fields.
#[doc(hidden)]
#[proc_macro_derive(_FakeDeriveSerde, attributes(serde))]
pub fn fake_derive_serde(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
