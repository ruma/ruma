//! Implementation of the `IdDst` derive macro.

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_quote;

use crate::util::{RumaCommon, RumaCommonReexport};

mod parse;

/// Generate the `Owned` version of an identifier and various trait implementations.
pub(crate) fn expand_id_dst(input: syn::ItemStruct) -> syn::Result<TokenStream> {
    let id_dst = IdDst::parse(input)?;

    let id = &id_dst.types.id;

    let as_str_and_bytes_impls = id_dst.expand_as_str_and_bytes_impls();
    let to_string_impls = id_dst.expand_to_string_impls(id);
    let unchecked_from_str_impls = id_dst.expand_unchecked_from_str_impls();
    let owned_id_struct = id_dst.expand_owned_id_struct();
    let fallible_from_str_impls = id_dst.expand_fallible_from_str_impls();
    let infallible_from_str_impls = id_dst.expand_infallible_from_str_impls();
    let partial_eq_impls = id_dst.expand_partial_eq_impls();

    Ok(quote! {
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

    /// The name of the owned type.
    owned_ident: syn::Ident,

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

    /// Common types.
    types: Types,

    /// `#[cfg]` attributes for the supported internal representations.
    storage_cfg: StorageCfg,

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
        let id = &self.types.id;

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
        let id = &self.types.id;

        quote! {
            #[automatically_derived]
            impl #impl_generics #id {
                pub(super) const fn from_borrowed_unchecked(s: &#str) -> &Self {
                    unsafe { ::std::mem::transmute(s) }
                }
            }
        }
    }

    /// Generate the `Owned{ident}` type and its implementations.
    fn expand_owned_id_struct(&self) -> TokenStream {
        let ident = &self.ident;
        let owned_ident = &self.owned_ident;
        let generics = &self.generics;
        let impl_generics = &self.impl_generics;

        let str = &self.types.str;
        let box_str = &self.types.box_str;
        let arc_str = &self.types.arc_str;
        let string = &self.types.string;
        let bytes = &self.types.bytes;
        let arcstr = &self.types.arcstr;
        let id = &self.types.id;
        let owned_id = &self.types.owned_id;

        let box_str_cfg = &self.storage_cfg.box_str;
        let arc_str_cfg = &self.storage_cfg.arc_str;
        let arcstr_cfg = &self.storage_cfg.arcstr;

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

        let doc_header = format!("Owned variant of [`{ident}`]");

        let to_string_impls = self.expand_to_string_impls(owned_id);

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
        let from_into_inner_impls =
            [(box_str_cfg, box_str), (arc_str_cfg, arc_str), (arcstr_cfg, arcstr)]
                .into_iter()
                .map(|(cfg, inner)| from_into_inner_cfg_impl(cfg, inner));

        quote! {
            #[doc = #doc_header]
            ///
            /// ## Inner representation
            ///
            /// By default, this type uses a `Box<str>` internally. The inner representation can be selected at
            /// compile time by using one of the following supported values:
            ///
            /// * `Arc` -- Use an `Arc<str>`.
            /// * `ArcStr` -- Use an `ArcStr` from the [`arcstr`](https://crates.io/crates/arcstr) crate.
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
            pub struct #owned_ident #generics {
                #box_str_cfg
                inner: #box_str,
                #arc_str_cfg
                inner: #arc_str,
                #arcstr_cfg
                inner: #arcstr,
                #phantom_decl
            }

            #[automatically_derived]
            impl #impl_generics #owned_id {
                pub(super) fn from_str_unchecked(s: &#str) -> Self {
                    Self {
                        #box_str_cfg
                        inner: s.into(),
                        #arc_str_cfg
                        inner: s.into(),
                        #arcstr_cfg
                        inner: s.into(),
                        #phantom_ctor
                    }
                }

                pub(super) fn from_box_str_unchecked(s: #box_str) -> Self {
                    Self {
                        #box_str_cfg
                        inner: s,
                        #arc_str_cfg
                        inner: s.into(),
                        #arcstr_cfg
                        inner: s.into(),
                        #phantom_ctor
                    }
                }

                pub(super) fn from_string_unchecked(s: #string) -> Self {
                    Self {
                        #box_str_cfg
                        inner: s.into(),
                        #arc_str_cfg
                        inner: s.into(),
                        #arcstr_cfg
                        inner: s.into(),
                        #phantom_ctor
                    }
                }

                /// Access the inner string without going through the borrowed type.
                pub(super) fn as_inner_str(&self) -> &#str {
                    #box_str_cfg
                    { &self.inner }
                    #arc_str_cfg
                    { &self.inner }
                    #arcstr_cfg
                    { &self.inner }
                }

                /// Access the inner bytes without going through the borrowed type.
                pub(super) fn as_inner_bytes(&self) -> &#bytes {
                    #box_str_cfg
                    { self.inner.as_bytes() }
                    #arc_str_cfg
                    { self.inner.as_bytes() }
                    #arcstr_cfg
                    { self.inner.as_bytes() }
                }

                #( #from_into_inner_impls )*
            }

            #[automatically_derived]
            impl #impl_generics ::std::clone::Clone for #owned_id {
                fn clone(&self) -> Self {
                    unsafe { Self::from_inner_unchecked(self.inner.clone()) }
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::cmp::PartialEq for #owned_id {
                fn eq(&self, other: &Self) -> bool {
                    self.as_inner_str() == other.as_inner_str()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::cmp::Eq for #owned_id {}

            #[automatically_derived]
            impl #impl_generics ::std::cmp::PartialOrd for #owned_id {
                fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
                    Some(self.cmp(other))
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::cmp::Ord for #owned_id {
                fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                    self.as_inner_str().cmp(other.as_inner_str())
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::hash::Hash for #owned_id {
                fn hash<H>(&self, state: &mut H)
                where
                    H: ::std::hash::Hasher,
                {
                    self.as_inner_str().hash(state)
                }
            }

            #to_string_impls

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
            impl #impl_generics ::std::convert::AsRef<#str> for #owned_id {
                fn as_ref(&self) -> &#str {
                    self.as_inner_str()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::AsRef<#bytes> for #owned_id {
                fn as_ref(&self) -> &#bytes {
                    self.as_inner_bytes()
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

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#owned_id> for #box_str {
                fn from(id: #owned_id) -> Self {
                    #box_str_cfg
                    { id.inner }
                    #arc_str_cfg
                    { id.as_inner_str().into() }
                    #arcstr_cfg
                    { id.as_inner_str().into() }
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#owned_id> for #string {
                fn from(id: #owned_id) -> Self {
                    #box_str_cfg
                    { id.inner.into() }
                    #arc_str_cfg
                    { id.as_inner_str().into() }
                    #arcstr_cfg
                    { id.as_inner_str().into() }
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
        let id = &self.types.id;
        let owned_id = &self.types.owned_id;

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
        let owned_ident = &self.owned_ident;
        let impl_generics = &self.impl_generics;
        let generic_params = &self.generics.params;

        let str = &self.types.str;
        let cow = &self.types.cow;
        let box_str = &self.types.box_str;
        let string = &self.types.string;
        let cow_str = &self.types.cow_str;
        let id = &self.types.id;
        let owned_id = &self.types.owned_id;

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

    /// Generate `std::fmt::Display`, `std::fmt::Debug` or `serde::Serialize` traits
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
        let id = &self.types.id;
        let owned_id = &self.types.owned_id;

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

    /// `ArcStr`.
    arcstr: syn::Type,

    /// `{id}`, the identifier type with generics, if any.
    id: syn::Type,

    /// `{owned_id}`, the owned identifier type with generics, if any.
    owned_id: syn::Type,
}

impl Types {
    fn new(
        ident: &syn::Ident,
        owned_ident: &syn::Ident,
        type_generics: syn::TypeGenerics<'_>,
        ruma_common: &RumaCommon,
    ) -> Self {
        let arcstr_crate = ruma_common.reexported(RumaCommonReexport::Arcstr);

        let str = parse_quote! { ::std::primitive::str };
        let cow = parse_quote! { ::std::borrow::Cow };

        let id = parse_quote! { #ident #type_generics };

        Self {
            box_str: parse_quote! { ::std::boxed::Box<#str> },
            arc_str: parse_quote! { ::std::sync::Arc<#str> },
            string: parse_quote! { ::std::string::String },
            cow_str: parse_quote! { #cow<'a, #str> },
            bytes: parse_quote! { [::std::primitive::u8] },
            arcstr: parse_quote! { #arcstr_crate::ArcStr },
            str,
            cow,
            id,
            owned_id: parse_quote! { #owned_ident #type_generics },
        }
    }
}

/// `#[cfg]` attributes for the supported internal representations.
struct StorageCfg {
    /// Attribute for the default internal representation, `Box<str>`.
    box_str: syn::Attribute,

    /// Attribute for the `Arc<str>` internal representation.
    arc_str: syn::Attribute,

    /// Attribute for the `ArcStr` internal representation.
    arcstr: syn::Attribute,
}

impl StorageCfg {
    fn new() -> Self {
        let key = quote! { ruma_identifiers_storage };

        let arc_str_value = quote! { "Arc" };
        let arcstr_value = quote! { "ArcStr" };
        let all_values = &[&arc_str_value, &arcstr_value];

        Self {
            box_str: parse_quote! { #[cfg(not(any(#( #key = #all_values ),*)))] },
            arc_str: parse_quote! { #[cfg(#key = #arc_str_value)] },
            arcstr: parse_quote! { #[cfg(#key = #arcstr_value)] },
        }
    }
}
