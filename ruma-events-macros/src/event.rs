//! Implementation of the top level `*Event` derive macro.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    Data, DataStruct, DeriveInput, Field, Fields, FieldsNamed, GenericParam, Ident, TypeParam,
};

/// Derive `Event` macro code generation.
pub fn expand_event(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;
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

    let content_trait = Ident::new(&format!("{}Content", ident), input.ident.span());
    let try_from_raw_fields = fields
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().unwrap();
            if name == "content" {
                quote! { content: C::try_from_raw(raw.content)? }
            } else if name == "prev_content" {
                quote! { prev_content: raw.prev_content.map(C::try_from_raw).transpose()? }
            } else {
                quote! { #name: raw.#name }
            }
        })
        .collect::<Vec<_>>();

    let try_from_raw_impl = quote! {
        impl<C> ::ruma_events::TryFromRaw for #ident<C>
        where
            C: ::ruma_events::#content_trait + ::ruma_events::TryFromRaw,
            C::Raw: ::ruma_events::RawEventContent,
        {
            type Raw = raw_event::#ident<C::Raw>;
            type Err = C::Err;

            fn try_from_raw(raw: Self::Raw) -> Result<Self, Self::Err> {
                Ok(Self {
                    #( #try_from_raw_fields ),*
                })
            }
        }
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

                    let timestamp = ::js_int::UInt::try_from(time_since_epoch.as_millis())
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
        impl<C: #content_trait> ::serde::ser::Serialize for #ident<C>
        where
            C::Raw: ::ruma_events::RawEventContent,
        {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::ser::Serializer,
            {
                use ::serde::ser::SerializeStruct as _;

                let event_type = self.content.event_type();

                let mut state = serializer.serialize_struct("StateEvent", 7)?;

                state.serialize_field("type", event_type)?;
                #( #serialize_fields )*
                state.end()
            }
        }
    };

    let raw_mod = expand_raw_state_event(&input, fields)?;

    Ok(quote! {
        #try_from_raw_impl

        #serialize_impl

        #raw_mod
    })
}

fn expand_raw_state_event(input: &DeriveInput, fields: Vec<Field>) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let content_ident = Ident::new(&format!("{}Content", ident), input.ident.span());

    // the raw version has no bounds on its type param
    let generics = {
        let mut gen = input.generics.clone();
        for p in &mut gen.params {
            if let GenericParam::Type(TypeParam { bounds, .. }) = p {
                bounds.clear();
            }
        }
        gen
    };

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
                quote! { Box<::serde_json::value::RawValue> }
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
                quote! {
                    let raw = content.ok_or_else(|| ::serde::de::Error::missing_field("content"))?;
                    let content = C::from_parts(&event_type, raw).map_err(A::Error::custom)?;
                }
            } else if name == "prev_content" {
                quote! {
                    let prev_content = if let Some(raw) = prev_content {
                        Some(C::from_parts(&event_type, raw).map_err(A::Error::custom)?)
                    } else {
                        None
                    };
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

    let deserialize_impl = quote! {
        impl<'de, C> ::serde::de::Deserialize<'de> for #ident<C>
        where
            C: ::ruma_events::RawEventContent,
        {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::de::Deserializer<'de>,
            {
                #[derive(serde::Deserialize)]
                #[serde(field_identifier, rename_all = "snake_case")]
                enum Field {
                    // since this is represented as an enum we have to add it so the JSON picks it up
                    Type,
                    #( #enum_variants ),*
                }

                /// Visits the fields of an event struct to handle deserialization of
                /// the `content` and `prev_content` fields.
                struct EventVisitor<C>(::std::marker::PhantomData<C>);

                impl<'de, C> ::serde::de::Visitor<'de> for EventVisitor<C>
                where
                    C: ::ruma_events::RawEventContent,
                {
                    type Value = #ident<C>;

                    fn expecting(&self, formatter: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                        write!(formatter, "struct implementing {}", stringify!(#content_ident))
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

                deserializer.deserialize_map(EventVisitor(::std::marker::PhantomData))
            }
        }
    };

    let raw_docs = format!("The raw version of {}, allows for deserialization.", ident);
    Ok(quote! {
        #[doc = #raw_docs]
        mod raw_event {
            use super::*;

            #[derive(Clone, Debug)]
            pub struct #ident #generics {
                #( #fields ),*
            }

            #deserialize_impl
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
