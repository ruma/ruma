//! Methods and types for generating identifiers.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Fields, ImplGenerics, Index, ItemStruct, LitStr, Path, Token,
};

pub struct IdentifierInput {
    pub dollar_crate: Path,
    pub id: LitStr,
}

impl Parse for IdentifierInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let dollar_crate = input.parse()?;
        let _: Token![,] = input.parse()?;
        let id = input.parse()?;

        Ok(Self { dollar_crate, id })
    }
}

pub fn expand_id_zst(input: ItemStruct) -> syn::Result<TokenStream> {
    let id = &input.ident;
    let owned = format_ident!("Owned{id}");

    let owned_decl = expand_owned_id(&input);

    let meta = input.attrs.iter().filter(|attr| attr.path().is_ident("ruma_id")).try_fold(
        IdZstMeta::default(),
        |meta, attr| {
            let list: Punctuated<IdZstMeta, Token![,]> =
                attr.parse_args_with(Punctuated::parse_terminated)?;

            list.into_iter().try_fold(meta, IdZstMeta::merge)
        },
    )?;

    let extra_impls = if let Some(validate) = meta.validate {
        expand_checked_impls(&input, validate)
    } else {
        assert!(
            input.generics.params.is_empty(),
            "generic unchecked IDs are not currently supported"
        );
        expand_unchecked_impls(&input)
    };

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    // So we don't have to insert #where_clause everywhere when it is always None in practice
    assert_eq!(where_clause, None, "where clauses on identifier types are not currently supported");

    let as_str_docs = format!("Creates a string slice from this `{id}`.");
    let as_bytes_docs = format!("Creates a byte slice from this `{id}`.");

    let as_str_impl = match &input.fields {
        Fields::Named(_) | Fields::Unit => {
            syn::Error::new(Span::call_site(), "Only tuple structs are supported currently.")
                .into_compile_error()
        }
        Fields::Unnamed(u) => {
            let last_idx = Index::from(u.unnamed.len() - 1);
            quote! { &self.#last_idx }
        }
    };

    let id_ty = quote! { #id #ty_generics };
    let owned_ty = quote! { #owned #ty_generics };

    let as_str_impls = expand_as_str_impls(id_ty.clone(), &impl_generics);
    // FIXME: Remove?
    let box_partial_eq_string = expand_partial_eq_string(quote! { Box<#id_ty> }, &impl_generics);

    Ok(quote! {
        #owned_decl

        #[automatically_derived]
        impl #impl_generics #id_ty {
            pub(super) const fn from_borrowed(s: &str) -> &Self {
                unsafe { std::mem::transmute(s) }
            }

            pub(super) fn from_box(s: Box<str>) -> Box<Self> {
                unsafe { Box::from_raw(Box::into_raw(s) as _) }
            }

            pub(super) fn from_rc(s: std::rc::Rc<str>) -> std::rc::Rc<Self> {
                unsafe { std::rc::Rc::from_raw(std::rc::Rc::into_raw(s) as _) }
            }

            pub(super) fn from_arc(s: std::sync::Arc<str>) -> std::sync::Arc<Self> {
                unsafe { std::sync::Arc::from_raw(std::sync::Arc::into_raw(s) as _) }
            }

            pub(super) fn into_owned(self: Box<Self>) -> Box<str> {
                unsafe { Box::from_raw(Box::into_raw(self) as _) }
            }

            #[doc = #as_str_docs]
            #[inline]
            pub fn as_str(&self) -> &str {
                #as_str_impl
            }

            #[doc = #as_bytes_docs]
            #[inline]
            pub fn as_bytes(&self) -> &[u8] {
                self.as_str().as_bytes()
            }
        }

        #[automatically_derived]
        impl #impl_generics Clone for Box<#id_ty> {
            fn clone(&self) -> Self {
                (**self).into()
            }
        }

        #[automatically_derived]
        impl #impl_generics ToOwned for #id_ty {
            type Owned = #owned_ty;

            fn to_owned(&self) -> Self::Owned {
                #owned::from_ref(self)
            }
        }

        #[automatically_derived]
        impl #impl_generics AsRef<#id_ty> for #id_ty {
            fn as_ref(&self) -> &#id_ty {
                self
            }
        }

        #[automatically_derived]
        impl #impl_generics AsRef<str> for #id_ty {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        #[automatically_derived]
        impl #impl_generics AsRef<str> for Box<#id_ty> {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        #[automatically_derived]
        impl #impl_generics AsRef<[u8]> for #id_ty {
            fn as_ref(&self) -> &[u8] {
                self.as_bytes()
            }
        }

        #[automatically_derived]
        impl #impl_generics AsRef<[u8]> for Box<#id_ty> {
            fn as_ref(&self) -> &[u8] {
                self.as_bytes()
            }
        }

        #[automatically_derived]
        impl #impl_generics From<&#id_ty> for String {
            fn from(id: &#id_ty) -> Self {
                id.as_str().to_owned()
            }
        }

        #[automatically_derived]
        impl #impl_generics From<Box<#id_ty>> for String {
            fn from(id: Box<#id_ty>) -> Self {
                id.into_owned().into()
            }
        }

        #[automatically_derived]
        impl #impl_generics From<&#id_ty> for Box<#id_ty> {
            fn from(id: &#id_ty) -> Self {
                <#id_ty>::from_box(id.as_str().into())
            }
        }

        #[automatically_derived]
        impl #impl_generics From<&#id_ty> for std::rc::Rc<#id_ty> {
            fn from(s: &#id_ty) -> std::rc::Rc<#id_ty> {
                let rc = std::rc::Rc::<str>::from(s.as_str());
                <#id_ty>::from_rc(rc)
            }
        }

        #[automatically_derived]
        impl #impl_generics From<&#id_ty> for std::sync::Arc<#id_ty> {
            fn from(s: &#id_ty) -> std::sync::Arc<#id_ty> {
                let arc = std::sync::Arc::<str>::from(s.as_str());
                <#id_ty>::from_arc(arc)
            }
        }

        #[automatically_derived]
        impl #impl_generics PartialEq<#id_ty> for Box<#id_ty> {
            fn eq(&self, other: &#id_ty) -> bool {
                self.as_str() == other.as_str()
            }
        }

        #[automatically_derived]
        impl #impl_generics PartialEq<&'_ #id_ty> for Box<#id_ty> {
            fn eq(&self, other: &&#id_ty) -> bool {
                self.as_str() == other.as_str()
            }
        }

        #[automatically_derived]
        impl #impl_generics PartialEq<Box<#id_ty>> for #id_ty {
            fn eq(&self, other: &Box<#id_ty>) -> bool {
                self.as_str() == other.as_str()
            }
        }

        #[automatically_derived]
        impl #impl_generics PartialEq<Box<#id_ty>> for &'_ #id_ty {
            fn eq(&self, other: &Box<#id_ty>) -> bool {
                self.as_str() == other.as_str()
            }
        }

        #as_str_impls
        #box_partial_eq_string
        #extra_impls
    })
}

fn expand_owned_id(input: &ItemStruct) -> TokenStream {
    let id = &input.ident;
    let owned = format_ident!("Owned{id}");

    let doc_header = format!("Owned variant of {id}");
    let (impl_generics, ty_generics, _where_clause) = input.generics.split_for_impl();

    let id_ty = quote! { #id #ty_generics };
    let owned_ty = quote! { #owned #ty_generics };

    let as_str_impls = expand_as_str_impls(owned_ty.clone(), &impl_generics);

    quote! {
        #[doc = #doc_header]
        ///
        /// The wrapper type for this type is variable, by default it'll use [`Box`],
        /// but you can change that by setting "`--cfg=ruma_identifiers_storage=...`" using
        /// `RUSTFLAGS` or `.cargo/config.toml` (under `[build]` -> `rustflags = ["..."]`)
        /// to the following;
        /// - `ruma_identifiers_storage="Arc"` to use [`Arc`](std::sync::Arc) as a wrapper type.
        pub struct #owned #impl_generics {
            #[cfg(not(any(ruma_identifiers_storage = "Arc")))]
            inner: Box<#id_ty>,
            #[cfg(ruma_identifiers_storage = "Arc")]
            inner: std::sync::Arc<#id_ty>,
        }

        #[automatically_derived]
        impl #impl_generics #owned_ty {
            fn from_ref(v: &#id_ty) -> Self {
                Self {
                    #[cfg(not(any(ruma_identifiers_storage = "Arc")))]
                    inner: #id::from_box(v.as_str().into()),
                    #[cfg(ruma_identifiers_storage = "Arc")]
                    inner: #id::from_arc(v.as_str().into()),
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics AsRef<#id_ty> for #owned_ty {
            fn as_ref(&self) -> &#id_ty {
                &*self.inner
            }
        }

        #[automatically_derived]
        impl #impl_generics AsRef<str> for #owned_ty {
            fn as_ref(&self) -> &str {
                self.inner.as_str()
            }
        }

        #[automatically_derived]
        impl #impl_generics AsRef<[u8]> for #owned_ty {
            fn as_ref(&self) -> &[u8] {
                self.inner.as_bytes()
            }
        }

        #[automatically_derived]
        impl #impl_generics From<#owned_ty> for String {
            fn from(id: #owned_ty) -> String {
                #[cfg(not(any(ruma_identifiers_storage = "Arc")))]
                { id.inner.into() }
                #[cfg(ruma_identifiers_storage = "Arc")]
                { id.inner.as_ref().into() }
            }
        }

        #[automatically_derived]
        impl #impl_generics std::clone::Clone for #owned_ty {
            fn clone(&self) -> Self {
                (&*self.inner).into()
            }
        }

        #[automatically_derived]
        impl #impl_generics std::ops::Deref for #owned_ty {
            type Target = #id_ty;

            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }

        #[automatically_derived]
        impl #impl_generics std::borrow::Borrow<#id_ty> for #owned_ty {
            fn borrow(&self) -> &#id_ty {
                self.as_ref()
            }
        }

        #[automatically_derived]
        impl #impl_generics From<&'_ #id_ty> for #owned_ty {
            fn from(id: &#id_ty) -> #owned_ty {
                #owned { inner: id.into() }
            }
        }

        #[automatically_derived]
        impl #impl_generics From<Box<#id_ty>> for #owned_ty {
            fn from(b: Box<#id_ty>) -> #owned_ty {
                Self { inner: b.into() }
            }
        }

        #[automatically_derived]
        impl #impl_generics From<std::sync::Arc<#id_ty>> for #owned_ty {
            fn from(a: std::sync::Arc<#id_ty>) -> #owned_ty {
                Self {
                    #[cfg(not(any(ruma_identifiers_storage = "Arc")))]
                    inner: a.as_ref().into(),
                    #[cfg(ruma_identifiers_storage = "Arc")]
                    inner: a,
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics From<#owned_ty> for Box<#id_ty> {
            fn from(a: #owned_ty) -> Box<#id_ty> {
                #[cfg(not(any(ruma_identifiers_storage = "Arc")))]
                { a.inner }
                #[cfg(ruma_identifiers_storage = "Arc")]
                { a.inner.as_ref().into() }
            }
        }

        #[automatically_derived]
        impl #impl_generics From<#owned_ty> for std::sync::Arc<#id_ty> {
            fn from(a: #owned_ty) -> std::sync::Arc<#id_ty> {
                #[cfg(not(any(ruma_identifiers_storage = "Arc")))]
                { a.inner.into() }
                #[cfg(ruma_identifiers_storage = "Arc")]
                { a.inner }
            }
        }

        #[automatically_derived]
        impl #impl_generics std::cmp::PartialEq for #owned_ty {
            fn eq(&self, other: &Self) -> bool {
                self.as_str() == other.as_str()
            }
        }

        #[automatically_derived]
        impl #impl_generics std::cmp::Eq for #owned_ty {}

        #[automatically_derived]
        impl #impl_generics std::cmp::PartialOrd for #owned_ty {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        #[automatically_derived]
        impl #impl_generics std::cmp::Ord for #owned_ty {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.as_str().cmp(other.as_str())
            }
        }

        #[automatically_derived]
        impl #impl_generics std::hash::Hash for #owned_ty {
            fn hash<H>(&self, state: &mut H)
            where
                H: std::hash::Hasher,
            {
                self.as_str().hash(state)
            }
        }

        #as_str_impls

        #[automatically_derived]
        impl #impl_generics PartialEq<#id_ty> for #owned_ty {
            fn eq(&self, other: &#id_ty) -> bool {
                AsRef::<#id_ty>::as_ref(self) == other
            }
        }

        #[automatically_derived]
        impl #impl_generics PartialEq<#owned_ty> for #id_ty {
            fn eq(&self, other: &#owned_ty) -> bool {
                self == AsRef::<#id_ty>::as_ref(other)
            }
        }

        #[automatically_derived]
        impl #impl_generics PartialEq<&#id_ty> for #owned_ty {
            fn eq(&self, other: &&#id_ty) -> bool {
                AsRef::<#id_ty>::as_ref(self) == *other
            }
        }

        #[automatically_derived]
        impl #impl_generics PartialEq<#owned_ty> for &#id_ty {
            fn eq(&self, other: &#owned_ty) -> bool {
                *self == AsRef::<#id_ty>::as_ref(other)
            }
        }

        #[automatically_derived]
        impl #impl_generics PartialEq<Box<#id_ty>> for #owned_ty {
            fn eq(&self, other: &Box<#id_ty>) -> bool {
                AsRef::<#id_ty>::as_ref(self) == AsRef::<#id_ty>::as_ref(other)
            }
        }

        #[automatically_derived]
        impl #impl_generics PartialEq<#owned_ty> for Box<#id_ty> {
            fn eq(&self, other: &#owned_ty) -> bool {
                AsRef::<#id_ty>::as_ref(self) == AsRef::<#id_ty>::as_ref(other)
            }
        }

        #[automatically_derived]
        impl #impl_generics PartialEq<std::sync::Arc<#id_ty>> for #owned_ty {
            fn eq(&self, other: &std::sync::Arc<#id_ty>) -> bool {
                AsRef::<#id_ty>::as_ref(self) == AsRef::<#id_ty>::as_ref(other)
            }
        }

        #[automatically_derived]
        impl #impl_generics PartialEq<#owned_ty> for std::sync::Arc<#id_ty> {
            fn eq(&self, other: &#owned_ty) -> bool {
                AsRef::<#id_ty>::as_ref(self) == AsRef::<#id_ty>::as_ref(other)
            }
        }
    }
}

fn expand_checked_impls(input: &ItemStruct, validate: Path) -> TokenStream {
    let id = &input.ident;
    let owned = format_ident!("Owned{id}");

    let (impl_generics, ty_generics, _where_clause) = input.generics.split_for_impl();
    let generic_params = &input.generics.params;

    let parse_doc_header = format!("Try parsing a `&str` into an `Owned{id}`.");
    let parse_box_doc_header = format!("Try parsing a `&str` into a `Box<{id}>`.");
    let parse_rc_docs = format!("Try parsing a `&str` into an `Rc<{id}>`.");
    let parse_arc_docs = format!("Try parsing a `&str` into an `Arc<{id}>`.");

    let id_ty = quote! { #id #ty_generics };
    let owned_ty = quote! { #owned #ty_generics };

    quote! {
        #[automatically_derived]
        impl #impl_generics #id_ty {
            #[doc = #parse_doc_header]
            ///
            /// The same can also be done using `FromStr`, `TryFrom` or `TryInto`.
            /// This function is simply more constrained and thus useful in generic contexts.
            pub fn parse(
                s: impl AsRef<str>,
            ) -> Result<#owned_ty, crate::IdParseError> {
                let s = s.as_ref();
                #validate(s)?;
                Ok(#id::from_borrowed(s).to_owned())
            }

            #[doc = #parse_box_doc_header]
            ///
            /// The same can also be done using `FromStr`, `TryFrom` or `TryInto`.
            /// This function is simply more constrained and thus useful in generic contexts.
            pub fn parse_box(
                s: impl AsRef<str> + Into<Box<str>>,
            ) -> Result<Box<Self>, crate::IdParseError> {
                #validate(s.as_ref())?;
                Ok(#id::from_box(s.into()))
            }

            #[doc = #parse_rc_docs]
            pub fn parse_rc(
                s: impl AsRef<str> + Into<std::rc::Rc<str>>,
            ) -> Result<std::rc::Rc<Self>, crate::IdParseError> {
                #validate(s.as_ref())?;
                Ok(#id::from_rc(s.into()))
            }

            #[doc = #parse_arc_docs]
            pub fn parse_arc(
                s: impl AsRef<str> + Into<std::sync::Arc<str>>,
            ) -> Result<std::sync::Arc<Self>, crate::IdParseError> {
                #validate(s.as_ref())?;
                Ok(#id::from_arc(s.into()))
            }
        }

        #[automatically_derived]
        impl<'de, #generic_params> serde::Deserialize<'de> for Box<#id_ty> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                use serde::de::Error;

                let s = String::deserialize(deserializer)?;

                match #id::parse_box(s) {
                    Ok(o) => Ok(o),
                    Err(e) => Err(D::Error::custom(e)),
                }
            }
        }

        #[automatically_derived]
        impl<'de, #generic_params> serde::Deserialize<'de> for #owned_ty {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                use serde::de::Error;

                let s = String::deserialize(deserializer)?;

                match #id::parse(s) {
                    Ok(o) => Ok(o),
                    Err(e) => Err(D::Error::custom(e)),
                }
            }
        }

        #[automatically_derived]
        impl<'a, #generic_params> std::convert::TryFrom<&'a str> for &'a #id_ty {
            type Error = crate::IdParseError;

            fn try_from(s: &'a str) -> Result<Self, Self::Error> {
                #validate(s)?;
                Ok(<#id_ty>::from_borrowed(s))
            }
        }

        #[automatically_derived]
        impl #impl_generics std::str::FromStr for Box<#id_ty> {
            type Err = crate::IdParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                <#id_ty>::parse_box(s)
            }
        }

        #[automatically_derived]
        impl #impl_generics std::convert::TryFrom<&str> for Box<#id_ty> {
            type Error = crate::IdParseError;

            fn try_from(s: &str) -> Result<Self, Self::Error> {
                <#id_ty>::parse_box(s)
            }
        }

        #[automatically_derived]
        impl #impl_generics std::convert::TryFrom<String> for Box<#id_ty> {
            type Error = crate::IdParseError;

            fn try_from(s: String) -> Result<Self, Self::Error> {
                <#id_ty>::parse_box(s)
            }
        }

        #[automatically_derived]
        impl #impl_generics std::str::FromStr for #owned_ty {
            type Err = crate::IdParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                <#id_ty>::parse(s)
            }
        }

        #[automatically_derived]
        impl #impl_generics std::convert::TryFrom<&str> for #owned_ty {
            type Error = crate::IdParseError;

            fn try_from(s: &str) -> Result<Self, Self::Error> {
                <#id_ty>::parse(s)
            }
        }

        #[automatically_derived]
        impl #impl_generics std::convert::TryFrom<String> for #owned_ty {
            type Error = crate::IdParseError;

            fn try_from(s: String) -> Result<Self, Self::Error> {
                <#id_ty>::parse(s)
            }
        }
    }
}

fn expand_unchecked_impls(input: &ItemStruct) -> TokenStream {
    let id = &input.ident;
    let owned = format_ident!("Owned{id}");

    quote! {
        #[automatically_derived]
        impl<'a> From<&'a str> for &'a #id {
            fn from(s: &'a str) -> Self {
                #id::from_borrowed(s)
            }
        }

        #[automatically_derived]
        impl From<&str> for #owned {
            fn from(s: &str) -> Self {
                <&#id>::from(s).into()
            }
        }

        #[automatically_derived]
        impl From<Box<str>> for #owned {
            fn from(s: Box<str>) -> Self {
                <&#id>::from(&*s).into()
            }
        }

        #[automatically_derived]
        impl From<String> for #owned {
            fn from(s: String) -> Self {
                <&#id>::from(s.as_str()).into()
            }
        }

        #[automatically_derived]
        impl From<&str> for Box<#id> {
            fn from(s: &str) -> Self {
                #id::from_box(s.into())
            }
        }

        #[automatically_derived]
        impl From<Box<str>> for Box<#id> {
            fn from(s: Box<str>) -> Self {
                #id::from_box(s)
            }
        }

        #[automatically_derived]
        impl From<String> for Box<#id> {
            fn from(s: String) -> Self {
                #id::from_box(s.into())
            }
        }

        #[automatically_derived]
        impl From<Box<#id>> for Box<str> {
            fn from(id: Box<#id>) -> Self {
                id.into_owned()
            }
        }

        #[automatically_derived]
        impl<'de> serde::Deserialize<'de> for Box<#id> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Box::<str>::deserialize(deserializer).map(#id::from_box)
            }
        }

        #[automatically_derived]
        impl<'de> serde::Deserialize<'de> for #owned {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                // FIXME: Deserialize inner, convert that
                Box::<str>::deserialize(deserializer).map(#id::from_box).map(Into::into)
            }
        }
    }
}

fn expand_as_str_impls(ty: TokenStream, impl_generics: &ImplGenerics<'_>) -> TokenStream {
    let partial_eq_string = expand_partial_eq_string(ty.clone(), impl_generics);

    quote! {
        #[automatically_derived]
        impl #impl_generics std::fmt::Display for #ty {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }

        #[automatically_derived]
        impl #impl_generics std::fmt::Debug for #ty {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                <str as std::fmt::Debug>::fmt(self.as_str(), f)
            }
        }

        #[automatically_derived]
        impl #impl_generics serde::Serialize for #ty {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        #partial_eq_string
    }
}

fn expand_partial_eq_string(ty: TokenStream, impl_generics: &ImplGenerics<'_>) -> TokenStream {
    IntoIterator::into_iter([
        (ty.clone(), quote! { str }),
        (ty.clone(), quote! { &str }),
        (ty.clone(), quote! { String }),
        (quote! { str }, ty.clone()),
        (quote! { &str }, ty.clone()),
        (quote! { String }, ty),
    ])
    .map(|(lhs, rhs)| {
        quote! {
            #[automatically_derived]
            impl #impl_generics PartialEq<#rhs> for #lhs {
                fn eq(&self, other: &#rhs) -> bool {
                    AsRef::<str>::as_ref(self)
                        == AsRef::<str>::as_ref(other)
                }
            }
        }
    })
    .collect()
}

mod kw {
    syn::custom_keyword!(validate);
}

#[derive(Default)]
struct IdZstMeta {
    validate: Option<Path>,
}

impl IdZstMeta {
    fn merge(self, other: IdZstMeta) -> syn::Result<Self> {
        let validate = match (self.validate, other.validate) {
            (None, None) => None,
            (Some(val), None) | (None, Some(val)) => Some(val),
            (Some(a), Some(b)) => {
                let mut error = syn::Error::new_spanned(b, "duplicate attribute argument");
                error.combine(syn::Error::new_spanned(a, "note: first one here"));
                return Err(error);
            }
        };

        Ok(Self { validate })
    }
}

impl Parse for IdZstMeta {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _: kw::validate = input.parse()?;
        let _: Token![=] = input.parse()?;
        let validate = Some(input.parse()?);
        Ok(Self { validate })
    }
}
