use std::{collections::BTreeSet, mem};

use syn::{
    braced,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    AngleBracketedGenericArguments, Attribute, BoundLifetimes, Field, GenericArgument, Ident,
    Lifetime, LifetimeDef, ParenthesizedGenericArguments, PathArguments, Token, Type, TypeArray,
    TypeBareFn, TypeGroup, TypeParen, TypePath, TypePtr, TypeReference, TypeSlice, TypeTuple,
};

use super::{
    attribute::{Meta, MetaNameValue},
    request::{RequestField, RequestFieldKind, RequestLifetimes},
    response::{ResponseField, ResponseFieldKind},
    Api, Metadata, Request, Response,
};
use crate::util;

mod kw {
    syn::custom_keyword!(error);
    syn::custom_keyword!(request);
    syn::custom_keyword!(response);
}

impl Parse for Api {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let metadata: Metadata = input.parse()?;
        let request: Request = input.parse()?;
        let response: Response = input.parse()?;

        // TODO: Use `bool::then` when MSRV >= 1.50
        let error_ty = if input.peek(kw::error) {
            let _: kw::error = input.parse()?;
            let _: Token![:] = input.parse()?;

            Some(input.parse()?)
        } else {
            None
        };

        let newtype_body_field = request.newtype_body_field();
        if metadata.method == "GET" && (request.has_body_fields() || newtype_body_field.is_some()) {
            let mut combined_error: Option<syn::Error> = None;
            let mut add_error = |field| {
                let error = syn::Error::new_spanned(field, "GET endpoints can't have body fields");
                if let Some(combined_error_ref) = &mut combined_error {
                    combined_error_ref.combine(error);
                } else {
                    combined_error = Some(error);
                }
            };

            for field in request.body_fields() {
                add_error(field);
            }

            if let Some(field) = newtype_body_field {
                add_error(field);
            }

            return Err(combined_error.unwrap());
        }

        Ok(Self { metadata, request, response, error_ty })
    }
}

impl Parse for Request {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let attributes = input.call(Attribute::parse_outer)?;
        let request_kw = input.parse::<kw::request>()?;
        input.parse::<Token![:]>()?;
        let fields;
        braced!(fields in input);

        let fields = fields.parse_terminated::<Field, Token![,]>(Field::parse_named)?;

        let mut newtype_body_field = None;
        let mut query_map_field = None;
        let mut lifetimes = RequestLifetimes::default();

        let fields = fields
            .into_iter()
            .map(|mut field| {
                let mut field_kind = None;
                let mut header = None;

                for attr in mem::take(&mut field.attrs) {
                    let meta = match Meta::from_attribute(&attr)? {
                        Some(m) => m,
                        None => {
                            field.attrs.push(attr);
                            continue;
                        }
                    };

                    if field_kind.is_some() {
                        return Err(syn::Error::new_spanned(
                            attr,
                            "There can only be one field kind attribute",
                        ));
                    }

                    field_kind = Some(match meta {
                        Meta::Word(ident) => match &ident.to_string()[..] {
                            attr @ "body" | attr @ "raw_body" => req_res_meta_word(
                                attr,
                                &field,
                                &mut newtype_body_field,
                                RequestFieldKind::NewtypeBody,
                                RequestFieldKind::NewtypeRawBody,
                            )?,
                            "path" => RequestFieldKind::Path,
                            "query" => RequestFieldKind::Query,
                            "query_map" => {
                                if let Some(f) = &query_map_field {
                                    let mut error = syn::Error::new_spanned(
                                        field,
                                        "There can only be one query map field",
                                    );
                                    error.combine(syn::Error::new_spanned(
                                        f,
                                        "Previous query map field",
                                    ));
                                    return Err(error);
                                }

                                query_map_field = Some(field.clone());
                                RequestFieldKind::QueryMap
                            }
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    ident,
                                    "Invalid #[ruma_api] argument, expected one of \
                                     `body`, `path`, `query`, `query_map`",
                                ));
                            }
                        },
                        Meta::NameValue(MetaNameValue { name, value }) => {
                            req_res_name_value(name, value, &mut header, RequestFieldKind::Header)?
                        }
                    });
                }

                match field_kind.unwrap_or(RequestFieldKind::Body) {
                    RequestFieldKind::Header => {
                        collect_lifetime_ident(&mut lifetimes.header, &field.ty)
                    }
                    RequestFieldKind::Body => {
                        collect_lifetime_ident(&mut lifetimes.body, &field.ty)
                    }
                    RequestFieldKind::NewtypeBody => {
                        collect_lifetime_ident(&mut lifetimes.body, &field.ty)
                    }
                    RequestFieldKind::NewtypeRawBody => {
                        collect_lifetime_ident(&mut lifetimes.body, &field.ty)
                    }
                    RequestFieldKind::Path => {
                        collect_lifetime_ident(&mut lifetimes.path, &field.ty)
                    }
                    RequestFieldKind::Query => {
                        collect_lifetime_ident(&mut lifetimes.query, &field.ty)
                    }
                    RequestFieldKind::QueryMap => {
                        collect_lifetime_ident(&mut lifetimes.query, &field.ty)
                    }
                }

                Ok(RequestField::new(field_kind.unwrap_or(RequestFieldKind::Body), field, header))
            })
            .collect::<syn::Result<Vec<_>>>()?;

        if newtype_body_field.is_some() && fields.iter().any(|f| f.is_body()) {
            // TODO: highlight conflicting fields,
            return Err(syn::Error::new_spanned(
                request_kw,
                "Can't have both a newtype body field and regular body fields",
            ));
        }

        if query_map_field.is_some() && fields.iter().any(|f| f.is_query()) {
            return Err(syn::Error::new_spanned(
                // TODO: raw,
                request_kw,
                "Can't have both a query map field and regular query fields",
            ));
        }

        // TODO when/if `&[(&str, &str)]` is supported remove this
        if query_map_field.is_some() && !lifetimes.query.is_empty() {
            return Err(syn::Error::new_spanned(
                request_kw,
                "Lifetimes are not allowed for query_map fields",
            ));
        }

        Ok(Self { attributes, fields, lifetimes, ruma_api_import: util::import_ruma_api() })
    }
}

impl Parse for Response {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let attributes = input.call(Attribute::parse_outer)?;
        let response_kw = input.parse::<kw::response>()?;
        input.parse::<Token![:]>()?;
        let fields;
        braced!(fields in input);

        let fields = fields
            .parse_terminated::<Field, Token![,]>(Field::parse_named)?
            .into_iter()
            .map(|f| {
                if has_lifetime(&f.ty) {
                    Err(syn::Error::new(
                        f.ident.span(),
                        "Lifetimes on Response fields cannot be supported until GAT are stable",
                    ))
                } else {
                    Ok(f)
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mut newtype_body_field = None;

        let fields = fields
            .into_iter()
            .map(|mut field| {
                let mut field_kind = None;
                let mut header = None;

                for attr in mem::take(&mut field.attrs) {
                    let meta = match Meta::from_attribute(&attr)? {
                        Some(m) => m,
                        None => {
                            field.attrs.push(attr);
                            continue;
                        }
                    };

                    if field_kind.is_some() {
                        return Err(syn::Error::new_spanned(
                            attr,
                            "There can only be one field kind attribute",
                        ));
                    }

                    field_kind = Some(match meta {
                        Meta::Word(ident) => match &ident.to_string()[..] {
                            s @ "body" | s @ "raw_body" => req_res_meta_word(
                                s,
                                &field,
                                &mut newtype_body_field,
                                ResponseFieldKind::NewtypeBody,
                                ResponseFieldKind::NewtypeRawBody,
                            )?,
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    ident,
                                    "Invalid #[ruma_api] argument with value, expected `body`",
                                ));
                            }
                        },
                        Meta::NameValue(MetaNameValue { name, value }) => {
                            req_res_name_value(name, value, &mut header, ResponseFieldKind::Header)?
                        }
                    });
                }

                Ok(match field_kind.unwrap_or(ResponseFieldKind::Body) {
                    ResponseFieldKind::Body => ResponseField::Body(field),
                    ResponseFieldKind::Header => {
                        ResponseField::Header(field, header.expect("missing header name"))
                    }
                    ResponseFieldKind::NewtypeBody => ResponseField::NewtypeBody(field),
                    ResponseFieldKind::NewtypeRawBody => ResponseField::NewtypeRawBody(field),
                })
            })
            .collect::<syn::Result<Vec<_>>>()?;

        if newtype_body_field.is_some() && fields.iter().any(|f| f.is_body()) {
            // TODO: highlight conflicting fields,
            return Err(syn::Error::new_spanned(
                response_kw,
                "Can't have both a newtype body field and regular body fields",
            ));
        }

        Ok(Self { attributes, fields, ruma_api_import: util::import_ruma_api() })
    }
}

fn has_lifetime(ty: &Type) -> bool {
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
        Type::BareFn(TypeBareFn { lifetimes: Some(BoundLifetimes { .. }), .. }) => true,
        _ => false,
    }
}

fn collect_lifetime_ident(lifetimes: &mut BTreeSet<Lifetime>, ty: &Type) {
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
            lifetimes: Some(BoundLifetimes { lifetimes: fn_lifetimes, .. }),
            ..
        }) => {
            for lt in fn_lifetimes {
                let LifetimeDef { lifetime, .. } = lt;
                lifetimes.insert(lifetime.clone());
            }
        }
        _ => {}
    }
}

fn req_res_meta_word<T>(
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

fn req_res_name_value<T>(
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
