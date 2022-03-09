//! Implementation of the top level `*Event` derive macro.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse_quote, Data, DataStruct, DeriveInput, Field, Fields, FieldsNamed, GenericParam, Meta,
    MetaList, NestedMeta,
};

use super::{
    event_parse::{to_kind_variation, EventKind, EventKindVariation},
    util::is_non_stripped_room_event,
};
use crate::{import_ruma_common, util::to_camel_case};

/// Derive `Event` macro code generation.
pub fn expand_event(input: DeriveInput) -> syn::Result<TokenStream> {
    let ruma_common = import_ruma_common();

    let ident = &input.ident;
    let (kind, var) = to_kind_variation(ident).ok_or_else(|| {
        syn::Error::new_spanned(ident, "not a valid ruma event struct identifier")
    })?;

    let fields: Vec<_> = if let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed { named, .. }),
        ..
    }) = &input.data
    {
        if !named.iter().any(|f| f.ident.as_ref().unwrap() == "content") {
            return Err(syn::Error::new(
                Span::call_site(),
                "struct must contain a `content` field",
            ));
        }

        named.iter().cloned().collect()
    } else {
        return Err(syn::Error::new_spanned(
            input.ident,
            "the `Event` derive only supports structs with named fields",
        ));
    };

    let mut res = TokenStream::new();

    res.extend(expand_serialize_event(&input, var, &fields, &ruma_common));
    res.extend(expand_deserialize_event(&input, kind, var, &fields, &ruma_common)?);

    if var.is_sync() {
        res.extend(expand_sync_from_into_full(&input, kind, var, &fields, &ruma_common));
    }

    if matches!(kind, EventKind::MessageLike | EventKind::State)
        && matches!(var, EventKindVariation::Full | EventKindVariation::Sync)
    {
        res.extend(expand_redact_event(&input, kind, var, &fields, &ruma_common));
    }

    if is_non_stripped_room_event(kind, var) {
        res.extend(expand_eq_ord_event(&input));
    }

    Ok(res)
}

fn expand_serialize_event(
    input: &DeriveInput,
    var: EventKindVariation,
    fields: &[Field],
    ruma_common: &TokenStream,
) -> TokenStream {
    let serde = quote! { #ruma_common::exports::serde };

    let ident = &input.ident;
    let (impl_gen, ty_gen, where_clause) = input.generics.split_for_impl();
    let serialize_fields: Vec<_> = fields
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().unwrap();
            if name == "content" && var.is_redacted() {
                quote! {
                    if #ruma_common::events::RedactedEventContent::has_serialize_fields(&self.content) {
                        state.serialize_field("content", &self.content)?;
                    }
                }
            } else if name == "unsigned" {
                quote! {
                    if !self.unsigned.is_empty() {
                        state.serialize_field("unsigned", &self.unsigned)?;
                    }
                }
            } else {
                let name_s = name.to_string();
                match &field.ty {
                    syn::Type::Path(syn::TypePath { path: syn::Path { segments, .. }, .. })
                        if segments.last().unwrap().ident == "Option" =>
                    {
                        quote! {
                            if let Some(content) = self.#name.as_ref() {
                                state.serialize_field(#name_s, content)?;
                            }
                        }
                    }
                    _ => quote! {
                        state.serialize_field(#name_s, &self.#name)?;
                    },
                }
            }
        })
        .collect();

    quote! {
        #[automatically_derived]
        impl #impl_gen #serde::ser::Serialize for #ident #ty_gen #where_clause {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: #serde::ser::Serializer,
            {
                use #serde::ser::{SerializeStruct as _, Error as _};

                let event_type = #ruma_common::events::EventContent::event_type(&self.content);

                let mut state = serializer.serialize_struct(stringify!(#ident), 7)?;

                state.serialize_field("type", event_type)?;
                #( #serialize_fields )*
                state.end()
            }
        }
    }
}

fn expand_deserialize_event(
    input: &DeriveInput,
    _kind: EventKind,
    var: EventKindVariation,
    fields: &[Field],
    ruma_common: &TokenStream,
) -> syn::Result<TokenStream> {
    let serde = quote! { #ruma_common::exports::serde };
    let serde_json = quote! { #ruma_common::exports::serde_json };

    let ident = &input.ident;
    // we know there is a content field already
    let content_type = &fields
        .iter()
        // we also know that the fields are named and have an ident
        .find(|f| f.ident.as_ref().unwrap() == "content")
        .unwrap()
        .ty;

    let (impl_generics, ty_gen, where_clause) = input.generics.split_for_impl();
    let is_generic = !input.generics.params.is_empty();

    let enum_variants: Vec<_> = fields
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().unwrap();
            to_camel_case(name)
        })
        .collect();

    let deserialize_var_types: Vec<_> = fields
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().unwrap();
            let ty = &field.ty;
            if name == "content" || name == "prev_content" {
                if is_generic {
                    quote! { ::std::boxed::Box<#serde_json::value::RawValue> }
                } else {
                    quote! { #content_type }
                }
            } else {
                #[allow(unused_mut)]
                let mut ty = quote! { #ty };

                #[cfg(feature = "compat")]
                if matches!(_kind, EventKind::State) && name == "unsigned" {
                    match var {
                        EventKindVariation::Full | EventKindVariation::Sync => {
                            ty = quote! { #ruma_common::events::UnsignedWithPrevContent };
                        }
                        EventKindVariation::Redacted | EventKindVariation::RedactedSync => {
                            ty = quote! { #ruma_common::events::RedactedUnsignedWithPrevContent };
                        }
                        EventKindVariation::Stripped | EventKindVariation::Initial => {
                            unreachable!()
                        }
                    }
                }

                ty
            }
        })
        .collect();

    let ok_or_else_fields: Vec<_> = fields
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().unwrap();
            Ok(if name == "content" {
                if is_generic && var.is_redacted() {
                    quote! {
                        let content = match C::has_deserialize_fields() {
                            #ruma_common::events::HasDeserializeFields::False => {
                                C::empty(&event_type).map_err(A::Error::custom)?
                            },
                            #ruma_common::events::HasDeserializeFields::True => {
                                let json = content.ok_or_else(
                                    || #serde::de::Error::missing_field("content"),
                                )?;
                                C::from_parts(&event_type, &json).map_err(A::Error::custom)?
                            },
                            #ruma_common::events::HasDeserializeFields::Optional => {
                                let json = content.unwrap_or(
                                    #serde_json::value::RawValue::from_string("{}".to_owned())
                                        .unwrap()
                                );
                                C::from_parts(&event_type, &json).map_err(A::Error::custom)?
                            },
                        };
                    }
                } else if is_generic {
                    quote! {
                        let content = {
                            let json = content
                                .ok_or_else(|| #serde::de::Error::missing_field("content"))?;
                            C::from_parts(&event_type, &json).map_err(A::Error::custom)?
                        };
                    }
                } else {
                    quote! {
                        let content = content.ok_or_else(
                            || #serde::de::Error::missing_field("content"),
                        )?;
                    }
                }
            } else if name == "prev_content" {
                if is_generic {
                    #[allow(unused_mut)]
                    let mut res = quote! {
                        let prev_content = prev_content.map(|json| {
                            C::from_parts(&event_type, &json).map_err(A::Error::custom)
                        }).transpose()?;
                    };

                    #[cfg(feature = "compat")]
                    if let EventKind::State = _kind {
                        res = quote! {
                            let prev_content = prev_content
                                .or_else(|| unsigned.as_mut().and_then(|u| u.prev_content.take()));
                            #res
                        };
                    };

                    res
                } else {
                    TokenStream::new()
                }
            } else if name == "unsigned" {
                #[allow(unused_mut)]
                let mut res = quote! {
                    let unsigned = unsigned.unwrap_or_default();
                };

                #[cfg(feature = "compat")]
                if matches!(_kind, EventKind::State) {
                    res = quote! {
                        let unsigned = unsigned.map_or_else(
                            ::std::default::Default::default,
                            ::std::convert::From::from,
                        );
                    };
                }

                res
            } else {
                let attrs: Vec<_> = field
                    .attrs
                    .iter()
                    .filter(|a| a.path.is_ident("ruma_event"))
                    .map(|a| a.parse_meta())
                    .collect::<syn::Result<_>>()?;

                let has_default_attr = attrs.iter().any(|a| {
                    matches!(
                        a,
                        Meta::List(MetaList { nested, .. })
                        if nested.iter().any(|n| {
                            matches!(n, NestedMeta::Meta(Meta::Path(p)) if p.is_ident("default"))
                        })
                    )
                });

                if has_default_attr {
                    quote! {
                        let #name = #name.unwrap_or_default();
                    }
                } else {
                    quote! {
                        let #name = #name.ok_or_else(|| {
                            #serde::de::Error::missing_field(stringify!(#name))
                        })?;
                    }
                }
            })
        })
        .collect::<syn::Result<_>>()?;

    let field_names: Vec<_> = fields.iter().flat_map(|f| &f.ident).collect();

    let deserialize_impl_gen = if is_generic {
        let gen = &input.generics.params;
        quote! { <'de, #gen> }
    } else {
        quote! { <'de> }
    };
    let deserialize_phantom_type = if is_generic {
        quote! { ::std::marker::PhantomData }
    } else {
        quote! {}
    };

    Ok(quote! {
        #[automatically_derived]
        impl #deserialize_impl_gen #serde::de::Deserialize<'de> for #ident #ty_gen #where_clause {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: #serde::de::Deserializer<'de>,
            {
                #[derive(#serde::Deserialize)]
                #[serde(field_identifier, rename_all = "snake_case")]
                enum Field {
                    // since this is represented as an enum we have to add it so the JSON picks it
                    // up
                    Type,
                    #( #enum_variants, )*
                    #[serde(other)]
                    Unknown,
                }

                /// Visits the fields of an event struct to handle deserialization of
                /// the `content` and `prev_content` fields.
                struct EventVisitor #impl_generics (#deserialize_phantom_type #ty_gen);

                #[automatically_derived]
                impl #deserialize_impl_gen #serde::de::Visitor<'de>
                    for EventVisitor #ty_gen #where_clause
                {
                    type Value = #ident #ty_gen;

                    fn expecting(
                        &self,
                        formatter: &mut ::std::fmt::Formatter<'_>,
                    ) -> ::std::fmt::Result {
                        write!(formatter, "struct implementing {}", stringify!(#content_type))
                    }

                    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                    where
                        A: #serde::de::MapAccess<'de>,
                    {
                        use #serde::de::Error as _;

                        let mut event_type: Option<String> = None;
                        #( let mut #field_names: Option<#deserialize_var_types> = None; )*

                        while let Some(key) = map.next_key()? {
                            match key {
                                Field::Unknown => {
                                    let _: #serde::de::IgnoredAny = map.next_value()?;
                                },
                                Field::Type => {
                                    if event_type.is_some() {
                                        return Err(#serde::de::Error::duplicate_field("type"));
                                    }
                                    event_type = Some(map.next_value()?);
                                }
                                #(
                                    Field::#enum_variants => {
                                        if #field_names.is_some() {
                                            return Err(#serde::de::Error::duplicate_field(
                                                stringify!(#field_names),
                                            ));
                                        }
                                        #field_names = Some(map.next_value()?);
                                    }
                                )*
                            }
                        }

                        let event_type =
                            event_type.ok_or_else(|| #serde::de::Error::missing_field("type"))?;
                        #( #ok_or_else_fields )*

                        Ok(#ident {
                            #( #field_names ),*
                        })
                    }
                }

                deserializer.deserialize_map(EventVisitor(#deserialize_phantom_type))
            }
        }
    })
}

fn expand_redact_event(
    input: &DeriveInput,
    kind: EventKind,
    var: EventKindVariation,
    fields: &[Field],
    ruma_common: &TokenStream,
) -> TokenStream {
    let redacted_type = kind.to_event_ident(var.to_redacted());
    let redacted_content_trait =
        format_ident!("{}Content", kind.to_event_ident(EventKindVariation::Redacted));
    let ident = &input.ident;

    let mut generics = input.generics.clone();
    if generics.params.is_empty() {
        return TokenStream::new();
    }

    assert_eq!(generics.params.len(), 1, "expected one generic parameter");
    let ty_param = match &generics.params[0] {
        GenericParam::Type(ty) => ty.ident.clone(),
        _ => panic!("expected a type parameter"),
    };

    let where_clause = generics.make_where_clause();
    where_clause.predicates.push(parse_quote! { #ty_param: #ruma_common::events::RedactContent });
    where_clause.predicates.push(parse_quote! {
        <#ty_param as #ruma_common::events::RedactContent>::Redacted:
            #ruma_common::events::#redacted_content_trait
    });

    let (impl_generics, ty_gen, where_clause) = generics.split_for_impl();

    let fields = fields.iter().filter_map(|field| {
        let ident = field.ident.as_ref().unwrap();

        if ident == "content" || ident == "prev_content" {
            None
        } else if ident == "unsigned" {
            Some(quote! {
                unsigned: #ruma_common::events::RedactedUnsigned::new_because(
                    ::std::boxed::Box::new(redaction),
                )
            })
        } else {
            Some(quote! {
                #ident: self.#ident
            })
        }
    });

    quote! {
        #[automatically_derived]
        impl #impl_generics #ruma_common::events::Redact for #ident #ty_gen #where_clause {
            type Redacted = #ruma_common::events::#redacted_type<
                <#ty_param as #ruma_common::events::RedactContent>::Redacted,
            >;

            fn redact(
                self,
                redaction: #ruma_common::events::room::redaction::SyncRoomRedactionEvent,
                version: &#ruma_common::RoomVersionId,
            ) -> Self::Redacted {
                let content = #ruma_common::events::RedactContent::redact(self.content, version);
                #ruma_common::events::#redacted_type {
                    content,
                    #(#fields),*
                }
            }
        }
    }
}

fn expand_sync_from_into_full(
    input: &DeriveInput,
    kind: EventKind,
    var: EventKindVariation,
    fields: &[Field],
    ruma_common: &TokenStream,
) -> TokenStream {
    let ident = &input.ident;
    let full_struct = kind.to_event_ident(var.to_full());
    let (impl_generics, ty_gen, where_clause) = input.generics.split_for_impl();
    let fields: Vec<_> = fields.iter().flat_map(|f| &f.ident).collect();

    quote! {
        #[automatically_derived]
        impl #impl_generics ::std::convert::From<#full_struct #ty_gen>
            for #ident #ty_gen #where_clause
        {
            fn from(event: #full_struct #ty_gen) -> Self {
                let #full_struct { #( #fields, )* .. } = event;
                Self { #( #fields, )* }
            }
        }

        #[automatically_derived]
        impl #impl_generics #ident #ty_gen #where_clause {
            /// Convert this sync event into a full event, one with a room_id field.
            pub fn into_full_event(
                self,
                room_id: ::std::boxed::Box<#ruma_common::RoomId>,
            ) -> #full_struct #ty_gen {
                let Self { #( #fields, )* } = self;
                #full_struct {
                    #( #fields, )*
                    room_id,
                }
            }
        }
    }
}

fn expand_eq_ord_event(input: &DeriveInput) -> TokenStream {
    let ident = &input.ident;
    let (impl_gen, ty_gen, where_clause) = input.generics.split_for_impl();

    quote! {
        #[automatically_derived]
        impl #impl_gen ::std::cmp::PartialEq for #ident #ty_gen #where_clause {
            /// Checks if two `EventId`s are equal.
            fn eq(&self, other: &Self) -> ::std::primitive::bool {
                self.event_id == other.event_id
            }
        }

        #[automatically_derived]
        impl #impl_gen ::std::cmp::Eq for #ident #ty_gen #where_clause {}

        #[automatically_derived]
        impl #impl_gen ::std::cmp::PartialOrd for #ident #ty_gen #where_clause {
            /// Compares `EventId`s and orders them lexicographically.
            fn partial_cmp(&self, other: &Self) -> ::std::option::Option<::std::cmp::Ordering> {
                self.event_id.partial_cmp(&other.event_id)
            }
        }

        #[automatically_derived]
        impl #impl_gen ::std::cmp::Ord for #ident #ty_gen #where_clause {
            /// Compares `EventId`s and orders them lexicographically.
            fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                self.event_id.cmp(&other.event_id)
            }
        }
    }
}
