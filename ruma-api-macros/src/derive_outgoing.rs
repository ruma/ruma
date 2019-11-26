use std::mem;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse_quote, punctuated::Pair, spanned::Spanned, Attribute, Data, DeriveInput, Fields,
    GenericArgument, Path, PathArguments, Type, TypePath,
};

mod wrap_incoming;

use wrap_incoming::Meta;

pub fn expand_derive_outgoing(input: DeriveInput) -> syn::Result<TokenStream> {
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            input.generics,
            "derive(Outgoing) doesn't currently support types with generics!",
        ));
    }

    let derive_deserialize = if no_deserialize_in_attrs(&input.attrs) {
        TokenStream::new()
    } else {
        quote!(#[derive(ruma_api::exports::serde::Deserialize)])
    };

    let mut fields: Vec<_> = match input.data {
        Data::Enum(_) | Data::Union(_) => {
            panic!("#[derive(Outgoing)] is only supported for structs")
        }
        Data::Struct(s) => match s.fields {
            Fields::Named(fs) => fs.named.into_pairs().map(Pair::into_value).collect(),
            Fields::Unnamed(fs) => fs.unnamed.into_pairs().map(Pair::into_value).collect(),
            Fields::Unit => return Ok(impl_send_recv_incoming_self(input.ident)),
        },
    };

    let mut any_attribute = false;

    for field in &mut fields {
        let mut field_meta = None;

        let mut remaining_attrs = Vec::new();
        for attr in mem::replace(&mut field.attrs, Vec::new()) {
            if let Some(meta) = Meta::from_attribute(&attr)? {
                if field_meta.is_some() {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "duplicate #[wrap_incoming] attribute",
                    ));
                }
                field_meta = Some(meta);
                any_attribute = true;
            } else {
                remaining_attrs.push(attr);
            }
        }
        field.attrs = remaining_attrs;

        if let Some(attr) = field_meta {
            if let Some(type_to_wrap) = attr.type_to_wrap {
                wrap_generic_arg(&type_to_wrap, &mut field.ty, attr.wrapper_type.as_ref())?;
            } else {
                wrap_ty(&mut field.ty, attr.wrapper_type)?;
            }
        }
    }

    if !any_attribute {
        return Ok(impl_send_recv_incoming_self(input.ident));
    }

    let vis = input.vis;
    let doc = format!("'Incoming' variant of [{ty}](struct.{ty}.html).", ty = input.ident);
    let original_ident = input.ident;
    let incoming_ident = Ident::new(&format!("Incoming{}", original_ident), Span::call_site());

    Ok(quote! {
        #[doc = #doc]
        #derive_deserialize
        #vis struct #incoming_ident {
            #(#fields,)*
        }

        impl ruma_api::Outgoing for #original_ident {
            type Incoming = #incoming_ident;
        }
    })
}

fn no_deserialize_in_attrs(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        match &attr.path {
            Path { leading_colon: None, segments }
                if segments.len() == 1 && segments[0].ident == "incoming_no_deserialize" =>
            {
                return true
            }
            _ => {}
        }
    }

    false
}

fn impl_send_recv_incoming_self(ident: Ident) -> TokenStream {
    quote! {
        impl ruma_api::Outgoing for #ident {
            type Incoming = Self;
        }
    }
}

fn wrap_ty(ty: &mut Type, path: Option<Path>) -> syn::Result<()> {
    if let Some(wrap_ty) = path {
        *ty = parse_quote!(#wrap_ty<#ty>);
    } else {
        match ty {
            Type::Path(TypePath { path, .. }) => {
                let ty_ident = &mut path.segments.last_mut().unwrap().ident;
                let ident = Ident::new(&format!("Incoming{}", ty_ident), Span::call_site());
                *ty_ident = parse_quote!(#ident);
            }
            _ => return Err(syn::Error::new_spanned(ty, "Can't wrap this type")),
        }
    }

    Ok(())
}

fn wrap_generic_arg(type_to_wrap: &Type, of: &mut Type, with: Option<&Path>) -> syn::Result<()> {
    let mut span = None;
    wrap_generic_arg_impl(type_to_wrap, of, with, &mut span)?;

    if span.is_some() {
        Ok(())
    } else {
        Err(syn::Error::new_spanned(
            of,
            format!(
                "Couldn't find generic argument `{}` in this type",
                type_to_wrap.to_token_stream()
            ),
        ))
    }
}

fn wrap_generic_arg_impl(
    type_to_wrap: &Type,
    of: &mut Type,
    with: Option<&Path>,
    span: &mut Option<Span>,
) -> syn::Result<()> {
    // TODO: Support things like array types?
    let ty_path = match of {
        Type::Path(TypePath { path, .. }) => path,
        _ => return Ok(()),
    };

    let args = match &mut ty_path.segments.last_mut().unwrap().arguments {
        PathArguments::AngleBracketed(ab) => &mut ab.args,
        _ => return Ok(()),
    };

    for arg in args.iter_mut() {
        let ty = match arg {
            GenericArgument::Type(ty) => ty,
            _ => continue,
        };

        if ty == type_to_wrap {
            if let Some(s) = span {
                let mut error = syn::Error::new(
                    *s,
                    format!(
                        "`{}` found multiple times, this is not currently supported",
                        type_to_wrap.to_token_stream()
                    ),
                );
                error.combine(syn::Error::new_spanned(ty, "second occurrence"));
                return Err(error);
            }

            *span = Some(ty.span());

            if let Some(wrapper_type) = with {
                *ty = parse_quote!(#wrapper_type<#ty>);
            } else if let Type::Path(TypePath { path, .. }) = ty {
                let ty_ident = &mut path.segments.last_mut().unwrap().ident;
                let ident = Ident::new(&format!("Incoming{}", ty_ident), Span::call_site());
                *ty_ident = parse_quote!(#ident);
            } else {
                return Err(syn::Error::new_spanned(ty, "Can't wrap this type"));
            }
        } else {
            wrap_generic_arg_impl(type_to_wrap, ty, with, span)?;
        }
    }

    Ok(())
}
