//! Functions to aid the `Api::to_tokens` method.

use proc_macro2::{Span, TokenStream};
use proc_macro_crate::crate_name;
use quote::quote;
use std::collections::BTreeSet;
use syn::{
    AngleBracketedGenericArguments, GenericArgument, Ident, Lifetime,
    ParenthesizedGenericArguments, PathArguments, Type, TypeArray, TypeBareFn, TypeGroup,
    TypeParen, TypePath, TypePtr, TypeReference, TypeSlice, TypeTuple,
};

use crate::api::{metadata::Metadata, request::Request};

pub fn collect_lifetime_ident(lifetimes: &mut BTreeSet<Lifetime>, ty: &Type) {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            for seg in &path.segments {
                match &seg.arguments {
                    PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                        args, ..
                    }) => {
                        for gen in args {
                            if let GenericArgument::Type(ty) = gen {
                                collect_lifetime_ident(lifetimes, &ty);
                            } else if let GenericArgument::Lifetime(lt) = gen {
                                lifetimes.insert(lt.clone());
                            }
                        }
                    }
                    PathArguments::Parenthesized(ParenthesizedGenericArguments {
                        inputs, ..
                    }) => {
                        for ty in inputs {
                            collect_lifetime_ident(lifetimes, ty);
                        }
                    }
                    _ => {}
                }
            }
        }
        Type::Reference(TypeReference { elem, lifetime, .. }) => {
            collect_lifetime_ident(lifetimes, &*elem);
            if let Some(lt) = lifetime {
                lifetimes.insert(lt.clone());
            }
        }
        Type::Tuple(TypeTuple { elems, .. }) => {
            for ty in elems {
                collect_lifetime_ident(lifetimes, ty);
            }
        }
        Type::Paren(TypeParen { elem, .. }) => collect_lifetime_ident(lifetimes, &*elem),
        Type::Group(TypeGroup { elem, .. }) => collect_lifetime_ident(lifetimes, &*elem),
        Type::Ptr(TypePtr { elem, .. }) => collect_lifetime_ident(lifetimes, &*elem),
        Type::Slice(TypeSlice { elem, .. }) => collect_lifetime_ident(lifetimes, &*elem),
        Type::Array(TypeArray { elem, .. }) => collect_lifetime_ident(lifetimes, &*elem),
        Type::BareFn(TypeBareFn {
            lifetimes: Some(syn::BoundLifetimes { lifetimes: fn_lifetimes, .. }),
            ..
        }) => {
            for lt in fn_lifetimes {
                let syn::LifetimeDef { lifetime, .. } = lt;
                lifetimes.insert(lifetime.clone());
            }
        }
        _ => {}
    }
}

/// Generates a `TokenStream` of lifetime identifiers `<'lifetime>`.
pub fn unique_lifetimes_to_tokens<'a, I: Iterator<Item = &'a Lifetime>>(
    lifetimes: I,
) -> TokenStream {
    let lifetimes = lifetimes.collect::<BTreeSet<_>>();
    if lifetimes.is_empty() {
        TokenStream::new()
    } else {
        let lifetimes = quote! { #( #lifetimes ),* };
        quote! { < #lifetimes > }
    }
}

pub fn has_lifetime(ty: &Type) -> bool {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            let mut found = false;
            for seg in &path.segments {
                match &seg.arguments {
                    PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                        args, ..
                    }) => {
                        for gen in args {
                            if let GenericArgument::Type(ty) = gen {
                                if has_lifetime(&ty) {
                                    found = true;
                                };
                            } else if let GenericArgument::Lifetime(_) = gen {
                                return true;
                            }
                        }
                    }
                    PathArguments::Parenthesized(ParenthesizedGenericArguments {
                        inputs, ..
                    }) => {
                        for ty in inputs {
                            if has_lifetime(ty) {
                                found = true;
                            }
                        }
                    }
                    _ => {}
                }
            }
            found
        }
        Type::Reference(TypeReference { elem, lifetime, .. }) => {
            if lifetime.is_some() {
                true
            } else {
                has_lifetime(&elem)
            }
        }
        Type::Tuple(TypeTuple { elems, .. }) => {
            let mut found = false;
            for ty in elems {
                if has_lifetime(ty) {
                    found = true;
                }
            }
            found
        }
        Type::Paren(TypeParen { elem, .. }) => has_lifetime(&elem),
        Type::Group(TypeGroup { elem, .. }) => has_lifetime(&*elem),
        Type::Ptr(TypePtr { elem, .. }) => has_lifetime(&*elem),
        Type::Slice(TypeSlice { elem, .. }) => has_lifetime(&*elem),
        Type::Array(TypeArray { elem, .. }) => has_lifetime(&*elem),
        Type::BareFn(TypeBareFn { lifetimes: Some(syn::BoundLifetimes { .. }), .. }) => true,
        _ => false,
    }
}

/// The first item in the tuple generates code for the request path from
/// the `Metadata` and `Request` structs. The second item in the returned tuple
/// is the code to generate a Request struct field created from any segments
/// of the path that start with ":".
///
/// The first `TokenStream` returned is the constructed url path. The second `TokenStream` is
/// used for implementing `TryFrom<http::Request<Vec<u8>>>`, from path strings deserialized to Ruma
/// types.
pub(crate) fn request_path_string_and_parse(
    request: &Request,
    metadata: &Metadata,
    ruma_api: &TokenStream,
) -> (TokenStream, TokenStream) {
    let percent_encoding = quote! { #ruma_api::exports::percent_encoding };

    if request.has_path_fields() {
        let path_string = metadata.path.value();

        assert!(path_string.starts_with('/'), "path needs to start with '/'");
        assert!(
            path_string.chars().filter(|c| *c == ':').count() == request.path_field_count(),
            "number of declared path parameters needs to match amount of placeholders in path"
        );

        let format_call = {
            let mut format_string = path_string.clone();
            let mut format_args = Vec::new();

            while let Some(start_of_segment) = format_string.find(':') {
                // ':' should only ever appear at the start of a segment
                assert_eq!(&format_string[start_of_segment - 1..start_of_segment], "/");

                let end_of_segment = match format_string[start_of_segment..].find('/') {
                    Some(rel_pos) => start_of_segment + rel_pos,
                    None => format_string.len(),
                };

                let path_var = Ident::new(
                    &format_string[start_of_segment + 1..end_of_segment],
                    Span::call_site(),
                );
                format_args.push(quote! {
                    #percent_encoding::utf8_percent_encode(
                        &self.#path_var.to_string(),
                        #percent_encoding::NON_ALPHANUMERIC,
                    )
                });
                format_string.replace_range(start_of_segment..end_of_segment, "{}");
            }

            quote! {
                format_args!(#format_string, #(#format_args),*)
            }
        };

        let path_fields =
            path_string[1..].split('/').enumerate().filter(|(_, s)| s.starts_with(':')).map(
                |(i, segment)| {
                    let path_var = &segment[1..];
                    let path_var_ident = Ident::new(path_var, Span::call_site());
                    quote! {
                        #path_var_ident: {
                            use #ruma_api::error::RequestDeserializationError;

                            let segment = path_segments.get(#i).unwrap().as_bytes();
                            let decoded = #ruma_api::try_deserialize!(
                                request,
                                #percent_encoding::percent_decode(segment)
                                    .decode_utf8(),
                            );

                            #ruma_api::try_deserialize!(
                                request,
                                ::std::convert::TryFrom::try_from(&*decoded),
                            )
                        }
                    }
                },
            );

        (format_call, quote! { #(#path_fields,)* })
    } else {
        (quote! { metadata.path.to_owned() }, TokenStream::new())
    }
}

/// The function determines the type of query string that needs to be built
/// and then builds it using `ruma_serde::urlencoded::to_string`.
pub(crate) fn build_query_string(request: &Request, ruma_api: &TokenStream) -> TokenStream {
    let ruma_serde = quote! { #ruma_api::exports::ruma_serde };

    if let Some(field) = request.query_map_field() {
        let field_name = field.ident.as_ref().expect("expected field to have identifier");

        quote!({
            // This function exists so that the compiler will throw an
            // error when the type of the field with the query_map
            // attribute doesn't implement IntoIterator<Item = (String, String)>
            //
            // This is necessary because the ruma_serde::urlencoded::to_string
            // call will result in a runtime error when the type cannot be
            // encoded as a list key-value pairs (?key1=value1&key2=value2)
            //
            // By asserting that it implements the iterator trait, we can
            // ensure that it won't fail.
            fn assert_trait_impl<T>(_: &T)
            where
                T: ::std::iter::IntoIterator<Item = (::std::string::String, ::std::string::String)>,
            {}

            let request_query = RequestQuery(self.#field_name);
            assert_trait_impl(&request_query.0);

            format_args!(
                "?{}",
                #ruma_serde::urlencoded::to_string(request_query)?
            )
        })
    } else if request.has_query_fields() {
        let request_query_init_fields = request.request_query_init_fields();

        quote!({
            let request_query = RequestQuery {
                #request_query_init_fields
            };

            format_args!(
                "?{}",
                #ruma_serde::urlencoded::to_string(request_query)?
            )
        })
    } else {
        quote! { "" }
    }
}

/// Deserialize the query string.
pub(crate) fn extract_request_query(request: &Request, ruma_api: &TokenStream) -> TokenStream {
    let ruma_serde = quote! { #ruma_api::exports::ruma_serde };

    if request.query_map_field().is_some() {
        quote! {
            let request_query = #ruma_api::try_deserialize!(
                request,
                #ruma_serde::urlencoded::from_str(
                    &request.uri().query().unwrap_or("")
                ),
            );
        }
    } else if request.has_query_fields() {
        quote! {
            let request_query: <RequestQuery as #ruma_serde::Outgoing>::Incoming =
                #ruma_api::try_deserialize!(
                    request,
                    #ruma_serde::urlencoded::from_str(
                        &request.uri().query().unwrap_or("")
                    ),
                );
        }
    } else {
        TokenStream::new()
    }
}

/// Generates the code to initialize a `Request`.
///
/// Used to construct an `http::Request`s body.
pub(crate) fn build_request_body(request: &Request, ruma_api: &TokenStream) -> TokenStream {
    let serde_json = quote! { #ruma_api::exports::serde_json };

    if let Some(field) = request.newtype_raw_body_field() {
        let field_name = field.ident.as_ref().expect("expected field to have an identifier");
        quote!(self.#field_name)
    } else if request.has_body_fields() || request.newtype_body_field().is_some() {
        let request_body_initializers = if let Some(field) = request.newtype_body_field() {
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");
            quote! { (self.#field_name) }
        } else {
            let initializers = request.request_body_init_fields();
            quote! { { #initializers } }
        };

        quote! {
            {
                let request_body = RequestBody #request_body_initializers;
                #serde_json::to_vec(&request_body)?
            }
        }
    } else {
        quote!(Vec::new())
    }
}

pub(crate) fn parse_request_body(request: &Request) -> TokenStream {
    if let Some(field) = request.newtype_body_field() {
        let field_name = field.ident.as_ref().expect("expected field to have an identifier");
        quote! {
            #field_name: request_body.0,
        }
    } else if let Some(field) = request.newtype_raw_body_field() {
        let field_name = field.ident.as_ref().expect("expected field to have an identifier");
        quote! {
            #field_name: request.into_body(),
        }
    } else {
        request.request_init_body_fields()
    }
}

pub(crate) fn req_res_meta_word<T>(
    attr_kind: &str,
    field: &syn::Field,
    newtype_body_field: &mut Option<syn::Field>,
    body_field_kind: T,
    raw_field_kind: T,
) -> syn::Result<T> {
    if let Some(f) = &newtype_body_field {
        let mut error = syn::Error::new_spanned(field, "There can only be one newtype body field");
        error.combine(syn::Error::new_spanned(f, "Previous newtype body field"));
        return Err(error);
    }

    *newtype_body_field = Some(field.clone());
    Ok(match attr_kind {
        "body" => body_field_kind,
        "raw_body" => raw_field_kind,
        _ => unreachable!(),
    })
}

pub(crate) fn req_res_name_value<T>(
    name: Ident,
    value: Ident,
    header: &mut Option<Ident>,
    field_kind: T,
) -> syn::Result<T> {
    if name != "header" {
        return Err(syn::Error::new_spanned(
            name,
            "Invalid #[ruma_api] argument with value, expected `header`",
        ));
    }

    *header = Some(value);
    Ok(field_kind)
}

pub(crate) fn is_valid_endpoint_path(string: &str) -> bool {
    string.as_bytes().iter().all(|b| (0x21..=0x7E).contains(b))
}

pub fn import_ruma_api() -> TokenStream {
    if let Ok(possibly_renamed) = crate_name("ruma-api") {
        let import = Ident::new(&possibly_renamed, Span::call_site());
        quote! { ::#import }
    } else if let Ok(possibly_renamed) = crate_name("ruma") {
        let import = Ident::new(&possibly_renamed, Span::call_site());
        quote! { ::#import::api }
    } else {
        quote! { ::ruma_api }
    }
}
