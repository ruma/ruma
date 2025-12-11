//! Implementation of the `IdDst` derive macro.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse_quote;

use crate::util::{RumaCommon, RumaCommonReexport};

mod parse;

/// Generate the `Owned` version of an identifier and various trait implementations.
pub(crate) fn expand_id_dst(input: syn::ItemStruct) -> syn::Result<TokenStream> {
    let id_dst = IdDst::parse(input)?;

    let borrowed_type = &id_dst.borrowed_type;
    let box_type = &id_dst.box_type;
    let arc_type = &id_dst.arc_type;
    let impl_generics = &id_dst.impl_generics;

    let as_str_and_bytes_impls = id_dst.expand_as_str_and_bytes_impls();
    let to_string_impls = id_dst.expand_to_string_impls(borrowed_type);
    let unchecked_from_str_impls = id_dst.expand_unchecked_from_str_impls();
    let owned_id_struct = id_dst.expand_owned_id_struct();
    let fallible_from_str_impls = id_dst.expand_fallible_from_str_impls();
    let infallible_from_str_impls = id_dst.expand_infallible_from_str_impls();
    let partial_eq_impls = id_dst.expand_partial_eq_impls();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::std::clone::Clone for #box_type {
            fn clone(&self) -> Self {
                (**self).into()
            }
        }

        #[automatically_derived]
        impl #impl_generics ::std::convert::From<&#borrowed_type> for #box_type {
            fn from(id: &#borrowed_type) -> Self {
                <#borrowed_type>::from_box(id.as_str().into())
            }
        }

        #[automatically_derived]
        impl #impl_generics ::std::convert::From<&#borrowed_type> for ::std::rc::Rc<#borrowed_type> {
            fn from(s: &#borrowed_type) -> Self {
                let rc = ::std::rc::Rc::<::std::primitive::str>::from(s.as_str());
                <#borrowed_type>::from_rc(rc)
            }
        }

        #[automatically_derived]
        impl #impl_generics ::std::convert::From<&#borrowed_type> for #arc_type {
            fn from(s: &#borrowed_type) -> Self {
                let arc = ::std::sync::Arc::<::std::primitive::str>::from(s.as_str());
                <#borrowed_type>::from_arc(arc)
            }
        }

        #as_str_and_bytes_impls
        #to_string_impls
        #unchecked_from_str_impls
        #owned_id_struct
        #fallible_from_str_impls
        #infallible_from_str_impls
        #partial_eq_impls
    })
}

/// The parsed input of the `IdDst` macro.
struct IdDst {
    /// The name of the borrowed type.
    ident: syn::Ident,

    /// The borrowed type with generics, if any.
    borrowed_type: syn::Type,

    /// The name of the owned type.
    owned_ident: syn::Ident,

    /// The owned type with generics, if any.
    owned_type: syn::Type,

    /// The type wrapped in a `Box` with generics, if any.
    box_type: syn::Type,

    /// The type wrapped in a `Arc` with generics, if any.
    arc_type: syn::Type,

    /// The generics on the borrowed type.
    generics: syn::Generics,

    /// The declaration of the generics of the borrowed type to use on `impl` blocks.
    impl_generics: TokenStream,

    /// The path to the function to use to validate the identifier.
    validate: Option<syn::Path>,

    /// The index of the `str` field.
    ///
    /// This is assumed too be the last field of the tuple struct.
    str_field_index: syn::Index,

    /// The path to use imports from the ruma-common crate.
    ruma_common: RumaCommon,
}

impl IdDst {
    /// Generate `AsRef<str>` and `AsRef<[u8]>` implementations and string conversions for this
    /// identifier.
    fn expand_as_str_and_bytes_impls(&self) -> TokenStream {
        let ident = &self.ident;
        let borrowed_type = &self.borrowed_type;
        let box_type = &self.box_type;
        let impl_generics = &self.impl_generics;
        let str_field_index = &self.str_field_index;

        let as_str_docs = format!("Extracts a string slice from this `{ident}`.");
        let as_bytes_docs = format!("Extracts a byte slice from this `{ident}`.");

        quote! {
            impl #impl_generics #borrowed_type {
                #[doc = #as_str_docs]
                #[inline]
                pub fn as_str(&self) -> &::std::primitive::str {
                    &self.#str_field_index
                }

                #[doc = #as_bytes_docs]
                #[inline]
                pub fn as_bytes(&self) -> &[::std::primitive::u8] {
                    self.as_str().as_bytes()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::AsRef<#borrowed_type> for #borrowed_type {
                fn as_ref(&self) -> &#borrowed_type {
                    self
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::AsRef<::std::primitive::str> for #borrowed_type {
                fn as_ref(&self) -> &str {
                    self.as_str()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::AsRef<::std::primitive::str> for #box_type {
                fn as_ref(&self) -> &str {
                    self.as_str()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::AsRef<[::std::primitive::u8]> for #borrowed_type {
                fn as_ref(&self) -> &[::std::primitive::u8] {
                    self.as_bytes()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::AsRef<[::std::primitive::u8]> for #box_type {
                fn as_ref(&self) -> &[::std::primitive::u8] {
                    self.as_bytes()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<&#borrowed_type> for ::std::string::String {
                fn from(id: &#borrowed_type) -> Self {
                    id.as_str().to_owned()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#box_type> for ::std::boxed::Box<::std::primitive::str> {
                fn from(id: #box_type) -> Self {
                    unsafe { ::std::boxed::Box::from_raw(std::boxed::Box::into_raw(id) as _) }
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#box_type> for ::std::string::String {
                fn from(id: #box_type) -> Self {
                    ::std::boxed::Box::<::std::primitive::str>::from(id).into()
                }
            }
        }
    }

    /// Generate unchecked private methods to convert a string type to the identifier.
    fn expand_unchecked_from_str_impls(&self) -> TokenStream {
        let borrowed_type = &self.borrowed_type;
        let impl_generics = &self.impl_generics;

        quote! {
            #[automatically_derived]
            impl #impl_generics #borrowed_type {
                pub(super) const fn from_borrowed(s: &::std::primitive::str) -> &Self {
                    unsafe { ::std::mem::transmute(s) }
                }

                pub(super) fn from_box(s: ::std::boxed::Box<::std::primitive::str>) -> ::std::boxed::Box<Self> {
                    unsafe { ::std::boxed::Box::from_raw(::std::boxed::Box::into_raw(s) as _) }
                }

                pub(super) fn from_rc(s: ::std::rc::Rc<::std::primitive::str>) -> ::std::rc::Rc<Self> {
                    unsafe { ::std::rc::Rc::from_raw(::std::rc::Rc::into_raw(s) as _) }
                }

                pub(super) fn from_arc(s: ::std::sync::Arc<::std::primitive::str>) -> ::std::sync::Arc<Self> {
                    unsafe { ::std::sync::Arc::from_raw(::std::sync::Arc::into_raw(s) as _) }
                }
            }
        }
    }

    /// Generate `FromStr` and other fallible string conversions implementations for this
    /// identifier, if it has a validation function.
    ///
    /// The error returned during conversion is `ruma_common::IdParseError`.
    fn expand_fallible_from_str_impls(&self) -> Option<TokenStream> {
        let validate = self.validate.as_ref()?;

        let ident = &self.ident;
        let owned_ident = &self.owned_ident;
        let borrowed_type = &self.borrowed_type;
        let owned_type = &self.owned_type;
        let box_type = &self.box_type;
        let generic_params = &self.generics.params;
        let impl_generics = &self.impl_generics;

        let ruma_common = &self.ruma_common;
        let serde = ruma_common.reexported(RumaCommonReexport::Serde);

        let parse_doc_header = format!("Try parsing a `&str` into an `{owned_ident}`.");
        let parse_box_doc_header = format!("Try parsing a `&str` into a `Box<{ident}>`.");
        let parse_rc_docs = format!("Try parsing a `&str` into an `Rc<{ident}>`.");
        let parse_arc_docs = format!("Try parsing a `&str` into an `Arc<{ident}>`.");

        let ref_str_type: syn::Type = parse_quote!(&::std::primitive::str);
        let string_type: syn::Type = parse_quote!(::std::string::String);

        // Generate `TryFrom<{from_type}> for {for_type}` which uses the given `parse_fn` from the
        // borrowed type.
        let expand_try_from_impl =
            |for_type: &syn::Type, from_type: &syn::Type, parse_fn: &syn::Ident| {
                quote! {
                    #[automatically_derived]
                    impl #impl_generics ::std::convert::TryFrom<#from_type> for #for_type {
                        type Error = #ruma_common::IdParseError;

                    fn try_from(s: #from_type) -> ::std::result::Result<Self, Self::Error> {
                            <#borrowed_type>::#parse_fn(s)
                        }
                    }
                }
            };

        // Generate `FromStr` `TryFrom<&str>` and `TryFrom<String>` implementations for the given
        // type, which use the given `parse_fn` from the borrowed type.
        let expand_from_str_impls = |ty: &syn::Type, parse_fn: &syn::Ident| -> TokenStream {
            let try_from_ref_str_impl = expand_try_from_impl(ty, &ref_str_type, parse_fn);
            let try_from_string_impl = expand_try_from_impl(ty, &string_type, parse_fn);

            quote! {
                #[automatically_derived]
                impl #impl_generics ::std::str::FromStr for #ty {
                    type Err = #ruma_common::IdParseError;

                    fn from_str(s: &::std::primitive::str) -> ::std::result::Result<Self, Self::Err> {
                        <#borrowed_type>::#parse_fn(s)
                    }
                }

                #try_from_ref_str_impl
                #try_from_string_impl
            }
        };

        let box_type_from_str_impls =
            expand_from_str_impls(box_type, &syn::Ident::new("parse_box", Span::call_site()));
        let owned_type_from_str_impls =
            expand_from_str_impls(owned_type, &syn::Ident::new("parse", Span::call_site()));

        Some(quote! {
            #[automatically_derived]
            impl #impl_generics #borrowed_type {
                #[doc = #parse_doc_header]
                ///
                /// The same can also be done using `FromStr`, `TryFrom` or `TryInto`.
                /// This function is simply more constrained and thus useful in generic contexts.
                pub fn parse(
                    s: impl ::std::convert::AsRef<::std::primitive::str>,
                ) -> ::std::result::Result<#owned_type, #ruma_common::IdParseError> {
                    let s = s.as_ref();
                    #validate(s)?;
                    ::std::result::Result::Ok(#ident::from_borrowed(s).to_owned())
                }

                #[doc = #parse_box_doc_header]
                ///
                /// The same can also be done using `FromStr`, `TryFrom` or `TryInto`.
                /// This function is simply more constrained and thus useful in generic contexts.
                pub fn parse_box(
                    s: impl ::std::convert::AsRef<::std::primitive::str> + ::std::convert::Into<::std::boxed::Box<::std::primitive::str>>,
                ) -> ::std::result::Result<::std::boxed::Box<Self>, #ruma_common::IdParseError> {
                    #validate(s.as_ref())?;
                    ::std::result::Result::Ok(#ident::from_box(s.into()))
                }

                #[doc = #parse_rc_docs]
                pub fn parse_rc(
                    s: impl ::std::convert::AsRef<::std::primitive::str> + ::std::convert::Into<::std::rc::Rc<::std::primitive::str>>,
                ) -> ::std::result::Result<::std::rc::Rc<Self>, #ruma_common::IdParseError> {
                    #validate(s.as_ref())?;
                    ::std::result::Result::Ok(#ident::from_rc(s.into()))
                }

                #[doc = #parse_arc_docs]
                pub fn parse_arc(
                    s: impl ::std::convert::AsRef<::std::primitive::str> + ::std::convert::Into<std::sync::Arc<::std::primitive::str>>,
                ) -> ::std::result::Result<::std::sync::Arc<Self>, #ruma_common::IdParseError> {
                    #validate(s.as_ref())?;
                    ::std::result::Result::Ok(#ident::from_arc(s.into()))
                }
            }

            #[automatically_derived]
            impl<'de, #generic_params> #serde::Deserialize<'de> for #box_type {
                fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
                where
                    D: #serde::Deserializer<'de>,
                {
                    use #serde::de::Error;

                    let s = #ruma_common::serde::deserialize_cow_str(deserializer)?;
                    #ident::parse_box(s.as_ref()).map_err(D::Error::custom)
                }
            }

            #[automatically_derived]
            impl<'de, #generic_params> #serde::Deserialize<'de> for #owned_type {
                fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
                where
                    D: #serde::Deserializer<'de>,
                {
                    use #serde::de::Error;

                    let s = #ruma_common::serde::deserialize_cow_str(deserializer)?;
                    #ident::parse(s.as_ref()).map_err(D::Error::custom)
                }
            }

            #[automatically_derived]
            impl<'a, #generic_params> ::std::convert::TryFrom<&'a ::std::primitive::str> for &'a #borrowed_type {
                type Error = #ruma_common::IdParseError;

                fn try_from(s: &'a ::std::primitive::str) -> ::std::result::Result<Self, Self::Error> {
                    #validate(s)?;
                    ::std::result::Result::Ok(<#borrowed_type>::from_borrowed(s))
                }
            }

            #box_type_from_str_impls
            #owned_type_from_str_impls
        })
    }

    /// Generate `From<&str>` and other infallible string conversions implementations for this
    /// identifier, if it doesn't have a validation function.
    fn expand_infallible_from_str_impls(&self) -> Option<TokenStream> {
        if self.validate.is_some() {
            return None;
        }

        let borrowed_type = &self.borrowed_type;
        let owned_type = &self.owned_type;
        let box_type = &self.box_type;
        let impl_generics = &self.impl_generics;
        let generic_params = &self.generics.params;

        let serde = self.ruma_common.reexported(RumaCommonReexport::Serde);

        Some(quote! {
            #[automatically_derived]
            impl<'a, #generic_params> ::std::convert::From<&'a ::std::primitive::str> for &'a #borrowed_type {
                fn from(s: &'a ::std::primitive::str) -> Self {
                    <#borrowed_type>::from_borrowed(s)
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<&::std::primitive::str> for #owned_type {
                fn from(s: &::std::primitive::str) -> Self {
                    <&#borrowed_type>::from(s).into()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<::std::boxed::Box<::std::primitive::str>> for #owned_type {
                fn from(s: ::std::boxed::Box<::std::primitive::str>) -> Self {
                    <&#borrowed_type>::from(&*s).into()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<::std::string::String> for #owned_type {
                fn from(s: ::std::string::String) -> Self {
                    <&#borrowed_type>::from(s.as_str()).into()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<&::std::primitive::str> for #box_type {
                fn from(s: &::std::primitive::str) -> Self {
                    <#borrowed_type>::from_box(s.into())
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<::std::boxed::Box<::std::primitive::str>> for #box_type {
                fn from(s: ::std::boxed::Box<::std::primitive::str>) -> Self {
                    <#borrowed_type>::from_box(s)
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<::std::string::String> for #box_type {
                fn from(s: String) -> Self {
                    <#borrowed_type>::from_box(s.into())
                }
            }

            #[automatically_derived]
            impl<'de, #generic_params> #serde::Deserialize<'de> for #box_type {
                fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
                where
                    D: #serde::Deserializer<'de>,
                {
                    ::std::boxed::Box::<::std::primitive::str>::deserialize(deserializer).map(<#borrowed_type>::from_box)
                }
            }

            #[automatically_derived]
            impl<'de, #generic_params> #serde::Deserialize<'de> for #owned_type {
                fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
                where
                    D: #serde::Deserializer<'de>,
                {
                    // FIXME: Deserialize inner, convert that
                    ::std::boxed::Box::<::std::primitive::str>::deserialize(deserializer).map(<#borrowed_type>::from_box).map(::std::convert::Into::into)
                }
            }
        })
    }

    /// Generate the `Owned{ident}` type and its implementations.
    fn expand_owned_id_struct(&self) -> TokenStream {
        let ident = &self.ident;
        let owned_ident = &self.owned_ident;
        let borrowed_type = &self.borrowed_type;
        let owned_type = &self.owned_type;
        let box_type = &self.box_type;
        let arc_type = &self.arc_type;
        let generics = &self.generics;
        let impl_generics = &self.impl_generics;

        let doc_header = format!("Owned variant of [`{ident}`]");

        let to_string_impls = self.expand_to_string_impls(owned_type);

        quote! {
            #[doc = #doc_header]
            ///
            /// The wrapper type for this type is variable, by default it'll use [`Box`],
            /// but you can change that by setting "`--cfg=ruma_identifiers_storage=...`" using
            /// `RUSTFLAGS` or `.cargo/config.toml` (under `[build]` -> `rustflags = ["..."]`)
            /// to the following;
            /// - `ruma_identifiers_storage="Arc"` to use [`Arc`](std::sync::Arc) as a wrapper type.
            pub struct #owned_ident #generics {
                #[cfg(not(any(ruma_identifiers_storage = "Arc")))]
                inner: #box_type,
                #[cfg(ruma_identifiers_storage = "Arc")]
                inner: #arc_type,
            }

            #[automatically_derived]
            impl #impl_generics ::std::borrow::ToOwned for #borrowed_type {
                type Owned = #owned_type;

                fn to_owned(&self) -> Self::Owned {
                    #owned_ident {
                        #[cfg(not(any(ruma_identifiers_storage = "Arc")))]
                        inner: #ident::from_box(self.as_str().into()),
                        #[cfg(ruma_identifiers_storage = "Arc")]
                        inner: #ident::from_arc(self.as_str().into()),
                    }
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::AsRef<#borrowed_type> for #owned_type {
                fn as_ref(&self) -> &#borrowed_type {
                    &*self.inner
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::AsRef<::std::primitive::str> for #owned_type {
                fn as_ref(&self) -> &::std::primitive::str {
                    self.inner.as_str()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::AsRef<[::std::primitive::u8]> for #owned_type {
                fn as_ref(&self) -> &[::std::primitive::u8] {
                    self.inner.as_bytes()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#owned_type> for ::std::string::String {
                fn from(id: #owned_type) -> Self {
                    #[cfg(not(any(ruma_identifiers_storage = "Arc")))]
                    { id.inner.into() }
                    #[cfg(ruma_identifiers_storage = "Arc")]
                    { id.inner.as_ref().into() }
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::clone::Clone for #owned_type {
                fn clone(&self) -> Self {
                    (&*self.inner).into()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::ops::Deref for #owned_type {
                type Target = #borrowed_type;

                fn deref(&self) -> &Self::Target {
                    &self.inner
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::borrow::Borrow<#borrowed_type> for #owned_type {
                fn borrow(&self) -> &#borrowed_type {
                    self.as_ref()
                }
            }

            #to_string_impls

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<&#borrowed_type> for #owned_type {
                fn from(id: &#borrowed_type) -> Self {
                    Self { inner: id.into() }
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#box_type> for #owned_type {
                fn from(b: #box_type) -> Self {
                    Self { inner: b.into() }
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#arc_type> for #owned_type {
                fn from(a: #arc_type) -> Self {
                    Self {
                        #[cfg(not(any(ruma_identifiers_storage = "Arc")))]
                        inner: a.as_ref().into(),
                        #[cfg(ruma_identifiers_storage = "Arc")]
                        inner: a,
                    }
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#owned_type> for #box_type {
                fn from(a: #owned_type) -> Self {
                    #[cfg(not(any(ruma_identifiers_storage = "Arc")))]
                    { a.inner }
                    #[cfg(ruma_identifiers_storage = "Arc")]
                    { a.inner.as_ref().into() }
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#owned_type> for #arc_type {
                fn from(a: #owned_type) -> Self {
                    #[cfg(not(any(ruma_identifiers_storage = "Arc")))]
                    { a.inner.into() }
                    #[cfg(ruma_identifiers_storage = "Arc")]
                    { a.inner }
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::cmp::PartialEq for #owned_type {
                fn eq(&self, other: &Self) -> bool {
                    self.as_str() == other.as_str()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::cmp::Eq for #owned_type {}

            #[automatically_derived]
            impl #impl_generics ::std::cmp::PartialOrd for #owned_type {
                fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
                    Some(self.cmp(other))
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::cmp::Ord for #owned_type {
                fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                    self.as_str().cmp(other.as_str())
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::hash::Hash for #owned_type {
                fn hash<H>(&self, state: &mut H)
                where
                    H: ::std::hash::Hasher,
                {
                    self.as_str().hash(state)
                }
            }
        }
    }

    /// Generate `std::fmt::Display`, `std::fmt::Debug` or `serde::Serialize` traits
    /// implementations, using it's `.as_str()` function.
    fn expand_to_string_impls(&self, ty: &syn::Type) -> TokenStream {
        let serde = self.ruma_common.reexported(RumaCommonReexport::Serde);
        let impl_generics = &self.impl_generics;

        quote! {
            #[automatically_derived]
            impl #impl_generics ::std::fmt::Display for #ty {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    write!(f, "{}", self.as_str())
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::fmt::Debug for #ty {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    <str as ::std::fmt::Debug>::fmt(self.as_str(), f)
                }
            }

            #[automatically_derived]
            impl #impl_generics #serde::Serialize for #ty {
                fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
                where
                    S: #serde::Serializer,
                {
                    serializer.serialize_str(self.as_str())
                }
            }
        }
    }

    /// Generate `std::cmp::PartialEq` implementations by comparing strings.
    fn expand_partial_eq_impls(&self) -> TokenStream {
        let impl_generics = &self.impl_generics;
        let borrowed_type = &self.borrowed_type;
        let ref_borrowed_type: syn::Type = parse_quote! { &#borrowed_type };
        let str_type: syn::Type = parse_quote! { ::std::primitive::str };
        let ref_str_type: syn::Type = parse_quote! { &#str_type };
        let string_type: syn::Type = parse_quote! { ::std::string::String };

        let self_ident = syn::Ident::new("self", Span::call_site());
        let other_ident = syn::Ident::new("other", Span::call_site());

        // Get the string representation of the type.
        let as_str_impl = |ty: &syn::Type, ident: &syn::Ident| {
            if *ty == str_type || *ty == ref_str_type {
                quote! { ::std::convert::AsRef::<::std::primitive::str>::as_ref(#ident) }
            } else {
                quote! { #ident.as_str() }
            }
        };

        // Implement `PartialEq` with the given lhs and rhs types.
        let expand_partial_eq = |lhs: &syn::Type, rhs: &syn::Type| {
            let self_as_str = as_str_impl(lhs, &self_ident);
            let other_as_str = as_str_impl(rhs, &other_ident);

            quote! {
                #[automatically_derived]
                impl #impl_generics ::std::cmp::PartialEq<#rhs> for #lhs {
                    fn eq(&self, other: &#rhs) -> bool {
                        #self_as_str == #other_as_str
                    }
                }
            }
        };

        // Implement reciprocal `PartialEq` implementation for the given type with the given other
        // types.
        let expand_partial_eq_impls_for_type =
            |ty: &syn::Type, others: &[&syn::Type]| -> TokenStream {
                others
                    .iter()
                    .flat_map(|other| [expand_partial_eq(ty, other), expand_partial_eq(other, ty)])
                    .collect()
            };

        [
            expand_partial_eq_impls_for_type(
                borrowed_type,
                &[&str_type, &ref_str_type, &string_type],
            ),
            expand_partial_eq_impls_for_type(
                &self.box_type,
                &[&str_type, &ref_str_type, &string_type, borrowed_type, &ref_borrowed_type],
            ),
            expand_partial_eq_impls_for_type(
                &self.owned_type,
                &[
                    &str_type,
                    &ref_str_type,
                    &string_type,
                    borrowed_type,
                    &ref_borrowed_type,
                    &self.box_type,
                    &self.arc_type,
                ],
            ),
        ]
        .into_iter()
        .collect()
    }
}
