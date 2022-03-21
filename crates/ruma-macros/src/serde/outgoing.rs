use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    AngleBracketedGenericArguments, Attribute, Data, DeriveInput, Field, Fields, GenericArgument,
    GenericParam, Generics, Ident, ImplGenerics, ParenthesizedGenericArguments, Path,
    PathArguments, Token, Type, TypeGenerics, TypePath, TypeReference, TypeSlice, Variant,
};

use crate::util::import_ruma_common;

enum StructKind {
    Struct,
    Tuple,
}

enum DataKind {
    Struct(Vec<Field>, StructKind),
    Enum(Vec<Variant>),
    Unit,
}

pub fn expand_derive_outgoing(input: DeriveInput) -> syn::Result<TokenStream> {
    let ruma_common = import_ruma_common();

    let mut derives = vec![quote! { Debug }];
    let mut derive_deserialize = true;

    derives.extend(
        input
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

    let input_attrs =
        input.attrs.iter().filter(|attr| filter_input_attrs(attr)).collect::<Vec<_>>();

    let data = match input.data.clone() {
        Data::Union(_) => panic!("#[derive(Outgoing)] does not support Union types"),
        Data::Enum(e) => DataKind::Enum(e.variants.into_iter().collect()),
        Data::Struct(s) => match s.fields {
            Fields::Named(fs) => {
                DataKind::Struct(fs.named.into_iter().collect(), StructKind::Struct)
            }
            Fields::Unnamed(fs) => {
                DataKind::Struct(fs.unnamed.into_iter().collect(), StructKind::Tuple)
            }
            Fields::Unit => DataKind::Unit,
        },
    };

    match data {
        DataKind::Unit => Ok(TokenStream::new()),
        DataKind::Enum(mut vars) => {
            let mut found_lifetime = false;
            for var in &mut vars {
                for field in &mut var.fields {
                    if strip_lifetimes(&mut field.ty) {
                        found_lifetime = true;
                    }
                }
            }

            if !found_lifetime {
                return Ok(TokenStream::new());
            }

            let vis = input.vis;
            let doc = format!("'Incoming' variant of [{ty}](enum.{ty}.html).", ty = &input.ident);
            let incoming_ident = format_ident!("Incoming{}", input.ident, span = Span::call_site());
            let mut gen_copy = input.generics.clone();
            let (_, ty_gen) = split_for_impl_lifetime_less(&mut gen_copy);

            Ok(quote! {
                #[doc = #doc]
                #[derive( #( #derives ),* )]
                #( #input_attrs )*
                #vis enum #incoming_ident #ty_gen { #( #vars, )* }
            })
        }
        DataKind::Struct(mut fields, struct_kind) => {
            let mut found_lifetime = false;
            for field in &mut fields {
                if !matches!(field.vis, syn::Visibility::Public(_)) {
                    return Err(syn::Error::new_spanned(field, "All fields must be marked `pub`"));
                }
                if strip_lifetimes(&mut field.ty) {
                    found_lifetime = true;
                }
            }

            if !found_lifetime {
                return Ok(TokenStream::new());
            }

            let vis = input.vis;
            let doc = format!("'Incoming' variant of [{ty}](struct.{ty}.html).", ty = &input.ident);
            let incoming_ident = format_ident!("Incoming{}", input.ident, span = Span::call_site());
            let mut gen_copy = input.generics.clone();
            let (_, ty_gen) = split_for_impl_lifetime_less(&mut gen_copy);

            let struct_def = match struct_kind {
                StructKind::Struct => quote! { { #(#fields,)* } },
                StructKind::Tuple => quote! { ( #(#fields,)* ); },
            };

            Ok(quote! {
                #[doc = #doc]
                #[derive( #( #derives ),* )]
                #( #input_attrs )*
                #vis struct #incoming_ident #ty_gen #struct_def
            })
        }
    }
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

fn split_for_impl_lifetime_less(generics: &mut Generics) -> (ImplGenerics<'_>, TypeGenerics<'_>) {
    generics.params = generics
        .params
        .clone()
        .into_iter()
        .filter(|param| !matches!(param, GenericParam::Lifetime(_)))
        .collect();

    let (impl_gen, ty_gen, _) = generics.split_for_impl();
    (impl_gen, ty_gen)
}

fn strip_lifetimes(field_type: &mut Type) -> bool {
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
                                    if strip_lifetimes(ty) {
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
                                if strip_lifetimes(&mut ty) {
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
                    } else if last_seg.ident == "ClientSecret"
                        || last_seg.ident == "DeviceId"
                        || last_seg.ident == "DeviceKeyId"
                        || last_seg.ident == "DeviceSigningKeyId"
                        || last_seg.ident == "EventId"
                        || last_seg.ident == "KeyId"
                        || last_seg.ident == "MxcUri"
                        || last_seg.ident == "ServerName"
                        || last_seg.ident == "SessionId"
                        || last_seg.ident == "RawJsonValue"
                        || last_seg.ident == "RoomAliasId"
                        || last_seg.ident == "RoomId"
                        || last_seg.ident == "RoomOrAliasId"
                        || last_seg.ident == "RoomName"
                        || last_seg.ident == "ServerSigningKeyId"
                        || last_seg.ident == "SigningKeyId"
                        || last_seg.ident == "TransactionId"
                        || last_seg.ident == "UserId"
                    {
                        // The identifiers that need to be boxed `Box<T>` since they are DST's.
                        Some(parse_quote! { ::std::boxed::Box<#path> })
                    } else {
                        None
                    }
                }
                // &[T] -> Vec<T>
                Type::Slice(TypeSlice { elem, .. }) => {
                    // Recursively strip the lifetimes of the slice's elements.
                    strip_lifetimes(&mut *elem);
                    Some(parse_quote! { Vec<#elem> })
                }
                _ => None,
            };

            *field_type = match special_replacement {
                Some(ty) => ty,
                None => {
                    // Strip lifetimes of `elem`.
                    strip_lifetimes(elem);
                    // Replace reference with `elem`.
                    (**elem).clone()
                }
            };

            true
        }
        Type::Tuple(syn::TypeTuple { elems, .. }) => {
            let mut has_lifetime = false;
            for elem in elems {
                if strip_lifetimes(elem) {
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
