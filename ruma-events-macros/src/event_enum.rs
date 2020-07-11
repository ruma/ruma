//! Implementation of event enum and event content enum macros.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse::{self, Parse, ParseStream},
    Attribute, Expr, ExprLit, Ident, Lit, LitStr, Token,
};

use crate::event_names::{
    ANY_EPHEMERAL_EVENT, ANY_MESSAGE_EVENT, ANY_STATE_EVENT, ANY_STRIPPED_STATE_EVENT,
    ANY_SYNC_MESSAGE_EVENT, ANY_SYNC_STATE_EVENT, ANY_TO_DEVICE_EVENT, REDACTED_MESSAGE_EVENT,
    REDACTED_STATE_EVENT, REDACTED_STRIPPED_STATE_EVENT, REDACTED_SYNC_MESSAGE_EVENT,
    REDACTED_SYNC_STATE_EVENT,
};

// Arrays of event enum names grouped by a field they share in common.
const ROOM_EVENT_KIND: &[&str] = &[
    ANY_MESSAGE_EVENT,
    ANY_SYNC_MESSAGE_EVENT,
    ANY_STATE_EVENT,
    ANY_SYNC_STATE_EVENT,
    REDACTED_MESSAGE_EVENT,
    REDACTED_STATE_EVENT,
    REDACTED_SYNC_MESSAGE_EVENT,
    REDACTED_SYNC_STATE_EVENT,
];

const ROOM_ID_KIND: &[&str] = &[
    ANY_MESSAGE_EVENT,
    ANY_STATE_EVENT,
    ANY_EPHEMERAL_EVENT,
    REDACTED_STATE_EVENT,
    REDACTED_MESSAGE_EVENT,
];

const EVENT_ID_KIND: &[&str] = &[
    ANY_MESSAGE_EVENT,
    ANY_SYNC_MESSAGE_EVENT,
    ANY_STATE_EVENT,
    ANY_SYNC_STATE_EVENT,
    REDACTED_SYNC_STATE_EVENT,
    REDACTED_SYNC_MESSAGE_EVENT,
    REDACTED_STATE_EVENT,
    REDACTED_MESSAGE_EVENT,
];

const SENDER_KIND: &[&str] = &[
    ANY_MESSAGE_EVENT,
    ANY_STATE_EVENT,
    ANY_SYNC_STATE_EVENT,
    ANY_TO_DEVICE_EVENT,
    ANY_SYNC_MESSAGE_EVENT,
    ANY_STRIPPED_STATE_EVENT,
    REDACTED_MESSAGE_EVENT,
    REDACTED_STATE_EVENT,
    REDACTED_STRIPPED_STATE_EVENT,
    REDACTED_SYNC_MESSAGE_EVENT,
    REDACTED_SYNC_STATE_EVENT,
];

const PREV_CONTENT_KIND: &[&str] = &[ANY_STATE_EVENT, ANY_SYNC_STATE_EVENT];

const STATE_KEY_KIND: &[&str] = &[
    ANY_STATE_EVENT,
    ANY_SYNC_STATE_EVENT,
    ANY_STRIPPED_STATE_EVENT,
    REDACTED_SYNC_STATE_EVENT,
    REDACTED_STRIPPED_STATE_EVENT,
    REDACTED_STATE_EVENT,
];

const REDACTED_EVENT_KIND: &[&str] = &[
    ANY_STATE_EVENT,
    ANY_SYNC_STATE_EVENT,
    ANY_STRIPPED_STATE_EVENT,
    ANY_MESSAGE_EVENT,
    ANY_SYNC_MESSAGE_EVENT,
];

/// This const is used to generate the accessor methods for the `Any*Event` enums.
///
/// DO NOT alter the field names unless the structs in `ruma_events::event_kinds` have changed.
const EVENT_FIELDS: &[(&str, &[&str])] = &[
    ("origin_server_ts", ROOM_EVENT_KIND),
    ("room_id", ROOM_ID_KIND),
    ("event_id", EVENT_ID_KIND),
    ("sender", SENDER_KIND),
    ("state_key", STATE_KEY_KIND),
    ("unsigned", ROOM_EVENT_KIND),
];

/// Create a content enum from `EventEnumInput`.
pub fn expand_event_enum(input: EventEnumInput) -> syn::Result<TokenStream> {
    let ident = &input.name;
    let full_event_enum = ident.to_full_event_enum();

    let event_enum = expand_any_enum_with_deserialize(&input, &full_event_enum)?;

    let event_stub_enum = if let Some(ident) = ident.to_event_stub_enum() {
        expand_stub_enum(&input, &ident)?
    } else {
        TokenStream::new()
    };

    let event_stripped_enum = if let Some(_) = ident.to_stripped_event_enum() {
        expand_stripped_enum(&input)?
    } else {
        TokenStream::new()
    };

    let redacted_event_enums = if needs_redacted(&full_event_enum) {
        expand_any_redacted_enum_with_deserialize(&input, &full_event_enum)?
    } else {
        TokenStream::new()
    };

    let event_content_enum = expand_content_enum(&input, &ident.to_content_enum())?;

    Ok(quote! {
        #event_enum

        #event_stub_enum

        #event_stripped_enum

        #redacted_event_enums

        #event_content_enum
    })
}

/// Create a "stub" enum from `EventEnumInput`.
pub fn expand_stub_enum(input: &EventEnumInput, ident: &Ident) -> syn::Result<TokenStream> {
    expand_any_enum_with_deserialize(input, &ident)
}

/// Create a "stripped" enum from `EventEnumInput`.
pub fn expand_stripped_enum(input: &EventEnumInput) -> syn::Result<TokenStream> {
    let ident = Ident::new("AnyStrippedStateEventStub", input.name.span());

    expand_any_enum_with_deserialize(input, &ident)
}

/// Generates the 3 redacted state enums, 2 redacted message enums,
/// and `Deserialize` implementations.
///
/// No content enums are generated since no part of the API deals with
/// redacted event's content. There are only five state variants that contain content.
fn expand_any_redacted_enum_with_deserialize(
    input: &EventEnumInput,
    ident: &Ident,
) -> syn::Result<TokenStream> {
    let name = input.name.to_full_event_struct();

    let redacted_enums_deserialize = if ident.to_string().contains("State") {
        let ident = format_ident!("AnyRedacted{}", name);
        let full = expand_any_enum_with_deserialize(input, &ident)?;

        let ident = format_ident!("AnyRedacted{}Stub", name);
        let stub = expand_any_enum_with_deserialize(input, &ident)?;

        let ident = format_ident!("AnyRedactedStripped{}Stub", name);
        let stripped = expand_any_enum_with_deserialize(input, &ident)?;

        quote! {
            #full

            #stub

            #stripped
        }
    } else {
        let ident = format_ident!("AnyRedacted{}", name);
        let full = expand_any_enum_with_deserialize(input, &ident)?;

        let ident = format_ident!("AnyRedacted{}Stub", name);
        let stub = expand_any_enum_with_deserialize(input, &ident)?;

        quote! {
            #full

            #stub
        }
    };

    Ok(quote! {
        #redacted_enums_deserialize
    })
}

/// Create a content enum from `EventEnumInput`.
pub fn expand_content_enum(input: &EventEnumInput, ident: &Ident) -> syn::Result<TokenStream> {
    let attrs = &input.attrs;
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
                            Ok(Self::#variants(content))
                        },
                    )*
                    ev_type => {
                        let content = ::ruma_events::custom::CustomEventContent::from_parts(ev_type, input)?;
                        Ok(Self::Custom(content))
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

    // This needs to remove the `Any` since every enum is generated by this method and
    // they all have an event structs that shares a name with `ident` minus `Any`.
    let event_struct = Ident::new(&ident.to_string().replace("Any", ""), ident.span());

    let variants = input.events.iter().map(to_camel_case).collect::<syn::Result<Vec<_>>>()?;
    let content =
        input.events.iter().map(|event| to_event_path(event, &event_struct)).collect::<Vec<_>>();

    let (custom_variant, custom_deserialize) = expand_custom_variant(ident, &event_struct);

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
            #custom_variant
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
                            Ok(Self::#variants(event))
                        },
                    )*
                    #custom_deserialize
                }
            }
        }
    };

    let field_accessor_impl = accessor_methods(ident, &variants);

    Ok(quote! {
        #any_enum

        #field_accessor_impl

        #event_deserialize_impl
    })
}

fn expand_custom_variant(ident: &Ident, event_struct: &Ident) -> (TokenStream, TokenStream) {
    if ident.to_string().contains("Redacted") {
        (
            quote! {
                /// A redacted event not defined by the Matrix specification
                Custom(::ruma_events::#event_struct<::ruma_events::custom::RedactedCustomEventContent>),
            },
            quote! {
                event => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::#event_struct<::ruma_events::custom::RedactedCustomEventContent>,
                    >(json.get())
                    .map_err(D::Error::custom)?;

                    Ok(Self::Custom(event))
                },
            },
        )
    } else {
        (
            quote! {
                /// An event not defined by the Matrix specification
                Custom(::ruma_events::#event_struct<::ruma_events::custom::CustomEventContent>),
            },
            quote! {
                event => {
                    let event =
                        ::serde_json::from_str::<::ruma_events::#event_struct<::ruma_events::custom::CustomEventContent>>(json.get())
                            .map_err(D::Error::custom)?;

                    Ok(Self::Custom(event))
                },
            },
        )
    }
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

fn accessor_methods(ident: &Ident, variants: &[Ident]) -> TokenStream {
    if ident.to_string().contains("Redacted") {
        return redacted_accessor_methods(ident, variants);
    }

    let fields = EVENT_FIELDS
        .iter()
        .map(|(name, has_field)| generate_accessor(name, ident, *has_field, variants));

    let any_content = ident.to_string().replace("Stub", "").replace("Stripped", "");
    let content_enum = Ident::new(&format!("{}Content", any_content), ident.span());

    let content = quote! {
        /// Returns the any content enum for this event.
        pub fn content(&self) -> #content_enum {
            match self {
                #(
                    Self::#variants(event) => #content_enum::#variants(event.content.clone()),
                )*
                Self::Custom(event) => #content_enum::Custom(event.content.clone()),
            }
        }
    };

    let prev_content = if PREV_CONTENT_KIND.contains(&ident.to_string().as_str()) {
        quote! {
            /// Returns the any content enum for this events prev_content.
            pub fn prev_content(&self) -> Option<#content_enum> {
                match self {
                    #(
                        Self::#variants(event) => {
                            event.prev_content.as_ref().map(|c| #content_enum::#variants(c.clone()))
                        },
                    )*
                    Self::Custom(event) => {
                        event.prev_content.as_ref().map(|c| #content_enum::Custom(c.clone()))
                    },
                }
            }
        }
    } else {
        TokenStream::new()
    };

    quote! {
        impl #ident {
            #content

            #prev_content

            #( #fields )*
        }
    }
}

/// Redacted events do NOT generate `content` or `prev_content` methods like
/// un-redacted events; otherwise, they are the same.
fn redacted_accessor_methods(ident: &Ident, variants: &[Ident]) -> TokenStream {
    let fields = EVENT_FIELDS
        .iter()
        .map(|(name, has_field)| generate_accessor(name, ident, *has_field, variants));

    quote! {
        impl #ident {
            #( #fields )*
        }
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
            let content = format_ident!("{}EventContent", event);
            quote! { ::ruma_events::#struct_name<::ruma_events::#( #path )::*::#content> }
        }
        struct_str if struct_str.contains("Redacted") => {
            let content = format_ident!("Redacted{}EventContent", event);
            quote! { ::ruma_events::#struct_name<::ruma_events::#( #path )::*::#content> }
        }
        _ => {
            let event_name = format_ident!("{}Event", event);
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

    let content_str = format_ident!("{}EventContent", event);
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

fn generate_accessor(
    name: &str,
    ident: &Ident,
    event_kind_list: &[&str],
    variants: &[Ident],
) -> TokenStream {
    if event_kind_list.contains(&ident.to_string().as_str()) {
        let field_type = field_return_type(name);

        let name = Ident::new(name, Span::call_site());
        let docs = format!("Returns this events {} field.", name);
        quote! {
            #[doc = #docs]
            pub fn #name(&self) -> &#field_type {
                match self {
                    #(
                        Self::#variants(event) => &event.#name,
                    )*
                    Self::Custom(event) => &event.#name,
                }
            }
        }
    } else {
        TokenStream::new()
    }
}

/// Returns true if the `ident` is a state or message event.
fn needs_redacted(ident: &Ident) -> bool {
    REDACTED_EVENT_KIND.contains(&ident.to_string().as_str())
}

fn field_return_type(name: &str) -> TokenStream {
    match name {
        "origin_server_ts" => quote! { ::std::time::SystemTime },
        "room_id" => quote! { ::ruma_identifiers::RoomId },
        "event_id" => quote! { ::ruma_identifiers::EventId },
        "sender" => quote! { ::ruma_identifiers::UserId },
        "state_key" => quote! { str },
        "unsigned" => quote! { ::ruma_events::UnsignedData },
        _ => panic!("the `ruma_events_macros::event_enum::EVENT_FIELD` const was changed"),
    }
}

/// Custom keywords for the `event_enum!` macro
mod kw {
    syn::custom_keyword!(kind);
    syn::custom_keyword!(events);
}

pub enum EnumKind {
    Basic(Ident),
    Ephemeral(Ident),
    Message(Ident),
    State(Ident),
    ToDevice(Ident),
}

impl EnumKind {
    /// Return the span of this ident.
    fn span(&self) -> Span {
        match &self {
            Self::Basic(i)
            | Self::Ephemeral(i)
            | Self::Message(i)
            | Self::State(i)
            | Self::ToDevice(i) => i.span(),
        }
    }
    /// `[Kind]Event`
    fn to_full_event_struct(&self) -> Ident {
        match &self {
            Self::Basic(i)
            | Self::Ephemeral(i)
            | Self::Message(i)
            | Self::State(i)
            | Self::ToDevice(i) => format_ident!("{}Event", i),
        }
    }

    /// `Any[Kind]Event`
    fn to_full_event_enum(&self) -> Ident {
        match self {
            Self::Basic(i)
            | Self::Ephemeral(i)
            | Self::Message(i)
            | Self::State(i)
            | Self::ToDevice(i) => format_ident!("Any{}Event", i),
        }
    }

    /// `Any[Kind]EventStub`
    fn to_event_stub_enum(&self) -> Option<Ident> {
        match self {
            Self::Ephemeral(i) | Self::Message(i) | Self::State(i) => {
                Some(format_ident!("Any{}EventStub", i))
            }
            Self::Basic(_) | Self::ToDevice(_) => None,
        }
    }

    /// `AnyStrippedStateEvent`
    fn to_stripped_event_enum(&self) -> Option<Ident> {
        match self {
            Self::State(i) => Some(format_ident!("AnyStripped{}EventStub", i)),
            Self::Ephemeral(_) | Self::Message(_) | Self::Basic(_) | Self::ToDevice(_) => None,
        }
    }

    /// `Any[kind]EventContent`
    fn to_content_enum(&self) -> Ident {
        match self {
            Self::Basic(i)
            | Self::Ephemeral(i)
            | Self::Message(i)
            | Self::State(i)
            | Self::ToDevice(i) => format_ident!("Any{}EventContent", i),
        }
    }
}

impl Parse for EnumKind {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(match input.parse::<Ident>()? {
            i if i.to_string().as_str() == "Basic" => EnumKind::Basic(i),
            i if i.to_string().as_str() == "EphemeralRoom" => EnumKind::Ephemeral(i),
            i if i.to_string().as_str() == "Message" => EnumKind::Message(i),
            i if i.to_string().as_str() == "State" => EnumKind::State(i),
            i if i.to_string().as_str() == "ToDevice" => EnumKind::ToDevice(i),
            id => {
                return Err(syn::Error::new(
                    input.span(),
                    format!(
                        "valid event kinds are Basic, EphemeralRoom, Message, State, ToDevice found `{}`",
                        id
                    ),
                ));
            }
        })
    }
}

/// The entire `event_enum!` macro structure directly as it appears in the source code.
pub struct EventEnumInput {
    /// Outer attributes on the field, such as a docstring.
    pub attrs: Vec<Attribute>,

    /// The name of the event.
    pub name: EnumKind,

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
        input.parse::<kw::kind>()?;
        input.parse::<Token![:]>()?;

        // the name of our event enum
        let name = input.parse::<EnumKind>()?;
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
