//! Implementation of the `ruma_id` attribute macro.

use std::borrow::Cow;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse_quote;

mod parse;

pub(crate) use self::parse::RumaIdAttrs;
use crate::util::{RumaCommon, RumaCommonReexport};

/// Generate the `Owned` version of an identifier and various trait implementations.
pub(crate) fn expand_ruma_id(
    ruma_id_attrs: RumaIdAttrs,
    item: syn::ItemStruct,
) -> syn::Result<TokenStream> {
    let ruma_id = RumaId::parse(ruma_id_attrs, item)?;

    let struct_impl = ruma_id.expand_struct();
    let id_to_string_impls = ruma_id.expand_to_string_impls();
    let fallible_from_str_impls = ruma_id.expand_fallible_from_str_impls();
    let infallible_from_str_impls = ruma_id.expand_infallible_from_str_impls();
    let partial_eq_impls = ruma_id.expand_partial_eq_impls();

    Ok(quote! {
        #struct_impl
        #id_to_string_impls
        #fallible_from_str_impls
        #infallible_from_str_impls
        #partial_eq_impls
    })
}

/// The parsed input of the `ruma_id` macro.
struct RumaId {
    /// The name of the identifier type.
    ident: syn::Ident,

    /// The attributes on the identifier type.
    attrs: Vec<syn::Attribute>,

    /// The visibility of the identifier type.
    vis: syn::Visibility,

    /// The identifier type with generics, if any.
    id_type: syn::Type,

    /// The generics on the identifier type.
    generics: syn::Generics,

    /// The declaration of the generics of the identifier type to use on `impl` blocks.
    impl_generics: TokenStream,

    /// The path to the function to use to validate the identifier.
    validate: Option<syn::Path>,

    /// The size of the inline array for the `SmallVec` inner representation.
    smallvec_inline_bytes: usize,

    /// `#[cfg]` attributes for the internal storage representations.
    storage_attrs: StorageCfgAttributes,

    /// Common types.
    types: Types,

    /// The path to use imports from the ruma-common crate.
    ruma_common: RumaCommon,
}

impl RumaId {
    /// Expand an implementation for all the internal storage representations by calling the given
    /// function for each value and concatenating the outputs gated behind the proper `#[cfg]`
    /// attribute.
    fn expand_for_each_storage_value<F: Fn(&StorageCfgValue) -> TokenStream>(
        &self,
        expand_value_fn: F,
    ) -> TokenStream {
        StorageCfgValue::ALL
            .iter()
            .map(|value| {
                let cfg_attr = value.cfg_attr(&self.storage_attrs);
                let expanded = expand_value_fn(value);

                quote! {
                    #cfg_attr
                    #expanded
                }
            })
            .collect()
    }

    /// Generate the identifier type and its basic implementations.
    pub(super) fn expand_struct(&self) -> TokenStream {
        let ident = &self.ident;
        let attrs = &self.attrs;
        let vis = &self.vis;
        let id = &self.id_type;
        let generics = &self.generics;
        let impl_generics = &self.impl_generics;
        let types = &self.types;
        let ruma_common = &self.ruma_common;

        let str = &types.str;
        let box_str = &types.box_str;
        let string = &types.string;
        let bytes = &types.bytes;

        // Expanded code to access the inner field.
        let self_inner_field = quote! { self.inner };
        let id_var = quote! { id };
        let id_inner_field = quote! { #id_var.inner };
        // The name of the string variable when constructing an identifier from a string.
        let string_var = syn::Ident::new("s", Span::call_site());

        let (phantom_decl, phantom_ctor) = if generics.params.is_empty() {
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

        let doc_values = StorageCfgValue::ALL
            .iter()
            .map(|value| StorageCfgValue::doc(value, self.smallvec_inline_bytes))
            .collect::<Vec<_>>()
            .join("\n* ");

        let inner_types_decl = self.expand_for_each_storage_value(|value| {
            let inner_type = value.inner_type(types);
            quote! { inner: #inner_type, }
        });

        let from_str_impls = self.expand_for_each_storage_value(|value| {
            let expanded = value.expand_from_str_impl(&string_var, types);
            quote! { inner: #expanded, }
        });
        let from_box_str_impls = self.expand_for_each_storage_value(|value| {
            let expanded = value.expand_from_box_str_impl(&string_var, types);
            quote! { inner: #expanded, }
        });
        let from_string_impls = self.expand_for_each_storage_value(|value| {
            let expanded = value.expand_from_string_impl(&string_var, types);
            quote! { inner: #expanded, }
        });

        let as_str_docs = format!("Extracts a string slice from this `{ident}`.");
        let as_str_impls = self.expand_for_each_storage_value(|value| {
            let expanded = value.expand_as_str_impl(&self_inner_field, types);
            quote! { { #expanded } }
        });

        let as_bytes_docs = format!("Extracts a byte slice from this `{ident}`.");
        let as_bytes_impls = self.expand_for_each_storage_value(|value| {
            let expanded = value.expand_as_bytes_impl(&self_inner_field);
            quote! { { #expanded } }
        });

        let into_inner_impls = self.expand_for_each_storage_value(|value| {
            let inner_type = value.inner_type(types);

            quote! {
                /// Consumes this identifier and returns its inner data.
                pub(super) fn into_inner(self) -> #inner_type {
                    #self_inner_field
                }
            }
        });
        let from_inner_impls = self.expand_for_each_storage_value(|value| {
            let inner_type = value.inner_type(types);

            quote! {
                /// Converts the inner data to this identifier, without checking that it is valid.
                ///
                /// # Safety
                ///
                /// This function is unsafe because it does not check that the data passed to it is
                /// valid for this identifier. If this constraint is violated, it may cause memory
                /// unsafety issues with future users of this type.
                pub(super) unsafe fn from_inner_unchecked(inner: #inner_type) -> Self {
                    Self {
                        inner,
                        #phantom_ctor
                    }
                }
            }
        });

        let zeroize_doc_header = format!(
            "Securely zero memory (aka [zeroize](https://en.wikipedia.org/wiki/Zeroisation)) of `{ident}`."
        );
        let zeroize_impls = self.expand_for_each_storage_value(|value| {
            let expanded = value.expand_zeroize_impl(&self_inner_field, ruma_common);
            quote! { { #expanded } }
        });

        let into_box_str_impls = self.expand_for_each_storage_value(|value| {
            let expanded = value.expand_into_box_str_impl(&id_var, &id_inner_field);
            quote! { { #expanded } }
        });
        let into_string_impls = self.expand_for_each_storage_value(|value| {
            let expanded = value.expand_into_string_impl(&id_var, &id_inner_field, types);
            quote! { { #expanded } }
        });

        quote! {
            #( #attrs )*
            ///
            /// ## Inner representation
            ///
            /// By default, this type uses a `Box<str>` internally. The inner representation can be selected at
            /// compile time by using one of the following supported values:
            ///
            #[doc = #doc_values]
            ///
            /// The selected value can be set by using the `ruma_identifiers_storage` compile-time `cfg` setting.
            /// This setting can be configured using the `RUSTFLAGS` environment variable at build time, like this:
            ///
            /// ```shell
            /// RUSTFLAGS="--cfg ruma_identifiers_storage=\"{value}\""
            /// ```
            ///
            /// Or in `.cargo/config.toml`:
            ///
            /// ```toml
            /// # General setting for all targets, overridden by per-target `rustflags` setting if set.
            /// [build]
            /// rustflags = ["--cfg", "ruma_identifiers_storage=\"{value}\""]
            ///
            /// # Per-target setting.
            /// [target.<triple/cfg>]
            /// rustflags = ["--cfg", "ruma_identifiers_storage=\"{value}\""]
            /// ```
            ///
            /// This setting can also be configured using the `RUMA_IDENTIFIERS_STORAGE` environment variable at
            /// compile time, which has the benefit of not requiring to re-compile the whole dependency chain
            /// when the value is changed, like this:
            ///
            /// ```shell
            /// RUMA_IDENTIFIERS_STORAGE="{value}"
            /// ```
            #vis struct #ident #generics {
                #inner_types_decl
                #phantom_decl
            }

            #[automatically_derived]
            impl #impl_generics #id {
                pub(super) fn from_str_unchecked(#string_var: &#str) -> Self {
                    Self {
                        #from_str_impls
                        #phantom_ctor
                    }
                }

                pub(super) fn from_box_str_unchecked(#string_var: #box_str) -> Self {
                    Self {
                        #from_box_str_impls
                        #phantom_ctor
                    }
                }

                pub(super) fn from_string_unchecked(#string_var: #string) -> Self {
                    Self {
                        #from_string_impls
                        #phantom_ctor
                    }
                }

                #[doc = #as_str_docs]
                #[inline]
                pub fn as_str(&self) -> &#str {
                    #as_str_impls
                }

                #[doc = #as_bytes_docs]
                #[inline]
                pub fn as_bytes(&self) -> &#bytes {
                    #as_bytes_impls
                }

                #into_inner_impls
                #from_inner_impls

                #[doc = #zeroize_doc_header]
                ///
                /// This method zeroizes this type by writing zeros in its
                /// memory location before freeing it. It internally uses
                /// [the `zeroize` crate][`zeroize`]. Note that this type
                /// doesn't implement the `zeroize::Zeroize` trait because the
                /// `Zeroize::zeroize` method takes a `&mut self`, which means
                /// we could put this type into an inconsistent state if it is
                /// used after calling that method. Instead, this method takes
                /// ownership of the type, ensuring it's impossible to misuse
                /// it.
                ///
                /// # Implementation details
                ///
                /// If the `ruma_identifiers_storage` configuration is set to
                /// `Arc` or `ThinArc`, this type will be zeroized if and only if
                /// there is no other strong or weak reference to this same location.
                ///
                /// [`zeroize`]: https://docs.rs/zeroize/
                pub fn zeroize(mut self) {
                    #zeroize_impls
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::clone::Clone for #id {
                fn clone(&self) -> Self {
                    unsafe { Self::from_inner_unchecked(self.inner.clone()) }
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::cmp::PartialEq for #id {
                fn eq(&self, other: &Self) -> ::std::primitive::bool {
                    self.inner.eq(&other.inner)
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::cmp::Eq for #id {}

            #[automatically_derived]
            impl #impl_generics ::std::cmp::PartialOrd for #id {
                fn partial_cmp(&self, other: &Self) -> ::std::option::Option<::std::cmp::Ordering> {
                    ::std::option::Option::Some(self.cmp(other))
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::cmp::Ord for #id {
                fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                    self.inner.cmp(&other.inner)
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

            #[automatically_derived]
            impl #impl_generics ::std::borrow::Borrow<#str> for #id {
                fn borrow(&self) -> &#str {
                    self.as_str()
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
            impl #impl_generics ::std::convert::From<#id> for #box_str {
                fn from(id: #id) -> Self {
                    #into_box_str_impls
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#id> for #string {
                fn from(id: #id) -> Self {
                    #into_string_impls
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<&#id> for #string {
                fn from(id: &#id) -> Self {
                    id.as_str().to_owned()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<&#id> for #id {
                fn from(id: &#id) -> Self {
                    id.clone()
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

        let ruma_common = &self.ruma_common;
        let serde = ruma_common.reexported(RumaCommonReexport::Serde);

        let parse_doc_header = format!("Try parsing a `&str` into an `{ident}`.");

        let str = &self.types.str;
        let cow = &self.types.cow;
        let box_str = &self.types.box_str;
        let string = &self.types.string;
        let cow_str = &self.types.cow_str;
        let id = &self.id_type;

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
                    #validate(&s)?;
                    ::std::result::Result::Ok(Self::from_string_unchecked(s))
                }
            }

            #[automatically_derived]
            impl<'a, #generic_params> ::std::convert::TryFrom<#cow_str> for #id {
                type Error = #ruma_common::IdParseError;

                fn try_from(s: #cow_str) -> ::std::result::Result<Self, Self::Error> {
                    #validate(&s)?;
                    Ok(match s {
                        #cow::Borrowed(s) => Self::from_str_unchecked(s),
                        #cow::Owned(s) => Self::from_string_unchecked(s),
                    })
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
        let id = &self.id_type;

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
                        #cow::Borrowed(s) => Self::from_str_unchecked(s),
                        #cow::Owned(s) => Self::from_string_unchecked(s),
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

    /// Generate `std::fmt::Display`, `std::fmt::Debug` and `serde::Serialize` traits
    /// implementations, using it's `.as_str()` function.
    fn expand_to_string_impls(&self) -> TokenStream {
        let serde = self.ruma_common.reexported(RumaCommonReexport::Serde);

        let id = &self.id_type;
        let impl_generics = &self.impl_generics;

        quote! {
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

    /// Generate `std::cmp::PartialEq` implementations by comparing strings.
    fn expand_partial_eq_impls(&self) -> TokenStream {
        let generics_params = &self.generics.params;
        let impl_generics = &self.impl_generics;

        let str = &self.types.str;
        let string = &self.types.string;
        let cow_str = &self.types.cow_str;
        let id = &self.id_type;

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

        // Implement reciprocal `PartialEq` implementation for the identifier type with common
        // string types.
        [str, &ref_str, string, cow_str, &ref_id]
            .iter()
            .flat_map(|other| [expand_partial_eq(id, other), expand_partial_eq(other, id)])
            .collect()
    }
}

/// The supported values for the identifiers internal storage representations.
enum StorageCfgValue {
    /// The default `Box<str>` internal representation.
    Default,

    /// `Arc`, the `Arc<str>` internal representation.
    Arc,

    /// `ThinArc`, the `triomphe::ThinArc<(), u8>` internal representation.
    ThinArc,

    /// `SmallVec`, the `smallvec::SmallVec<[u8; N]>` internal representation.
    SmallVec,
}

impl StorageCfgValue {
    /// All the possible values.
    const ALL: &'static [Self] = &[Self::Default, Self::Arc, Self::ThinArc, Self::SmallVec];

    /// The string representation for this value.
    ///
    /// Returns `None` for the `Default` variant.
    fn as_str(&self) -> Option<&'static str> {
        Some(match self {
            Self::Default => None?,
            Self::Arc => "Arc",
            Self::ThinArc => "ThinArc",
            Self::SmallVec => "SmallVec",
        })
    }

    /// The `#[cfg]` attribute for this value.
    fn cfg_attr<'a>(&self, attrs: &'a StorageCfgAttributes) -> &'a syn::Attribute {
        match self {
            Self::Default => &attrs.default,
            Self::Arc => &attrs.arc,
            Self::ThinArc => &attrs.thin_arc,
            Self::SmallVec => &attrs.small_vec,
        }
    }

    /// The docs for this value.
    ///
    /// This should be a doc string that looks like `` `{value}` -- Use a `{type}`.``.
    fn doc(&self, smallvec_inline_bytes: usize) -> Cow<'static, str> {
        match self {
            Self::Default => Cow::Borrowed(""),
            Self::Arc => Cow::Borrowed("`Arc` -- Use an `Arc<str>`."),
            Self::ThinArc => Cow::Borrowed(
                "`ThinArc` -- Use a `triomphe::ThinArc<(), u8>`. \
                 Requires the `triomphe` cargo feature.",
            ),
            Self::SmallVec => Cow::Owned(format!(
                "`SmallVec` -- Use a `smallvec::SmallVec<[u8; {smallvec_inline_bytes}]>`. \
                 Requires the `smallvec` cargo feature.",
            )),
        }
    }

    /// The type of the internal storage.
    fn inner_type<'a>(&self, types: &'a Types) -> &'a syn::Type {
        match self {
            Self::Default => &types.box_str,
            Self::Arc => &types.arc_str,
            Self::ThinArc => &types.thin_arc_bytes,
            Self::SmallVec => &types.small_vec_bytes,
        }
    }

    /// Expand the implementation to convert a `&str` to the inner type.
    fn expand_from_str_impl(&self, string_var: &syn::Ident, types: &Types) -> TokenStream {
        match self {
            Self::Default | Self::Arc => quote! {
                #string_var.into()
            },
            Self::ThinArc => {
                let thin_arc_bytes = &types.thin_arc_bytes;
                quote! {
                    <#thin_arc_bytes>::from_header_and_slice((), #string_var.as_bytes())
                }
            }
            Self::SmallVec => {
                let small_vec_bytes = &types.small_vec_bytes;
                quote! {
                    <#small_vec_bytes>::from_slice(#string_var.as_bytes())
                }
            }
        }
    }

    /// Expand the implementation to convert a `Box<str>` to the inner type.
    fn expand_from_box_str_impl(&self, string_var: &syn::Ident, types: &Types) -> TokenStream {
        match self {
            Self::Default => quote! { #string_var },
            Self::Arc => quote! {
                #string_var.into()
            },
            Self::ThinArc => {
                let thin_arc_bytes = &types.thin_arc_bytes;
                quote! {
                    <#thin_arc_bytes>::from_header_and_slice((), #string_var.as_bytes())
                }
            }
            Self::SmallVec => {
                let str = &types.str;
                let small_vec_bytes = &types.small_vec_bytes;
                quote! {
                    <#small_vec_bytes>::from_vec(#str::into_boxed_bytes(#string_var).into())
                }
            }
        }
    }

    /// Expand the implementation to convert a `String` to the inner type.
    fn expand_from_string_impl(&self, string_var: &syn::Ident, types: &Types) -> TokenStream {
        match self {
            Self::Default | Self::Arc => quote! {
                #string_var.into()
            },
            Self::ThinArc => {
                let thin_arc_bytes = &types.thin_arc_bytes;
                quote! {
                    <#thin_arc_bytes>::from_header_and_slice((), #string_var.as_bytes())
                }
            }
            Self::SmallVec => {
                let small_vec_bytes = &types.small_vec_bytes;
                quote! {
                    <#small_vec_bytes>::from_vec(#string_var.into_bytes())
                }
            }
        }
    }

    /// Expand the implementation to access the inner type as a `&str`.
    fn expand_as_str_impl(&self, inner_field: &TokenStream, types: &Types) -> TokenStream {
        match self {
            Self::Default | Self::Arc => quote! {
                &#inner_field
            },
            Self::ThinArc => {
                let str = &types.str;
                quote! {
                    unsafe { #str::from_utf8_unchecked(&#inner_field.slice) }
                }
            }
            Self::SmallVec => {
                let str = &types.str;
                quote! {
                    unsafe { #str::from_utf8_unchecked(&#inner_field) }
                }
            }
        }
    }

    /// Expand the implementation to access the inner type as a `&[u8]`.
    fn expand_as_bytes_impl(&self, inner_field: &TokenStream) -> TokenStream {
        match self {
            Self::Default | Self::Arc => quote! {
                #inner_field.as_bytes()
            },
            Self::ThinArc => quote! {
                &#inner_field.slice
            },
            Self::SmallVec => quote! {
                &#inner_field
            },
        }
    }

    /// Expand the implementation to zeroize the inner type.
    fn expand_zeroize_impl(
        &self,
        inner_field: &TokenStream,
        ruma_common: &RumaCommon,
    ) -> TokenStream {
        match self {
            Self::Default => quote! {
                ::zeroize::Zeroize::zeroize(&mut #inner_field);
            },
            Self::Arc => quote! {
                if let Some(value) = ::std::sync::Arc::get_mut(&mut #inner_field) {
                    ::zeroize::Zeroize::zeroize(value);
                }
            },
            Self::ThinArc => {
                let triomphe = ruma_common.reexported(RumaCommonReexport::Triomphe);
                quote! {
                    #inner_field.with_arc_mut(|this| {
                        if let Some(value) = #triomphe::Arc::get_mut(this) {
                            ::zeroize::Zeroize::zeroize(value.slice_mut());
                        }
                    })
                }
            }
            Self::SmallVec => quote! {
                ::zeroize::Zeroize::zeroize(#inner_field.as_mut_slice());
            },
        }
    }

    /// Expand the implementation to convert the inner type to a `Box<str>`.
    fn expand_into_box_str_impl(
        &self,
        id_var: &TokenStream,
        inner_field: &TokenStream,
    ) -> TokenStream {
        match self {
            Self::Default => quote! { #inner_field },
            Self::Arc | Self::ThinArc => quote! {
                #id_var.as_str().into()
            },
            Self::SmallVec => {
                quote! {
                    unsafe { ::std::str::from_boxed_utf8_unchecked(#inner_field.into_boxed_slice()) }
                }
            }
        }
    }

    /// Expand the implementation to convert the inner type to a `String`.
    fn expand_into_string_impl(
        &self,
        id_var: &TokenStream,
        inner_field: &TokenStream,
        types: &Types,
    ) -> TokenStream {
        match self {
            Self::Default => quote! { #inner_field.into() },
            Self::Arc | Self::ThinArc => quote! {
                #id_var.as_str().into()
            },
            Self::SmallVec => {
                let string = &types.string;
                quote! {
                    unsafe { #string::from_utf8_unchecked(#inner_field.into_vec()) }
                }
            }
        }
    }
}

/// `#[cfg]` attributes for the identifiers internal representations.
struct StorageCfgAttributes {
    /// Attribute for the default internal representation.
    default: syn::Attribute,

    /// Attribute for the `Arc` value.
    arc: syn::Attribute,

    /// Attribute for the `SmallVec` value.
    small_vec: syn::Attribute,

    /// Attribute for the `ThinArc` value.
    thin_arc: syn::Attribute,
}

impl StorageCfgAttributes {
    fn new() -> Self {
        let key = quote! { ruma_identifiers_storage };

        let all_values = StorageCfgValue::ALL.iter().filter_map(StorageCfgValue::as_str);

        let value_to_attribute = |value: StorageCfgValue| {
            let value_str = value.as_str().expect("should not be StorageCfgValue::Default");
            parse_quote! { #[cfg(#key = #value_str)] }
        };

        Self {
            default: parse_quote! { #[cfg(not(any(#( #key = #all_values ),*)))] },
            arc: value_to_attribute(StorageCfgValue::Arc),
            small_vec: value_to_attribute(StorageCfgValue::SmallVec),
            thin_arc: value_to_attribute(StorageCfgValue::ThinArc),
        }
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

    /// `smallvec::SmallVec<[u8; N]`.
    small_vec_bytes: syn::Type,
}

impl Types {
    fn new(ruma_common: &RumaCommon, smallvec_inline_bytes: usize) -> Self {
        let str = parse_quote! { ::std::primitive::str };
        let byte = quote! { ::std::primitive::u8 };
        let cow = parse_quote! { ::std::borrow::Cow };

        let triomphe = ruma_common.reexported(RumaCommonReexport::Triomphe);
        let smallvec = ruma_common.reexported(RumaCommonReexport::Smallvec);

        Self {
            box_str: parse_quote! { ::std::boxed::Box<#str> },
            arc_str: parse_quote! { ::std::sync::Arc<#str> },
            string: parse_quote! { ::std::string::String },
            cow_str: parse_quote! { #cow<'a, #str> },
            bytes: parse_quote! { [#byte] },
            thin_arc_bytes: parse_quote! { #triomphe::ThinArc<(), #byte> },
            small_vec_bytes: parse_quote! { #smallvec::SmallVec<[#byte; #smallvec_inline_bytes]> },
            str,
            cow,
        }
    }
}
