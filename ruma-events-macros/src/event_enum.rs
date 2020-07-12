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
    let name = &input.name;
    let events = &input.events;
    let attrs = &input.attrs;

    let event_enum =
        expand_any_enum_with_deserialize(name, events, attrs, &EventKindVariation::Full)?;

    let event_stub_enum =
        expand_any_enum_with_deserialize(name, events, attrs, &EventKindVariation::Stub)?;

    let event_stripped_enum =
        expand_any_enum_with_deserialize(name, events, attrs, &EventKindVariation::Stripped)?;

    let redacted_event_enums = expand_any_redacted_enum_with_deserialize(name, events, attrs)?;

    let event_content_enum = expand_content_enum(name, events, attrs)?;

    Ok(quote! {
        #event_enum

        #event_stub_enum

        #event_stripped_enum

        #redacted_event_enums

        #event_content_enum
    })
}

fn expand_any_enum_with_deserialize(
    name: &EventKind,
    events: &[LitStr],
    attrs: &[Attribute],
    var: &EventKindVariation,
) -> syn::Result<TokenStream> {
    let event_struct = if let Some(i) = name.to_event_ident(var) {
        i
    } else {
        return Ok(TokenStream::new());
    };
    let ident = if let Some(i) = name.to_event_enum_ident(var) {
        i
    } else {
        return Ok(TokenStream::new());
    };

    let attrs = attrs;
    let event_type_str = events;

    let variants = events.iter().map(to_camel_case).collect::<syn::Result<Vec<_>>>()?;
    let content =
        events.iter().map(|event| to_event_path(event, &event_struct)).collect::<Vec<_>>();

    let (custom_variant, custom_deserialize) = expand_custom_variant(&event_struct);

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

    let field_accessor_impl = accessor_methods(&ident, &variants);

    Ok(quote! {
        #any_enum

        #field_accessor_impl

        #event_deserialize_impl
    })
}

/// Generates the 3 redacted state enums, 2 redacted message enums,
/// and `Deserialize` implementations.
///
/// No content enums are generated since no part of the API deals with
/// redacted event's content. There are only five state variants that contain content.
fn expand_any_redacted_enum_with_deserialize(
    name: &EventKind,
    events: &[LitStr],
    attrs: &[Attribute],
) -> syn::Result<TokenStream> {
    if name.is_state() {
        let state_full =
            expand_any_enum_with_deserialize(name, events, attrs, &EventKindVariation::Redacted)?;
        let state_stub = expand_any_enum_with_deserialize(
            name,
            events,
            attrs,
            &EventKindVariation::RedactedStub,
        )?;
        let state_stripped = expand_any_enum_with_deserialize(
            name,
            events,
            attrs,
            &EventKindVariation::RedactedStripped,
        )?;

        Ok(quote! {
            #state_full

            #state_stub

            #state_stripped
        })
    } else if name.is_message() {
        let message_full =
            expand_any_enum_with_deserialize(name, events, attrs, &EventKindVariation::Redacted)?;
        let message_stub = expand_any_enum_with_deserialize(
            name,
            events,
            attrs,
            &EventKindVariation::RedactedStub,
        )?;

        Ok(quote! {
            #message_full

            #message_stub
        })
    } else {
        Ok(TokenStream::new())
    }
}

/// Create a content enum from `EventEnumInput`.
pub fn expand_content_enum(
    name: &EventKind,
    events: &[LitStr],
    attrs: &[Attribute],
) -> syn::Result<TokenStream> {
    let ident = name.to_content_enum();
    let event_type_str = events;

    let variants = events.iter().map(to_camel_case).collect::<syn::Result<Vec<_>>>()?;
    let content = events.iter().map(to_event_content_path).collect::<Vec<_>>();

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

fn expand_custom_variant(event_struct: &Ident) -> (TokenStream, TokenStream) {
    if event_struct.to_string().contains("Redacted") {
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

pub enum EventKindVariation {
    Full,
    Stub,
    Stripped,
    Redacted,
    RedactedStub,
    RedactedStripped,
}

pub enum EventKind {
    Basic(Ident),
    Ephemeral(Ident),
    Message(Ident),
    State(Ident),
    ToDevice(Ident),
}

impl EventKind {
    fn is_state(&self) -> bool {
        if let Self::State(_) = self {
            true
        } else {
            false
        }
    }

    fn is_message(&self) -> bool {
        if let Self::Message(_) = self {
            true
        } else {
            false
        }
    }

    fn to_event_ident(&self, var: &EventKindVariation) -> Option<Ident> {
        use EventKindVariation::*;

        match var {
            // all `EventKind`s are valid event structs
            Full => Some(format_ident!("{}Event", get_ident(self))),
            Stub => match self {
                Self::Ephemeral(i) | Self::State(i) | Self::Message(i) => {
                    Some(format_ident!("{}EventStub", i))
                }
                _ => None,
            },
            Stripped => {
                if let Self::State(i) = self {
                    Some(format_ident!("Stripped{}EventStub", i))
                } else {
                    None
                }
            }
            Redacted => Some(format_ident!("Redacted{}Event", get_ident(self))),
            RedactedStub => match self {
                Self::State(i) | Self::Message(i) => Some(format_ident!("Redacted{}EventStub", i)),
                _ => None,
            },
            RedactedStripped => {
                if let Self::State(i) = self {
                    Some(format_ident!("RedactedStripped{}EventStub", i))
                } else {
                    None
                }
            }
        }
    }

    fn to_event_enum_ident(&self, var: &EventKindVariation) -> Option<Ident> {
        Some(format_ident!("Any{}", self.to_event_ident(var)?))
    }

    /// `Any[kind]EventContent`
    fn to_content_enum(&self) -> Ident {
        format_ident!("Any{}EventContent", get_ident(self))
    }
}

impl Parse for EventKind {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        Ok(match ident.to_string().as_str() {
            "Basic" => EventKind::Basic(ident),
            "EphemeralRoom" => EventKind::Ephemeral(ident),
            "Message" => EventKind::Message(ident),
            "State" => EventKind::State(ident),
            "ToDevice" => EventKind::ToDevice(ident),
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

fn get_ident(ident: &EventKind) -> &Ident {
    match ident {
        EventKind::Basic(i)
        | EventKind::Ephemeral(i)
        | EventKind::Message(i)
        | EventKind::State(i)
        | EventKind::ToDevice(i) => i,
    }
}

/// The entire `event_enum!` macro structure directly as it appears in the source code.
pub struct EventEnumInput {
    /// Outer attributes on the field, such as a docstring.
    pub attrs: Vec<Attribute>,

    /// The name of the event.
    pub name: EventKind,

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
        let name = input.parse::<EventKind>()?;
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
