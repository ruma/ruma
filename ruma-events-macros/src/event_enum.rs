//! Implementation of event enum and event content enum macros.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{self, Parse, ParseStream},
    Attribute, Expr, ExprLit, Ident, Lit, LitStr, Token,
};

/// Create a content enum from `EventEnumInput`.
pub fn expand_event_enum(input: EventEnumInput) -> syn::Result<TokenStream> {
    let ident = &input.name;

    let event_enum = expand_any_enum_with_deserialize(&input, ident)?;

    let needs_event_content = ident == "AnyStateEvent"
        || ident == "AnyMessageEvent"
        || ident == "AnyToDeviceEvent"
        || ident == "AnyEphemeralRoomEvent"
        || ident == "AnyBasicEvent";

    let needs_event_stub =
        ident == "AnyStateEvent" || ident == "AnyMessageEvent" || ident == "AnyEphemeralRoomEvent";

    let needs_stripped_event = ident == "AnyStateEvent";

    let event_stub_enum =
        if needs_event_stub { expand_stub_enum(&input)? } else { TokenStream::new() };

    let event_stripped_enum =
        if needs_stripped_event { expand_stripped_enum(&input)? } else { TokenStream::new() };

    let event_content_enum =
        if needs_event_content { expand_content_enum(&input)? } else { TokenStream::new() };

    Ok(quote! {
        #event_enum

        #event_stub_enum

        #event_stripped_enum

        #event_content_enum
    })
}

/// Create a "stub" enum from `EventEnumInput`.
pub fn expand_stub_enum(input: &EventEnumInput) -> syn::Result<TokenStream> {
    let ident = Ident::new(&format!("{}Stub", input.name.to_string()), input.name.span());

    expand_any_enum_with_deserialize(input, &ident)
}

/// Create a "stripped" enum from `EventEnumInput`.
pub fn expand_stripped_enum(input: &EventEnumInput) -> syn::Result<TokenStream> {
    let ident = Ident::new("AnyStrippedStateEventStub", input.name.span());

    expand_any_enum_with_deserialize(input, &ident)
}

/// Create a content enum from `EventEnumInput`.
pub fn expand_content_enum(input: &EventEnumInput) -> syn::Result<TokenStream> {
    let attrs = &input.attrs;
    let ident = Ident::new(&format!("{}Content", input.name.to_string()), input.name.span());
    let event_type_str = &input.events;

    let variants = input.events.iter().map(to_camel_case).collect::<syn::Result<Vec<_>>>()?;
    let content = input.events.iter().map(to_event_content_path).collect::<Vec<_>>();

    let content_enum = quote! {
        #( #attrs )*
        #[derive(Clone, Debug, ::serde::Serialize)]
        #[serde(untagged)]
        #[allow(clippy::large_enum_variant)]
        pub enum #ident {
            #(
                #[doc = #event_type_str]
                #variants(#content),
            )*
            /// Content of an event not defined by the Matrix specification.
            Custom(::ruma_events::custom::CustomEventContent),
        }
    };

    let event_content_impl = quote! {
        impl ::ruma_events::EventContent for #ident {
            fn event_type(&self) -> &str {
                match self {
                    #( Self::#variants(content) => content.event_type(), )*
                    Self::Custom(content) => content.event_type(),
                }
            }

            fn from_parts(event_type: &str, input: Box<::serde_json::value::RawValue>) -> Result<Self, ::serde_json::Error> {
                match event_type {
                    #(
                        #event_type_str => {
                            let content = #content::from_parts(event_type, input)?;
                            Ok(#ident::#variants(content))
                        },
                    )*
                    ev_type => {
                        let content = ::ruma_events::custom::CustomEventContent::from_parts(ev_type, input)?;
                        Ok(#ident::Custom(content))
                    },
                }
            }
        }
    };

    let marker_trait_impls = marker_traits(&ident);

    Ok(quote! {
        #content_enum

        #event_content_impl

        #marker_trait_impls
    })
}

fn expand_any_enum_with_deserialize(
    input: &EventEnumInput,
    ident: &Ident,
) -> syn::Result<TokenStream> {
    let attrs = &input.attrs;
    let event_type_str = &input.events;
    let event_struct = Ident::new(&ident.to_string().trim_start_matches("Any"), ident.span());

    let variants = input.events.iter().map(to_camel_case).collect::<syn::Result<Vec<_>>>()?;
    let content =
        input.events.iter().map(|event| to_event_path(event, &event_struct)).collect::<Vec<_>>();

    let any_enum = quote! {
        #( #attrs )*
        #[derive(Clone, Debug, ::serde::Serialize)]
        #[serde(untagged)]
        #[allow(clippy::large_enum_variant)]
        pub enum #ident {
            #(
                #[doc = #event_type_str]
                #variants(#content),
            )*
            /// An event not defined by the Matrix specification
            Custom(::ruma_events::#event_struct<::ruma_events::custom::CustomEventContent>),
        }
    };

    let event_deserialize_impl = quote! {
        impl<'de> ::serde::de::Deserialize<'de> for #ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::de::Deserializer<'de>,
            {
                use ::serde::de::Error as _;

                let json = Box::<::serde_json::value::RawValue>::deserialize(deserializer)?;
                let ::ruma_events::EventDeHelper { ev_type, .. } = ::ruma_events::from_raw_json_value(&json)?;
                match ev_type.as_str() {
                    #(
                        #event_type_str => {
                            let event = ::serde_json::from_str::<#content>(json.get()).map_err(D::Error::custom)?;
                            Ok(#ident::#variants(event))
                        },
                    )*
                    event => {
                        let event =
                            ::serde_json::from_str::<::ruma_events::#event_struct<::ruma_events::custom::CustomEventContent>>(json.get())
                                .map_err(D::Error::custom)?;

                        Ok(Self::Custom(event))
                    },
                }
            }
        }
    };

    Ok(quote! {
        #any_enum

        #event_deserialize_impl
    })
}

fn marker_traits(ident: &Ident) -> TokenStream {
    match ident.to_string().as_str() {
        "AnyStateEventContent" => quote! {
            impl ::ruma_events::RoomEventContent for #ident {}
            impl ::ruma_events::StateEventContent for #ident {}
        },
        "AnyMessageEventContent" => quote! {
            impl ::ruma_events::RoomEventContent for #ident {}
            impl ::ruma_events::MessageEventContent for #ident {}
        },
        "AnyEphemeralRoomEventContent" => quote! {
            impl ::ruma_events::EphemeralRoomEventContent for #ident {}
        },
        "AnyBasicEventContent" => quote! {
            impl ::ruma_events::BasicEventContent for #ident {}
        },
        _ => TokenStream::new(),
    }
}

fn to_event_path(name: &LitStr, struct_name: &Ident) -> TokenStream {
    let span = name.span();
    let name = name.value();

    assert_eq!(&name[..2], "m.");

    let path = name[2..].split('.').collect::<Vec<_>>();

    let event_str = path.last().unwrap();
    let event = event_str
        .split('_')
        .map(|s| s.chars().next().unwrap().to_uppercase().to_string() + &s[1..])
        .collect::<String>();

    let path = path.iter().map(|s| Ident::new(s, span));

    match struct_name.to_string().as_str() {
        "MessageEvent" | "MessageEventStub" if name == "m.room.redaction" => {
            let redaction = if struct_name == "MessageEvent" {
                quote! { RedactionEvent }
            } else {
                quote! { RedactionEventStub }
            };
            quote! { ::ruma_events::room::redaction::#redaction }
        }
        "ToDeviceEvent" | "StateEventStub" | "StrippedStateEventStub" | "MessageEventStub" => {
            let content = Ident::new(&format!("{}EventContent", event), span);
            quote! { ::ruma_events::#struct_name<::ruma_events::#( #path )::*::#content> }
        }
        _ => {
            let event_name = Ident::new(&format!("{}Event", event), span);
            quote! { ::ruma_events::#( #path )::*::#event_name }
        }
    }
}

fn to_event_content_path(name: &LitStr) -> TokenStream {
    let span = name.span();
    let name = name.value();

    assert_eq!(&name[..2], "m.");

    let path = name[2..].split('.').collect::<Vec<_>>();

    let event_str = path.last().unwrap();
    let event = event_str
        .split('_')
        .map(|s| s.chars().next().unwrap().to_uppercase().to_string() + &s[1..])
        .collect::<String>();

    let content_str = Ident::new(&format!("{}EventContent", event), span);
    let path = path.iter().map(|s| Ident::new(s, span));
    quote! {
        ::ruma_events::#( #path )::*::#content_str
    }
}

/// Splits the given `event_type` string on `.` and `_` removing the `m.room.` then
/// camel casing to give the `Event` struct name.
pub(crate) fn to_camel_case(name: &LitStr) -> syn::Result<Ident> {
    let span = name.span();
    let name = name.value();

    if &name[..2] != "m." {
        return Err(syn::Error::new(
            span,
            format!("well-known matrix events have to start with `m.` found `{}`", name),
        ));
    }

    let s = name[2..]
        .split(&['.', '_'] as &[char])
        .map(|s| s.chars().next().unwrap().to_uppercase().to_string() + &s[1..])
        .collect::<String>();

    Ok(Ident::new(&s, span))
}

/// Custom keywords for the `event_enum!` macro
mod kw {
    syn::custom_keyword!(name);
    syn::custom_keyword!(events);
}

/// The entire `event_enum!` macro structure directly as it appears in the source code.
pub struct EventEnumInput {
    /// Outer attributes on the field, such as a docstring.
    pub attrs: Vec<Attribute>,

    /// The name of the event.
    pub name: Ident,

    /// An array of valid matrix event types. This will generate the variants of the event type "name".
    /// There needs to be a corresponding variant in `ruma_events::EventType` for
    /// this event (converted to a valid Rust-style type name by stripping `m.`, replacing the
    /// remaining dots by underscores and then converting from snake_case to CamelCase).
    pub events: Vec<LitStr>,
}

impl Parse for EventEnumInput {
    fn parse(input: ParseStream<'_>) -> parse::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        // "name" field
        input.parse::<kw::name>()?;
        input.parse::<Token![:]>()?;

        // the name of our event enum
        let name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        // "events" field
        input.parse::<kw::events>()?;
        input.parse::<Token![:]>()?;

        // an array of event names `["m.room.whatever", ...]`
        let ev_array = input.parse::<syn::ExprArray>()?;
        let events = ev_array
            .elems
            .into_iter()
            .map(|item| {
                if let Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }) = item {
                    Ok(lit_str)
                } else {
                    let msg = "values of field `events` are required to be a string literal";
                    Err(syn::Error::new_spanned(item, msg))
                }
            })
            .collect::<syn::Result<_>>()?;

        Ok(Self { attrs, name, events })
    }
}
