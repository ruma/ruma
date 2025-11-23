use proc_macro2::{Span, TokenStream};
use quote::quote;

use super::util::{RenameAll, RumaEnumAttrs, UnitVariant, VariantWithSingleField};

/// Generate the `From<T> where T: AsRef<str> + Into<Box<str>>` implementation for the given enum.
pub fn expand_enum_from_string(input: &syn::ItemEnum) -> syn::Result<TokenStream> {
    let ruma_enum = RumaEnumWithFallbackVariant::try_from(input)?;

    let ident = &input.ident;

    let unit_variants = ruma_enum.unit_variants_data().map(|(variant, string, aliases)| {
        quote! {
            #string #( | #aliases )* => Self::#variant,
        }
    });

    let fallback_variant = {
        let variant = ruma_enum.fallback_variant.expand_variable();
        let field_ty = &ruma_enum.fallback_variant.field.ty;

        quote! {
            _ => {
                let inner = #field_ty(s.into());
                Self::#variant
            }
        }
    };

    Ok(quote! {
        #[automatically_derived]
        #[allow(deprecated)]
        impl<T> ::std::convert::From<T> for #ident
        where
            T: ::std::convert::AsRef<::std::primitive::str>
                + ::std::convert::Into<::std::boxed::Box<::std::primitive::str>>
        {
            fn from(s: T) -> Self {
                match s.as_ref() {
                    #( #unit_variants )*
                    #fallback_variant
                }
            }
        }
    })
}

/// A parsed enum with `ruma_enum` attributes and a single fallback variant.
pub(crate) struct RumaEnumWithFallbackVariant {
    /// The unit variants of the enum.
    unit_variants: Vec<UnitVariant>,

    /// The fallback variant of the enum.
    fallback_variant: VariantWithSingleField,

    /// The global renaming rule for the variants.
    rename_all: RenameAll,
}

impl RumaEnumWithFallbackVariant {
    /// The names, string representations and aliases of the unit variants.
    pub(super) fn unit_variants_data(
        &self,
    ) -> impl Iterator<Item = (&syn::Ident, String, &[syn::LitStr])> {
        self.unit_variants.iter().map(|variant| {
            (
                &variant.ident,
                variant.string_representation(&self.rename_all),
                variant.aliases.as_slice(),
            )
        })
    }
}

impl TryFrom<&syn::ItemEnum> for RumaEnumWithFallbackVariant {
    type Error = syn::Error;

    fn try_from(input: &syn::ItemEnum) -> Result<Self, Self::Error> {
        let enum_attrs = RumaEnumAttrs::parse(&input.attrs)?;

        let mut fallback_variant = None;
        let mut unit_variants = Vec::new();

        // Parse enum variants.
        for variant in &input.variants {
            match &variant.fields {
                syn::Fields::Named(_) | syn::Fields::Unnamed(_) => {
                    if fallback_variant.is_some() {
                        return Err(syn::Error::new_spanned(
                            variant,
                            "cannot have multiple fallback variants",
                        ));
                    }

                    fallback_variant = Some(VariantWithSingleField::try_from(variant)?);
                }
                syn::Fields::Unit => {
                    unit_variants.push(UnitVariant::try_from(variant)?);
                }
            }
        }

        let ruma_enum = Self {
            unit_variants,
            fallback_variant: fallback_variant.ok_or_else(|| {
                syn::Error::new(Span::call_site(), "required fallback variant not found")
            })?,
            rename_all: enum_attrs.rename_all,
        };

        Ok(ruma_enum)
    }
}
