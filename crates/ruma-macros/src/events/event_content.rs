//! Implementations of the EventContent derive macro.
#![allow(clippy::too_many_arguments)] // FIXME

use std::borrow::Cow;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{DeriveInput, Field, Ident, Meta, Token, Type, parse_quote, punctuated::Punctuated};

mod parse;

use self::parse::{ContentAttrs, ContentMeta, EventContentKind, EventFieldMeta, EventTypeFragment};
use super::enums::{
    EventContentTraitVariation, EventContentVariation, EventKind, EventTypes, EventVariation,
};
use crate::{events::enums::EventType, import_ruma_common, util::PrivateField};

/// `EventContent` derive macro code generation.
pub fn expand_event_content(
    input: &DeriveInput,
    ruma_events: &TokenStream,
) -> syn::Result<TokenStream> {
    let content_meta = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("ruma_event"))
        .try_fold(ContentMeta::default(), |meta, attr| {
            let list: Punctuated<ContentMeta, Token![,]> =
                attr.parse_args_with(Punctuated::parse_terminated)?;

            list.into_iter().try_fold(meta, ContentMeta::merge)
        })?;

    let ContentAttrs {
        types,
        kind,
        state_key_type,
        unsigned_type,
        is_custom_redacted,
        is_custom_possibly_redacted,
        has_without_relation,
    } = content_meta.try_into()?;

    let variations = kind.event_content_variations();

    let generate_redacted =
        !is_custom_redacted && variations.contains(&EventContentVariation::Redacted);
    let generate_possibly_redacted = !is_custom_possibly_redacted
        && variations.contains(&EventContentVariation::PossiblyRedacted);

    let ident = &input.ident;
    let fields = match &input.data {
        syn::Data::Struct(syn::DataStruct { fields, .. }) => Some(fields.iter()),
        _ => {
            if generate_redacted {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "To generate a redacted event content, the event content type needs to be a struct. Disable this with the custom_redacted attribute",
                ));
            }

            if generate_possibly_redacted {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "To generate a possibly redacted event content, the event content type needs to be a struct. Disable this with the custom_possibly_redacted attribute",
                ));
            }

            if has_without_relation {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "To generate an event content without relation, the event content type needs to be a struct. Disable this by removing the without_relation attribute",
                ));
            }

            None
        }
    };

    let event_type_fragment = EventTypeFragment::try_from_parts(&types.ev_type, fields.clone())?;

    // We only generate redacted content structs for state and message-like events
    let redacted_event_content = generate_redacted.then(|| {
        generate_redacted_event_content(
            ident,
            &input.vis,
            fields.clone().unwrap(),
            &types,
            kind,
            event_type_fragment.as_ref(),
            state_key_type.as_ref(),
            unsigned_type.clone(),
            ruma_events,
        )
        .unwrap_or_else(syn::Error::into_compile_error)
    });

    // We only generate possibly redacted content structs for state events.
    let possibly_redacted_event_content = generate_possibly_redacted.then(|| {
        generate_possibly_redacted_event_content(
            ident,
            &input.vis,
            fields.clone().unwrap(),
            &types,
            event_type_fragment.as_ref(),
            state_key_type.as_ref(),
            unsigned_type.clone(),
            generate_redacted,
            ruma_events,
        )
        .unwrap_or_else(syn::Error::into_compile_error)
    });

    let event_content_without_relation = has_without_relation.then(|| {
        generate_event_content_without_relation(
            ident,
            &input.vis,
            fields.clone().unwrap(),
            ruma_events,
        )
        .unwrap_or_else(syn::Error::into_compile_error)
    });

    let event_content_impl = generate_event_content_impl(
        ident,
        fields,
        &types,
        kind,
        EventContentVariation::Original,
        event_type_fragment.as_ref(),
        state_key_type.as_ref(),
        unsigned_type,
        ruma_events,
    )
    .unwrap_or_else(syn::Error::into_compile_error);
    let static_event_content_impl = generate_static_event_content_impl(ident, &types, ruma_events);
    let type_aliases = generate_event_type_aliases(kind, ident, &input.vis, &types, ruma_events)
        .unwrap_or_else(syn::Error::into_compile_error);

    let json_castable_impl = generate_json_castable_impl(ident, &[]);

    Ok(quote! {
        #redacted_event_content
        #possibly_redacted_event_content
        #event_content_without_relation
        #event_content_impl
        #static_event_content_impl
        #type_aliases
        #json_castable_impl
    })
}

/// Generate the `Redacted*EventContent` variation of the type.
fn generate_redacted_event_content<'a>(
    ident: &Ident,
    vis: &syn::Visibility,
    fields: impl Iterator<Item = &'a Field>,
    types: &EventTypes,
    kind: EventContentKind,
    event_type_fragment: Option<&EventTypeFragment<'_>>,
    state_key_type: Option<&TokenStream>,
    unsigned_type: Option<TokenStream>,
    ruma_events: &TokenStream,
) -> syn::Result<TokenStream> {
    assert!(
        !types.is_prefix(),
        "Event type shouldn't contain a `*`, this should have been checked previously"
    );

    let ruma_common = quote! { #ruma_events::exports::ruma_common };
    let serde = quote! { #ruma_events::exports::serde };

    let doc = format!("Redacted form of [`{ident}`]");
    let redacted_ident = format_ident!("Redacted{ident}");

    let kept_redacted_fields: Vec<_> = fields
        .map(|f| {
            let mut keep_field = false;
            let attrs = f
                .attrs
                .iter()
                .map(|a| -> syn::Result<_> {
                    if a.path().is_ident("ruma_event") {
                        if let EventFieldMeta::SkipRedaction = a.parse_args()? {
                            keep_field = true;
                        }

                        // don't re-emit our `ruma_event` attributes
                        Ok(None)
                    } else {
                        Ok(Some(a.clone()))
                    }
                })
                .filter_map(Result::transpose)
                .collect::<syn::Result<_>>()?;

            if keep_field { Ok(Some(Field { attrs, ..f.clone() })) } else { Ok(None) }
        })
        .filter_map(Result::transpose)
        .collect::<syn::Result<_>>()?;

    let redaction_struct_fields = kept_redacted_fields.iter().flat_map(|f| &f.ident);

    let constructor = kept_redacted_fields.is_empty().then(|| {
        let doc = format!("Creates an empty {redacted_ident}.");
        quote! {
            impl #redacted_ident {
                #[doc = #doc]
                #vis fn new() -> Self {
                    Self {}
                }
            }
        }
    });

    let redacted_event_content = generate_event_content_impl(
        &redacted_ident,
        Some(kept_redacted_fields.iter()),
        types,
        kind,
        EventContentVariation::Redacted,
        event_type_fragment,
        state_key_type,
        unsigned_type,
        ruma_events,
    )
    .unwrap_or_else(syn::Error::into_compile_error);

    let static_event_content_impl =
        generate_static_event_content_impl(&redacted_ident, types, ruma_events);

    let json_castable_impl = generate_json_castable_impl(&redacted_ident, &[ident]);

    Ok(quote! {
        // this is the non redacted event content's impl
        #[automatically_derived]
        impl #ruma_events::RedactContent for #ident {
            type Redacted = #redacted_ident;

            fn redact(self, _rules: &#ruma_common::room_version_rules::RedactionRules) -> #redacted_ident {
                #redacted_ident {
                    #( #redaction_struct_fields: self.#redaction_struct_fields, )*
                }
            }
        }

        #[doc = #doc]
        #[derive(Clone, Debug, #serde::Deserialize, #serde::Serialize)]
        #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
        #vis struct #redacted_ident {
            #( #kept_redacted_fields, )*
        }

        #constructor

        #redacted_event_content

        #static_event_content_impl

        #json_castable_impl
    })
}

/// Generate the `PossiblyRedacted*EventContent` variation of the type.
fn generate_possibly_redacted_event_content<'a>(
    ident: &Ident,
    vis: &syn::Visibility,
    fields: impl Iterator<Item = &'a Field>,
    types: &EventTypes,
    event_type_fragment: Option<&EventTypeFragment<'_>>,
    state_key_type: Option<&TokenStream>,
    unsigned_type: Option<TokenStream>,
    generate_redacted: bool,
    ruma_events: &TokenStream,
) -> syn::Result<TokenStream> {
    assert!(
        !types.is_prefix(),
        "Event type shouldn't contain a `*`, this should have been checked previously"
    );

    let serde = quote! { #ruma_events::exports::serde };

    let doc = format!(
        "The possibly redacted form of [`{ident}`].\n\n\
        This type is used when it's not obvious whether the content is redacted or not."
    );
    let possibly_redacted_ident = format_ident!("PossiblyRedacted{ident}");

    let mut field_changed = false;
    let possibly_redacted_fields: Vec<_> = fields
        .map(|f| {
            let mut keep_field = false;
            let mut unsupported_serde_attribute = None;

            if let Type::Path(type_path) = &f.ty
                && type_path.path.segments.first().filter(|s| s.ident == "Option").is_some()
            {
                // Keep the field if it's an `Option`.
                keep_field = true;
            }

            let mut attrs = f
                .attrs
                .iter()
                .map(|a| -> syn::Result<_> {
                    if a.path().is_ident("ruma_event") {
                        // Keep the field if it is not redacted.
                        if let EventFieldMeta::SkipRedaction = a.parse_args()? {
                            keep_field = true;
                        }

                        // Don't re-emit our `ruma_event` attributes.
                        Ok(None)
                    } else {
                        if a.path().is_ident("serde")
                            && let Meta::List(list) = &a.meta
                        {
                            let nested: Punctuated<Meta, Token![,]> =
                                list.parse_args_with(Punctuated::parse_terminated)?;
                            for meta in &nested {
                                if meta.path().is_ident("default") {
                                    // Keep the field if it deserializes to its default value.
                                    keep_field = true;
                                } else if !meta.path().is_ident("rename")
                                    && !meta.path().is_ident("alias")
                                    && unsupported_serde_attribute.is_none()
                                {
                                    // Error if the field is not kept and uses an unsupported
                                    // serde attribute.
                                    unsupported_serde_attribute = Some(syn::Error::new_spanned(
                                        meta,
                                        "Can't generate PossiblyRedacted struct with \
                                                 unsupported serde attribute\n\
                                                 Expected one of `default`, `rename` or `alias`\n\
                                                 Use the `custom_possibly_redacted` attribute \
                                                 and create the struct manually",
                                    ));
                                }
                            }
                        }

                        Ok(Some(a.clone()))
                    }
                })
                .filter_map(Result::transpose)
                .collect::<syn::Result<_>>()?;

            if keep_field {
                Ok(Field { attrs, ..f.clone() })
            } else if let Some(err) = unsupported_serde_attribute {
                Err(err)
            } else if f.ident.is_none() {
                // If the field has no `ident`, it's a tuple struct. Since `content` is an object,
                // it will need a custom struct to deserialize from an empty object.
                Err(syn::Error::new(
                    Span::call_site(),
                    "Can't generate PossiblyRedacted struct for tuple structs\n\
                    Use the `custom_possibly_redacted` attribute and create the struct manually",
                ))
            } else {
                // Change the field to an `Option`.
                field_changed = true;

                let old_type = &f.ty;
                let ty = parse_quote! { Option<#old_type> };
                attrs.push(parse_quote! { #[serde(skip_serializing_if = "Option::is_none")] });

                Ok(Field { attrs, ty, ..f.clone() })
            }
        })
        .collect::<syn::Result<_>>()?;

    // If at least one field needs to change, generate a new struct, else use a type alias.
    if field_changed {
        let possibly_redacted_event_content = generate_event_content_impl(
            &possibly_redacted_ident,
            Some(possibly_redacted_fields.iter()),
            types,
            EventKind::State.into(),
            EventContentVariation::PossiblyRedacted,
            event_type_fragment,
            state_key_type,
            unsigned_type,
            ruma_events,
        )
        .unwrap_or_else(syn::Error::into_compile_error);

        let static_event_content_impl =
            generate_static_event_content_impl(&possibly_redacted_ident, types, ruma_events);

        let json_castable_impl = if generate_redacted {
            let redacted_ident = format_ident!("Redacted{ident}");
            generate_json_castable_impl(&possibly_redacted_ident, &[ident, &redacted_ident])
        } else {
            generate_json_castable_impl(&possibly_redacted_ident, &[ident])
        };

        Ok(quote! {
            #[doc = #doc]
            #[derive(Clone, Debug, #serde::Deserialize, #serde::Serialize)]
            #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
            #vis struct #possibly_redacted_ident {
                #( #possibly_redacted_fields, )*
            }

            #possibly_redacted_event_content

            #static_event_content_impl

            #json_castable_impl
        })
    } else {
        let event_content_kind_trait_impl = generate_event_content_kind_trait_impl(
            ident,
            types,
            EventKind::State.into(),
            EventContentTraitVariation::PossiblyRedacted,
            event_type_fragment,
            state_key_type,
            ruma_events,
        );

        Ok(quote! {
            #[doc = #doc]
            #vis type #possibly_redacted_ident = #ident;

            #event_content_kind_trait_impl
        })
    }
}

/// Generate the `*EventContentWithoutRelation` variation of the type.
fn generate_event_content_without_relation<'a>(
    ident: &Ident,
    vis: &syn::Visibility,
    fields: impl Iterator<Item = &'a Field>,
    ruma_events: &TokenStream,
) -> syn::Result<TokenStream> {
    let serde = quote! { #ruma_events::exports::serde };

    let type_doc = format!(
        "Form of [`{ident}`] without relation.\n\n\
        To construct this type, construct a [`{ident}`] and then use one of its `::from()` / `.into()` methods."
    );
    let without_relation_ident = format_ident!("{ident}WithoutRelation");

    let with_relation_fn_doc =
        format!("Transform `self` into a [`{ident}`] with the given relation.");

    let (relates_to, other_fields) = fields.partition::<Vec<_>, _>(|f| {
        f.ident.as_ref().filter(|ident| *ident == "relates_to").is_some()
    });

    let relates_to_type = relates_to.into_iter().next().map(|f| &f.ty).ok_or_else(|| {
        syn::Error::new(
            Span::call_site(),
            "`without_relation` can only be used on events with a `relates_to` field",
        )
    })?;

    let without_relation_fields = other_fields.iter().flat_map(|f| &f.ident).collect::<Vec<_>>();
    let without_relation_struct = if other_fields.is_empty() {
        quote! { ; }
    } else {
        quote! {
            { #( #other_fields, )* }
        }
    };

    let json_castable_impl = generate_json_castable_impl(&without_relation_ident, &[ident]);

    Ok(quote! {
        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl ::std::convert::From<#ident> for #without_relation_ident {
            fn from(c: #ident) -> Self {
                Self {
                    #( #without_relation_fields: c.#without_relation_fields, )*
                }
            }
        }

        #[doc = #type_doc]
        #[derive(Clone, Debug, #serde::Deserialize, #serde::Serialize)]
        #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
        #vis struct #without_relation_ident #without_relation_struct

        impl #without_relation_ident {
            #[doc = #with_relation_fn_doc]
            #vis fn with_relation(self, relates_to: #relates_to_type) -> #ident {
                #ident {
                    #( #without_relation_fields: self.#without_relation_fields, )*
                    relates_to,
                }
            }
        }

        #json_castable_impl
    })
}

/// Generate the type aliases for the event.
fn generate_event_type_aliases(
    kind: EventContentKind,
    ident: &Ident,
    vis: &syn::Visibility,
    types: &EventTypes,
    ruma_events: &TokenStream,
) -> syn::Result<TokenStream> {
    // The redaction module has its own event types.
    if ident == "RoomRedactionEventContent" {
        return Ok(quote! {});
    }

    let event_type = &types.ev_type;
    let ident_s = ident.to_string();
    let ev_type_s = ident_s.strip_suffix("Content").ok_or_else(|| {
        syn::Error::new_spanned(ident, "Expected content struct name ending in `Content`")
    })?;

    let type_aliases = kind
        .event_variations()
        .iter()
        .filter_map(|&var| Some((var, kind.to_event_idents(var)?)))
        .flat_map(|(var, type_prefixes_and_event_idents)| {
            type_prefixes_and_event_idents.into_iter().map(move |(type_prefix, ev_struct)| {
                let ev_type = format_ident!("{var}{type_prefix}{ev_type_s}");

                let doc_text = match var {
                    EventVariation::None | EventVariation::Original => "",
                    EventVariation::Sync | EventVariation::OriginalSync => {
                        " from a `sync_events` response"
                    }
                    EventVariation::Stripped => " from an invited room preview",
                    EventVariation::Redacted => " that has been redacted",
                    EventVariation::RedactedSync => {
                        " from a `sync_events` response that has been redacted"
                    }
                    EventVariation::Initial => " for creating a room",
                };

                let ev_type_doc = if type_prefix.is_empty() {
                    format!("An `{event_type}` event{doc_text}.")
                } else {
                    format!("A {} `{event_type}` event{doc_text}.", type_prefix.to_lowercase())
                };

                let content_struct = if var.is_redacted() {
                    Cow::Owned(format_ident!("Redacted{ident}"))
                } else if let EventVariation::Stripped = var {
                    Cow::Owned(format_ident!("PossiblyRedacted{ident}"))
                } else {
                    Cow::Borrowed(ident)
                };

                quote! {
                    #[doc = #ev_type_doc]
                    #vis type #ev_type = #ruma_events::#ev_struct<#content_struct>;
                }
            })
        })
        .flatten()
        .collect();

    Ok(type_aliases)
}

/// Generate the `*EventContent` trait implementations of the type.
fn generate_event_content_impl<'a>(
    ident: &Ident,
    fields: Option<impl Iterator<Item = &'a Field>>,
    types: &EventTypes,
    kind: EventContentKind,
    variation: EventContentVariation,
    event_type_fragment: Option<&EventTypeFragment<'_>>,
    state_key_type: Option<&TokenStream>,
    unsigned_type: Option<TokenStream>,
    ruma_events: &TokenStream,
) -> syn::Result<TokenStream> {
    let serde = quote! { #ruma_events::exports::serde };
    let serde_json = quote! { #ruma_events::exports::serde_json };

    let possible_variations = kind.event_content_variations();

    let event_content_kind_trait_impl = generate_event_content_kind_trait_impl(
        ident,
        types,
        kind,
        variation.into(),
        event_type_fragment,
        state_key_type,
        ruma_events,
    );

    let static_state_event_content_impl = (possible_variations
        .contains(&EventContentVariation::PossiblyRedacted)
        && variation == EventContentVariation::Original)
        .then(|| {
            let event_content_kind_trait_name =
                EventKind::State.to_content_kind_trait(EventContentTraitVariation::Static);
            let possibly_redacted_ident = format_ident!("PossiblyRedacted{ident}");

            let unsigned_type = unsigned_type
                .unwrap_or_else(|| quote! { #ruma_events::StateUnsigned<Self::PossiblyRedacted> });

            quote! {
                #[automatically_derived]
                impl #ruma_events::#event_content_kind_trait_name for #ident {
                    type PossiblyRedacted = #possibly_redacted_ident;
                    type Unsigned = #unsigned_type;
                }
            }
        });

    let event_content_from_type_impl = event_type_fragment.map(|type_fragment_field| {
        let type_prefixes = types.iter().map(EventType::without_wildcard);
        let type_prefixes = quote! {
            [#(#type_prefixes,)*]
        };
        let fields_without_type_fragment = fields
            .unwrap()
            .filter(|f| {
                !f.attrs.iter().any(|a| {
                    a.path().is_ident("ruma_event")
                        && matches!(a.parse_args(), Ok(EventFieldMeta::TypeFragment))
                })
            })
            .map(PrivateField)
            .collect::<Vec<_>>();
        let fields_ident_without_type_fragment =
            fields_without_type_fragment.iter().filter_map(|f| f.0.ident.as_ref());

        quote! {
            impl #ruma_events::EventContentFromType for #ident {
                fn from_parts(
                    ev_type: &::std::primitive::str,
                    content: &#serde_json::value::RawValue,
                ) -> #serde_json::Result<Self> {
                    #[derive(#serde::Deserialize)]
                    struct WithoutTypeFragment {
                        #( #fields_without_type_fragment, )*
                    }

                    if let ::std::option::Option::Some(type_fragment) =
                        #type_prefixes.iter().find_map(|prefix| ev_type.strip_prefix(prefix))
                    {
                        let c: WithoutTypeFragment = #serde_json::from_str(content.get())?;

                        ::std::result::Result::Ok(Self {
                            #(
                                #fields_ident_without_type_fragment:
                                    c.#fields_ident_without_type_fragment,
                            )*
                            #type_fragment_field: type_fragment.to_owned(),
                        })
                    } else {
                        ::std::result::Result::Err(#serde::de::Error::custom(
                            ::std::format!(
                                "expected event type starting with one of `{:?}`, found `{}`",
                                #type_prefixes, ev_type,
                            )
                        ))
                    }
                }
            }
        }
    });

    Ok(quote! {
        #event_content_from_type_impl
        #event_content_kind_trait_impl
        #static_state_event_content_impl
    })
}

/// Generate the `*EventContent` trait implementation of the type.
fn generate_event_content_kind_trait_impl(
    ident: &Ident,
    types: &EventTypes,
    kind: EventContentKind,
    variation: EventContentTraitVariation,
    event_type_fragment: Option<&EventTypeFragment<'_>>,
    state_key_type: Option<&TokenStream>,
    ruma_events: &TokenStream,
) -> TokenStream {
    let event_type = types.ev_type.without_wildcard();
    let event_type_fn_impl = if let Some(field) = event_type_fragment {
        let format = event_type.to_owned() + "{}";

        quote! {
            ::std::convert::From::from(::std::format!(#format, self.#field))
        }
    } else {
        quote! { ::std::convert::From::from(#event_type) }
    };

    let state_key = kind.is_state().then(|| {
        assert!(state_key_type.is_some());

        quote! {
            type StateKey = #state_key_type;
        }
    });

    kind.to_content_kind_enums_and_traits(variation)
        .into_iter()
        .map(|(event_type_enum, event_content_kind_trait_name)| {
            quote! {
                #[automatically_derived]
                impl #ruma_events::#event_content_kind_trait_name for #ident {
                    #state_key

                    fn event_type(&self) -> #ruma_events::#event_type_enum {
                        #event_type_fn_impl
                    }
                }
            }
        })
        .collect()
}

/// Generate the `StaticEventContent` trait implementation of the type.
fn generate_static_event_content_impl(
    ident: &Ident,
    types: &EventTypes,
    ruma_events: &TokenStream,
) -> TokenStream {
    let event_type = types.ev_type.without_wildcard();
    let static_event_type = quote! { #event_type };

    let is_prefix = if types.is_prefix() {
        quote! { #ruma_events::True }
    } else {
        quote! { #ruma_events::False }
    };

    quote! {
        impl #ruma_events::StaticEventContent for #ident {
            const TYPE: &'static ::std::primitive::str = #static_event_type;
            type IsPrefix = #is_prefix;
        }
    }
}

/// Implement `JsonCastable<JsonObject> for {ident}` and `JsonCastable<{ident}> for {other}`.
fn generate_json_castable_impl(ident: &Ident, others: &[&Ident]) -> TokenStream {
    let ruma_common = import_ruma_common();

    let mut json_castable_impls = quote! {
        #[automatically_derived]
        impl #ruma_common::serde::JsonCastable<#ruma_common::serde::JsonObject> for #ident {}
    };

    json_castable_impls.extend(others.iter().map(|other| {
        quote! {
            #[automatically_derived]
            impl #ruma_common::serde::JsonCastable<#ident> for #other {}
        }
    }));

    json_castable_impls
}
