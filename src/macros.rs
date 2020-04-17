macro_rules! impl_enum {
    ($name:ident { $($variant:ident => $s:expr,)+ }) => {
        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
                let variant = match *self {
                    $($name::$variant => $s,)*
                    $name::__Nonexhaustive => panic!("__Nonexhaustive enum variant is not intended for use."),
                };

                write!(f, "{}", variant)
            }
        }

        impl ::std::str::FromStr for $name {
            type Err = $crate::FromStrError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($s => Ok($name::$variant),)*
                    _ => Err($crate::FromStrError),
                }
            }
        }
    }
}

macro_rules! impl_event {
    ($name:ident, $content_name:ident, $event_type:path) => {
        impl crate::Event for $name {
            /// The type of this event's `content` field.
            type Content = $content_name;

            /// The event's content.
            fn content(&self) -> &Self::Content {
                &self.content
            }

            /// The type of the event.
            fn event_type(&self) -> crate::EventType {
                $event_type
            }
        }
    };
}

macro_rules! impl_room_event {
    ($name:ident, $content_name:ident, $event_type:path) => {
        impl_event!($name, $content_name, $event_type);

        impl crate::RoomEvent for $name {
            /// The unique identifier for the event.
            fn event_id(&self) -> &EventId {
                &self.event_id
            }

            /// Time on originating homeserver when this event was sent.
            fn origin_server_ts(&self) -> ::std::time::SystemTime {
                self.origin_server_ts
            }

            /// The unique identifier for the room associated with this event.
            ///
            /// This can be `None` if the event came from a context where there is
            /// no ambiguity which room it belongs to, like a `/sync` response for example.
            fn room_id(&self) -> Option<&::ruma_identifiers::RoomId> {
                self.room_id.as_ref()
            }

            /// The unique identifier for the user who sent this event.
            fn sender(&self) -> &::ruma_identifiers::UserId {
                &self.sender
            }

            /// Additional key-value pairs not signed by the homeserver.
            fn unsigned(&self) -> &::serde_json::Map<String, ::serde_json::Value> {
                &self.unsigned
            }
        }
    };
}

macro_rules! impl_state_event {
    ($name:ident, $content_name:ident, $event_type:path) => {
        impl_room_event!($name, $content_name, $event_type);

        impl crate::StateEvent for $name {
            /// The previous content for this state key, if any.
            fn prev_content(&self) -> Option<&Self::Content> {
                self.prev_content.as_ref()
            }

            /// A key that determines which piece of room state the event represents.
            fn state_key(&self) -> &str {
                &self.state_key
            }
        }
    };
}

macro_rules! impl_from_for_enum {
    ($self_ty:ident, $inner_ty:ty, $variant:ident) => {
        impl ::std::convert::From<$inner_ty> for $self_ty {
            fn from(event: $inner_ty) -> Self {
                $self_ty::$variant(event)
            }
        }
    };
}
