//! Methods and types for generating identifiers.

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, ItemStruct, LitStr, Path, Token,
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

    let owned_decl = expand_owned_id(id, &owned);

    let meta = input.attrs.iter().filter(|attr| attr.path.is_ident("ruma_id")).try_fold(
        IdZstMeta::default(),
        |meta, attr| {
            let list: Punctuated<IdZstMeta, Token![,]> =
                attr.parse_args_with(Punctuated::parse_terminated)?;

            list.into_iter().try_fold(meta, IdZstMeta::merge)
        },
    )?;

    let extra_impls = if let Some(validate) = meta.validate {
        expand_checked_impls(id, &owned, validate)
    } else {
        expand_unchecked_impls(id, &owned)
    };

    let as_str_docs = format!("Creates a string slice from this `{}`.", id);
    let as_bytes_docs = format!("Creates a byte slice from this `{}`.", id);

    let partial_eq_string = expand_partial_eq_string(id);
    // FIXME: Remove?
    let box_partial_eq_string = expand_partial_eq_string(quote! { Box<#id> });

    Ok(quote! {
        #owned_decl

        impl #id {
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
            pub fn as_str(&self) -> &str {
                &self.0
            }

            #[doc = #as_bytes_docs]
            pub fn as_bytes(&self) -> &[u8] {
                self.0.as_bytes()
            }
        }

        impl Clone for Box<#id> {
            fn clone(&self) -> Self {
                (**self).into()
            }
        }

        impl ToOwned for #id {
            type Owned = Box<#id>;

            fn to_owned(&self) -> Self::Owned {
                Self::from_box(self.0.into())
            }
        }

        impl AsRef<str> for #id {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl AsRef<str> for Box<#id> {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl From<&#id> for String {
            fn from(id: &#id) -> Self {
                id.as_str().to_owned()
            }
        }

        impl From<Box<#id>> for String {
            fn from(id: Box<#id>) -> Self {
                id.into_owned().into()
            }
        }

        impl From<&#id> for Box<#id> {
            fn from(id: &#id) -> Self {
                #id::from_box(id.0.into())
            }
        }

        impl From<&#id> for std::rc::Rc<#id> {
            fn from(s: &#id) -> std::rc::Rc<#id> {
                let rc = std::rc::Rc::<str>::from(s.as_str());
                <#id>::from_rc(rc)
            }
        }

        impl From<&#id> for std::sync::Arc<#id> {
            fn from(s: &#id) -> std::sync::Arc<#id> {
                let arc = std::sync::Arc::<str>::from(s.as_str());
                <#id>::from_arc(arc)
            }
        }

        impl PartialEq<#id> for Box<#id> {
            fn eq(&self, other: &#id) -> bool {
                self.as_str() == other.as_str()
            }
        }

        impl PartialEq<&'_ #id> for Box<#id> {
            fn eq(&self, other: &&#id) -> bool {
                self.as_str() == other.as_str()
            }
        }

        impl PartialEq<Box<#id>> for #id {
            fn eq(&self, other: &Box<#id>) -> bool {
                self.as_str() == other.as_str()
            }
        }

        impl PartialEq<Box<#id>> for &'_ #id {
            fn eq(&self, other: &Box<#id>) -> bool {
                self.as_str() == other.as_str()
            }
        }

        impl std::fmt::Debug for #id {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                <str as std::fmt::Debug>::fmt(self.as_str(), f)
            }
        }

        impl std::fmt::Display for #id {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }

        impl serde::Serialize for #id {
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

fn expand_owned_id(id: &Ident, owned: &Ident) -> TokenStream {
    let doc_header = format!("Owned variant of {}", id);
    let partial_eq_string = expand_partial_eq_string(owned);

    quote! {
        #[doc = #doc_header]
        ///
        /// The wrapper type for this type is variable, by default it'll use [`Box`],
        /// but you can change that by setting "`--cfg=ruma_identifiers_storage=...`" using
        /// `RUSTFLAGS` or `.cargo/config.toml` (under `[build]` -> `rustflags = ["..."]`)
        /// to the following;
        /// - `ruma_identifiers_storage="Arc"` to use [`Arc`](std::sync::Arc) as a wrapper type.
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct #owned {
            #[cfg(not(any(ruma_identifiers_storage = "Arc")))]
            inner: Box<#id>,
            #[cfg(ruma_identifiers_storage = "Arc")]
            inner: std::sync::Arc<#id>,
        }

        impl AsRef<#id> for #owned {
            fn as_ref(&self) -> &#id {
                &*self.inner
            }
        }

        impl AsRef<str> for #owned {
            fn as_ref(&self) -> &str {
                (*self.inner).as_ref()
            }
        }

        impl std::ops::Deref for #owned {
            type Target = #id;

            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }

        impl std::borrow::Borrow<#id> for #owned {
            fn borrow(&self) -> &#id {
                self.as_ref()
            }
        }

        impl From<&'_ #id> for #owned {
            fn from(id: &#id) -> #owned {
                #owned { inner: id.into() }
            }
        }

        impl From<Box<#id>> for #owned {
            fn from(b: Box<#id>) -> #owned {
                Self { inner: b.into() }
            }
        }

        impl From<std::sync::Arc<#id>> for #owned {
            fn from(a: std::sync::Arc<#id>) -> #owned {
                Self {
                    #[cfg(not(any(ruma_identifiers_storage = "Arc")))]
                    inner: a.as_ref().into(),
                    #[cfg(ruma_identifiers_storage = "Arc")]
                    inner: a,
                }
            }
        }

        impl std::fmt::Display for #owned {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }

        impl serde::Serialize for #owned {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        #partial_eq_string

        impl PartialEq<#id> for #owned {
            fn eq(&self, other: &#id) -> bool {
                AsRef::<#id>::as_ref(self) == other
            }
        }

        impl PartialEq<#owned> for #id {
            fn eq(&self, other: &#owned) -> bool {
                self == AsRef::<#id>::as_ref(other)
            }
        }

        impl PartialEq<&#id> for #owned {
            fn eq(&self, other: &&#id) -> bool {
                AsRef::<#id>::as_ref(self) == *other
            }
        }

        impl PartialEq<#owned> for &#id {
            fn eq(&self, other: &#owned) -> bool {
                *self == AsRef::<#id>::as_ref(other)
            }
        }

        impl PartialEq<Box<#id>> for #owned {
            fn eq(&self, other: &Box<#id>) -> bool {
                AsRef::<#id>::as_ref(self) == AsRef::<#id>::as_ref(other)
            }
        }

        impl PartialEq<#owned> for Box<#id> {
            fn eq(&self, other: &#owned) -> bool {
                AsRef::<#id>::as_ref(self) == AsRef::<#id>::as_ref(other)
            }
        }
    }
}

fn expand_checked_impls(id: &Ident, owned: &Ident, validate: Path) -> TokenStream {
    let parse_doc_header = format!("Try parsing a `&str` into a `Box<{}>`.", id);
    let parse_rc_docs = format!("Try parsing a `&str` into an `Rc<{}>`.", id);
    let parse_arc_docs = format!("Try parsing a `&str` into an `Arc<{}>`.", id);

    quote! {
        impl #id {
            #[doc = #parse_doc_header]
            ///
            /// The same can also be done using `FromStr`, `TryFrom` or `TryInto`.
            /// This function is simply more constrained and thus useful in generic contexts.
            pub fn parse(
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

        impl<'de> serde::Deserialize<'de> for Box<#id> {
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

        impl<'de> serde::Deserialize<'de> for #owned {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                use serde::de::Error;

                let s = String::deserialize(deserializer)?;

                match #id::parse(s) {
                    Ok(o) => Ok(o.into()),
                    Err(e) => Err(D::Error::custom(e)),
                }
            }
        }

        impl<'a> std::convert::TryFrom<&'a str> for &'a #id {
            type Error = crate::IdParseError;

            fn try_from(s: &'a str) -> Result<Self, Self::Error> {
                #validate(s)?;
                Ok(#id::from_borrowed(s))
            }
        }

        impl std::str::FromStr for Box<#id> {
            type Err = crate::IdParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                #id::parse(s)
            }
        }

        impl std::convert::TryFrom<&str> for Box<#id> {
            type Error = crate::IdParseError;

            fn try_from(s: &str) -> Result<Self, Self::Error> {
                #id::parse(s)
            }
        }

        impl std::convert::TryFrom<String> for Box<#id> {
            type Error = crate::IdParseError;

            fn try_from(s: String) -> Result<Self, Self::Error> {
                #id::parse(s)
            }
        }
    }
}

fn expand_unchecked_impls(id: &Ident, owned: &Ident) -> TokenStream {
    quote! {
        impl<'a> From<&'a str> for &'a #id {
            fn from(s: &'a str) -> Self {
                #id::from_borrowed(s)
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

fn expand_partial_eq_string(id: impl ToTokens) -> TokenStream {
    fn single_impl(lhs: impl ToTokens, rhs: impl ToTokens) -> TokenStream {
        quote! {
            impl PartialEq<#rhs> for #lhs {
                fn eq(&self, other: &#rhs) -> bool {
                    AsRef::<str>::as_ref(self)
                        == AsRef::<str>::as_ref(other)
                }
            }
        }
    }

    let id = &id;

    let mut res = TokenStream::new();
    res.extend(single_impl(id, quote! { str }));
    res.extend(single_impl(id, quote! { &str }));
    res.extend(single_impl(id, quote! { String }));
    res.extend(single_impl(quote! { str }, id));
    res.extend(single_impl(quote! { &str }, id));
    res.extend(single_impl(quote! { String }, id));
    res
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
