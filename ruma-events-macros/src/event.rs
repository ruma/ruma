//! Implementation of the top level `*Event` derive macro.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Field, Fields, FieldsNamed, Ident};

use crate::{
    event_parse::{to_kind_variation, EventKind, EventKindVariation},
    import_ruma_events,
};

/// Derive `Event` macro code generation.
pub fn expand_event(input: DeriveInput) -> syn::Result<TokenStream> {
    let ruma_events = import_ruma_events();

    let ident = &input.ident;
    let (kind, var) = to_kind_variation(ident).ok_or_else(|| {
        syn::Error::new(Span::call_site(), "not a valid ruma event struct identifier")
    })?;

    let fields = if let Data::Struct(DataStruct { fields, .. }) = input.data.clone() {
        if let Fields::Named(FieldsNamed { named, .. }) = fields {
            if !named.iter().any(|f| f.ident.as_ref().unwrap() == "content") {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "struct must contain a `content` field",
                ));
            }

            named.into_iter().collect::<Vec<_>>()
        } else {
            return Err(syn::Error::new_spanned(
                fields,
                "the `Event` derive only supports named fields",
            ));
        }
    } else {
        return Err(syn::Error::new_spanned(
            input.ident,
            "the `Event` derive only supports structs with named fields",
        ));
    };

    let serialize_impl = expand_serialize_event(&input, &var, &fields, &ruma_events);
    let deserialize_impl = expand_deserialize_event(&input, &var, &fields, &ruma_events);
    let conversion_impl = expand_from_into(&input, &kind, &var, &fields, &ruma_events);

    let eq_impl = expand_eq_ord_event(&input, &fields);

    Ok(quote! {
        #conversion_impl
        #serialize_impl
        #deserialize_impl
        #eq_impl
    })
}

fn expand_serialize_event(
    input: &DeriveInput,
    var: &EventKindVariation,
    fields: &[Field],
    ruma_events: &TokenStream,
) -> TokenStream {
    let js_int = quote! { #ruma_events::exports::js_int };
    let serde = quote! { #ruma_events::exports::serde };

    let ident = &input.ident;
    let (impl_gen, ty_gen, where_clause) = input.generics.split_for_impl();
    let serialize_fields = fields
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().unwrap();
            if name == "content" && var.is_redacted() {
                quote! {
                    if #ruma_events::RedactedEventContent::has_serialize_fields(&self.content) {
                        state.serialize_field("content", &self.content)?;
                    }
                }
            } else if name == "prev_content" {
                quote! {
                    if let Some(content) = self.prev_content.as_ref() {
                        state.serialize_field("prev_content", content)?;
                    }
                }
            } else if name == "origin_server_ts" {
                quote! {
                    let time_since_epoch =
                        self.origin_server_ts.duration_since(::std::time::UNIX_EPOCH).unwrap();

                    let timestamp = <#js_int::UInt as ::std::convert::TryFrom<_>>::try_from(
                        time_since_epoch.as_millis(),
                    ).map_err(S::Error::custom)?;

                    state.serialize_field("origin_server_ts", &timestamp)?;
                }
            } else if name == "unsigned" {
                quote! {
                    if !self.unsigned.is_empty() {
                        state.serialize_field("unsigned", &self.unsigned)?;
                    }
                }
            } else {
                quote! {
                    state.serialize_field(stringify!(#name), &self.#name)?;
                }
            }
        })
        .collect::<Vec<_>>();

    quote! {
        #[automatically_derived]
        impl #impl_gen #serde::ser::Serialize for #ident #ty_gen #where_clause {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: #serde::ser::Serializer,
            {
                use #serde::ser::{SerializeStruct as _, Error as _};

                let event_type = #ruma_events::EventContent::event_type(&self.content);

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
    var: &EventKindVariation,
    fields: &[Field],
    ruma_events: &TokenStream,
) -> TokenStream {
    let js_int = quote! { #ruma_events::exports::js_int };
    let serde = quote! { #ruma_events::exports::serde };
    let serde_json = quote! { #ruma_events::exports::serde_json };

    let ident = &input.ident;
    // we know there is a content field already
    let content_type = fields
        .iter()
        // we also know that the fields are named and have an ident
        .find(|f| f.ident.as_ref().unwrap() == "content")
        .map(|f| f.ty.clone())
        .unwrap();

    let (impl_generics, ty_gen, where_clause) = input.generics.split_for_impl();
    let is_generic = !input.generics.params.is_empty();

    let enum_variants = fields
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().unwrap();
            to_camel_case(name)
        })
        .collect::<Vec<_>>();

    let deserialize_var_types = fields
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().unwrap();
            let ty = &field.ty;
            if name == "content" || name == "prev_content" {
                if is_generic {
                    quote! { Box<#serde_json::value::RawValue> }
                } else {
                    quote! { #content_type }
                }
            } else if name == "origin_server_ts" {
                quote! { #js_int::UInt }
            } else {
                quote! { #ty }
            }
        })
        .collect::<Vec<_>>();

    let ok_or_else_fields = fields
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().unwrap();
            if name == "content" {
                if is_generic && var.is_redacted() {
                    quote! {
                        let content = match C::has_deserialize_fields() {
                            #ruma_events::HasDeserializeFields::False => {
                                C::empty(&event_type).map_err(A::Error::custom)?
                            },
                            #ruma_events::HasDeserializeFields::True => {
                                let json = content.ok_or_else(
                                    || #serde::de::Error::missing_field("content"),
                                )?;
                                C::from_parts(&event_type, json).map_err(A::Error::custom)?
                            },
                            #ruma_events::HasDeserializeFields::Optional => {
                                let json = content.unwrap_or(
                                    #serde_json::value::RawValue::from_string("{}".to_string())
                                        .unwrap()
                                );
                                C::from_parts(&event_type, json).map_err(A::Error::custom)?
                            },
                        };
                    }
                } else if is_generic {
                    quote! {
                        let json =
                            content.ok_or_else(|| #serde::de::Error::missing_field("content"))?;
                        let content = C::from_parts(&event_type, json).map_err(A::Error::custom)?;
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
                    quote! {
                        let prev_content = if let Some(json) = prev_content {
                            Some(C::from_parts(&event_type, json).map_err(A::Error::custom)?)
                        } else {
                            None
                        };
                    }
                } else {
                    quote! {
                        let prev_content = if let Some(content) = prev_content {
                            Some(content)
                        } else {
                            None
                        };
                    }
                }
            } else if name == "origin_server_ts" {
                quote! {
                    let origin_server_ts = origin_server_ts
                        .map(|time| {
                            let t = time.into();
                            ::std::time::UNIX_EPOCH + ::std::time::Duration::from_millis(t)
                        })
                        .ok_or_else(|| #serde::de::Error::missing_field("origin_server_ts"))?;
                }
            } else if name == "unsigned" {
                quote! { let unsigned = unsigned.unwrap_or_default(); }
            } else {
                quote! {
                    let #name = #name.ok_or_else(|| {
                        #serde::de::Error::missing_field(stringify!(#name))
                    })?;
                }
            }
        })
        .collect::<Vec<_>>();

    let field_names = fields.iter().flat_map(|f| &f.ident).collect::<Vec<_>>();

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

    quote! {
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
    }
}

fn expand_from_into(
    input: &DeriveInput,
    kind: &EventKind,
    var: &EventKindVariation,
    fields: &[Field],
    ruma_events: &TokenStream,
) -> Option<TokenStream> {
    let ruma_identifiers = quote! { #ruma_events::exports::ruma_identifiers };

    let ident = &input.ident;

    let (impl_generics, ty_gen, where_clause) = input.generics.split_for_impl();

    let fields = fields.iter().flat_map(|f| &f.ident).collect::<Vec<_>>();

    let fields_without_unsigned =
        fields.iter().filter(|id| id.to_string().as_str() != "unsigned").collect::<Vec<_>>();

    let (into, into_full_event) = if var.is_redacted() {
        (quote! { unsigned: unsigned.into(), }, quote! { unsigned: unsigned.into_full(room_id), })
    } else if kind == &EventKind::Ephemeral {
        (TokenStream::new(), TokenStream::new())
    } else {
        (quote! { unsigned, }, quote! { unsigned, })
    };

    if let EventKindVariation::Sync | EventKindVariation::RedactedSync = var {
        let full_struct = kind.to_event_ident(&var.to_full_variation());
        Some(quote! {
            #[automatically_derived]
            impl #impl_generics From<#full_struct #ty_gen> for #ident #ty_gen #where_clause {
                fn from(event: #full_struct #ty_gen) -> Self {
                    let #full_struct {
                        #( #fields, )* ..
                    } = event;
                    Self { #( #fields_without_unsigned, )* #into }
                }
            }

            #[automatically_derived]
            impl #impl_generics #ident #ty_gen #where_clause {
                /// Convert this sync event into a full event, one with a room_id field.
                pub fn into_full_event(
                    self,
                    room_id: #ruma_identifiers::RoomId,
                ) -> #full_struct #ty_gen {
                    let Self { #( #fields, )* } = self;
                    #full_struct {
                        #( #fields_without_unsigned, )*
                        room_id: room_id.clone(),
                        #into_full_event
                    }
                }
            }
        })
    } else {
        None
    }
}

fn expand_eq_ord_event(input: &DeriveInput, fields: &[Field]) -> Option<TokenStream> {
    if fields.iter().flat_map(|f| f.ident.as_ref()).any(|f| f == "event_id") {
        let ident = &input.ident;
        let (impl_gen, ty_gen, where_clause) = input.generics.split_for_impl();

        Some(quote! {
            #[automatically_derived]
            impl #impl_gen ::std::cmp::PartialEq for #ident #ty_gen #where_clause {
                /// This checks if two `EventId`s are equal.
                fn eq(&self, other: &Self) -> ::std::primitive::bool {
                    self.event_id == other.event_id
                }
            }

            #[automatically_derived]
            impl #impl_gen ::std::cmp::Eq for #ident #ty_gen #where_clause {}

            #[automatically_derived]
            impl #impl_gen ::std::cmp::PartialOrd for #ident #ty_gen #where_clause {
                /// This compares `EventId`s and orders them lexicographically.
                fn partial_cmp(&self, other: &Self) -> ::std::option::Option<::std::cmp::Ordering> {
                    self.event_id.partial_cmp(&other.event_id)
                }
            }

            #[automatically_derived]
            impl #impl_gen ::std::cmp::Ord for #ident #ty_gen #where_clause {
                /// This compares `EventId`s and orders them lexicographically.
                fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                    self.event_id.cmp(&other.event_id)
                }
            }
        })
    } else {
        None
    }
}

/// CamelCase's a field ident like "foo_bar" to "FooBar".
fn to_camel_case(name: &Ident) -> Ident {
    let span = name.span();
    let name = name.to_string();

    let s = name
        .split('_')
        .map(|s| s.chars().next().unwrap().to_uppercase().to_string() + &s[1..])
        .collect::<String>();
    Ident::new(&s, span)
}
