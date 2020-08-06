use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse_quote, AngleBracketedGenericArguments, Attribute, Data, DeriveInput, Field, Fields,
    GenericArgument, GenericParam, Generics, ImplGenerics, ParenthesizedGenericArguments,
    PathArguments, Type, TypeGenerics, TypePath, TypeReference, TypeSlice, Variant,
};

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
    let derive_deserialize = if no_deserialize_in_attrs(&input.attrs) {
        TokenStream::new()
    } else {
        quote!(::ruma_api::exports::serde::Deserialize)
    };

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
        DataKind::Unit => Ok(impl_outgoing_with_incoming_self(&input)),
        DataKind::Enum(mut vars) => {
            let mut found_lifetime = false;
            for var in &mut vars {
                for field in &mut var.fields {
                    if strip_lifetimes(&mut field.ty) {
                        found_lifetime = true;
                    }
                }
            }

            let original_ident = &input.ident;
            let (original_impl_gen, original_ty_gen, _) = input.generics.split_for_impl();

            if !found_lifetime {
                return Ok(impl_outgoing_with_incoming_self(&input));
            }

            let vis = input.vis;
            let doc = format!("'Incoming' variant of [{ty}](enum.{ty}.html).", ty = &input.ident);
            let incoming_ident =
                format_ident!("Incoming{}", original_ident, span = Span::call_site());
            let mut gen_copy = input.generics.clone();
            let (impl_gen, ty_gen) = split_for_impl_lifetime_less(&mut gen_copy);

            Ok(quote! {
                #[doc = #doc]
                #[derive(Debug, #derive_deserialize)]
                #vis enum #incoming_ident #ty_gen { #( #vars, )* }

                impl #original_impl_gen ::ruma_api::Outgoing for #original_ident #original_ty_gen {
                    type Incoming = #incoming_ident #impl_gen;
                }
            })
        }
        DataKind::Struct(mut fields, struct_kind) => {
            let mut found_lifetime = false;
            for field in &mut fields {
                if strip_lifetimes(&mut field.ty) {
                    found_lifetime = true;
                }
            }

            let original_ident = &input.ident;
            let (original_impl_gen, original_ty_gen, _) = input.generics.split_for_impl();

            if !found_lifetime {
                return Ok(impl_outgoing_with_incoming_self(&input));
            }

            let vis = input.vis;
            let doc = format!("'Incoming' variant of [{ty}](struct.{ty}.html).", ty = &input.ident);
            let incoming_ident =
                format_ident!("Incoming{}", original_ident, span = Span::call_site());
            let mut gen_copy = input.generics.clone();
            let (impl_gen, ty_gen) = split_for_impl_lifetime_less(&mut gen_copy);

            let struct_def = match struct_kind {
                StructKind::Struct => quote! { { #(#fields,)* } },
                StructKind::Tuple => quote! { ( #(#fields,)* ); },
            };

            Ok(quote! {
                #[doc = #doc]
                #[derive(Debug, #derive_deserialize)]
                #vis struct #incoming_ident #ty_gen #struct_def

                impl #original_impl_gen ::ruma_api::Outgoing for #original_ident #original_ty_gen {
                    type Incoming = #incoming_ident #impl_gen;
                }
            })
        }
    }
}

fn no_deserialize_in_attrs(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path.is_ident("incoming_no_deserialize"))
}

fn impl_outgoing_with_incoming_self(input: &DeriveInput) -> TokenStream {
    let ident = &input.ident;
    let (impl_gen, ty_gen, _) = input.generics.split_for_impl();

    quote! {
        impl #impl_gen ::ruma_api::Outgoing for #ident #ty_gen {
            type Incoming = Self;
        }
    }
}

fn split_for_impl_lifetime_less(generics: &mut Generics) -> (ImplGenerics, TypeGenerics) {
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
            let mut generic_lifetime = false;

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
                                    generic_lifetime = true;
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

            // If a type has a generic lifetime parameter there must be an `Incoming` variant of that type.
            if generic_lifetime {
                if let Some(name) = path.segments.last_mut() {
                    let incoming_ty_ident = format_ident!("Incoming{}", name.ident);
                    name.ident = incoming_ty_ident;
                }
            }

            has_lifetimes || generic_lifetime
        }
        Type::Reference(TypeReference { elem, .. }) => match &mut **elem {
            Type::Path(ty_path) => {
                let TypePath { path, .. } = ty_path;
                let segs = path
                    .segments
                    .clone()
                    .into_iter()
                    .map(|seg| seg.ident.to_string())
                    .collect::<Vec<_>>();

                if path.is_ident("str") {
                    // &str -> String
                    *field_type = parse_quote! { ::std::string::String };
                } else if segs.contains(&"DeviceId".into()) || segs.contains(&"ServerName".into()) {
                    // The identifiers that need to be boxed `Box<T>` since they are DST's.
                    *field_type = parse_quote! { ::std::boxed::Box<#path> };
                } else {
                    // &T -> T
                    *field_type = Type::Path(ty_path.clone());
                }
                true
            }
            // &[T] -> Vec<T>
            Type::Slice(TypeSlice { elem, .. }) => {
                // Recursively strip the lifetimes of the slice's elements.
                strip_lifetimes(&mut *elem);
                *field_type = parse_quote! { Vec<#elem> };
                true
            }
            _ => false,
        },
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
