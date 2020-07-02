//! Functions to aid the `Api::to_tokens` method.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

use crate::api::{metadata::Metadata, request::Request};

/// The first item in the tuple generates code for the request path from
/// the `Metadata` and `Request` structs. The second item in the returned tuple
/// is the code to generate a Request struct field created from any segments
/// of the path that start with ":".
///
/// The first `TokenStream` returned is the constructed url path. The second `TokenStream` is
/// used for implementing `TryFrom<http::Request<Vec<u8>>>`, from path strings deserialized to ruma types.
pub(crate) fn request_path_string_and_parse(
    request: &Request,
    metadata: &Metadata,
) -> (TokenStream, TokenStream) {
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
                    ruma_api::exports::percent_encoding::utf8_percent_encode(
                        &request.#path_var.to_string(),
                        ruma_api::exports::percent_encoding::NON_ALPHANUMERIC,
                    )
                });
                format_string.replace_range(start_of_segment..end_of_segment, "{}");
            }

            quote! {
                format!(#format_string, #(#format_args),*)
            }
        };

        let path_fields =
            path_string[1..].split('/').enumerate().filter(|(_, s)| s.starts_with(':')).map(
                |(i, segment)| {
                    let path_var = &segment[1..];
                    let path_var_ident = Ident::new(path_var, Span::call_site());

                    quote! {
                        #path_var_ident: {
                            use std::ops::Deref as _;
                            use ruma_api::error::RequestDeserializationError;

                            let segment = path_segments.get(#i).unwrap().as_bytes();
                            let decoded = match ruma_api::exports::percent_encoding::percent_decode(
                                segment
                            ).decode_utf8() {
                                Ok(x) => x,
                                Err(err) => {
                                    return Err(
                                        RequestDeserializationError::new(err, request).into()
                                    );
                                }
                            };
                            match std::convert::TryFrom::try_from(decoded.deref()) {
                                Ok(val) => val,
                                Err(err) => {
                                    return Err(
                                        RequestDeserializationError::new(err, request).into()
                                    );
                                }
                            }
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
pub(crate) fn build_query_string(request: &Request) -> TokenStream {
    if let Some(field) = request.query_map_field() {
        let field_name = field.ident.as_ref().expect("expected field to have identifier");
        let field_type = &field.ty;

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
            fn assert_trait_impl<T>()
            where
                T: std::iter::IntoIterator<Item = (std::string::String, std::string::String)>,
            {}
            assert_trait_impl::<#field_type>();

            let request_query = RequestQuery(request.#field_name);
            format!("?{}", ruma_api::exports::ruma_serde::urlencoded::to_string(request_query)?)
        })
    } else if request.has_query_fields() {
        let request_query_init_fields = request.request_query_init_fields();

        quote!({
            let request_query = RequestQuery {
                #request_query_init_fields
            };

            format!("?{}", ruma_api::exports::ruma_serde::urlencoded::to_string(request_query)?)
        })
    } else {
        quote! {
            String::new()
        }
    }
}

/// Deserialize the query string.
pub(crate) fn extract_request_query(request: &Request) -> TokenStream {
    if request.query_map_field().is_some() {
        quote! {
            let request_query = match ruma_api::exports::ruma_serde::urlencoded::from_str(
                &request.uri().query().unwrap_or("")
            ) {
                Ok(query) => query,
                Err(err) => {
                    return Err(
                        ruma_api::error::RequestDeserializationError::new(err, request).into()
                    );
                }
            };
        }
    } else if request.has_query_fields() {
        quote! {
            let request_query: RequestQuery =
                match ruma_api::exports::ruma_serde::urlencoded::from_str(
                    &request.uri().query().unwrap_or("")
                ) {
                    Ok(query) => query,
                    Err(err) => {
                        return Err(
                            ruma_api::error::RequestDeserializationError::new(err, request)
                                .into()
                        );
                    }
                };
        }
    } else {
        TokenStream::new()
    }
}

/// Generates the code to initialize a `Request`.
///
/// Used to construct an `http::Request`s body.
pub(crate) fn build_request_body(request: &Request) -> TokenStream {
    if let Some(field) = request.newtype_raw_body_field() {
        let field_name = field.ident.as_ref().expect("expected field to have an identifier");
        quote!(request.#field_name)
    } else if request.has_body_fields() || request.newtype_body_field().is_some() {
        let request_body_initializers = if let Some(field) = request.newtype_body_field() {
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");
            quote! { (request.#field_name) }
        } else {
            let initializers = request.request_body_init_fields();
            quote! { { #initializers } }
        };

        quote! {
            {
                let request_body = RequestBody #request_body_initializers;
                ruma_api::exports::serde_json::to_vec(&request_body)?
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

pub(crate) fn is_ascii_printable(string: &str) -> bool {
    string.as_bytes().iter().all(|b| (0x20..=0x7E).contains(b))
}
