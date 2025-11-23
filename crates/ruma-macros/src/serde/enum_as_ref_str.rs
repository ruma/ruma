use proc_macro2::TokenStream;
use quote::quote;

use super::util::{RenameAll, RumaEnumAttrs, UnitVariant, VariantWithSingleField};

/// Generate the `AsRef<str>` implementation for the given enum.
pub fn expand_enum_as_ref_str(input: &syn::ItemEnum) -> syn::Result<TokenStream> {
    let ruma_enum = RumaEnumWithAnyVariants::try_from(input)?;

    let ident = &input.ident;

    let unit_variants = ruma_enum.unit_variants_data().map(|(variant, string)| {
        quote! {
            Self::#variant => #string,
        }
    });

    let field_variants = ruma_enum.expand_field_variants_variables().map(|variant| {
        quote! {
            Self::#variant => &inner.0,
        }
    });

    Ok(quote! {
        #[automatically_derived]
        #[allow(deprecated)]
        impl ::std::convert::AsRef<::std::primitive::str> for #ident {
            fn as_ref(&self) -> &::std::primitive::str {
                match self {
                    #( #unit_variants )*
                    #( #field_variants )*
                }
            }
        }
    })
}

/// A parsed enum with `ruma_enum` attributes and any [`UnitVariant`] or [`VariantWithSingleField`].
pub(crate) struct RumaEnumWithAnyVariants {
    /// The unit variants of the enum.
    unit_variants: Vec<UnitVariant>,

    /// The variants of the enum containing a single field.
    field_variants: Vec<VariantWithSingleField>,

    /// The global renaming rule for the variants.
    rename_all: RenameAll,
}

impl RumaEnumWithAnyVariants {
    /// The names and string representations of the unit variants.
    pub(super) fn unit_variants_data(&self) -> impl Iterator<Item = (&syn::Ident, String)> {
        self.unit_variants
            .iter()
            .map(|variant| (&variant.ident, variant.string_representation(&self.rename_all)))
    }

    /// Generate the code to extract or set the inner value of the field variants into or from a
    /// variable called `inner`.
    pub(super) fn expand_field_variants_variables(&self) -> impl Iterator<Item = TokenStream> {
        self.field_variants.iter().map(|variant| variant.expand_variable())
    }
}

impl TryFrom<&syn::ItemEnum> for RumaEnumWithAnyVariants {
    type Error = syn::Error;

    fn try_from(input: &syn::ItemEnum) -> Result<Self, Self::Error> {
        let enum_attrs = RumaEnumAttrs::parse(&input.attrs)?;

        let mut field_variants = Vec::new();
        let mut unit_variants = Vec::new();

        // Parse enum variants.
        for variant in &input.variants {
            match &variant.fields {
                syn::Fields::Named(_) | syn::Fields::Unnamed(_) => {
                    field_variants.push(VariantWithSingleField::try_from(variant)?);
                }
                syn::Fields::Unit => {
                    unit_variants.push(UnitVariant::try_from(variant)?);
                }
            }
        }

        Ok(Self { unit_variants, field_variants, rename_all: enum_attrs.rename_all })
    }
}
