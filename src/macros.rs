macro_rules! impl_enum {
    ($name:ident { $($variant:ident => $s:expr,)+ }) => {
        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
                let variant = match *self {
                    $($name::$variant => $s,)*
                };

                write!(f, "{}", variant)
            }
        }

        impl ::std::str::FromStr for $name {
            type Err = $crate::ParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($s => Ok($name::$variant),)*
                    _ => Err($crate::ParseError),
                }
            }
        }
    }
}

macro_rules! event {
    (   $(#[$attr:meta])*
        pub struct $name:ident($content_type:ty) {
            $(
                $(#[$field_attr:meta])*
                pub $field_name:ident: $field_type:ty
            ),*
        }
    ) => {
        $(#[$attr])*
        #[derive(Clone, Debug, Deserialize, Serialize)]
        pub struct $name {
            /// The event's content.
            pub content: $content_type,

            /// The type of the event.
            #[serde(rename="type")]
            pub event_type: $crate::EventType,

            $(
                $(#[$field_attr])*
                pub $field_name: $field_type
            ),*
        }

        impl_event!($name, $content_type);
    }
}

macro_rules! impl_event {
    ($name:ident, $content_type:ty) => {
        impl $crate::Event for $name {
            type Content = $content_type;

            fn content(&self) -> &<$name as $crate::Event>::Content {
                &self.content
            }

            fn event_type(&self) -> &$crate::EventType {
                &self.event_type
            }
        }
    }
}

macro_rules! room_event {
    (   $(#[$attr:meta])*
        pub struct $name:ident($content_type:ty) {
            $(
                $(#[$field_attr:meta])*
                pub $field_name:ident: $field_type:ty
            ),*
        }
    ) => {
        $(#[$attr])*
        #[derive(Clone, Debug, Deserialize, Serialize)]
        pub struct $name {
            /// The event's content.
            pub content: $content_type,

            /// The unique identifier for the event.
            pub event_id: ::ruma_identifiers::EventId,

            /// The type of the event.
            #[serde(rename="type")]
            pub event_type: $crate::EventType,

            /// The unique identifier for the room associated with this event.
            pub room_id: ::ruma_identifiers::RoomId,

            /// Additional key-value pairs not signed by the homeserver.
            #[serde(skip_serializing_if="Option::is_none")]
            pub unsigned: Option<::serde_json::Value>,

            /// The unique identifier for the user associated with this event.
            #[serde(rename="sender")]
            pub user_id: ::ruma_identifiers::UserId,

            $(
                $(#[$field_attr])*
                pub $field_name: $field_type
            ),*
        }

        impl_room_event!($name, $content_type);
    }
}

macro_rules! impl_room_event {
    ($name:ident, $content_type:ty) => {
        impl_event!($name, $content_type);

        impl $crate::RoomEvent for $name {
            fn event_id(&self) -> &::ruma_identifiers::EventId {
                &self.event_id
            }

            fn room_id(&self) -> &::ruma_identifiers::RoomId {
                &self.room_id
            }

            fn unsigned(&self) -> Option<&::serde_json::Value> {
                self.unsigned.as_ref()
            }

            fn user_id(&self) -> &::ruma_identifiers::UserId {
                &self.user_id
            }
        }
    }
}

macro_rules! state_event {
    (   $(#[$attr:meta])*
        pub struct $name:ident($content_type:ty) {
            $(
                $(#[$field_attr:meta])*
                pub $field_name:ident: $field_type:ty
            ),*
        }
    ) => {
        $(#[$attr])*
        #[allow(missing_docs)]
        #[derive(Clone, Debug, Deserialize, Serialize)]
        pub struct $name {
            /// The event's content.
            pub content: $content_type,

            /// The unique identifier for the event.
            pub event_id: ::ruma_identifiers::EventId,

            /// The type of the event.
            #[serde(rename="type")]
            pub event_type: $crate::EventType,

            /// The previous content for this state key, if any.
            #[serde(skip_serializing_if="Option::is_none")]
            pub prev_content: Option<$content_type>,

            /// The unique identifier for the room associated with this event.
            pub room_id: ::ruma_identifiers::RoomId,

            /// A key that determines which piece of room state the event represents.
            pub state_key: String,

            /// Additional key-value pairs not signed by the homeserver.
            #[serde(skip_serializing_if="Option::is_none")]
            pub unsigned: Option<::serde_json::Value>,

            /// The unique identifier for the user associated with this event.
            #[serde(rename="sender")]
            pub user_id: ::ruma_identifiers::UserId,

            $(
                $(#[$field_attr])*
                pub $field_name: $field_type
            ),*
        }

        impl_state_event!($name, $content_type);
    }
}

macro_rules! impl_state_event {
    ($name:ident, $content_type:ty) => {
        impl_room_event!($name, $content_type);

        impl $crate::StateEvent for $name {
            fn prev_content(&self) -> Option<&Self::Content> {
                self.prev_content.as_ref()
            }

            fn state_key(&self) -> &str {
                &self.state_key
            }
        }
    }
}
