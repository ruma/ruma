//! Implementation of the `RumaId` derive macro.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse_quote;

use crate::util::{RumaCommon, RumaCommonReexport};

mod parse;

pub(crate) use self::parse::RumaIdAttrs;

/// Generate an identifier type and various methods and trait implementations.
pub(crate) fn expand_ruma_id(
    attrs: RumaIdAttrs,
    input: syn::ItemStruct,
) -> syn::Result<TokenStream> {
    let ruma_id = RumaId::parse(attrs, input)?;
    Ok(ruma_id.expand())
}

/// The parsed input of the `RumaId` macro.
struct RumaId {
    /// The attributes on the identifier type.
    attrs: Vec<syn::Attribute>,

    /// The name of the identifier type.
    ident: syn::Ident,

    /// The generics on the identifier type.
    generics: syn::Generics,

    /// The declaration of the generics of the identifier type to use on `impl` blocks.
    impl_generics: TokenStream,

    /// The path to the function to use to validate the identifier.
    validate: Option<syn::Path>,

    /// Common types.
    types: Types,

    /// `#[cfg]` attributes for the supported internal representations.
    storage_cfg: StorageCfg,

    /// The path to use imports from the ruma-common crate.
    ruma_common: RumaCommon,
}

impl RumaId {
    /// Generate the identifier type and its various methods and trait implementations.
    fn expand(&self) -> TokenStream {
        let attrs = &self.attrs;
        let ident = &self.ident;
        let generics = &self.generics;
        let impl_generics = &self.impl_generics;

        let str = &self.types.str;
        let box_str = &self.types.box_str;
        let arc_str = &self.types.arc_str;
        let string = &self.types.string;
        let bytes = &self.types.bytes;
        let id = &self.types.id;

        let box_str_cfg = &self.storage_cfg.box_str;
        let arc_str_cfg = &self.storage_cfg.arc_str;

        let (phantom_decl, phantom_ctor) = if self.generics.params.is_empty() {
            None
        } else {
            let phantom_data = quote! { ::std::marker::PhantomData };
            let generic_types = generics.type_params().map(|param| &param.ident);

            Some((
                quote! { phantom: #phantom_data<( #(#generic_types,)* )>, },
                quote! { phantom: #phantom_data, },
            ))
        }
        .unzip();

        let as_str_docs = format!("Extracts a string slice from this `{ident}`.");
        let as_bytes_docs = format!("Extracts a byte slice from this `{ident}`.");

        let as_str_and_bytes_impls = self.expand_as_str_and_bytes_impls();
        let to_string_impls = self.expand_to_string_impls();
        let partial_eq_str_impls = self.expand_partial_eq_str_impls();
        let fallible_from_str_impls = self.expand_fallible_from_str_impls();
        let infallible_from_str_impls = self.expand_infallible_from_str_impls();

        // Implement `into_inner()` and `from_inner_unchecked()` methods behind the given `cfg`
        // attribute for all the inner representations.
        let from_into_inner_cfg_impl = |cfg: &syn::Attribute, inner: &syn::Type| {
            quote! {
                /// Consumes this identifier and returns its inner data.
                #cfg
                pub(super) fn into_inner(self) -> #inner {
                    self.inner
                }

                /// Converts the inner data to this identifier, without checking that it is valid.
                ///
                /// # Safety
                ///
                /// This function is unsafe because it does not check that the data passed to it is
                /// valid for this identifier. If this constraint is violated, it may cause memory
                /// unsafety issues with future users of this type.
                #cfg
                pub(super) unsafe fn from_inner_unchecked(inner: #inner) -> Self {
                    Self {
                        inner,
                        #phantom_ctor
                    }
                }
            }
        };
        let from_into_inner_impls = [(box_str_cfg, box_str), (arc_str_cfg, arc_str)]
            .into_iter()
            .map(|(cfg, inner)| from_into_inner_cfg_impl(cfg, inner));

        quote! {
            #( #attrs )*
            ///
            /// The inner representation for this type is variable, by default it'll use `Box<str>`,
            /// but you can change that by setting "`--cfg=ruma_identifiers_storage=...`" using
            /// `RUSTFLAGS` or `.cargo/config.toml` (under `[build]` -> `rustflags = ["..."]`)
            /// to the following;
            ///
            /// - `ruma_identifiers_storage="Arc"` to use `Arc<str>`.
            pub struct #ident #generics {
                #box_str_cfg
                inner: #box_str,
                #arc_str_cfg
                inner: #arc_str,
                #phantom_decl
            }


            #[automatically_derived]
            impl #impl_generics #id {
                #[doc = #as_str_docs]
                #[inline]
                pub fn as_str(&self) -> &#str {
                    #box_str_cfg
                    { &self.inner }
                    #arc_str_cfg
                    { &self.inner }
                }

                #[doc = #as_bytes_docs]
                #[inline]
                pub fn as_bytes(&self) -> &#bytes {
                    #box_str_cfg
                    { self.inner.as_bytes() }
                    #arc_str_cfg
                    { self.inner.as_bytes() }
                }

                #[inline]
                pub(super) fn from_str_unchecked(s: &#str) -> Self {
                    Self {
                        #box_str_cfg
                        inner: s.into(),
                        #arc_str_cfg
                        inner: s.into(),
                        #phantom_ctor
                    }
                }

                #[inline]
                pub(super) fn from_box_str_unchecked(s: #box_str) -> Self {
                    Self {
                        #box_str_cfg
                        inner: s,
                        #arc_str_cfg
                        inner: s.into(),
                        #phantom_ctor
                    }
                }

                #[inline]
                pub(super) fn from_string_unchecked(s: #string) -> Self {
                    Self {
                        #box_str_cfg
                        inner: s.into(),
                        #arc_str_cfg
                        inner: s.into(),
                        #phantom_ctor
                    }
                }

                #( #from_into_inner_impls )*
            }

            #[automatically_derived]
            impl #impl_generics ::std::clone::Clone for #id {
                fn clone(&self) -> Self {
                    unsafe { Self::from_inner_unchecked(self.inner.clone()) }
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<&#id> for #id {
                fn from(id: &#id) -> Self {
                    id.to_owned()
                }
            }

            #as_str_and_bytes_impls
            #to_string_impls
            #partial_eq_str_impls
            #fallible_from_str_impls
            #infallible_from_str_impls
        }
    }

    /// Generate `AsRef<str>`, `AsRef<[u8]>`, `(Partial)Eq`, `(Partial)Ord` and `Hash`
    /// implementations.
    fn expand_as_str_and_bytes_impls(&self) -> TokenStream {
        let impl_generics = &self.impl_generics;

        let str = &self.types.str;
        let bytes = &self.types.bytes;
        let id = &self.types.id;

        quote! {
            #[automatically_derived]
            impl #impl_generics ::std::convert::AsRef<#str> for #id {
                fn as_ref(&self) -> &#str {
                    self.as_str()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::AsRef<#bytes> for #id {
                fn as_ref(&self) -> &#bytes {
                    self.as_bytes()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::cmp::PartialEq for #id {
                fn eq(&self, other: &Self) -> bool {
                    self.as_str() == other.as_str()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::cmp::Eq for #id {}

            #[automatically_derived]
            impl #impl_generics ::std::cmp::PartialOrd for #id {
                fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
                    Some(self.cmp(other))
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::cmp::Ord for #id {
                fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                    self.as_str().cmp(other.as_str())
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::hash::Hash for #id {
                fn hash<H>(&self, state: &mut H)
                where
                    H: ::std::hash::Hasher,
                {
                    self.as_str().hash(state)
                }
            }
        }
    }

    /// Generate `std::fmt::Display`, `std::fmt::Debug` and `serde::Serialize` implementations and
    /// conversions to `Box<str>` and `String`.
    fn expand_to_string_impls(&self) -> TokenStream {
        let impl_generics = &self.impl_generics;

        let box_str = &self.types.box_str;
        let string = &self.types.string;
        let id = &self.types.id;

        let box_str_cfg = &self.storage_cfg.box_str;
        let arc_str_cfg = &self.storage_cfg.arc_str;

        let serde = self.ruma_common.reexported(RumaCommonReexport::Serde);

        quote! {
            #[automatically_derived]
            impl #impl_generics ::std::convert::From<&#id> for #box_str {
                fn from(id: &#id) -> Self {
                    id.as_str().into()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<&#id> for #string {
                fn from(id: &#id) -> Self {
                    id.as_str().into()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#id> for #box_str {
                fn from(id: #id) -> Self {
                    #box_str_cfg
                    { id.inner }
                    #arc_str_cfg
                    { id.as_str().into() }
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#id> for #string {
                fn from(id: #id) -> Self {
                    #box_str_cfg
                    { id.inner.into() }
                    #arc_str_cfg
                    { id.as_str().into() }
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::fmt::Display for #id {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    self.as_str().fmt(f)
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::fmt::Debug for #id {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    self.as_str().fmt(f)
                }
            }

            #[automatically_derived]
            impl #impl_generics #serde::Serialize for #id {
                fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
                where
                    S: #serde::Serializer,
                {
                    serializer.serialize_str(self.as_str())
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
        let generic_params = &self.generics.params;
        let impl_generics = &self.impl_generics;

        let str = &self.types.str;
        let cow = &self.types.cow;
        let box_str = &self.types.box_str;
        let string = &self.types.string;
        let cow_str = &self.types.cow_str;
        let id = &self.types.id;

        let ruma_common = &self.ruma_common;
        let serde = ruma_common.reexported(RumaCommonReexport::Serde);

        let parse_doc_header = format!("Try parsing a `&str` into an `{ident}`.");

        Some(quote! {
            #[automatically_derived]
            impl #impl_generics #id {
                #[doc = #parse_doc_header]
                ///
                /// The same can also be done using `FromStr`, `TryFrom` or `TryInto`.
                /// This function is simply more constrained and thus useful in generic contexts.
                pub fn parse(
                    s: impl ::std::convert::AsRef<#str>,
                ) -> ::std::result::Result<Self, #ruma_common::IdParseError> {
                    let s = s.as_ref();
                    #validate(s)?;
                    ::std::result::Result::Ok(Self::from_str_unchecked(s))
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::str::FromStr for #id {
                type Err = #ruma_common::IdParseError;

                fn from_str(s: &#str) -> ::std::result::Result<Self, Self::Err> {
                    Self::parse(s)
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::TryFrom<&#str> for #id {
                type Error = #ruma_common::IdParseError;

                fn try_from(s: &#str) -> ::std::result::Result<Self, Self::Error> {
                    Self::parse(s)
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::TryFrom<#box_str> for #id {
                type Error = #ruma_common::IdParseError;

                fn try_from(s: #box_str) -> ::std::result::Result<Self, Self::Error> {
                    #validate(&s)?;
                    ::std::result::Result::Ok(Self::from_box_str_unchecked(s))
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::TryFrom<#string> for #id {
                type Error = #ruma_common::IdParseError;

                fn try_from(s: #string) -> ::std::result::Result<Self, Self::Error> {
                    #validate(s.as_ref())?;
                    ::std::result::Result::Ok(Self::from_string_unchecked(s))
                }
            }

            #[automatically_derived]
            impl<'a, #generic_params> ::std::convert::TryFrom<#cow_str> for #id {
                type Error = #ruma_common::IdParseError;

                fn try_from(s: #cow_str) -> ::std::result::Result<Self, Self::Error> {
                    match s {
                        #cow::Borrowed(s) => s.try_into(),
                        #cow::Owned(s) => s.try_into(),
                    }
                }
            }

            #[automatically_derived]
            impl<'de, #generic_params> #serde::Deserialize<'de> for #id {
                fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
                where
                    D: #serde::Deserializer<'de>,
                {
                    use #serde::de::Error;

                    // We always deserialize as a string to make sure that it is valid UTF-8,
                    // regardless of the inner representation.
                    #ruma_common::serde::deserialize_cow_str(deserializer)?
                        .try_into()
                        .map_err(D::Error::custom)
                }
            }
        })
    }

    /// Generate `From<&str>` and other infallible string conversions implementations for this
    /// identifier, if it doesn't have a validation function.
    fn expand_infallible_from_str_impls(&self) -> Option<TokenStream> {
        if self.validate.is_some() {
            return None;
        }

        let impl_generics = &self.impl_generics;
        let generic_params = &self.generics.params;

        let str = &self.types.str;
        let cow = &self.types.cow;
        let box_str = &self.types.box_str;
        let string = &self.types.string;
        let cow_str = &self.types.cow_str;
        let id = &self.types.id;

        let ruma_common = &self.ruma_common;
        let serde = ruma_common.reexported(RumaCommonReexport::Serde);

        Some(quote! {
            #[automatically_derived]
            impl #impl_generics ::std::convert::From<&#str> for #id {
                fn from(s: &#str) -> Self {
                    Self::from_str_unchecked(s)
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#box_str> for #id {
                fn from(s: #box_str) -> Self {
                    Self::from_box_str_unchecked(s)
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#string> for #id {
                fn from(s: #string) -> Self {
                    Self::from_string_unchecked(s)
                }
            }

            #[automatically_derived]
            impl<'a, #generic_params> ::std::convert::From<#cow_str> for #id {
                fn from(s: #cow_str) -> Self {
                    match s {
                        #cow::Borrowed(s) => s.into(),
                        #cow::Owned(s) => s.into(),
                    }
                }
            }

            #[automatically_derived]
            impl<'de, #generic_params> #serde::Deserialize<'de> for #id {
                fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
                where
                    D: #serde::Deserializer<'de>,
                {
                    // We always deserialize as a string to make sure that it is valid UTF-8,
                    // regardless of the inner representation.
                    #ruma_common::serde::deserialize_cow_str(deserializer).map(::std::convert::Into::into)
                }
            }
        })
    }

    /// Generate `std::cmp::PartialEq` implementations by comparing strings.
    fn expand_partial_eq_str_impls(&self) -> TokenStream {
        let str = &self.types.str;
        let string = &self.types.string;
        let cow_str = &self.types.cow_str;
        let id = &self.types.id;

        let ref_str: syn::Type = parse_quote! { &#str };

        let self_ident = syn::Ident::new("self", Span::call_site());
        let other_ident = syn::Ident::new("other", Span::call_site());

        // Get the string representation of the type.
        let as_ref_str_impl = |ident: &syn::Ident| {
            quote! { ::std::convert::AsRef::<#str>::as_ref(#ident) }
        };

        // Implement `PartialEq` with the given lhs and rhs types.
        let expand_partial_eq = |lhs: &syn::Type, rhs: &syn::Type| {
            let self_as_str = as_ref_str_impl(&self_ident);
            let other_as_str = as_ref_str_impl(&other_ident);

            let impl_generics = if lhs == cow_str || rhs == cow_str {
                let generic_params = &self.generics.params;
                &quote! { <'a, #generic_params> }
            } else {
                &self.impl_generics
            };

            quote! {
                #[automatically_derived]
                impl #impl_generics ::std::cmp::PartialEq<#rhs> for #lhs {
                    fn eq(&self, other: &#rhs) -> bool {
                        #self_as_str == #other_as_str
                    }
                }
            }
        };

        // Implement reciprocal `PartialEq` implementation for the identifier type with the common
        // string types.
        [str, &ref_str, string, cow_str]
            .into_iter()
            .flat_map(|other| [expand_partial_eq(id, other), expand_partial_eq(other, id)])
            .collect()
    }
}

/// Common types.
struct Types {
    /// `str`.
    str: syn::Type,

    /// `Cow`.
    cow: syn::Type,

    /// `Box<str>`.
    box_str: syn::Type,

    /// `Arc<str>`.
    arc_str: syn::Type,

    /// `String`.
    string: syn::Type,

    /// `Cow<'a, str>`.
    cow_str: syn::Type,

    /// `[u8]`.
    bytes: syn::Type,

    /// `{id}`, the identifier type with generics, if any.
    id: syn::Type,
}

impl Types {
    fn new(ident: &syn::Ident, type_generics: syn::TypeGenerics<'_>) -> Self {
        let str = parse_quote! { ::std::primitive::str };
        let cow = parse_quote! { ::std::borrow::Cow };

        Self {
            box_str: parse_quote! { ::std::boxed::Box<#str> },
            arc_str: parse_quote! { ::std::sync::Arc<#str> },
            string: parse_quote! { ::std::string::String },
            cow_str: parse_quote! { #cow<'a, #str> },
            bytes: parse_quote! { [::std::primitive::u8] },
            id: parse_quote! { #ident #type_generics },
            str,
            cow,
        }
    }
}

/// `#[cfg]` attributes for the supported internal representations.
struct StorageCfg {
    /// Attribute for the default internal representation, `Box<str>`.
    box_str: syn::Attribute,

    /// Attribute for the `Arc<str>` internal representation.
    arc_str: syn::Attribute,
}

impl StorageCfg {
    fn new() -> Self {
        let key = quote! { ruma_identifiers_storage };

        Self {
            box_str: parse_quote! { #[cfg(not(#key = "Arc"))] },
            arc_str: parse_quote! { #[cfg(#key = "Arc")] },
        }
    }
}
