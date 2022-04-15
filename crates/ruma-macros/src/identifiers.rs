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
    let owned = format_ident!("Owned{}", id);

    let owned_decl = expand_owned_id(&input);

    let meta = input.attrs.iter().filter(|attr| attr.path.is_ident("ruma_id")).try_fold(
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

    let as_str_docs = format!("Creates a string slice from this `{}`.", id);
    let as_bytes_docs = format!("Creates a byte slice from this `{}`.", id);

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

    let partial_eq_string = expand_partial_eq_string(quote! { #id #ty_generics }, &impl_generics);
    // FIXME: Remove?
    let box_partial_eq_string =
        expand_partial_eq_string(quote! { Box<#id #ty_generics> }, &impl_generics);

    Ok(quote! {
        #owned_decl

        impl #impl_generics #id #ty_generics {
            pub(super) fn from_borrowed(s: &str) -> &Self {
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

        impl #impl_generics Clone for Box<#id #ty_generics> {
            fn clone(&self) -> Self {
                (**self).into()
            }
        }

        impl #impl_generics ToOwned for #id #ty_generics {
            type Owned = #owned #ty_generics;

            fn to_owned(&self) -> Self::Owned {
                #owned::from_ref(self)
            }
        }

        impl #impl_generics AsRef<str> for #id #ty_generics {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl #impl_generics AsRef<str> for Box<#id #ty_generics> {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl #impl_generics From<&#id #ty_generics> for String {
            fn from(id: &#id #ty_generics) -> Self {
                id.as_str().to_owned()
            }
        }

        impl #impl_generics From<Box<#id #ty_generics>> for String {
            fn from(id: Box<#id #ty_generics>) -> Self {
                id.into_owned().into()
            }
        }

        impl #impl_generics From<&#id #ty_generics> for Box<#id #ty_generics> {
            fn from(id: &#id #ty_generics) -> Self {
                <#id #ty_generics>::from_box(id.as_str().into())
            }
        }

        impl #impl_generics From<&#id #ty_generics> for std::rc::Rc<#id #ty_generics> {
            fn from(s: &#id #ty_generics) -> std::rc::Rc<#id #ty_generics> {
                let rc = std::rc::Rc::<str>::from(s.as_str());
                <#id #ty_generics>::from_rc(rc)
            }
        }

        impl #impl_generics From<&#id #ty_generics> for std::sync::Arc<#id #ty_generics> {
            fn from(s: &#id #ty_generics) -> std::sync::Arc<#id #ty_generics> {
                let arc = std::sync::Arc::<str>::from(s.as_str());
                <#id #ty_generics>::from_arc(arc)
            }
        }

        impl #impl_generics PartialEq<#id #ty_generics> for Box<#id #ty_generics> {
            fn eq(&self, other: &#id #ty_generics) -> bool {
                self.as_str() == other.as_str()
            }
        }

        impl #impl_generics PartialEq<&'_ #id #ty_generics> for Box<#id #ty_generics> {
            fn eq(&self, other: &&#id #ty_generics) -> bool {
                self.as_str() == other.as_str()
            }
        }

        impl #impl_generics PartialEq<Box<#id #ty_generics>> for #id #ty_generics {
            fn eq(&self, other: &Box<#id #ty_generics>) -> bool {
                self.as_str() == other.as_str()
            }
        }

        impl #impl_generics PartialEq<Box<#id #ty_generics>> for &'_ #id #ty_generics {
            fn eq(&self, other: &Box<#id #ty_generics>) -> bool {
                self.as_str() == other.as_str()
            }
        }

        impl #impl_generics std::fmt::Debug for #id #ty_generics {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                <str as std::fmt::Debug>::fmt(self.as_str(), f)
            }
        }

        impl #impl_generics std::fmt::Display for #id #ty_generics {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }

        impl #impl_generics serde::Serialize for #id #ty_generics {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        #partial_eq_string
        #box_partial_eq_string
        #extra_impls
    })
}

fn expand_owned_id(input: &ItemStruct) -> TokenStream {
    let id = &input.ident;
    let owned = format_ident!("Owned{}", id);

    let doc_header = format!("Owned variant of {}", id);
    let (impl_generics, ty_generics, _where_clause) = input.generics.split_for_impl();
    let partial_eq_string =
        expand_partial_eq_string(quote! { #owned #ty_generics }, &impl_generics);

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
            inner: Box<#id #ty_generics>,
            #[cfg(ruma_identifiers_storage = "Arc")]
            inner: std::sync::Arc<#id #ty_generics>,
        }

        impl #impl_generics #owned #ty_generics {
            fn from_ref(v: &#id #ty_generics) -> Self {
                Self {
                    #[cfg(not(any(ruma_identifiers_storage = "Arc")))]
                    inner: #id::from_box(v.as_str().into()),
                    #[cfg(ruma_identifiers_storage = "Arc")]
                    inner: #id::from_arc(v.as_str().into()),
                }
            }
        }

        impl #impl_generics AsRef<#id #ty_generics> for #owned #ty_generics {
            fn as_ref(&self) -> &#id #ty_generics {
                &*self.inner
            }
        }

        impl #impl_generics AsRef<str> for #owned #ty_generics {
            fn as_ref(&self) -> &str {
                (*self.inner).as_ref()
            }
        }

        impl #impl_generics std::clone::Clone for #owned #ty_generics {
            fn clone(&self) -> Self {
                (&*self.inner).into()
            }
        }

        impl #impl_generics std::ops::Deref for #owned #ty_generics {
            type Target = #id #ty_generics;

            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }

        impl #impl_generics std::borrow::Borrow<#id #ty_generics> for #owned #ty_generics {
            fn borrow(&self) -> &#id #ty_generics {
                self.as_ref()
            }
        }

        impl #impl_generics From<&'_ #id #ty_generics> for #owned #ty_generics {
            fn from(id: &#id #ty_generics) -> #owned #ty_generics {
                #owned { inner: id.into() }
            }
        }

        impl #impl_generics From<Box<#id #ty_generics>> for #owned #ty_generics {
            fn from(b: Box<#id #ty_generics>) -> #owned #ty_generics {
                Self { inner: b.into() }
            }
        }

        impl #impl_generics From<std::sync::Arc<#id #ty_generics>> for #owned #ty_generics {
            fn from(a: std::sync::Arc<#id #ty_generics>) -> #owned #ty_generics {
                Self {
                    #[cfg(not(any(ruma_identifiers_storage = "Arc")))]
                    inner: a.as_ref().into(),
                    #[cfg(ruma_identifiers_storage = "Arc")]
                    inner: a,
                }
            }
        }

        impl #impl_generics std::fmt::Display for #owned #ty_generics {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }

        impl #impl_generics std::fmt::Debug for #owned #ty_generics {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                <str as std::fmt::Debug>::fmt(self.as_str(), f)
            }
        }

        impl #impl_generics std::cmp::PartialEq for #owned #ty_generics {
            fn eq(&self, other: &Self) -> bool {
                self.as_str() == other.as_str()
            }
        }

        impl #impl_generics std::cmp::Eq for #owned #ty_generics {}

        impl #impl_generics std::cmp::PartialOrd for #owned #ty_generics {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl #impl_generics std::cmp::Ord for #owned #ty_generics {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.as_str().cmp(other.as_str())
            }
        }

        impl #impl_generics std::hash::Hash for #owned #ty_generics {
            fn hash<H>(&self, state: &mut H)
            where
                H: std::hash::Hasher,
            {
                self.as_str().hash(state)
            }
        }

        impl #impl_generics serde::Serialize for #owned #ty_generics {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        #partial_eq_string

        impl #impl_generics PartialEq<#id #ty_generics> for #owned #ty_generics {
            fn eq(&self, other: &#id #ty_generics) -> bool {
                AsRef::<#id #ty_generics>::as_ref(self) == other
            }
        }

        impl #impl_generics PartialEq<#owned #ty_generics> for #id #ty_generics {
            fn eq(&self, other: &#owned #ty_generics) -> bool {
                self == AsRef::<#id #ty_generics>::as_ref(other)
            }
        }

        impl #impl_generics PartialEq<&#id #ty_generics> for #owned #ty_generics {
            fn eq(&self, other: &&#id #ty_generics) -> bool {
                AsRef::<#id #ty_generics>::as_ref(self) == *other
            }
        }

        impl #impl_generics PartialEq<#owned #ty_generics> for &#id #ty_generics {
            fn eq(&self, other: &#owned #ty_generics) -> bool {
                *self == AsRef::<#id #ty_generics>::as_ref(other)
            }
        }

        impl #impl_generics PartialEq<Box<#id #ty_generics>> for #owned #ty_generics {
            fn eq(&self, other: &Box<#id #ty_generics>) -> bool {
                AsRef::<#id #ty_generics>::as_ref(self) == AsRef::<#id #ty_generics>::as_ref(other)
            }
        }

        impl #impl_generics PartialEq<#owned #ty_generics> for Box<#id #ty_generics> {
            fn eq(&self, other: &#owned #ty_generics) -> bool {
                AsRef::<#id #ty_generics>::as_ref(self) == AsRef::<#id #ty_generics>::as_ref(other)
            }
        }
    }
}

fn expand_checked_impls(input: &ItemStruct, validate: Path) -> TokenStream {
    let id = &input.ident;
    let owned = format_ident!("Owned{}", id);

    let (impl_generics, ty_generics, _where_clause) = input.generics.split_for_impl();
    let generic_params = &input.generics.params;

    let parse_doc_header = format!("Try parsing a `&str` into an `Owned{}`.", id);
    let parse_box_doc_header = format!("Try parsing a `&str` into a `Box<{}>`.", id);
    let parse_rc_docs = format!("Try parsing a `&str` into an `Rc<{}>`.", id);
    let parse_arc_docs = format!("Try parsing a `&str` into an `Arc<{}>`.", id);

    quote! {
        impl #impl_generics #id #ty_generics {
            #[doc = #parse_doc_header]
            ///
            /// The same can also be done using `FromStr`, `TryFrom` or `TryInto`.
            /// This function is simply more constrained and thus useful in generic contexts.
            pub fn parse(
                s: impl AsRef<str>,
            ) -> Result<#owned #ty_generics, crate::IdParseError> {
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

        impl<'de, #generic_params> serde::Deserialize<'de> for Box<#id #ty_generics> {
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

        impl<'de, #generic_params> serde::Deserialize<'de> for #owned #ty_generics {
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

        impl<'a, #generic_params> std::convert::TryFrom<&'a str> for &'a #id #ty_generics {
            type Error = crate::IdParseError;

            fn try_from(s: &'a str) -> Result<Self, Self::Error> {
                #validate(s)?;
                Ok(<#id #ty_generics>::from_borrowed(s))
            }
        }

        impl #impl_generics std::str::FromStr for Box<#id #ty_generics> {
            type Err = crate::IdParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                <#id #ty_generics>::parse_box(s)
            }
        }

        impl #impl_generics std::convert::TryFrom<&str> for Box<#id #ty_generics> {
            type Error = crate::IdParseError;

            fn try_from(s: &str) -> Result<Self, Self::Error> {
                <#id #ty_generics>::parse_box(s)
            }
        }

        impl #impl_generics std::convert::TryFrom<String> for Box<#id #ty_generics> {
            type Error = crate::IdParseError;

            fn try_from(s: String) -> Result<Self, Self::Error> {
                <#id #ty_generics>::parse_box(s)
            }
        }

        impl #impl_generics std::str::FromStr for #owned #ty_generics {
            type Err = crate::IdParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                <#id #ty_generics>::parse(s)
            }
        }

        impl #impl_generics std::convert::TryFrom<&str> for #owned #ty_generics {
            type Error = crate::IdParseError;

            fn try_from(s: &str) -> Result<Self, Self::Error> {
                <#id #ty_generics>::parse(s)
            }
        }

        impl #impl_generics std::convert::TryFrom<String> for #owned #ty_generics {
            type Error = crate::IdParseError;

            fn try_from(s: String) -> Result<Self, Self::Error> {
                <#id #ty_generics>::parse(s)
            }
        }
    }
}

fn expand_unchecked_impls(input: &ItemStruct) -> TokenStream {
    let id = &input.ident;
    let owned = format_ident!("Owned{}", id);

    quote! {
        impl<'a> From<&'a str> for &'a #id {
            fn from(s: &'a str) -> Self {
                #id::from_borrowed(s)
            }
        }

        impl From<&str> for #owned {
            fn from(s: &str) -> Self {
                <&#id>::from(s).into()
            }
        }

        impl From<Box<str>> for #owned {
            fn from(s: Box<str>) -> Self {
                <&#id>::from(&*s).into()
            }
        }

        impl From<String> for #owned {
            fn from(s: String) -> Self {
                <&#id>::from(s.as_str()).into()
            }
        }

        impl From<&str> for Box<#id> {
            fn from(s: &str) -> Self {
                #id::from_box(s.into())
            }
        }

        impl From<Box<str>> for Box<#id> {
            fn from(s: Box<str>) -> Self {
                #id::from_box(s)
            }
        }

        impl From<String> for Box<#id> {
            fn from(s: String) -> Self {
                #id::from_box(s.into())
            }
        }

        impl From<Box<#id>> for Box<str> {
            fn from(id: Box<#id>) -> Self {
                id.into_owned()
            }
        }

        impl<'de> serde::Deserialize<'de> for Box<#id> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Box::<str>::deserialize(deserializer).map(#id::from_box)
            }
        }

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
