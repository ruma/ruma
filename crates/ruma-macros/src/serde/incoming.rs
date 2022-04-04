use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    AngleBracketedGenericArguments, Attribute, Data, DeriveInput, GenericArgument, GenericParam,
    Generics, Ident, ItemType, ParenthesizedGenericArguments, Path, PathArguments, Token, Type,
    TypePath, TypeReference, TypeSlice,
};

use crate::util::import_ruma_common;

pub fn expand_derive_incoming(mut ty_def: DeriveInput) -> syn::Result<TokenStream> {
    let ruma_common = import_ruma_common();

    let mut found_lifetime = false;
    match &mut ty_def.data {
        Data::Union(_) => panic!("#[derive(Incoming)] does not support Union types"),
        Data::Enum(e) => {
            for var in &mut e.variants {
                for field in &mut var.fields {
                    if strip_lifetimes(&mut field.ty, &ruma_common) {
                        found_lifetime = true;
                    }
                }
            }
        }
        Data::Struct(s) => {
            for field in &mut s.fields {
                if !matches!(field.vis, syn::Visibility::Public(_)) {
                    return Err(syn::Error::new_spanned(field, "All fields must be marked `pub`"));
                }
                if strip_lifetimes(&mut field.ty, &ruma_common) {
                    found_lifetime = true;
                }
            }
        }
    }

    let ident = format_ident!("Incoming{}", ty_def.ident, span = Span::call_site());

    if !found_lifetime {
        let doc = format!(
            "Convenience type alias for [{}], for consistency with other [{}] types.",
            &ty_def.ident, ident
        );

        let mut type_alias: ItemType = parse_quote! { type X = Y; };
        type_alias.vis = ty_def.vis.clone();
        type_alias.ident = ident;
        type_alias.generics = ty_def.generics.clone();
        type_alias.ty =
            Box::new(TypePath { qself: None, path: ty_def.ident.clone().into() }.into());

        return Ok(quote! {
            #[doc = #doc]
            #type_alias
        });
    }

    let mut derives = vec![quote! { Debug }];
    let mut derive_deserialize = true;

    derives.extend(
        ty_def
            .attrs
            .iter()
            .filter(|attr| attr.path.is_ident("incoming_derive"))
            .map(|attr| attr.parse_args())
            .collect::<syn::Result<Vec<Meta>>>()?
            .into_iter()
            .flat_map(|meta| meta.derive_macs)
            .filter_map(|derive_mac| match derive_mac {
                DeriveMac::Regular(id) => Some(quote! { #id }),
                DeriveMac::NegativeDeserialize => {
                    derive_deserialize = false;
                    None
                }
            }),
    );

    derives.push(if derive_deserialize {
        quote! { #ruma_common::exports::serde::Deserialize }
    } else {
        quote! { #ruma_common::serde::_FakeDeriveSerde }
    });

    ty_def.attrs.retain(filter_input_attrs);
    clean_generics(&mut ty_def.generics);

    let doc = format!("'Incoming' variant of [{}].", &ty_def.ident);
    ty_def.ident = ident;

    Ok(quote! {
        #[doc = #doc]
        #[derive( #( #derives ),* )]
        #ty_def
    })
}

/// Keep any `cfg`, `cfg_attr`, `serde` or `non_exhaustive` attributes found and pass them to the
/// Incoming variant.
fn filter_input_attrs(attr: &Attribute) -> bool {
    attr.path.is_ident("cfg")
        || attr.path.is_ident("cfg_attr")
        || attr.path.is_ident("serde")
        || attr.path.is_ident("non_exhaustive")
        || attr.path.is_ident("allow")
}

fn clean_generics(generics: &mut Generics) {
    generics.params = generics
        .params
        .clone()
        .into_iter()
        .filter(|param| !matches!(param, GenericParam::Lifetime(_)))
        .collect();
}

fn strip_lifetimes(field_type: &mut Type, ruma_common: &TokenStream) -> bool {
    match field_type {
        // T<'a> -> IncomingT
        // The IncomingT has to be declared by the user of this derive macro.
        Type::Path(TypePath { path, .. }) => {
            let mut has_lifetimes = false;
            let mut is_lifetime_generic = false;

            for seg in &mut path.segments {
                // strip generic lifetimes
                match &mut seg.arguments {
                    PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                        args, ..
                    }) => {
                        *args = args
                            .clone()
                            .into_iter()
                            .map(|mut ty| {
                                if let GenericArgument::Type(ty) = &mut ty {
                                    if strip_lifetimes(ty, ruma_common) {
                                        has_lifetimes = true;
                                    };
                                }
                                ty
                            })
                            .filter(|arg| {
                                if let GenericArgument::Lifetime(_) = arg {
                                    is_lifetime_generic = true;
                                    false
                                } else {
                                    true
                                }
                            })
                            .collect();
                    }
                    PathArguments::Parenthesized(ParenthesizedGenericArguments {
                        inputs, ..
                    }) => {
                        *inputs = inputs
                            .clone()
                            .into_iter()
                            .map(|mut ty| {
                                if strip_lifetimes(&mut ty, ruma_common) {
                                    has_lifetimes = true;
                                };
                                ty
                            })
                            .collect();
                    }
                    _ => {}
                }
            }

            // If a type has a generic lifetime parameter there must be an `Incoming` variant of
            // that type.
            if is_lifetime_generic {
                if let Some(name) = path.segments.last_mut() {
                    let incoming_ty_ident = format_ident!("Incoming{}", name.ident);
                    name.ident = incoming_ty_ident;
                }
            }

            has_lifetimes || is_lifetime_generic
        }
        Type::Reference(TypeReference { elem, .. }) => {
            let special_replacement = match &mut **elem {
                Type::Path(ty) => {
                    let path = &ty.path;
                    let last_seg = path.segments.last().unwrap();

                    if last_seg.ident == "str" {
                        // &str -> String
                        Some(parse_quote! { ::std::string::String })
                    } else if last_seg.ident == "RawJsonValue" {
                        Some(parse_quote! { ::std::boxed::Box<#path> })
                    } else if last_seg.ident == "ClientSecret"
                        || last_seg.ident == "DeviceId"
                        || last_seg.ident == "DeviceKeyId"
                        || last_seg.ident == "DeviceSigningKeyId"
                        || last_seg.ident == "EventId"
                        || last_seg.ident == "KeyId"
                        || last_seg.ident == "MxcUri"
                        || last_seg.ident == "ServerName"
                        || last_seg.ident == "SessionId"
                        || last_seg.ident == "RoomAliasId"
                        || last_seg.ident == "RoomId"
                        || last_seg.ident == "RoomOrAliasId"
                        || last_seg.ident == "RoomName"
                        || last_seg.ident == "ServerSigningKeyId"
                        || last_seg.ident == "SigningKeyId"
                        || last_seg.ident == "TransactionId"
                        || last_seg.ident == "UserId"
                    {
                        let ident = format_ident!("Owned{}", last_seg.ident);
                        Some(parse_quote! { #ruma_common::#ident })
                    } else {
                        None
                    }
                }
                // &[T] -> Vec<T>
                Type::Slice(TypeSlice { elem, .. }) => {
                    // Recursively strip the lifetimes of the slice's elements.
                    strip_lifetimes(&mut *elem, ruma_common);
                    Some(parse_quote! { Vec<#elem> })
                }
                _ => None,
            };

            *field_type = match special_replacement {
                Some(ty) => ty,
                None => {
                    // Strip lifetimes of `elem`.
                    strip_lifetimes(elem, ruma_common);
                    // Replace reference with `elem`.
                    (**elem).clone()
                }
            };

            true
        }
        Type::Tuple(syn::TypeTuple { elems, .. }) => {
            let mut has_lifetime = false;
            for elem in elems {
                if strip_lifetimes(elem, ruma_common) {
                    has_lifetime = true;
                }
            }
            has_lifetime
        }
        _ => false,
    }
}

pub struct Meta {
    derive_macs: Vec<DeriveMac>,
}

impl Parse for Meta {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            derive_macs: Punctuated::<_, Token![,]>::parse_terminated(input)?.into_iter().collect(),
        })
    }
}

pub enum DeriveMac {
    Regular(Path),
    NegativeDeserialize,
}

impl Parse for DeriveMac {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.peek(Token![!]) {
            let _: Token![!] = input.parse()?;
            let mac: Ident = input.parse()?;

            if mac != "Deserialize" {
                return Err(syn::Error::new_spanned(
                    mac,
                    "Negative incoming_derive can only be used for Deserialize",
                ));
            }

            Ok(Self::NegativeDeserialize)
        } else {
            let mac = input.parse()?;
            Ok(Self::Regular(mac))
        }
    }
}
