//! Implementation of the `Event` derive macro.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Fields, FieldsNamed, parse_quote};

mod parse;

use self::parse::{ParsedEventField, parse_event_struct_ident_to_kind_variation};
use super::enums::{CommonEventField, EventKind, EventVariation};
use crate::util::{RumaEvents, RumaEventsReexport, to_camel_case};

/// `Event` derive macro code generation.
pub fn expand_event(input: DeriveInput) -> syn::Result<TokenStream> {
    let ruma_events = RumaEvents::new();

    let ident = &input.ident;
    let (kind, var) = parse_event_struct_ident_to_kind_variation(ident).ok_or_else(|| {
        syn::Error::new_spanned(ident, "not a supported ruma event struct identifier")
    })?;

    let fields = if let Data::Struct(DataStruct {
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

        named.iter().cloned().map(ParsedEventField::parse).collect::<Result<Vec<_>, _>>()?
    } else {
        return Err(syn::Error::new_spanned(
            input.ident,
            "the `Event` derive only supports structs with named fields",
        ));
    };

    let mut res = TokenStream::new();

    res.extend(
        expand_deserialize_event(&input, var, &fields, &ruma_events)
            .unwrap_or_else(syn::Error::into_compile_error),
    );

    if var.is_sync() {
        res.extend(expand_sync_from_into_full(&input, kind, var, &fields, &ruma_events));
    }

    if CommonEventField::EventId.is_present(kind, var) {
        res.extend(expand_eq_ord_event(&input));
    }

    Ok(res)
}

/// Implement `Deserialize` for the event struct.
fn expand_deserialize_event(
    input: &DeriveInput,
    var: EventVariation,
    fields: &[ParsedEventField],
    ruma_events: &RumaEvents,
) -> syn::Result<TokenStream> {
    let serde = ruma_events.reexported(RumaEventsReexport::Serde);
    let serde_json = ruma_events.reexported(RumaEventsReexport::SerdeJson);

    let ident = &input.ident;
    // we know there is a content field already
    let content_type = &fields
        .iter()
        // we also know that the fields are named and have an ident
        .find(|f| f.name() == "content")
        .unwrap()
        .ty();

    let (impl_generics, ty_gen, where_clause) = input.generics.split_for_impl();
    let is_generic = !input.generics.params.is_empty();

    let enum_variants: Vec<_> = fields.iter().map(|field| to_camel_case(field.name())).collect();
    let enum_variants_serde_attributes = fields
        .iter()
        .map(|field| {
            let mut attrs = Vec::new();

            if let Some(rename) = &field.rename {
                attrs.push(quote! { rename = #rename });
            }

            attrs.extend(field.aliases.iter().map(|alias| {
                quote! { alias = #alias }
            }));

            (!attrs.is_empty()).then(|| {
                quote! { #[serde(#( #attrs, )*)] }
            })
        })
        .collect::<Vec<_>>();
    let serialized_field_names =
        fields.iter().map(ParsedEventField::serialized_name).collect::<Vec<_>>();

    let deserialize_var_types: Vec<_> = fields
        .iter()
        .map(|field| {
            let name = field.name();
            if name == "content" {
                if is_generic {
                    quote! { ::std::boxed::Box<#serde_json::value::RawValue> }
                } else {
                    quote! { #content_type }
                }
            } else if name == "state_key" && var == EventVariation::Initial {
                quote! { ::std::string::String }
            } else {
                let ty = field.ty();
                quote! { #ty }
            }
        })
        .collect();

    let ok_or_else_fields: Vec<_> = fields
        .iter()
        .zip(&serialized_field_names)
        .map(|(field, serialized_name)| {
            let name = field.name();

            Ok(if name == "content" && is_generic {
                quote! {
                    let content = {
                        let json = content
                            .ok_or_else(|| #serde::de::Error::missing_field("content"))?;
                        C::from_parts(&event_type, &json).map_err(#serde::de::Error::custom)?
                    };
                }
            } else if field.default || (name == "unsigned" && !var.is_redacted()) {
                quote! {
                    let #name = #name.unwrap_or_default();
                }
            } else if name == "state_key" && var == EventVariation::Initial {
                let ty = field.ty();
                quote! {
                    let state_key: ::std::string::String = state_key.unwrap_or_default();
                    let state_key: #ty = <#ty as #serde::de::Deserialize>::deserialize(
                        #serde::de::IntoDeserializer::<A::Error>::into_deserializer(state_key),
                    )?;
                }
            } else {
                quote! {
                    let #name = #name.ok_or_else(|| {
                        #serde::de::Error::missing_field(#serialized_name)
                    })?;
                }
            })
        })
        .collect::<syn::Result<_>>()?;

    let field_names: Vec<_> = fields.iter().map(ParsedEventField::name).collect();
    let field_error_handlers = fields
        .iter()
        .map(|field| {
            if field.default_on_error {
                quote! {
                    .map_err(|error| {
                        tracing::debug!("deserialization error, using default value: {error}");
                    })
                    .unwrap_or_default()
                }
            } else {
                quote! { ? }
            }
        })
        .collect::<Vec<_>>();

    let deserialize_impl_gen = if is_generic {
        let generic_params = &input.generics.params;
        quote! { <'de, #generic_params> }
    } else {
        quote! { <'de> }
    };
    let deserialize_phantom_type = if is_generic {
        quote! { ::std::marker::PhantomData }
    } else {
        quote! {}
    };
    let where_clause = if is_generic {
        let predicate = parse_quote! { C: #ruma_events::EventContentFromType };
        if let Some(mut where_clause) = where_clause.cloned() {
            where_clause.predicates.push(predicate);
            Some(where_clause)
        } else {
            Some(parse_quote! { where #predicate })
        }
    } else {
        where_clause.cloned()
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
                    #( #enum_variants_serde_attributes #enum_variants, )*
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
                                                #serialized_field_names,
                                            ));
                                        }
                                        #field_names = Some(map.next_value() #field_error_handlers);
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

/// Implement `From<{full}>` and `.into_full_event()` for a sync event struct.
fn expand_sync_from_into_full(
    input: &DeriveInput,
    kind: EventKind,
    var: EventVariation,
    fields: &[ParsedEventField],
    ruma_events: &RumaEvents,
) -> syn::Result<TokenStream> {
    let ruma_common = ruma_events.ruma_common();

    let ident = &input.ident;
    let full_struct = kind.to_event_ident(var.to_full())?;
    let (impl_generics, ty_gen, where_clause) = input.generics.split_for_impl();
    let fields: Vec<_> = fields.iter().map(ParsedEventField::name).collect();

    Ok(quote! {
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
                room_id: #ruma_common::OwnedRoomId,
            ) -> #full_struct #ty_gen {
                let Self { #( #fields, )* } = self;
                #full_struct {
                    #( #fields, )*
                    room_id,
                }
            }
        }
    })
}

/// Implement `PartialEq`, `Eq`, `PartialOrd`, `Ord` for the event struct by comparing the
/// `event_id`.
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
