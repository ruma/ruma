//! Details of generating code for the `ruma_event` procedural macro.

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Attribute, Field, Ident};

use crate::parse::{EventKind, RumaEventInput};

/// The result of processing the `ruma_event` macro, ready for output back to source code.
pub struct RumaEvent {
    /// Outer attributes on the field, such as a docstring.
    attrs: Vec<Attribute>,

    /// The name of the type of the event's `content` field.
    content_name: Ident,

    /// Additional named struct fields in the top level event struct.
    fields: Option<Vec<Field>>,

    /// The kind of event.
    kind: EventKind,

    /// The name of the event.
    name: Ident,
}

impl RumaEvent {
    /// Fills in the event's struct definition with fields common to all basic events.
    fn common_basic_event_fields(&self) -> TokenStream {
        let content_name = &self.content_name;

        quote! {
            /// The event's content.
            pub content: #content_name,
        }
    }

    /// Fills in the event's struct definition with fields common to all room events.
    fn common_room_event_fields(&self) -> TokenStream {
        let common_basic_event_fields = self.common_basic_event_fields();

        quote! {
            #common_basic_event_fields

            /// The unique identifier for the event.
            pub event_id: ruma_identifiers::EventId,

            /// Timestamp (milliseconds since the UNIX epoch) on originating homeserver when this
            /// event was sent.
            pub origin_server_ts: js_int::UInt,

            /// The unique identifier for the room associated with this event.
            pub room_id: Option<ruma_identifiers::RoomId>,

            /// Additional key-value pairs not signed by the homeserver.
            pub unsigned: Option<serde_json::Value>,

            /// The unique identifier for the user who sent this event.
            pub sender: ruma_identifiers::UserId,
        }
    }

    /// Fills in the event's struct definition with fields common to all state events.
    fn common_state_event_fields(&self) -> TokenStream {
        let content_name = &self.content_name;
        let common_room_event_fields = self.common_room_event_fields();

        quote! {
            #common_room_event_fields

            /// The previous content for this state key, if any.
            pub prev_content: Option<#content_name>,

            /// A key that determines which piece of room state the event represents.
            pub state_key: String,
        }
    }
}

impl From<RumaEventInput> for RumaEvent {
    fn from(input: RumaEventInput) -> Self {
        Self {
            attrs: input.attrs,
            content_name: Ident::new(&format!("{}Content", input.name), Span::call_site()),
            fields: input.fields,
            kind: input.kind,
            name: input.name,
        }
    }
}

impl ToTokens for RumaEvent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let attrs = &self.attrs;
        let content_name = &self.content_name;

        let common_event_fields = match self.kind {
            EventKind::Event => self.common_basic_event_fields(),
            EventKind::RoomEvent => self.common_room_event_fields(),
            EventKind::StateEvent => self.common_state_event_fields(),
        };

        let event_fields = match &self.fields {
            Some(fields) => fields.clone(),
            None => vec![],
        };

        let name = &self.name;

        let content_docstring = format!("The payload for `{}`.", name);

        let output = quote!(
            #(#attrs),*
            #[derive(Clone, Debug)]
            pub struct #name {
                #common_event_fields
                #(#event_fields),*
            }

            #[doc = #content_docstring]
            #[derive(Clone, Debug)]
            pub struct #content_name {
            }
        );

        output.to_tokens(tokens);
    }
}
