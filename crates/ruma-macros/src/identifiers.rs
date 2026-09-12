//! Methods and types for generating identifiers.

use std::borrow::Cow;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{meta::ParseNestedMeta, parse_quote};

use crate::util::{RumaCommon, RumaCommonReexport};

pub(crate) mod constructor;
pub(crate) mod id_dst;
pub(crate) mod ruma_id;

/// The default size of the inline array for the `SmallVec` inner representation.
const SMALLVEC_INLINE_BYTES_DEFAULT: usize = 32;

/// The parsed attributes of the `ruma_id` attribute of the `IdDst` derive macro or of the `ruma_id`
/// attribute macro.
#[derive(Default)]
pub(crate) struct RumaIdAttrs {
    /// The path to the function to use to validate the identifier.
    validate: Option<syn::Path>,

    /// The size of the inline array for the `SmallVec` inner representation.
    smallvec_inline_bytes: Option<usize>,
}

impl RumaIdAttrs {
    /// Set the path to the function to use to validate the identifier.
    ///
    /// Returns an error if it is already set.
    fn set_validate(&mut self, validate: syn::Path, meta: &ParseNestedMeta<'_>) -> syn::Result<()> {
        if self.validate.is_some() {
            return Err(meta.error("cannot have multiple values for `validate` attribute"));
        }

        self.validate = Some(validate);
        Ok(())
    }

    /// Set the size of the inline array for the `SmallVec` inner representation.
    ///
    /// Returns an error if it is already set or if the value doesn't fit into a `usize`.
    fn set_smallvec_inline_bytes(
        &mut self,
        inline_bytes: syn::LitInt,
        meta: &ParseNestedMeta<'_>,
    ) -> syn::Result<()> {
        if self.smallvec_inline_bytes.is_some() {
            return Err(
                meta.error("cannot have multiple values for `smallvec_inline_bytes` attribute")
            );
        }

        self.smallvec_inline_bytes = Some(inline_bytes.base10_parse()?);
        Ok(())
    }

    /// Try to parse the given meta item and merge it into this `IdDstAttrs`.
    ///
    /// Returns an error if an unknown `ruma_id` attribute is encountered, or if an attribute
    /// that accepts a single value appears several times.
    pub(crate) fn try_merge(&mut self, meta: ParseNestedMeta<'_>) -> syn::Result<()> {
        if meta.path.is_ident("validate") {
            return self.set_validate(meta.value()?.parse()?, &meta);
        }

        if meta.path.is_ident("smallvec_inline_bytes") {
            return self.set_smallvec_inline_bytes(meta.value()?.parse()?, &meta);
        }

        Err(meta.error("unsupported `ruma_id` attribute"))
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
