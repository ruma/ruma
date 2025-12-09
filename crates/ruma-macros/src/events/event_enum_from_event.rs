//! Implementation of the `EventEnumFromEvent` derive macro.

use proc_macro2::{Span, TokenStream};
use quote::quote;

/// `EventEnumFromEvent` derive macro code generation.
pub(crate) fn expand_event_enum_from_event(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let event_enum_from_event = EventEnumFromEvent::parse(input)?;

    Ok(event_enum_from_event.expand_from_impls())
}

/// The parsed `EventEnumFromEvent` container data.
struct EventEnumFromEvent {
    /// The name of the event enum.
    ident: syn::Ident,

    /// The unit variants of the enum.
    variants: Vec<EventEnumFromEventVariant>,
}

impl EventEnumFromEvent {
    /// Parse the given input as an `EventEnumFromEvent`.
    fn parse(input: syn::DeriveInput) -> syn::Result<Self> {
        let syn::Data::Enum(syn::DataEnum { variants, .. }) = input.data else {
            return Err(syn::Error::new_spanned(
                input,
                "the `EventEnumFromEvent` derive macro only works on enums",
            ));
        };

        let variants = variants
            .into_iter()
            .map(EventEnumFromEventVariant::parse)
            .collect::<syn::Result<_>>()?;

        Ok(Self { ident: input.ident, variants })
    }

    /// Generate the `From<{variant_inner_type}> for {ident}` implementations.
    fn expand_from_impls(&self) -> TokenStream {
        let ident = &self.ident;

        self.variants
            .iter()
            .map(|variant| {
                let variant_ident = &variant.ident;
                let variant_inner_type = &variant.field.ty;

                quote! {
                    #[automatically_derived]
                    impl ::std::convert::From<#variant_inner_type> for #ident {
                        fn from(c: #variant_inner_type) -> Self {
                            Self::#variant_ident(c)
                        }
                    }
                }
            })
            .collect()
    }
}

/// A parsed unit variant of [`EventEnumFromEvent`].
struct EventEnumFromEventVariant {
    /// The name of the variant.
    ident: syn::Ident,

    /// The field of the variant.
    field: syn::Field,
}

impl EventEnumFromEventVariant {
    /// Parse the given variant as an `EventEnumFromEventVariant`.
    fn parse(variant: syn::Variant) -> syn::Result<Self> {
        if let syn::Fields::Unnamed(fields) = variant.fields
            && fields.unnamed.len() == 1
        {
            Ok(Self {
                ident: variant.ident,
                field: fields
                    .unnamed
                    .into_iter()
                    .next()
                    .expect("variant should have one unnamed field"),
            })
        } else {
            Err(syn::Error::new(
                Span::call_site(),
                "the `EventEnumFromEvent` derive macro only works with unit variants",
            ))
        }
    }
}
