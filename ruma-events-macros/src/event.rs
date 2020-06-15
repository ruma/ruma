//! Implementation of the top level `*Event` derive macro.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Field, Fields, FieldsNamed, Ident};

/// Derive `Event` macro code generation.
pub fn expand_event(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let (impl_gen, ty_gen, where_clause) = input.generics.split_for_impl();
    let is_generic = !input.generics.params.is_empty();

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

    let serialize_fields = fields
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().unwrap();
            if name == "prev_content" {
                quote! {
                    if let Some(content) = self.prev_content.as_ref() {
                        state.serialize_field("prev_content", content)?;
                    }
                }
            } else if name == "origin_server_ts" {
                quote! {
                    let time_since_epoch =
                        self.origin_server_ts.duration_since(::std::time::UNIX_EPOCH).unwrap();

                    let timestamp = <::js_int::UInt as ::std::convert::TryFrom<_>>::try_from(time_since_epoch.as_millis())
                        .map_err(S::Error::custom)?;

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

    let serialize_impl = quote! {
        impl #impl_gen ::serde::ser::Serialize for #ident #ty_gen #where_clause {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::ser::Serializer,
            {
                use ::serde::ser::{SerializeStruct as _, Error as _};

                let event_type = ::ruma_events::EventContent::event_type(&self.content);

                let mut state = serializer.serialize_struct(stringify!(#ident), 7)?;

                state.serialize_field("type", event_type)?;
                #( #serialize_fields )*
                state.end()
            }
        }
    };

    let deserialize_impl = expand_deserialize_event(is_generic, input, fields)?;

    Ok(quote! {
        #serialize_impl

        #deserialize_impl
    })
}

fn expand_deserialize_event(
    is_generic: bool,
    input: DeriveInput,
    fields: Vec<Field>,
) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    // we know there is a content field already
    let content_type = fields
        .iter()
        // we also know that the fields are named and have an ident
        .find(|f| f.ident.as_ref().unwrap() == "content")
        .map(|f| f.ty.clone())
        .unwrap();

    let (impl_generics, ty_gen, where_clause) = input.generics.split_for_impl();

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
                    quote! { Box<::serde_json::value::RawValue> }
                } else {
                    quote! { #content_type }
                }
            } else if name == "origin_server_ts" {
                quote! { ::js_int::UInt }
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
            if is_generic {
                quote! {
                    let json = content.ok_or_else(|| ::serde::de::Error::missing_field("content"))?;
                    let content = C::from_parts(&event_type, json).map_err(A::Error::custom)?;
                }
            } else {
                quote! {
                    let content = content.ok_or_else(|| ::serde::de::Error::missing_field("content"))?;
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
                    .ok_or_else(|| ::serde::de::Error::missing_field("origin_server_ts"))?;
            }
        } else if name == "unsigned" {
            quote! { let unsigned = unsigned.unwrap_or_default(); }
        } else {
            quote! {
                let #name = #name.ok_or_else(|| {
                    ::serde::de::Error::missing_field(stringify!(#name))
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

    Ok(quote! {
        impl #deserialize_impl_gen ::serde::de::Deserialize<'de> for #ident #ty_gen #where_clause {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::de::Deserializer<'de>,
            {
                #[derive(::serde::Deserialize)]
                #[serde(field_identifier, rename_all = "snake_case")]
                enum Field {
                    // since this is represented as an enum we have to add it so the JSON picks it up
                    Type,
                    #( #enum_variants, )*
                    #[serde(other)]
                    Unknown,
                }

                /// Visits the fields of an event struct to handle deserialization of
                /// the `content` and `prev_content` fields.
                struct EventVisitor #impl_generics (#deserialize_phantom_type #ty_gen);

                impl #deserialize_impl_gen ::serde::de::Visitor<'de> for EventVisitor #ty_gen #where_clause {
                    type Value = #ident #ty_gen;

                    fn expecting(&self, formatter: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                        write!(formatter, "struct implementing {}", stringify!(#content_type))
                    }

                    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                    where
                        A: ::serde::de::MapAccess<'de>,
                    {
                        use ::serde::de::Error as _;

                        let mut event_type: Option<String> = None;
                        #( let mut #field_names: Option<#deserialize_var_types> = None; )*

                        while let Some(key) = map.next_key()? {
                            match key {
                                Field::Unknown => continue,
                                Field::Type => {
                                    if event_type.is_some() {
                                        return Err(::serde::de::Error::duplicate_field("type"));
                                    }
                                    event_type = Some(map.next_value()?);
                                }
                                #(
                                    Field::#enum_variants => {
                                        if #field_names.is_some() {
                                            return Err(::serde::de::Error::duplicate_field(stringify!(#field_names)));
                                        }
                                        #field_names = Some(map.next_value()?);
                                    }
                                )*
                            }
                        }

                        let event_type = event_type.ok_or_else(|| ::serde::de::Error::missing_field("type"))?;
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
