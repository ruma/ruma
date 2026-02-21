//! Implementation of the `IdDst` derive macro.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse_quote;

use crate::util::{RumaCommon, RumaCommonReexport};

mod parse;

/// Generate the `Owned` version of an identifier and various trait implementations.
pub(crate) fn expand_id_dst(input: syn::ItemStruct) -> syn::Result<TokenStream> {
    let id_dst = IdDst::parse(input)?;

    let ident = &id_dst.ident;
    let id = &id_dst.types.id;
    let box_id = &id_dst.types.box_id;
    let arc_id = &id_dst.types.arc_id;
    let rc_id = &id_dst.types.rc_id;
    let impl_generics = &id_dst.impl_generics;

    let as_str_and_bytes_impls = id_dst.expand_as_str_and_bytes_impls();
    let to_string_impls = id_dst.expand_to_string_impls(id);
    let unchecked_from_str_impls = id_dst.expand_unchecked_from_str_impls();
    let owned_id_struct = id_dst.expand_owned_id_struct();
    let fallible_from_str_impls = id_dst.expand_fallible_from_str_impls();
    let infallible_from_str_impls = id_dst.expand_infallible_from_str_impls();
    let partial_eq_impls = id_dst.expand_partial_eq_impls();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::std::clone::Clone for #box_id {
            fn clone(&self) -> Self {
                (**self).into()
            }
        }

        #[automatically_derived]
        impl #impl_generics ::std::convert::From<&#id> for #box_id {
            fn from(id: &#id) -> Self {
                #ident::from_box_unchecked(id.as_str().into())
            }
        }

        #[automatically_derived]
        impl #impl_generics ::std::convert::From<&#id> for #rc_id {
            fn from(id: &#id) -> Self {
                #ident::from_rc_unchecked(id.as_str().into())
            }
        }

        #[automatically_derived]
        impl #impl_generics ::std::convert::From<&#id> for #arc_id {
            fn from(id: &#id) -> Self {
                #ident::from_arc_unchecked(id.as_str().into())
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
        let boxed = &self.types.boxed;
        let bytes = &self.types.bytes;
        let box_str = &self.types.box_str;
        let string = &self.types.string;
        let id = &self.types.id;
        let box_id = &self.types.box_id;

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
            impl #impl_generics ::std::convert::AsRef<#str> for #box_id {
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
            impl #impl_generics ::std::convert::AsRef<#bytes> for #box_id {
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

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#box_id> for #box_str {
                fn from(id: #box_id) -> Self {
                    unsafe { #boxed::from_raw(#boxed::into_raw(id) as _) }
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#box_id> for #string {
                fn from(id: #box_id) -> Self {
                    <#box_str>::from(id).into()
                }
            }
        }
    }

    /// Generate unchecked private methods to convert a string type to the identifier.
    fn expand_unchecked_from_str_impls(&self) -> TokenStream {
        let impl_generics = &self.impl_generics;

        let str = &self.types.str;
        let boxed = &self.types.boxed;
        let arc = &self.types.arc;
        let rc = &self.types.rc;
        let box_str = &self.types.box_str;
        let arc_str = &self.types.arc_str;
        let rc_str = &self.types.rc_str;
        let id = &self.types.id;

        quote! {
            #[automatically_derived]
            impl #impl_generics #id {
                pub(super) const fn from_borrowed_unchecked(s: &#str) -> &Self {
                    unsafe { ::std::mem::transmute(s) }
                }

                pub(super) fn from_box_unchecked(s: #box_str) -> #boxed<Self> {
                    unsafe { #boxed::from_raw(#boxed::into_raw(s) as _) }
                }

                pub(super) fn from_rc_unchecked(s: #rc_str) -> #rc<Self> {
                    unsafe { #rc::from_raw(#rc::into_raw(s) as _) }
                }

                pub(super) fn from_arc_unchecked(s: #arc_str) -> #arc<Self> {
                    unsafe { #arc::from_raw(#arc::into_raw(s) as _) }
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
        let boxed = &self.types.boxed;
        let arc = &self.types.arc;
        let box_str = &self.types.box_str;
        let string = &self.types.string;
        let bytes = &self.types.bytes;
        let id = &self.types.id;
        let box_id = &self.types.box_id;
        let arc_id = &self.types.arc_id;
        let owned_id = &self.types.owned_id;

        let box_cfg = &self.storage_cfg.boxed;
        let arc_cfg = &self.storage_cfg.arc;

        let doc_header = format!("Owned variant of [`{ident}`]");
        let doc_box_cfg = format!("By default, this type uses a `Box<{ident}>` internally.");
        let doc_arc_cfg = format!("* `Arc` -- Use an `Arc<{ident}>`.");

        let to_string_impls = self.expand_to_string_impls(owned_id);

        quote! {
            #[doc = #doc_header]
            ///
            /// ## Inner representation
            ///
            #[doc = #doc_box_cfg]
            /// The inner representation can be selected at compile time by using one of the following supported values:
            ///
            #[doc = #doc_arc_cfg]
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
                #box_cfg
                inner: #box_id,
                #arc_cfg
                inner: #arc_id,
            }

            #[automatically_derived]
            impl #impl_generics #owned_id {
                pub(super) fn from_str_unchecked(s: &#str) -> Self {
                    #ident::from_borrowed_unchecked(s).to_owned()
                }

                pub(super) fn from_box_str_unchecked(s: #box_str) -> Self {
                    Self {
                        #box_cfg
                        inner: #ident::from_box_unchecked(s),
                        #arc_cfg
                        inner: #ident::from_arc_unchecked(s.into()),
                    }
                }

                pub(super) fn from_string_unchecked(s: #string) -> Self {
                    Self {
                        #box_cfg
                        inner: #ident::from_box_unchecked(s.into()),
                        #arc_cfg
                        inner: #ident::from_arc_unchecked(s.into()),
                    }
                }

                /// Consumes this ID and returns a raw pointer to its inner data.
                ///
                /// The pointer must later be passed to [`Self::from_raw`] to avoid a memory leak.
                pub(super) fn into_raw(self) -> *const #id {
                    #box_cfg
                    { #boxed::into_raw(self.inner).cast_const() }
                    #arc_cfg
                    { #arc::into_raw(self.inner) }
                }

                /// Reconstruct this ID from a raw pointer created by [`Self::into_raw`].
                ///
                /// # Safety
                ///
                /// `ptr` must have been returned by an `into_raw` method generated by `IdDst`
                /// for a compatible ID type and it must not have been passed to `from_raw` before.
                pub(super) unsafe fn from_raw(ptr: *const #id) -> Self {
                    Self {
                        #box_cfg
                        inner: unsafe { #boxed::from_raw(ptr.cast_mut()) },
                        #arc_cfg
                        inner: unsafe { #arc::from_raw(ptr) },
                    }
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::clone::Clone for #owned_id {
                fn clone(&self) -> Self {
                    Self { inner: self.inner.clone() }
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::cmp::PartialEq for #owned_id {
                fn eq(&self, other: &Self) -> bool {
                    self.as_str() == other.as_str()
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
                    self.as_str().cmp(other.as_str())
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::hash::Hash for #owned_id {
                fn hash<H>(&self, state: &mut H)
                where
                    H: ::std::hash::Hasher,
                {
                    self.as_str().hash(state)
                }
            }

            #to_string_impls

            #[automatically_derived]
            impl #impl_generics ::std::ops::Deref for #owned_id {
                type Target = #id;

                fn deref(&self) -> &Self::Target {
                    &self.inner
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
                    &*self.inner
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::AsRef<#str> for #owned_id {
                fn as_ref(&self) -> &#str {
                    self.inner.as_str()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::AsRef<#bytes> for #owned_id {
                fn as_ref(&self) -> &#bytes {
                    self.inner.as_bytes()
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::borrow::ToOwned for #id {
                type Owned = #owned_id;

                fn to_owned(&self) -> Self::Owned {
                    #owned_ident {
                        #box_cfg
                        inner: #ident::from_box_unchecked(self.as_str().into()),
                        #arc_cfg
                        inner: #ident::from_arc_unchecked(self.as_str().into()),
                    }
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<&#id> for #owned_id {
                fn from(id: &#id) -> Self {
                    Self { inner: id.into() }
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#box_id> for #owned_id {
                fn from(b: #box_id) -> Self {
                    Self {
                        #box_cfg
                        inner: b,
                        #arc_cfg
                        inner: b.into(),
                    }
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#arc_id> for #owned_id {
                fn from(a: #arc_id) -> Self {
                    Self {
                        #box_cfg
                        inner: a.as_ref().into(),
                        #arc_cfg
                        inner: a,
                    }
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#owned_id> for #box_id {
                fn from(a: #owned_id) -> Self {
                    #box_cfg
                    { a.inner }
                    #arc_cfg
                    { a.inner.as_ref().into() }
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#owned_id> for #arc_id {
                fn from(a: #owned_id) -> Self {
                    #box_cfg
                    { a.inner.into() }
                    #arc_cfg
                    { a.inner }
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#owned_id> for #string {
                fn from(id: #owned_id) -> Self {
                    #box_cfg
                    { id.inner.into() }
                    #arc_cfg
                    { id.inner.as_ref().into() }
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
        let parse_box_doc_header = format!("Try parsing a `&str` into a `Box<{ident}>`.");
        let parse_rc_docs = format!("Try parsing a `&str` into an `Rc<{ident}>`.");
        let parse_arc_docs = format!("Try parsing a `&str` into an `Arc<{ident}>`.");

        let str = &self.types.str;
        let boxed = &self.types.boxed;
        let arc = &self.types.arc;
        let rc = &self.types.rc;
        let cow = &self.types.cow;
        let box_str = &self.types.box_str;
        let arc_str = &self.types.arc_str;
        let rc_str = &self.types.rc_str;
        let string = &self.types.string;
        let cow_str = &self.types.cow_str;
        let ref_str: syn::Type = parse_quote!(&#str);
        let id = &self.types.id;
        let box_id = &self.types.box_id;
        let owned_id = &self.types.owned_id;

        // Generate `FromStr` and `TryFrom<&str>` implementations for the given type, using the
        // given `parse_fn` from the identifier type.
        let expand_from_str_impls = |ty: &syn::Type, parse_fn: &syn::Ident| -> TokenStream {
            quote! {
                #[automatically_derived]
                impl #impl_generics ::std::str::FromStr for #ty {
                    type Err = #ruma_common::IdParseError;

                    fn from_str(s: #ref_str) -> ::std::result::Result<Self, Self::Err> {
                        #ident::#parse_fn(s)
                    }
                }

                #[automatically_derived]
                impl #impl_generics ::std::convert::TryFrom<&#str> for #ty {
                    type Error = #ruma_common::IdParseError;

                    fn try_from(s: &#str) -> ::std::result::Result<Self, Self::Error> {
                        #ident::#parse_fn(s)
                    }
                }
            }
        };

        let box_id_from_str_impls =
            expand_from_str_impls(box_id, &syn::Ident::new("parse_box", Span::call_site()));
        let owned_id_from_str_impls =
            expand_from_str_impls(owned_id, &syn::Ident::new("parse", Span::call_site()));

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

                #[doc = #parse_box_doc_header]
                ///
                /// The same can also be done using `FromStr`, `TryFrom` or `TryInto`.
                /// This function is simply more constrained and thus useful in generic contexts.
                pub fn parse_box(
                    s: impl ::std::convert::AsRef<#str> + ::std::convert::Into<#box_str>,
                ) -> ::std::result::Result<#boxed<Self>, #ruma_common::IdParseError> {
                    #validate(s.as_ref())?;
                    ::std::result::Result::Ok(#ident::from_box_unchecked(s.into()))
                }

                #[doc = #parse_rc_docs]
                pub fn parse_rc(
                    s: impl ::std::convert::AsRef<#str> + ::std::convert::Into<#rc_str>,
                ) -> ::std::result::Result<#rc<Self>, #ruma_common::IdParseError> {
                    #validate(s.as_ref())?;
                    ::std::result::Result::Ok(#ident::from_rc_unchecked(s.into()))
                }

                #[doc = #parse_arc_docs]
                pub fn parse_arc(
                    s: impl ::std::convert::AsRef<#str> + ::std::convert::Into<#arc_str>,
                ) -> ::std::result::Result<#arc<Self>, #ruma_common::IdParseError> {
                    #validate(s.as_ref())?;
                    ::std::result::Result::Ok(#ident::from_arc_unchecked(s.into()))
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

            #box_id_from_str_impls

            #[automatically_derived]
            impl #impl_generics ::std::convert::TryFrom<#box_str> for #box_id {
                type Error = #ruma_common::IdParseError;

                fn try_from(s: #box_str) -> ::std::result::Result<Self, Self::Error> {
                    #ident::parse_box(s)
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::TryFrom<#string> for #box_id {
                type Error = #ruma_common::IdParseError;

                fn try_from(s: #string) -> ::std::result::Result<Self, Self::Error> {
                    #ident::parse_box(s)
                }
            }

            #[automatically_derived]
            impl<'a, #generic_params> ::std::convert::TryFrom<#cow_str> for #box_id {
                type Error = #ruma_common::IdParseError;

                fn try_from(s: #cow_str) -> ::std::result::Result<Self, Self::Error> {
                    match s {
                        #cow::Borrowed(s) => s.try_into(),
                        #cow::Owned(s) => s.try_into(),
                    }
                }
            }

            #[automatically_derived]
            impl<'de, #generic_params> #serde::Deserialize<'de> for #box_id {
                fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
                where
                    D: #serde::Deserializer<'de>,
                {
                    use #serde::de::Error;

                    <#box_str>::deserialize(deserializer)?.try_into().map_err(D::Error::custom)
                }
            }

            #owned_id_from_str_impls

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
        let box_id = &self.types.box_id;
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
            impl #impl_generics ::std::convert::From<&#str> for #box_id {
                fn from(s: &#str) -> Self {
                    #ident::from_box_unchecked(s.into())
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#box_str> for #box_id {
                fn from(s: #box_str) -> Self {
                    #ident::from_box_unchecked(s)
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#string> for #box_id {
                fn from(s: #string) -> Self {
                    #ident::from_box_unchecked(s.into())
                }
            }

            #[automatically_derived]
            impl<'a, #generic_params> ::std::convert::From<#cow_str> for #box_id {
                fn from(s: #cow_str) -> Self {
                    match s {
                        #cow::Borrowed(s) => s.into(),
                        #cow::Owned(s) => s.into(),
                    }
                }
            }

            #[automatically_derived]
            impl<'de, #generic_params> #serde::Deserialize<'de> for #box_id {
                fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
                where
                    D: #serde::Deserializer<'de>,
                {
                    <#box_str>::deserialize(deserializer).map(::std::convert::Into::into)
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

        quote! {
            #[automatically_derived]
            impl #impl_generics ::std::fmt::Display for #ty {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    self.as_str().fmt(f)
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::fmt::Debug for #ty {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    self.as_str().fmt(f)
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
        let generics_params = &self.generics.params;
        let impl_generics = &self.impl_generics;

        let str = &self.types.str;
        let string = &self.types.string;
        let cow_str = &self.types.cow_str;
        let id = &self.types.id;
        let box_id = &self.types.box_id;
        let arc_id = &self.types.arc_id;
        let owned_id = &self.types.owned_id;

        let ref_id: syn::Type = parse_quote! { &#id };
        let ref_str: syn::Type = parse_quote! { &#str };
        let cow_generics = quote! { <'a, #generics_params> };

        let self_ident = syn::Ident::new("self", Span::call_site());
        let other_ident = syn::Ident::new("other", Span::call_site());

        // Get the string representation of the type.
        let as_str_impl = |ty: &syn::Type, ident: &syn::Ident| {
            if *ty == *str || *ty == ref_str || *ty == *cow_str {
                quote! { ::std::convert::AsRef::<#str>::as_ref(#ident) }
            } else {
                quote! { #ident.as_str() }
            }
        };

        // Implement `PartialEq` with the given lhs and rhs types.
        let expand_partial_eq = |lhs: &syn::Type, rhs: &syn::Type| {
            let self_as_str = as_str_impl(lhs, &self_ident);
            let other_as_str = as_str_impl(rhs, &other_ident);

            let impl_generics =
                if *lhs == *cow_str || *rhs == *cow_str { &cow_generics } else { impl_generics };

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
            expand_partial_eq_impls_for_type(id, &[str, &ref_str, string]),
            expand_partial_eq_impls_for_type(box_id, &[str, &ref_str, string, id, &ref_id]),
            expand_partial_eq_impls_for_type(
                owned_id,
                &[str, &ref_str, string, id, &ref_id, box_id, arc_id],
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

    /// `Box`.
    boxed: syn::Type,

    /// `Arc`.
    arc: syn::Type,

    /// `Rc`.
    rc: syn::Type,

    /// `Cow`.
    cow: syn::Type,

    /// `Box<str>`.
    box_str: syn::Type,

    /// `Arc<str>`.
    arc_str: syn::Type,

    /// `Rc<str>`.
    rc_str: syn::Type,

    /// `String`.
    string: syn::Type,

    /// `Cow<'a, str>`.
    cow_str: syn::Type,

    /// `[u8]`.
    bytes: syn::Type,

    /// `{id}`, the identifier type with generics, if any.
    id: syn::Type,

    /// `Box<{id}>`.
    box_id: syn::Type,

    /// `Arc<{id}>`.
    arc_id: syn::Type,

    /// `Rc<{id}>`.
    rc_id: syn::Type,

    /// `{owned_id}`, the owned identifier type with generics, if any.
    owned_id: syn::Type,
}

impl Types {
    fn new(
        ident: &syn::Ident,
        owned_ident: &syn::Ident,
        type_generics: syn::TypeGenerics<'_>,
    ) -> Self {
        let str = parse_quote! { ::std::primitive::str };
        let boxed = parse_quote! { ::std::boxed::Box };
        let arc = parse_quote! { ::std::sync::Arc };
        let rc = parse_quote! { ::std::rc::Rc };
        let cow = parse_quote! { ::std::borrow::Cow };

        let id = parse_quote! { #ident #type_generics };

        Self {
            box_str: parse_quote! { #boxed<#str> },
            arc_str: parse_quote! { #arc<#str> },
            rc_str: parse_quote! { #rc<#str> },
            string: parse_quote! { ::std::string::String },
            cow_str: parse_quote! { #cow<'a, #str> },
            bytes: parse_quote! { [::std::primitive::u8] },
            str,
            box_id: parse_quote! { #boxed<#id> },
            arc_id: parse_quote! { #arc<#id> },
            rc_id: parse_quote! { #rc<#id> },
            id,
            owned_id: parse_quote! { #owned_ident #type_generics },
            boxed,
            arc,
            rc,
            cow,
        }
    }
}

/// `#[cfg]` attributes for the supported internal representations.
struct StorageCfg {
    /// Attribute for the default internal representation, `Box<{id}}>`.
    boxed: syn::Attribute,

    /// Attribute for the `Arc<{id}>` internal representation.
    arc: syn::Attribute,
}

impl StorageCfg {
    fn new() -> Self {
        let key = quote! { ruma_identifiers_storage };

        Self {
            boxed: parse_quote! { #[cfg(not(#key = "Arc"))] },
            arc: parse_quote! { #[cfg(#key = "Arc")] },
        }
    }
}
