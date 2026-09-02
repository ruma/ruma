//! Types and functions to handle the identifiers internal storage representations.

use std::borrow::Cow;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse_quote;

use super::{IdDst, Types};
use crate::util::{RumaCommon, RumaCommonReexport};

/// Data for the owned variant of an identifier.
pub(super) struct OwnedId {
    /// The name of the owned type.
    pub(super) ident: syn::Ident,

    /// The owned type with generics, if any.
    pub(super) id_type: syn::Type,

    /// The size of the inline array for the `SmallVec` inner representation.
    pub(super) smallvec_inline_bytes: usize,

    /// `#[cfg]` attributes for the internal storage representations.
    storage_attrs: StorageCfgAttributes,
}

impl OwnedId {
    /// Construct a new `OwnedId`.
    pub(super) fn new(ident: syn::Ident, id_type: syn::Type, smallvec_inline_bytes: usize) -> Self {
        Self { ident, id_type, smallvec_inline_bytes, storage_attrs: StorageCfgAttributes::new() }
    }

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

    /// Generate the `Owned{ident}` type and its implementations.
    pub(super) fn expand_struct(&self, id_dst: &IdDst) -> TokenStream {
        let owned_ident = &self.ident;
        let owned_id = &self.id_type;

        let generics = &id_dst.generics;
        let impl_generics = &id_dst.impl_generics;
        let types = &id_dst.types;
        let ruma_common = &id_dst.ruma_common;

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

        let doc_header = format!("Owned variant of [`{}`].", id_dst.ident);
        let doc_values = StorageCfgValue::ALL
            .iter()
            .map(|value| StorageCfgValue::doc(value, self))
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
        let as_str_impls = self.expand_for_each_storage_value(|value| {
            let expanded = value.expand_as_str_impl(&self_inner_field, types);
            quote! { { #expanded } }
        });
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
            "Securely zero memory (aka [zeroize](https://en.wikipedia.org/wiki/Zeroisation)) of `{owned_ident}`."
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
            #[doc = #doc_header]
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
            pub struct #owned_ident #generics {
                #inner_types_decl
                #phantom_decl
            }

            #[automatically_derived]
            impl #impl_generics #owned_id {
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

                /// Access the inner string without going through the borrowed type.
                pub(super) fn as_inner_str(&self) -> &#str {
                    #as_str_impls
                }

                /// Access the inner bytes without going through the borrowed type.
                pub(super) fn as_inner_bytes(&self) -> &#bytes {
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
            impl #impl_generics ::std::clone::Clone for #owned_id {
                fn clone(&self) -> Self {
                    unsafe { Self::from_inner_unchecked(self.inner.clone()) }
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::cmp::PartialEq for #owned_id {
                fn eq(&self, other: &Self) -> ::std::primitive::bool {
                    self.inner.eq(&other.inner)
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::cmp::Eq for #owned_id {}

            #[automatically_derived]
            impl #impl_generics ::std::cmp::PartialOrd for #owned_id {
                fn partial_cmp(&self, other: &Self) -> ::std::option::Option<::std::cmp::Ordering> {
                    ::std::option::Option::Some(self.cmp(other))
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::cmp::Ord for #owned_id {
                fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                    self.inner.cmp(&other.inner)
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

            #[automatically_derived]
            impl #impl_generics ::std::borrow::Borrow<#str> for #owned_id {
                fn borrow(&self) -> &#str {
                    self.as_inner_str()
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
            impl #impl_generics ::std::convert::From<#owned_id> for #box_str {
                fn from(id: #owned_id) -> Self {
                    #into_box_str_impls
                }
            }

            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#owned_id> for #string {
                fn from(id: #owned_id) -> Self {
                    #into_string_impls
                }
            }
        }
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
    fn doc(&self, owned_id: &OwnedId) -> Cow<'static, str> {
        match self {
            Self::Default => Cow::Borrowed(""),
            Self::Arc => Cow::Borrowed("`Arc` -- Use an `Arc<str>`."),
            Self::ThinArc => Cow::Borrowed(
                "`ThinArc` -- Use a `triomphe::ThinArc<(), u8>`. \
                 Requires the `triomphe` cargo feature.",
            ),
            Self::SmallVec => Cow::Owned(format!(
                "`SmallVec` -- Use a `smallvec::SmallVec<[u8; {}]>`. \
                 Requires the `smallvec` cargo feature.",
                owned_id.smallvec_inline_bytes
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
                #id_var.as_inner_str().into()
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
                #id_var.as_inner_str().into()
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
