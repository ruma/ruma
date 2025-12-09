//! Implementation of the `Event` derive macro.

use std::borrow::Cow;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_quote;

mod parse;

use super::enums::{CommonEventField, EventKind, EventVariation};
use crate::util::{RumaEvents, RumaEventsReexport, to_camel_case};

/// `Event` derive macro code generation.
pub(crate) fn expand_event(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let event = Event::parse(input)?;

    let deserialize_impl = event.expand_deserialize_impl();
    let sync_from_and_into_full = event.expand_sync_from_and_into_full();

    let eq_and_ord_impl = event.expand_eq_and_ord_impl();

    Ok(quote! {
        #deserialize_impl
        #sync_from_and_into_full
        #eq_and_ord_impl
    })
}

/// The parsed `Event` container data.
struct Event {
    /// The name of the event struct.
    ident: syn::Ident,

    /// The generics on the event struct.
    generics: syn::Generics,

    /// The kind of event.
    kind: EventKind,

    /// The variation of the event struct.
    variation: EventVariation,

    /// The fields of the struct.
    fields: Vec<EventField>,

    /// The path to imports from the ruma-events crate.
    ruma_events: RumaEvents,
}

impl Event {
    /// Generate the `serde::Deserialize` implementation for this struct.
    fn expand_deserialize_impl(&self) -> TokenStream {
        let ruma_events = &self.ruma_events;
        let serde = ruma_events.reexported(RumaEventsReexport::Serde);
        let serde_json = ruma_events.reexported(RumaEventsReexport::SerdeJson);

        let ident = &self.ident;
        let is_content_generic = !self.generics.params.is_empty();
        let (impl_generics, ty_gen, where_clause) = self.generics.split_for_impl();

        let field_idents = self.fields.iter().map(EventField::ident).collect::<Vec<_>>();
        let serialized_field_names =
            self.fields.iter().map(EventField::serialized_name).collect::<Vec<_>>();
        let enum_variants = field_idents.iter().copied().map(to_camel_case).collect::<Vec<_>>();
        let enum_variants_serde_attributes = self.fields.iter().map(EventField::serde_attribute);

        // Get the type of each field to deserialize to.
        let field_types = self.fields.iter().map(|field| {
            let field_ident = field.ident();

            if *field_ident == "content" && is_content_generic {
                // Deserialize the content to a `Box<RawValue>` so we can use the
                // `EventContentFromType` implementation later.
                quote! { ::std::boxed::Box<#serde_json::value::RawValue> }
            } else if *field_ident == "state_key" && self.variation == EventVariation::Initial {
                // Because the state key is allowed to be missing if it is empty when sending an
                // initial state event during creation, we default to deserializing a string first
                // so we can default to an empty string if it is missing.
                quote! { ::std::string::String }
            } else {
                let field_type = &field.inner.ty;
                quote! { #field_type }
            }
        });

        // Validate the deserialized values of the fields.
        let validate_field_values = self
            .fields
            .iter()
            .zip(&serialized_field_names)
            .map(|(field, serialized_name)| {
                let field_ident = field.ident();

                if *field_ident == "content" && is_content_generic {
                    // Return an error if the content is missing, and use the `EventContentFromType`
                    // implementation to deserialize the `RawValue`.
                    quote! {
                        let content = {
                            let json = content
                                .ok_or_else(|| #serde::de::Error::missing_field("content"))?;
                            C::from_parts(&event_type, &json).map_err(#serde::de::Error::custom)?
                        };
                    }
                } else if field.default
                    || (*field_ident == "unsigned" && !self.variation.is_redacted())
                {
                    // The field is allowed to be missing, and uses its `Default` implementation.
                    quote! {
                        let #field_ident = #field_ident.unwrap_or_default();
                    }
                } else if *field_ident == "state_key" && self.variation == EventVariation::Initial {
                    // The state key is allowed to be missing if it is empty, when sending an
                    // initial state event during creation.
                    let field_type = &field.inner.ty;
                    quote! {
                        let state_key = <#field_type as #serde::de::Deserialize>::deserialize(
                            #serde::de::IntoDeserializer::<A::Error>::into_deserializer(
                                state_key.unwrap_or_default(),
                            ),
                        )?;
                    }
                } else {
                    // The default behavior is to return an error if the field is missing.
                    quote! {
                        let #field_ident = #field_ident.ok_or_else(|| {
                            #serde::de::Error::missing_field(#serialized_name)
                        })?;
                    }
                }
            })
            .collect::<Vec<_>>();

        // Handle deserialization errors for the fields.
        let field_deserialize_error_handlers = self.fields.iter().map(|field| {
            if field.default_on_error {
                // Just log the deserialization error and use the `Default` implementation instead.
                quote! {
                    .map_err(|error| {
                        tracing::debug!("deserialization error, using default value: {error}");
                    })
                    .unwrap_or_default()
                }
            } else {
                // Just forward the deserialization error.
                quote! { ? }
            }
        });

        // Add the deserialization lifetime to the list of generics.
        let deserialize_generics = if is_content_generic {
            let generic_params = &self.generics.params;
            quote! { <'de, #generic_params> }
        } else {
            quote! { <'de> }
        };

        // If the struct has generics, it needs to be forwarded to the `EventVisitor` as
        // `PhantomData`.
        let visitor_phantom_type = if is_content_generic {
            quote! { ::std::marker::PhantomData }
        } else {
            quote! {}
        };

        // If the content is generic, we must add a bound for the `EventContentFromType`
        // implementation.
        let where_clause = if is_content_generic {
            let predicate = parse_quote! { C: #ruma_events::EventContentFromType };

            let where_clause = if let Some(mut where_clause) = where_clause.cloned() {
                where_clause.predicates.push(predicate);
                where_clause
            } else {
                parse_quote! { where #predicate }
            };

            Some(Cow::Owned(where_clause))
        } else {
            where_clause.map(Cow::Borrowed)
        };

        quote! {
            #[automatically_derived]
            impl #deserialize_generics #serde::de::Deserialize<'de> for #ident #ty_gen #where_clause {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: #serde::de::Deserializer<'de>,
                {
                    #[derive(#serde::Deserialize)]
                    #[serde(field_identifier, rename_all = "snake_case")]
                    enum Field {
                        // This field is hidden as the content type in Ruma, but it is always a
                        // valid field and we need to extract it to deserialize the content type.
                        Type,
                        #( #enum_variants_serde_attributes #enum_variants, )*
                        #[serde(other)]
                        Unknown,
                    }

                    /// Visits the fields of an event struct, in particular to handle deserialization of
                    /// the `content` field.
                    struct EventVisitor #impl_generics (#visitor_phantom_type #ty_gen);

                    #[automatically_derived]
                    impl #deserialize_generics #serde::de::Visitor<'de>
                        for EventVisitor #ty_gen #where_clause
                    {
                        type Value = #ident #ty_gen;

                        fn expecting(
                            &self,
                            formatter: &mut ::std::fmt::Formatter<'_>,
                        ) -> ::std::fmt::Result {
                            write!(formatter, "a key-value map")
                        }

                        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                        where
                            A: #serde::de::MapAccess<'de>,
                        {
                            let mut event_type: Option<String> = None;
                            #( let mut #field_idents: Option<#field_types> = None; )*

                            while let Some(key) = map.next_key()? {
                                match key {
                                    // We ignore unknown fields, for forwards compatibility.
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
                                            if #field_idents.is_some() {
                                                return Err(#serde::de::Error::duplicate_field(
                                                    #serialized_field_names,
                                                ));
                                            }
                                            #field_idents = Some(map.next_value() #field_deserialize_error_handlers);
                                        }
                                    )*
                                }
                            }

                            let event_type =
                                event_type.ok_or_else(|| #serde::de::Error::missing_field("type"))?;
                            #( #validate_field_values )*

                            Ok(#ident {
                                #( #field_idents ),*
                            })
                        }
                    }

                    deserializer.deserialize_map(EventVisitor(#visitor_phantom_type))
                }
            }
        }
    }

    /// Generate `From<{full_event}>` and `.into_full_event()` implementations if this is a "sync"
    /// event struct.
    fn expand_sync_from_and_into_full(&self) -> Option<TokenStream> {
        let full_ident = self.kind.to_event_ident(self.variation.to_full()?).ok()?;

        let ruma_common = self.ruma_events.ruma_common();
        let ident = &self.ident;
        let (impl_generics, ty_gen, where_clause) = self.generics.split_for_impl();
        let field_idents = self.fields.iter().map(EventField::ident).collect::<Vec<_>>();

        Some(quote! {
            #[automatically_derived]
            impl #impl_generics ::std::convert::From<#full_ident #ty_gen>
                for #ident #ty_gen #where_clause
            {
                fn from(event: #full_ident #ty_gen) -> Self {
                    let #full_ident { #( #field_idents, )* .. } = event;
                    Self { #( #field_idents, )* }
                }
            }

            #[automatically_derived]
            impl #impl_generics #ident #ty_gen #where_clause {
                /// Convert this sync event into a full event, one with a `room_id` field.
                pub fn into_full_event(
                    self,
                    room_id: #ruma_common::OwnedRoomId,
                ) -> #full_ident #ty_gen {
                    let Self { #( #field_idents, )* } = self;
                    #full_ident {
                        #( #field_idents, )*
                        room_id,
                    }
                }
            }
        })
    }

    /// Implement `std::cmp::PartialEq`, `std::cmp::Eq`, `std::cmp::PartialOrd`, `std::cmp::Ord` for
    /// this event struct by comparing the `event_id`, if this field is present.
    fn expand_eq_and_ord_impl(&self) -> Option<TokenStream> {
        if !CommonEventField::EventId.is_present(self.kind, self.variation) {
            return None;
        }

        let ident = &self.ident;
        let (impl_gen, ty_gen, where_clause) = self.generics.split_for_impl();

        Some(quote! {
            #[automatically_derived]
            impl #impl_gen ::std::cmp::PartialEq for #ident #ty_gen #where_clause {
                /// Checks if the `EventId`s of the events are equal.
                fn eq(&self, other: &Self) -> ::std::primitive::bool {
                    self.event_id == other.event_id
                }
            }

            #[automatically_derived]
            impl #impl_gen ::std::cmp::Eq for #ident #ty_gen #where_clause {}

            #[automatically_derived]
            impl #impl_gen ::std::cmp::PartialOrd for #ident #ty_gen #where_clause {
                /// Compares the `EventId`s of the events and orders them lexicographically.
                fn partial_cmp(&self, other: &Self) -> ::std::option::Option<::std::cmp::Ordering> {
                    self.event_id.partial_cmp(&other.event_id)
                }
            }

            #[automatically_derived]
            impl #impl_gen ::std::cmp::Ord for #ident #ty_gen #where_clause {
                /// Compares the `EventId`s of the events and orders them lexicographically.
                fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                    self.event_id.cmp(&other.event_id)
                }
            }
        })
    }
}

/// A parsed field of an [`Event`].
struct EventField {
    /// The parsed field, without the `ruma_event` attributes.
    inner: syn::Field,

    /// Whether this field should deserialize to the default value if it is missing.
    default: bool,

    /// Whether this field should deserialize to the default value if an error occurs during
    /// deserialization.
    default_on_error: bool,

    /// The name to use when (de)serializing this field.
    ///
    /// If this is not set, the name of the field will be used.
    rename: Option<syn::LitStr>,

    /// The alternate names to recognize when deserializing this field.
    aliases: Vec<syn::LitStr>,
}

impl EventField {
    /// The ident of this field.
    fn ident(&self) -> &syn::Ident {
        self.inner.ident.as_ref().expect(
            "all fields of Event struct should be named; \
             this should have been checked during parsing",
        )
    }

    /// The name of this field in its serialized form.
    fn serialized_name(&self) -> Cow<'_, syn::LitStr> {
        self.rename.as_ref().map(Cow::Borrowed).unwrap_or_else(|| {
            let ident = self.ident();
            Cow::Owned(syn::LitStr::new(&ident.to_string(), ident.span()))
        })
    }

    /// The serde attribute to apply to this field.
    fn serde_attribute(&self) -> Option<TokenStream> {
        let mut attrs = Vec::new();

        if let Some(rename) = &self.rename {
            attrs.push(quote! { rename = #rename });
        }

        attrs.extend(self.aliases.iter().map(|alias| {
            quote! { alias = #alias }
        }));

        (!attrs.is_empty()).then(|| {
            quote! { #[serde(#( #attrs ),*)] }
        })
    }
}
