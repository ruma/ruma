//! Implementation of the `IdDst` derive macro.

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_quote;

use crate::util::{RumaCommon, RumaCommonReexport};

mod owned_id;
mod parse;

use self::owned_id::OwnedId;

/// Generate the `Owned` version of an identifier and various trait implementations.
pub(crate) fn expand_id_dst(input: syn::ItemStruct) -> syn::Result<TokenStream> {
    let id_dst = IdDst::parse(input)?;

    let as_str_and_bytes_impls = id_dst.expand_as_str_and_bytes_impls();
    let id_to_string_impls = id_dst.expand_to_string_impls(&id_dst.id_type);
    let unchecked_from_str_impls = id_dst.expand_unchecked_from_str_impls();
    let fallible_from_str_impls = id_dst.expand_fallible_from_str_impls();
    let infallible_from_str_impls = id_dst.expand_infallible_from_str_impls();
    let partial_eq_impls = id_dst.expand_partial_eq_impls();

    let owned_id_struct = id_dst.owned_id.expand_struct(&id_dst);
    let owned_id_to_string_impls = id_dst.expand_to_string_impls(&id_dst.owned_id.id_type);
    let id_from_into_owned_impls = id_dst.expand_id_from_into_owned_impls();

    Ok(quote! {
        #as_str_and_bytes_impls
        #id_to_string_impls
        #unchecked_from_str_impls
        #owned_id_struct
        #owned_id_to_string_impls
        #id_from_into_owned_impls
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
    id_type: syn::Type,

    /// The generics on the borrowed type.
    generics: syn::Generics,

    /// The declaration of the generics of the borrowed type to use on `impl` blocks.
    impl_generics: TokenStream,

    /// The path to the function to use to validate the identifier.
    validate: Option<syn::Path>,

    /// The index of the `str` field.
    ///
    /// This is assumed to be the last field of the tuple struct.
    str_field_index: syn::Index,

    /// Data about the owned type.
    owned_id: OwnedId,

    /// Common types.
    types: Types,

    /// The path to use imports from the ruma-common crate.
    ruma_common: RumaCommon,
}

impl IdDst {
    /// Generate `AsRef<str>` and `AsRef<[u8]>` implementations and string conversions for this
    /// identifier.
    fn expand_as_str_and_bytes_impls(&self) -> TokenStream {
        let ident = &self.ident;
        let impl_generics = &self.impl_generics;
        let str_field_index = &self.str_field_index;

        let str = &self.types.str;
        let bytes = &self.types.bytes;
        let string = &self.types.string;
        let id = &self.id_type;

        let as_str_docs = format!("Extracts a string slice from this `{ident}`.");
        let as_bytes_docs = format!("Extracts a byte slice from this `{ident}`.");

        quote! {
            impl #impl_generics #id {
                #[doc = #as_str_docs]
                #[inline]
                pub fn as_str(&self) -> &#str {
                    &self.#str_field_index
                }

                #[doc = #as_bytes_docs]
                #[inline]
                pub fn as_bytes(&self) -> &#bytes {
                    self.as_str().as_bytes()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::borrow::Borrow<#str> for #id {
                fn borrow(&self) -> &#str {
                    self.as_str()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::AsRef<#id> for #id {
                fn as_ref(&self) -> &#id {
                    self
                }
            }

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
            impl #impl_generics ::std::convert::From<&#id> for #string {
                fn from(id: &#id) -> Self {
                    id.as_str().to_owned()
                }
            }
        }
    }

    /// Generate unchecked private methods to convert a string type to the identifier.
    fn expand_unchecked_from_str_impls(&self) -> TokenStream {
        let impl_generics = &self.impl_generics;

        let str = &self.types.str;
        let id = &self.id_type;

        quote! {
            #[automatically_derived]
            impl #impl_generics #id {
                pub(super) const fn from_borrowed_unchecked(s: &#str) -> &Self {
                    unsafe { ::std::mem::transmute(s) }
                }
            }
        }
    }

    /// Expand conversions of the borrowed type from and to the owned type.
    fn expand_id_from_into_owned_impls(&self) -> TokenStream {
        let ident = &self.ident;
        let id = &self.id_type;
        let owned_ident = &self.owned_id.ident;
        let owned_id = &self.owned_id.id_type;
        let impl_generics = &self.impl_generics;

        quote! {
            #[automatically_derived]
            impl #impl_generics ::std::ops::Deref for #owned_id {
                type Target = #id;

                fn deref(&self) -> &Self::Target {
                    self.as_ref()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::borrow::Borrow<#id> for #owned_id {
                fn borrow(&self) -> &#id {
                    self.as_ref()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::AsRef<#id> for #owned_id {
                fn as_ref(&self) -> &#id {
                    #ident::from_borrowed_unchecked(self.as_inner_str())
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::borrow::ToOwned for #id {
                type Owned = #owned_id;

                fn to_owned(&self) -> Self::Owned {
                    #owned_ident::from_str_unchecked(self.as_str())
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<&#id> for #owned_id {
                fn from(id: &#id) -> Self {
                    id.to_owned()
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
        let owned_ident = &self.owned_id.ident;
        let generic_params = &self.generics.params;
        let impl_generics = &self.impl_generics;

        let ruma_common = &self.ruma_common;
        let serde = ruma_common.reexported(RumaCommonReexport::Serde);

        let parse_doc_header = format!("Try parsing a `&str` into an `{owned_ident}`.");

        let str = &self.types.str;
        let cow = &self.types.cow;
        let box_str = &self.types.box_str;
        let string = &self.types.string;
        let cow_str = &self.types.cow_str;
        let id = &self.id_type;
        let owned_id = &self.owned_id.id_type;

        Some(quote! {
            #[automatically_derived]
            impl #impl_generics #id {
                #[doc = #parse_doc_header]
                ///
                /// The same can also be done using `FromStr`, `TryFrom` or `TryInto`.
                /// This function is simply more constrained and thus useful in generic contexts.
                pub fn parse(
                    s: impl ::std::convert::AsRef<#str>,
                ) -> ::std::result::Result<#owned_id, #ruma_common::IdParseError> {
                    let s = s.as_ref();
                    #validate(s)?;
                    ::std::result::Result::Ok(#owned_ident::from_str_unchecked(s))
                }
            }

            #[automatically_derived]
            impl<'a, #generic_params> ::std::convert::TryFrom<&'a #str> for &'a #id {
                type Error = #ruma_common::IdParseError;

                fn try_from(s: &'a #str) -> ::std::result::Result<Self, Self::Error> {
                    #validate(s)?;
                    ::std::result::Result::Ok(#ident::from_borrowed_unchecked(s))
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::str::FromStr for #owned_id {
                type Err = #ruma_common::IdParseError;

                fn from_str(s: &#str) -> ::std::result::Result<Self, Self::Err> {
                    #ident::parse(s)
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::TryFrom<&#str> for #owned_id {
                type Error = #ruma_common::IdParseError;

                fn try_from(s: &#str) -> ::std::result::Result<Self, Self::Error> {
                    #ident::parse(s)
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::TryFrom<#box_str> for #owned_id {
                type Error = #ruma_common::IdParseError;

                fn try_from(s: #box_str) -> ::std::result::Result<Self, Self::Error> {
                    #validate(&s)?;
                    ::std::result::Result::Ok(#owned_ident::from_box_str_unchecked(s))
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::TryFrom<#string> for #owned_id {
                type Error = #ruma_common::IdParseError;

                fn try_from(s: #string) -> ::std::result::Result<Self, Self::Error> {
                    #validate(&s)?;
                    ::std::result::Result::Ok(#owned_ident::from_string_unchecked(s))
                }
            }

            #[automatically_derived]
            impl<'a, #generic_params> ::std::convert::TryFrom<#cow_str> for #owned_id {
                type Error = #ruma_common::IdParseError;

                fn try_from(s: #cow_str) -> ::std::result::Result<Self, Self::Error> {
                    match s {
                        #cow::Borrowed(s) => s.try_into(),
                        #cow::Owned(s) => s.try_into(),
                    }
                }
            }

            #[automatically_derived]
            impl<'de, #generic_params> #serde::Deserialize<'de> for #owned_id {
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

        let ident = &self.ident;
        let owned_ident = &self.owned_id.ident;
        let impl_generics = &self.impl_generics;
        let generic_params = &self.generics.params;

        let str = &self.types.str;
        let cow = &self.types.cow;
        let box_str = &self.types.box_str;
        let string = &self.types.string;
        let cow_str = &self.types.cow_str;
        let id = &self.id_type;
        let owned_id = &self.owned_id.id_type;

        let ruma_common = &self.ruma_common;
        let serde = ruma_common.reexported(RumaCommonReexport::Serde);

        Some(quote! {
            #[automatically_derived]
            impl<'a, #generic_params> ::std::convert::From<&'a #str> for &'a #id {
                fn from(s: &'a #str) -> Self {
                    #ident::from_borrowed_unchecked(s)
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<&#str> for #owned_id {
                fn from(s: &#str) -> Self {
                    #owned_ident::from_str_unchecked(s)
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#box_str> for #owned_id {
                fn from(s: #box_str) -> Self {
                    #owned_ident::from_box_str_unchecked(s)
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#string> for #owned_id {
                fn from(s: #string) -> Self {
                    #owned_ident::from_string_unchecked(s)
                }
            }

            #[automatically_derived]
            impl<'a, #generic_params> ::std::convert::From<#cow_str> for #owned_id {
                fn from(s: #cow_str) -> Self {
                    match s {
                        #cow::Borrowed(s) => s.into(),
                        #cow::Owned(s) => s.into(),
                    }
                }
            }

            #[automatically_derived]
            impl<'de, #generic_params> #serde::Deserialize<'de> for #owned_id {
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

    /// Generate `std::fmt::Display`, `std::fmt::Debug` and `serde::Serialize` traits
    /// implementations, using it's `.as_str()` function.
    fn expand_to_string_impls(&self, ty: &syn::Type) -> TokenStream {
        let serde = self.ruma_common.reexported(RumaCommonReexport::Serde);

        let impl_generics = &self.impl_generics;
        let str = &self.types.str;

        quote! {
            #[automatically_derived]
            impl #impl_generics ::std::fmt::Display for #ty {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    ::std::convert::AsRef::<#str>::as_ref(self).fmt(f)
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::fmt::Debug for #ty {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    ::std::convert::AsRef::<#str>::as_ref(self).fmt(f)
                }
            }

            #[automatically_derived]
            impl #impl_generics #serde::Serialize for #ty {
                fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
                where
                    S: #serde::Serializer,
                {
                    serializer.serialize_str(::std::convert::AsRef::<#str>::as_ref(self))
                }
            }
        }
    }

    /// Generate `std::cmp::PartialEq` implementations by comparing strings.
    fn expand_partial_eq_impls(&self) -> TokenStream {
        let generics_params = &self.generics.params;
        let impl_generics = &self.impl_generics;

        let str = &self.types.str;
        let string = &self.types.string;
        let cow_str = &self.types.cow_str;
        let id = &self.id_type;
        let owned_id = &self.owned_id.id_type;

        let ref_id: syn::Type = parse_quote! { &#id };
        let ref_str: syn::Type = parse_quote! { &#str };
        let cow_generics = quote! { <'a, #generics_params> };

        // Implement `PartialEq` with the given lhs and rhs types.
        let expand_partial_eq = |lhs: &syn::Type, rhs: &syn::Type| {
            let impl_generics =
                if *lhs == *cow_str || *rhs == *cow_str { &cow_generics } else { impl_generics };

            quote! {
                #[automatically_derived]
                impl #impl_generics ::std::cmp::PartialEq<#rhs> for #lhs {
                    fn eq(&self, other: &#rhs) -> bool {
                        ::std::convert::AsRef::<#str>::as_ref(self) == ::std::convert::AsRef::<#str>::as_ref(other)
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
            expand_partial_eq_impls_for_type(id, &[str, &ref_str, string, cow_str]),
            expand_partial_eq_impls_for_type(
                owned_id,
                &[str, &ref_str, string, cow_str, id, &ref_id],
            ),
        ]
        .into_iter()
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

    /// `triomphe::ThinArc<(), u8>`.
    thin_arc_bytes: syn::Type,
}

impl Types {
    fn new(ruma_common: &RumaCommon) -> Self {
        let str = parse_quote! { ::std::primitive::str };
        let byte = quote! { ::std::primitive::u8 };
        let cow = parse_quote! { ::std::borrow::Cow };
        let triomphe = ruma_common.reexported(RumaCommonReexport::Triomphe);

        Self {
            box_str: parse_quote! { ::std::boxed::Box<#str> },
            arc_str: parse_quote! { ::std::sync::Arc<#str> },
            string: parse_quote! { ::std::string::String },
            cow_str: parse_quote! { #cow<'a, #str> },
            bytes: parse_quote! { [#byte] },
            thin_arc_bytes: parse_quote! { #triomphe::ThinArc<(), #byte> },
            str,
            cow,
        }
    }
}
