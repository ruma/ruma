mod enums {
    use crate::{from_raw_json_value, EventDeHelper};
    use ruma_events_macros::event_enum;
    use serde::{de, Serialize};
    use serde_json::value::RawValue as RawJsonValue;
    /// Any basic event.
    #[allow(clippy::large_enum_variant)]
    pub enum AnyBasicEvent {
        ///m.direct
        Direct(::ruma_events::direct::DirectEvent),
        ///m.dummy
        Dummy(::ruma_events::dummy::DummyEvent),
        ///m.ignored_user_list
        IgnoredUserList(::ruma_events::ignored_user_list::IgnoredUserListEvent),
        ///m.presence
        Presence(::ruma_events::presence::PresenceEvent),
        ///m.push_rules
        PushRules(::ruma_events::push_rules::PushRulesEvent),
        ///m.room_key
        RoomKey(::ruma_events::room_key::RoomKeyEvent),
        ///m.tag
        Tag(::ruma_events::tag::TagEvent),
        /// An event not defined by the Matrix specification
        Custom(::ruma_events::BasicEvent<::ruma_events::custom::CustomEventContent>),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::clone::Clone for AnyBasicEvent {
        #[inline]
        fn clone(&self) -> AnyBasicEvent {
            match (&*self,) {
                (&AnyBasicEvent::Direct(ref __self_0),) => {
                    AnyBasicEvent::Direct(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyBasicEvent::Dummy(ref __self_0),) => {
                    AnyBasicEvent::Dummy(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyBasicEvent::IgnoredUserList(ref __self_0),) => {
                    AnyBasicEvent::IgnoredUserList(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyBasicEvent::Presence(ref __self_0),) => {
                    AnyBasicEvent::Presence(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyBasicEvent::PushRules(ref __self_0),) => {
                    AnyBasicEvent::PushRules(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyBasicEvent::RoomKey(ref __self_0),) => {
                    AnyBasicEvent::RoomKey(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyBasicEvent::Tag(ref __self_0),) => {
                    AnyBasicEvent::Tag(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyBasicEvent::Custom(ref __self_0),) => {
                    AnyBasicEvent::Custom(::core::clone::Clone::clone(&(*__self_0)))
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::fmt::Debug for AnyBasicEvent {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&AnyBasicEvent::Direct(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Direct");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyBasicEvent::Dummy(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Dummy");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyBasicEvent::IgnoredUserList(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("IgnoredUserList");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyBasicEvent::Presence(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Presence");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyBasicEvent::PushRules(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("PushRules");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyBasicEvent::RoomKey(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomKey");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyBasicEvent::Tag(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Tag");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyBasicEvent::Custom(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Custom");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(rust_2018_idioms, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for AnyBasicEvent {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::export::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    AnyBasicEvent::Direct(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyBasicEvent::Dummy(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyBasicEvent::IgnoredUserList(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyBasicEvent::Presence(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyBasicEvent::PushRules(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyBasicEvent::RoomKey(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyBasicEvent::Tag(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyBasicEvent::Custom(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                }
            }
        }
    };
    impl AnyBasicEvent {
        /// Returns the any content enum for this event.
        pub fn content(&self) -> AnyBasicEventContent {
            match self {
                Self::Direct(event) => AnyBasicEventContent::Direct(event.content.clone()),
                Self::Dummy(event) => AnyBasicEventContent::Dummy(event.content.clone()),
                Self::IgnoredUserList(event) => {
                    AnyBasicEventContent::IgnoredUserList(event.content.clone())
                }
                Self::Presence(event) => AnyBasicEventContent::Presence(event.content.clone()),
                Self::PushRules(event) => AnyBasicEventContent::PushRules(event.content.clone()),
                Self::RoomKey(event) => AnyBasicEventContent::RoomKey(event.content.clone()),
                Self::Tag(event) => AnyBasicEventContent::Tag(event.content.clone()),
                Self::Custom(event) => AnyBasicEventContent::Custom(event.content.clone()),
            }
        }
    }
    impl<'de> ::serde::de::Deserialize<'de> for AnyBasicEvent {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: ::serde::de::Deserializer<'de>,
        {
            use ::serde::de::Error as _;
            let json = Box::<::serde_json::value::RawValue>::deserialize(deserializer)?;
            let ::ruma_events::EventDeHelper { ev_type, .. } =
                ::ruma_events::from_raw_json_value(&json)?;
            match ev_type.as_str() {
                "m.direct" => {
                    let event =
                        ::serde_json::from_str::<::ruma_events::direct::DirectEvent>(json.get())
                            .map_err(D::Error::custom)?;
                    Ok(Self::Direct(event))
                }
                "m.dummy" => {
                    let event =
                        ::serde_json::from_str::<::ruma_events::dummy::DummyEvent>(json.get())
                            .map_err(D::Error::custom)?;
                    Ok(Self::Dummy(event))
                }
                "m.ignored_user_list" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::ignored_user_list::IgnoredUserListEvent,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::IgnoredUserList(event))
                }
                "m.presence" => {
                    let event = ::serde_json::from_str::<::ruma_events::presence::PresenceEvent>(
                        json.get(),
                    )
                    .map_err(D::Error::custom)?;
                    Ok(Self::Presence(event))
                }
                "m.push_rules" => {
                    let event =
                        ::serde_json::from_str::<::ruma_events::push_rules::PushRulesEvent>(
                            json.get(),
                        )
                        .map_err(D::Error::custom)?;
                    Ok(Self::PushRules(event))
                }
                "m.room_key" => {
                    let event =
                        ::serde_json::from_str::<::ruma_events::room_key::RoomKeyEvent>(json.get())
                            .map_err(D::Error::custom)?;
                    Ok(Self::RoomKey(event))
                }
                "m.tag" => {
                    let event = ::serde_json::from_str::<::ruma_events::tag::TagEvent>(json.get())
                        .map_err(D::Error::custom)?;
                    Ok(Self::Tag(event))
                }
                event => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::BasicEvent<::ruma_events::custom::CustomEventContent>,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::Custom(event))
                }
            }
        }
    }
    /// Any basic event.
    #[allow(clippy::large_enum_variant)]
    pub enum AnyBasicEventContent {
        ///m.direct
        Direct(::ruma_events::direct::DirectEventContent),
        ///m.dummy
        Dummy(::ruma_events::dummy::DummyEventContent),
        ///m.ignored_user_list
        IgnoredUserList(::ruma_events::ignored_user_list::IgnoredUserListEventContent),
        ///m.presence
        Presence(::ruma_events::presence::PresenceEventContent),
        ///m.push_rules
        PushRules(::ruma_events::push_rules::PushRulesEventContent),
        ///m.room_key
        RoomKey(::ruma_events::room_key::RoomKeyEventContent),
        ///m.tag
        Tag(::ruma_events::tag::TagEventContent),
        /// Content of an event not defined by the Matrix specification.
        Custom(::ruma_events::custom::CustomEventContent),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::clone::Clone for AnyBasicEventContent {
        #[inline]
        fn clone(&self) -> AnyBasicEventContent {
            match (&*self,) {
                (&AnyBasicEventContent::Direct(ref __self_0),) => {
                    AnyBasicEventContent::Direct(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyBasicEventContent::Dummy(ref __self_0),) => {
                    AnyBasicEventContent::Dummy(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyBasicEventContent::IgnoredUserList(ref __self_0),) => {
                    AnyBasicEventContent::IgnoredUserList(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyBasicEventContent::Presence(ref __self_0),) => {
                    AnyBasicEventContent::Presence(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyBasicEventContent::PushRules(ref __self_0),) => {
                    AnyBasicEventContent::PushRules(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyBasicEventContent::RoomKey(ref __self_0),) => {
                    AnyBasicEventContent::RoomKey(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyBasicEventContent::Tag(ref __self_0),) => {
                    AnyBasicEventContent::Tag(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyBasicEventContent::Custom(ref __self_0),) => {
                    AnyBasicEventContent::Custom(::core::clone::Clone::clone(&(*__self_0)))
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::fmt::Debug for AnyBasicEventContent {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&AnyBasicEventContent::Direct(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Direct");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyBasicEventContent::Dummy(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Dummy");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyBasicEventContent::IgnoredUserList(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("IgnoredUserList");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyBasicEventContent::Presence(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Presence");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyBasicEventContent::PushRules(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("PushRules");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyBasicEventContent::RoomKey(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomKey");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyBasicEventContent::Tag(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Tag");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyBasicEventContent::Custom(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Custom");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(rust_2018_idioms, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for AnyBasicEventContent {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::export::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    AnyBasicEventContent::Direct(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyBasicEventContent::Dummy(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyBasicEventContent::IgnoredUserList(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyBasicEventContent::Presence(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyBasicEventContent::PushRules(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyBasicEventContent::RoomKey(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyBasicEventContent::Tag(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyBasicEventContent::Custom(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                }
            }
        }
    };
    impl ::ruma_events::EventContent for AnyBasicEventContent {
        fn event_type(&self) -> &str {
            match self {
                Self::Direct(content) => content.event_type(),
                Self::Dummy(content) => content.event_type(),
                Self::IgnoredUserList(content) => content.event_type(),
                Self::Presence(content) => content.event_type(),
                Self::PushRules(content) => content.event_type(),
                Self::RoomKey(content) => content.event_type(),
                Self::Tag(content) => content.event_type(),
                Self::Custom(content) => content.event_type(),
            }
        }
        fn from_parts(
            event_type: &str,
            input: Box<::serde_json::value::RawValue>,
        ) -> Result<Self, ::serde_json::Error> {
            match event_type {
                "m.direct" => {
                    let content =
                        ::ruma_events::direct::DirectEventContent::from_parts(event_type, input)?;
                    Ok(Self::Direct(content))
                }
                "m.dummy" => {
                    let content =
                        ::ruma_events::dummy::DummyEventContent::from_parts(event_type, input)?;
                    Ok(Self::Dummy(content))
                }
                "m.ignored_user_list" => {
                    let content =
                        ::ruma_events::ignored_user_list::IgnoredUserListEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::IgnoredUserList(content))
                }
                "m.presence" => {
                    let content = ::ruma_events::presence::PresenceEventContent::from_parts(
                        event_type, input,
                    )?;
                    Ok(Self::Presence(content))
                }
                "m.push_rules" => {
                    let content = ::ruma_events::push_rules::PushRulesEventContent::from_parts(
                        event_type, input,
                    )?;
                    Ok(Self::PushRules(content))
                }
                "m.room_key" => {
                    let content = ::ruma_events::room_key::RoomKeyEventContent::from_parts(
                        event_type, input,
                    )?;
                    Ok(Self::RoomKey(content))
                }
                "m.tag" => {
                    let content =
                        ::ruma_events::tag::TagEventContent::from_parts(event_type, input)?;
                    Ok(Self::Tag(content))
                }
                ev_type => {
                    let content =
                        ::ruma_events::custom::CustomEventContent::from_parts(ev_type, input)?;
                    Ok(Self::Custom(content))
                }
            }
        }
    }
    impl ::ruma_events::BasicEventContent for AnyBasicEventContent {}
    /// Any ephemeral room event.
    #[allow(clippy::large_enum_variant)]
    pub enum AnyEphemeralRoomEvent {
        ///m.fully_read
        FullyRead(::ruma_events::fully_read::FullyReadEvent),
        ///m.receipt
        Receipt(::ruma_events::receipt::ReceiptEvent),
        ///m.typing
        Typing(::ruma_events::typing::TypingEvent),
        /// An event not defined by the Matrix specification
        Custom(::ruma_events::EphemeralRoomEvent<::ruma_events::custom::CustomEventContent>),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::clone::Clone for AnyEphemeralRoomEvent {
        #[inline]
        fn clone(&self) -> AnyEphemeralRoomEvent {
            match (&*self,) {
                (&AnyEphemeralRoomEvent::FullyRead(ref __self_0),) => {
                    AnyEphemeralRoomEvent::FullyRead(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyEphemeralRoomEvent::Receipt(ref __self_0),) => {
                    AnyEphemeralRoomEvent::Receipt(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyEphemeralRoomEvent::Typing(ref __self_0),) => {
                    AnyEphemeralRoomEvent::Typing(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyEphemeralRoomEvent::Custom(ref __self_0),) => {
                    AnyEphemeralRoomEvent::Custom(::core::clone::Clone::clone(&(*__self_0)))
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::fmt::Debug for AnyEphemeralRoomEvent {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&AnyEphemeralRoomEvent::FullyRead(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("FullyRead");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyEphemeralRoomEvent::Receipt(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Receipt");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyEphemeralRoomEvent::Typing(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Typing");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyEphemeralRoomEvent::Custom(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Custom");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(rust_2018_idioms, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for AnyEphemeralRoomEvent {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::export::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    AnyEphemeralRoomEvent::FullyRead(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyEphemeralRoomEvent::Receipt(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyEphemeralRoomEvent::Typing(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyEphemeralRoomEvent::Custom(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                }
            }
        }
    };
    impl AnyEphemeralRoomEvent {
        /// Returns the any content enum for this event.
        pub fn content(&self) -> AnyEphemeralRoomEventContent {
            match self {
                Self::FullyRead(event) => {
                    AnyEphemeralRoomEventContent::FullyRead(event.content.clone())
                }
                Self::Receipt(event) => {
                    AnyEphemeralRoomEventContent::Receipt(event.content.clone())
                }
                Self::Typing(event) => AnyEphemeralRoomEventContent::Typing(event.content.clone()),
                Self::Custom(event) => AnyEphemeralRoomEventContent::Custom(event.content.clone()),
            }
        }
        ///Returns this events room_id field.
        pub fn room_id(&self) -> &::ruma_identifiers::RoomId {
            match self {
                Self::FullyRead(event) => &event.room_id,
                Self::Receipt(event) => &event.room_id,
                Self::Typing(event) => &event.room_id,
                Self::Custom(event) => &event.room_id,
            }
        }
    }
    impl<'de> ::serde::de::Deserialize<'de> for AnyEphemeralRoomEvent {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: ::serde::de::Deserializer<'de>,
        {
            use ::serde::de::Error as _;
            let json = Box::<::serde_json::value::RawValue>::deserialize(deserializer)?;
            let ::ruma_events::EventDeHelper { ev_type, .. } =
                ::ruma_events::from_raw_json_value(&json)?;
            match ev_type.as_str() {
                "m.fully_read" => {
                    let event =
                        ::serde_json::from_str::<::ruma_events::fully_read::FullyReadEvent>(
                            json.get(),
                        )
                        .map_err(D::Error::custom)?;
                    Ok(Self::FullyRead(event))
                }
                "m.receipt" => {
                    let event =
                        ::serde_json::from_str::<::ruma_events::receipt::ReceiptEvent>(json.get())
                            .map_err(D::Error::custom)?;
                    Ok(Self::Receipt(event))
                }
                "m.typing" => {
                    let event =
                        ::serde_json::from_str::<::ruma_events::typing::TypingEvent>(json.get())
                            .map_err(D::Error::custom)?;
                    Ok(Self::Typing(event))
                }
                event => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::EphemeralRoomEvent<
                            ::ruma_events::custom::CustomEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::Custom(event))
                }
            }
        }
    }
    /// Any ephemeral room event.
    #[allow(clippy::large_enum_variant)]
    pub enum AnyEphemeralRoomEventStub {
        ///m.fully_read
        FullyRead(::ruma_events::fully_read::FullyReadEvent),
        ///m.receipt
        Receipt(::ruma_events::receipt::ReceiptEvent),
        ///m.typing
        Typing(::ruma_events::typing::TypingEvent),
        /// An event not defined by the Matrix specification
        Custom(::ruma_events::EphemeralRoomEventStub<::ruma_events::custom::CustomEventContent>),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::clone::Clone for AnyEphemeralRoomEventStub {
        #[inline]
        fn clone(&self) -> AnyEphemeralRoomEventStub {
            match (&*self,) {
                (&AnyEphemeralRoomEventStub::FullyRead(ref __self_0),) => {
                    AnyEphemeralRoomEventStub::FullyRead(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyEphemeralRoomEventStub::Receipt(ref __self_0),) => {
                    AnyEphemeralRoomEventStub::Receipt(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyEphemeralRoomEventStub::Typing(ref __self_0),) => {
                    AnyEphemeralRoomEventStub::Typing(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyEphemeralRoomEventStub::Custom(ref __self_0),) => {
                    AnyEphemeralRoomEventStub::Custom(::core::clone::Clone::clone(&(*__self_0)))
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::fmt::Debug for AnyEphemeralRoomEventStub {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&AnyEphemeralRoomEventStub::FullyRead(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("FullyRead");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyEphemeralRoomEventStub::Receipt(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Receipt");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyEphemeralRoomEventStub::Typing(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Typing");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyEphemeralRoomEventStub::Custom(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Custom");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(rust_2018_idioms, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for AnyEphemeralRoomEventStub {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::export::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    AnyEphemeralRoomEventStub::FullyRead(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyEphemeralRoomEventStub::Receipt(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyEphemeralRoomEventStub::Typing(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyEphemeralRoomEventStub::Custom(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                }
            }
        }
    };
    impl AnyEphemeralRoomEventStub {
        /// Returns the any content enum for this event.
        pub fn content(&self) -> AnyEphemeralRoomEventContent {
            match self {
                Self::FullyRead(event) => {
                    AnyEphemeralRoomEventContent::FullyRead(event.content.clone())
                }
                Self::Receipt(event) => {
                    AnyEphemeralRoomEventContent::Receipt(event.content.clone())
                }
                Self::Typing(event) => AnyEphemeralRoomEventContent::Typing(event.content.clone()),
                Self::Custom(event) => AnyEphemeralRoomEventContent::Custom(event.content.clone()),
            }
        }
    }
    impl<'de> ::serde::de::Deserialize<'de> for AnyEphemeralRoomEventStub {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: ::serde::de::Deserializer<'de>,
        {
            use ::serde::de::Error as _;
            let json = Box::<::serde_json::value::RawValue>::deserialize(deserializer)?;
            let ::ruma_events::EventDeHelper { ev_type, .. } =
                ::ruma_events::from_raw_json_value(&json)?;
            match ev_type.as_str() {
                "m.fully_read" => {
                    let event =
                        ::serde_json::from_str::<::ruma_events::fully_read::FullyReadEvent>(
                            json.get(),
                        )
                        .map_err(D::Error::custom)?;
                    Ok(Self::FullyRead(event))
                }
                "m.receipt" => {
                    let event =
                        ::serde_json::from_str::<::ruma_events::receipt::ReceiptEvent>(json.get())
                            .map_err(D::Error::custom)?;
                    Ok(Self::Receipt(event))
                }
                "m.typing" => {
                    let event =
                        ::serde_json::from_str::<::ruma_events::typing::TypingEvent>(json.get())
                            .map_err(D::Error::custom)?;
                    Ok(Self::Typing(event))
                }
                event => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::EphemeralRoomEventStub<
                            ::ruma_events::custom::CustomEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::Custom(event))
                }
            }
        }
    }
    /// Any ephemeral room event.
    #[allow(clippy::large_enum_variant)]
    pub enum AnyEphemeralRoomEventContent {
        ///m.fully_read
        FullyRead(::ruma_events::fully_read::FullyReadEventContent),
        ///m.receipt
        Receipt(::ruma_events::receipt::ReceiptEventContent),
        ///m.typing
        Typing(::ruma_events::typing::TypingEventContent),
        /// Content of an event not defined by the Matrix specification.
        Custom(::ruma_events::custom::CustomEventContent),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::clone::Clone for AnyEphemeralRoomEventContent {
        #[inline]
        fn clone(&self) -> AnyEphemeralRoomEventContent {
            match (&*self,) {
                (&AnyEphemeralRoomEventContent::FullyRead(ref __self_0),) => {
                    AnyEphemeralRoomEventContent::FullyRead(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyEphemeralRoomEventContent::Receipt(ref __self_0),) => {
                    AnyEphemeralRoomEventContent::Receipt(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyEphemeralRoomEventContent::Typing(ref __self_0),) => {
                    AnyEphemeralRoomEventContent::Typing(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyEphemeralRoomEventContent::Custom(ref __self_0),) => {
                    AnyEphemeralRoomEventContent::Custom(::core::clone::Clone::clone(&(*__self_0)))
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::fmt::Debug for AnyEphemeralRoomEventContent {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&AnyEphemeralRoomEventContent::FullyRead(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("FullyRead");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyEphemeralRoomEventContent::Receipt(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Receipt");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyEphemeralRoomEventContent::Typing(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Typing");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyEphemeralRoomEventContent::Custom(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Custom");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(rust_2018_idioms, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for AnyEphemeralRoomEventContent {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::export::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    AnyEphemeralRoomEventContent::FullyRead(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyEphemeralRoomEventContent::Receipt(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyEphemeralRoomEventContent::Typing(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyEphemeralRoomEventContent::Custom(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                }
            }
        }
    };
    impl ::ruma_events::EventContent for AnyEphemeralRoomEventContent {
        fn event_type(&self) -> &str {
            match self {
                Self::FullyRead(content) => content.event_type(),
                Self::Receipt(content) => content.event_type(),
                Self::Typing(content) => content.event_type(),
                Self::Custom(content) => content.event_type(),
            }
        }
        fn from_parts(
            event_type: &str,
            input: Box<::serde_json::value::RawValue>,
        ) -> Result<Self, ::serde_json::Error> {
            match event_type {
                "m.fully_read" => {
                    let content = ::ruma_events::fully_read::FullyReadEventContent::from_parts(
                        event_type, input,
                    )?;
                    Ok(Self::FullyRead(content))
                }
                "m.receipt" => {
                    let content =
                        ::ruma_events::receipt::ReceiptEventContent::from_parts(event_type, input)?;
                    Ok(Self::Receipt(content))
                }
                "m.typing" => {
                    let content =
                        ::ruma_events::typing::TypingEventContent::from_parts(event_type, input)?;
                    Ok(Self::Typing(content))
                }
                ev_type => {
                    let content =
                        ::ruma_events::custom::CustomEventContent::from_parts(ev_type, input)?;
                    Ok(Self::Custom(content))
                }
            }
        }
    }
    impl ::ruma_events::EphemeralRoomEventContent for AnyEphemeralRoomEventContent {}
    /// Any message event.
    #[allow(clippy::large_enum_variant)]
    pub enum AnyMessageEvent {
        ///m.call.answer
        CallAnswer(::ruma_events::call::answer::AnswerEvent),
        ///m.call.invite
        CallInvite(::ruma_events::call::invite::InviteEvent),
        ///m.call.hangup
        CallHangup(::ruma_events::call::hangup::HangupEvent),
        ///m.call.candidates
        CallCandidates(::ruma_events::call::candidates::CandidatesEvent),
        ///m.room.encrypted
        RoomEncrypted(::ruma_events::room::encrypted::EncryptedEvent),
        ///m.room.message
        RoomMessage(::ruma_events::room::message::MessageEvent),
        ///m.room.message.feedback
        RoomMessageFeedback(::ruma_events::room::message::feedback::FeedbackEvent),
        ///m.room.redaction
        RoomRedaction(::ruma_events::room::redaction::RedactionEvent),
        ///m.sticker
        Sticker(::ruma_events::sticker::StickerEvent),
        /// An event not defined by the Matrix specification
        Custom(::ruma_events::MessageEvent<::ruma_events::custom::CustomEventContent>),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::clone::Clone for AnyMessageEvent {
        #[inline]
        fn clone(&self) -> AnyMessageEvent {
            match (&*self,) {
                (&AnyMessageEvent::CallAnswer(ref __self_0),) => {
                    AnyMessageEvent::CallAnswer(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyMessageEvent::CallInvite(ref __self_0),) => {
                    AnyMessageEvent::CallInvite(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyMessageEvent::CallHangup(ref __self_0),) => {
                    AnyMessageEvent::CallHangup(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyMessageEvent::CallCandidates(ref __self_0),) => {
                    AnyMessageEvent::CallCandidates(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyMessageEvent::RoomEncrypted(ref __self_0),) => {
                    AnyMessageEvent::RoomEncrypted(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyMessageEvent::RoomMessage(ref __self_0),) => {
                    AnyMessageEvent::RoomMessage(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyMessageEvent::RoomMessageFeedback(ref __self_0),) => {
                    AnyMessageEvent::RoomMessageFeedback(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyMessageEvent::RoomRedaction(ref __self_0),) => {
                    AnyMessageEvent::RoomRedaction(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyMessageEvent::Sticker(ref __self_0),) => {
                    AnyMessageEvent::Sticker(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyMessageEvent::Custom(ref __self_0),) => {
                    AnyMessageEvent::Custom(::core::clone::Clone::clone(&(*__self_0)))
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::fmt::Debug for AnyMessageEvent {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&AnyMessageEvent::CallAnswer(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("CallAnswer");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyMessageEvent::CallInvite(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("CallInvite");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyMessageEvent::CallHangup(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("CallHangup");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyMessageEvent::CallCandidates(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("CallCandidates");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyMessageEvent::RoomEncrypted(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomEncrypted");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyMessageEvent::RoomMessage(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomMessage");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyMessageEvent::RoomMessageFeedback(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomMessageFeedback");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyMessageEvent::RoomRedaction(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomRedaction");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyMessageEvent::Sticker(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Sticker");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyMessageEvent::Custom(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Custom");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(rust_2018_idioms, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for AnyMessageEvent {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::export::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    AnyMessageEvent::CallAnswer(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyMessageEvent::CallInvite(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyMessageEvent::CallHangup(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyMessageEvent::CallCandidates(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyMessageEvent::RoomEncrypted(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyMessageEvent::RoomMessage(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyMessageEvent::RoomMessageFeedback(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyMessageEvent::RoomRedaction(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyMessageEvent::Sticker(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyMessageEvent::Custom(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                }
            }
        }
    };
    impl AnyMessageEvent {
        /// Returns the any content enum for this event.
        pub fn content(&self) -> AnyMessageEventContent {
            match self {
                Self::CallAnswer(event) => {
                    AnyMessageEventContent::CallAnswer(event.content.clone())
                }
                Self::CallInvite(event) => {
                    AnyMessageEventContent::CallInvite(event.content.clone())
                }
                Self::CallHangup(event) => {
                    AnyMessageEventContent::CallHangup(event.content.clone())
                }
                Self::CallCandidates(event) => {
                    AnyMessageEventContent::CallCandidates(event.content.clone())
                }
                Self::RoomEncrypted(event) => {
                    AnyMessageEventContent::RoomEncrypted(event.content.clone())
                }
                Self::RoomMessage(event) => {
                    AnyMessageEventContent::RoomMessage(event.content.clone())
                }
                Self::RoomMessageFeedback(event) => {
                    AnyMessageEventContent::RoomMessageFeedback(event.content.clone())
                }
                Self::RoomRedaction(event) => {
                    AnyMessageEventContent::RoomRedaction(event.content.clone())
                }
                Self::Sticker(event) => AnyMessageEventContent::Sticker(event.content.clone()),
                Self::Custom(event) => AnyMessageEventContent::Custom(event.content.clone()),
            }
        }
        ///Returns this events origin_server_ts field.
        pub fn origin_server_ts(&self) -> &::std::time::SystemTime {
            match self {
                Self::CallAnswer(event) => &event.origin_server_ts,
                Self::CallInvite(event) => &event.origin_server_ts,
                Self::CallHangup(event) => &event.origin_server_ts,
                Self::CallCandidates(event) => &event.origin_server_ts,
                Self::RoomEncrypted(event) => &event.origin_server_ts,
                Self::RoomMessage(event) => &event.origin_server_ts,
                Self::RoomMessageFeedback(event) => &event.origin_server_ts,
                Self::RoomRedaction(event) => &event.origin_server_ts,
                Self::Sticker(event) => &event.origin_server_ts,
                Self::Custom(event) => &event.origin_server_ts,
            }
        }
        ///Returns this events room_id field.
        pub fn room_id(&self) -> &::ruma_identifiers::RoomId {
            match self {
                Self::CallAnswer(event) => &event.room_id,
                Self::CallInvite(event) => &event.room_id,
                Self::CallHangup(event) => &event.room_id,
                Self::CallCandidates(event) => &event.room_id,
                Self::RoomEncrypted(event) => &event.room_id,
                Self::RoomMessage(event) => &event.room_id,
                Self::RoomMessageFeedback(event) => &event.room_id,
                Self::RoomRedaction(event) => &event.room_id,
                Self::Sticker(event) => &event.room_id,
                Self::Custom(event) => &event.room_id,
            }
        }
        ///Returns this events event_id field.
        pub fn event_id(&self) -> &::ruma_identifiers::EventId {
            match self {
                Self::CallAnswer(event) => &event.event_id,
                Self::CallInvite(event) => &event.event_id,
                Self::CallHangup(event) => &event.event_id,
                Self::CallCandidates(event) => &event.event_id,
                Self::RoomEncrypted(event) => &event.event_id,
                Self::RoomMessage(event) => &event.event_id,
                Self::RoomMessageFeedback(event) => &event.event_id,
                Self::RoomRedaction(event) => &event.event_id,
                Self::Sticker(event) => &event.event_id,
                Self::Custom(event) => &event.event_id,
            }
        }
        ///Returns this events sender field.
        pub fn sender(&self) -> &::ruma_identifiers::UserId {
            match self {
                Self::CallAnswer(event) => &event.sender,
                Self::CallInvite(event) => &event.sender,
                Self::CallHangup(event) => &event.sender,
                Self::CallCandidates(event) => &event.sender,
                Self::RoomEncrypted(event) => &event.sender,
                Self::RoomMessage(event) => &event.sender,
                Self::RoomMessageFeedback(event) => &event.sender,
                Self::RoomRedaction(event) => &event.sender,
                Self::Sticker(event) => &event.sender,
                Self::Custom(event) => &event.sender,
            }
        }
        ///Returns this events unsigned field.
        pub fn unsigned(&self) -> &::ruma_events::UnsignedData {
            match self {
                Self::CallAnswer(event) => &event.unsigned,
                Self::CallInvite(event) => &event.unsigned,
                Self::CallHangup(event) => &event.unsigned,
                Self::CallCandidates(event) => &event.unsigned,
                Self::RoomEncrypted(event) => &event.unsigned,
                Self::RoomMessage(event) => &event.unsigned,
                Self::RoomMessageFeedback(event) => &event.unsigned,
                Self::RoomRedaction(event) => &event.unsigned,
                Self::Sticker(event) => &event.unsigned,
                Self::Custom(event) => &event.unsigned,
            }
        }
    }
    impl<'de> ::serde::de::Deserialize<'de> for AnyMessageEvent {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: ::serde::de::Deserializer<'de>,
        {
            use ::serde::de::Error as _;
            let json = Box::<::serde_json::value::RawValue>::deserialize(deserializer)?;
            let ::ruma_events::EventDeHelper { ev_type, .. } =
                ::ruma_events::from_raw_json_value(&json)?;
            match ev_type.as_str() {
                "m.call.answer" => {
                    let event = ::serde_json::from_str::<::ruma_events::call::answer::AnswerEvent>(
                        json.get(),
                    )
                    .map_err(D::Error::custom)?;
                    Ok(Self::CallAnswer(event))
                }
                "m.call.invite" => {
                    let event = ::serde_json::from_str::<::ruma_events::call::invite::InviteEvent>(
                        json.get(),
                    )
                    .map_err(D::Error::custom)?;
                    Ok(Self::CallInvite(event))
                }
                "m.call.hangup" => {
                    let event = ::serde_json::from_str::<::ruma_events::call::hangup::HangupEvent>(
                        json.get(),
                    )
                    .map_err(D::Error::custom)?;
                    Ok(Self::CallHangup(event))
                }
                "m.call.candidates" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::call::candidates::CandidatesEvent,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::CallCandidates(event))
                }
                "m.room.encrypted" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::room::encrypted::EncryptedEvent,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomEncrypted(event))
                }
                "m.room.message" => {
                    let event =
                        ::serde_json::from_str::<::ruma_events::room::message::MessageEvent>(
                            json.get(),
                        )
                        .map_err(D::Error::custom)?;
                    Ok(Self::RoomMessage(event))
                }
                "m.room.message.feedback" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::room::message::feedback::FeedbackEvent,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomMessageFeedback(event))
                }
                "m.room.redaction" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::room::redaction::RedactionEvent,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomRedaction(event))
                }
                "m.sticker" => {
                    let event =
                        ::serde_json::from_str::<::ruma_events::sticker::StickerEvent>(json.get())
                            .map_err(D::Error::custom)?;
                    Ok(Self::Sticker(event))
                }
                event => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::MessageEvent<::ruma_events::custom::CustomEventContent>,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::Custom(event))
                }
            }
        }
    }
    /// Any message event.
    #[allow(clippy::large_enum_variant)]
    pub enum AnyMessageEventStub {
        ///m.call.answer
        CallAnswer(
            ::ruma_events::MessageEventStub<::ruma_events::call::answer::AnswerEventContent>,
        ),
        ///m.call.invite
        CallInvite(
            ::ruma_events::MessageEventStub<::ruma_events::call::invite::InviteEventContent>,
        ),
        ///m.call.hangup
        CallHangup(
            ::ruma_events::MessageEventStub<::ruma_events::call::hangup::HangupEventContent>,
        ),
        ///m.call.candidates
        CallCandidates(
            ::ruma_events::MessageEventStub<
                ::ruma_events::call::candidates::CandidatesEventContent,
            >,
        ),
        ///m.room.encrypted
        RoomEncrypted(
            ::ruma_events::MessageEventStub<::ruma_events::room::encrypted::EncryptedEventContent>,
        ),
        ///m.room.message
        RoomMessage(
            ::ruma_events::MessageEventStub<::ruma_events::room::message::MessageEventContent>,
        ),
        ///m.room.message.feedback
        RoomMessageFeedback(
            ::ruma_events::MessageEventStub<
                ::ruma_events::room::message::feedback::FeedbackEventContent,
            >,
        ),
        ///m.room.redaction
        RoomRedaction(::ruma_events::room::redaction::RedactionEventStub),
        ///m.sticker
        Sticker(::ruma_events::MessageEventStub<::ruma_events::sticker::StickerEventContent>),
        /// An event not defined by the Matrix specification
        Custom(::ruma_events::MessageEventStub<::ruma_events::custom::CustomEventContent>),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::clone::Clone for AnyMessageEventStub {
        #[inline]
        fn clone(&self) -> AnyMessageEventStub {
            match (&*self,) {
                (&AnyMessageEventStub::CallAnswer(ref __self_0),) => {
                    AnyMessageEventStub::CallAnswer(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyMessageEventStub::CallInvite(ref __self_0),) => {
                    AnyMessageEventStub::CallInvite(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyMessageEventStub::CallHangup(ref __self_0),) => {
                    AnyMessageEventStub::CallHangup(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyMessageEventStub::CallCandidates(ref __self_0),) => {
                    AnyMessageEventStub::CallCandidates(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyMessageEventStub::RoomEncrypted(ref __self_0),) => {
                    AnyMessageEventStub::RoomEncrypted(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyMessageEventStub::RoomMessage(ref __self_0),) => {
                    AnyMessageEventStub::RoomMessage(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyMessageEventStub::RoomMessageFeedback(ref __self_0),) => {
                    AnyMessageEventStub::RoomMessageFeedback(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyMessageEventStub::RoomRedaction(ref __self_0),) => {
                    AnyMessageEventStub::RoomRedaction(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyMessageEventStub::Sticker(ref __self_0),) => {
                    AnyMessageEventStub::Sticker(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyMessageEventStub::Custom(ref __self_0),) => {
                    AnyMessageEventStub::Custom(::core::clone::Clone::clone(&(*__self_0)))
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::fmt::Debug for AnyMessageEventStub {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&AnyMessageEventStub::CallAnswer(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("CallAnswer");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyMessageEventStub::CallInvite(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("CallInvite");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyMessageEventStub::CallHangup(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("CallHangup");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyMessageEventStub::CallCandidates(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("CallCandidates");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyMessageEventStub::RoomEncrypted(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomEncrypted");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyMessageEventStub::RoomMessage(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomMessage");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyMessageEventStub::RoomMessageFeedback(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomMessageFeedback");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyMessageEventStub::RoomRedaction(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomRedaction");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyMessageEventStub::Sticker(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Sticker");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyMessageEventStub::Custom(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Custom");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(rust_2018_idioms, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for AnyMessageEventStub {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::export::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    AnyMessageEventStub::CallAnswer(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyMessageEventStub::CallInvite(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyMessageEventStub::CallHangup(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyMessageEventStub::CallCandidates(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyMessageEventStub::RoomEncrypted(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyMessageEventStub::RoomMessage(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyMessageEventStub::RoomMessageFeedback(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyMessageEventStub::RoomRedaction(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyMessageEventStub::Sticker(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyMessageEventStub::Custom(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                }
            }
        }
    };
    impl AnyMessageEventStub {
        /// Returns the any content enum for this event.
        pub fn content(&self) -> AnyMessageEventContent {
            match self {
                Self::CallAnswer(event) => {
                    AnyMessageEventContent::CallAnswer(event.content.clone())
                }
                Self::CallInvite(event) => {
                    AnyMessageEventContent::CallInvite(event.content.clone())
                }
                Self::CallHangup(event) => {
                    AnyMessageEventContent::CallHangup(event.content.clone())
                }
                Self::CallCandidates(event) => {
                    AnyMessageEventContent::CallCandidates(event.content.clone())
                }
                Self::RoomEncrypted(event) => {
                    AnyMessageEventContent::RoomEncrypted(event.content.clone())
                }
                Self::RoomMessage(event) => {
                    AnyMessageEventContent::RoomMessage(event.content.clone())
                }
                Self::RoomMessageFeedback(event) => {
                    AnyMessageEventContent::RoomMessageFeedback(event.content.clone())
                }
                Self::RoomRedaction(event) => {
                    AnyMessageEventContent::RoomRedaction(event.content.clone())
                }
                Self::Sticker(event) => AnyMessageEventContent::Sticker(event.content.clone()),
                Self::Custom(event) => AnyMessageEventContent::Custom(event.content.clone()),
            }
        }
        ///Returns this events origin_server_ts field.
        pub fn origin_server_ts(&self) -> &::std::time::SystemTime {
            match self {
                Self::CallAnswer(event) => &event.origin_server_ts,
                Self::CallInvite(event) => &event.origin_server_ts,
                Self::CallHangup(event) => &event.origin_server_ts,
                Self::CallCandidates(event) => &event.origin_server_ts,
                Self::RoomEncrypted(event) => &event.origin_server_ts,
                Self::RoomMessage(event) => &event.origin_server_ts,
                Self::RoomMessageFeedback(event) => &event.origin_server_ts,
                Self::RoomRedaction(event) => &event.origin_server_ts,
                Self::Sticker(event) => &event.origin_server_ts,
                Self::Custom(event) => &event.origin_server_ts,
            }
        }
        ///Returns this events event_id field.
        pub fn event_id(&self) -> &::ruma_identifiers::EventId {
            match self {
                Self::CallAnswer(event) => &event.event_id,
                Self::CallInvite(event) => &event.event_id,
                Self::CallHangup(event) => &event.event_id,
                Self::CallCandidates(event) => &event.event_id,
                Self::RoomEncrypted(event) => &event.event_id,
                Self::RoomMessage(event) => &event.event_id,
                Self::RoomMessageFeedback(event) => &event.event_id,
                Self::RoomRedaction(event) => &event.event_id,
                Self::Sticker(event) => &event.event_id,
                Self::Custom(event) => &event.event_id,
            }
        }
        ///Returns this events sender field.
        pub fn sender(&self) -> &::ruma_identifiers::UserId {
            match self {
                Self::CallAnswer(event) => &event.sender,
                Self::CallInvite(event) => &event.sender,
                Self::CallHangup(event) => &event.sender,
                Self::CallCandidates(event) => &event.sender,
                Self::RoomEncrypted(event) => &event.sender,
                Self::RoomMessage(event) => &event.sender,
                Self::RoomMessageFeedback(event) => &event.sender,
                Self::RoomRedaction(event) => &event.sender,
                Self::Sticker(event) => &event.sender,
                Self::Custom(event) => &event.sender,
            }
        }
        ///Returns this events unsigned field.
        pub fn unsigned(&self) -> &::ruma_events::UnsignedData {
            match self {
                Self::CallAnswer(event) => &event.unsigned,
                Self::CallInvite(event) => &event.unsigned,
                Self::CallHangup(event) => &event.unsigned,
                Self::CallCandidates(event) => &event.unsigned,
                Self::RoomEncrypted(event) => &event.unsigned,
                Self::RoomMessage(event) => &event.unsigned,
                Self::RoomMessageFeedback(event) => &event.unsigned,
                Self::RoomRedaction(event) => &event.unsigned,
                Self::Sticker(event) => &event.unsigned,
                Self::Custom(event) => &event.unsigned,
            }
        }
    }
    impl<'de> ::serde::de::Deserialize<'de> for AnyMessageEventStub {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: ::serde::de::Deserializer<'de>,
        {
            use ::serde::de::Error as _;
            let json = Box::<::serde_json::value::RawValue>::deserialize(deserializer)?;
            let ::ruma_events::EventDeHelper { ev_type, .. } =
                ::ruma_events::from_raw_json_value(&json)?;
            match ev_type.as_str() {
                "m.call.answer" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::MessageEventStub<
                            ::ruma_events::call::answer::AnswerEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::CallAnswer(event))
                }
                "m.call.invite" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::MessageEventStub<
                            ::ruma_events::call::invite::InviteEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::CallInvite(event))
                }
                "m.call.hangup" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::MessageEventStub<
                            ::ruma_events::call::hangup::HangupEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::CallHangup(event))
                }
                "m.call.candidates" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::MessageEventStub<
                            ::ruma_events::call::candidates::CandidatesEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::CallCandidates(event))
                }
                "m.room.encrypted" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::MessageEventStub<
                            ::ruma_events::room::encrypted::EncryptedEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomEncrypted(event))
                }
                "m.room.message" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::MessageEventStub<
                            ::ruma_events::room::message::MessageEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomMessage(event))
                }
                "m.room.message.feedback" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::MessageEventStub<
                            ::ruma_events::room::message::feedback::FeedbackEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomMessageFeedback(event))
                }
                "m.room.redaction" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::room::redaction::RedactionEventStub,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomRedaction(event))
                }
                "m.sticker" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::MessageEventStub<
                            ::ruma_events::sticker::StickerEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::Sticker(event))
                }
                event => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::MessageEventStub<::ruma_events::custom::CustomEventContent>,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::Custom(event))
                }
            }
        }
    }
    /// Any message event.
    #[allow(clippy::large_enum_variant)]
    pub enum AnyRedactedMessageEventContent {
        ///m.call.answer
        CallAnswer(::ruma_events::call::answer::AnswerEventContent),
        ///m.call.invite
        CallInvite(::ruma_events::call::invite::InviteEventContent),
        ///m.call.hangup
        CallHangup(::ruma_events::call::hangup::HangupEventContent),
        ///m.call.candidates
        CallCandidates(::ruma_events::call::candidates::CandidatesEventContent),
        ///m.room.encrypted
        RoomEncrypted(::ruma_events::room::encrypted::EncryptedEventContent),
        ///m.room.message
        RoomMessage(::ruma_events::room::message::MessageEventContent),
        ///m.room.message.feedback
        RoomMessageFeedback(::ruma_events::room::message::feedback::FeedbackEventContent),
        ///m.room.redaction
        RoomRedaction(::ruma_events::room::redaction::RedactionEventContent),
        ///m.sticker
        Sticker(::ruma_events::sticker::StickerEventContent),
        /// Content of an event not defined by the Matrix specification.
        Custom(::ruma_events::custom::CustomEventContent),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::clone::Clone for AnyRedactedMessageEventContent {
        #[inline]
        fn clone(&self) -> AnyRedactedMessageEventContent {
            match (&*self,) {
                (&AnyRedactedMessageEventContent::CallAnswer(ref __self_0),) => {
                    AnyRedactedMessageEventContent::CallAnswer(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedMessageEventContent::CallInvite(ref __self_0),) => {
                    AnyRedactedMessageEventContent::CallInvite(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedMessageEventContent::CallHangup(ref __self_0),) => {
                    AnyRedactedMessageEventContent::CallHangup(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedMessageEventContent::CallCandidates(ref __self_0),) => {
                    AnyRedactedMessageEventContent::CallCandidates(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedMessageEventContent::RoomEncrypted(ref __self_0),) => {
                    AnyRedactedMessageEventContent::RoomEncrypted(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedMessageEventContent::RoomMessage(ref __self_0),) => {
                    AnyRedactedMessageEventContent::RoomMessage(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedMessageEventContent::RoomMessageFeedback(ref __self_0),) => {
                    AnyRedactedMessageEventContent::RoomMessageFeedback(
                        ::core::clone::Clone::clone(&(*__self_0)),
                    )
                }
                (&AnyRedactedMessageEventContent::RoomRedaction(ref __self_0),) => {
                    AnyRedactedMessageEventContent::RoomRedaction(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedMessageEventContent::Sticker(ref __self_0),) => {
                    AnyRedactedMessageEventContent::Sticker(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedMessageEventContent::Custom(ref __self_0),) => {
                    AnyRedactedMessageEventContent::Custom(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::fmt::Debug for AnyRedactedMessageEventContent {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&AnyRedactedMessageEventContent::CallAnswer(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("CallAnswer");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedMessageEventContent::CallInvite(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("CallInvite");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedMessageEventContent::CallHangup(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("CallHangup");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedMessageEventContent::CallCandidates(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("CallCandidates");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedMessageEventContent::RoomEncrypted(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomEncrypted");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedMessageEventContent::RoomMessage(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomMessage");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedMessageEventContent::RoomMessageFeedback(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomMessageFeedback");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedMessageEventContent::RoomRedaction(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomRedaction");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedMessageEventContent::Sticker(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Sticker");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedMessageEventContent::Custom(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Custom");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(rust_2018_idioms, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for AnyRedactedMessageEventContent {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::export::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    AnyRedactedMessageEventContent::CallAnswer(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedMessageEventContent::CallInvite(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedMessageEventContent::CallHangup(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedMessageEventContent::CallCandidates(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedMessageEventContent::RoomEncrypted(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedMessageEventContent::RoomMessage(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedMessageEventContent::RoomMessageFeedback(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedMessageEventContent::RoomRedaction(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedMessageEventContent::Sticker(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedMessageEventContent::Custom(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                }
            }
        }
    };
    impl ::ruma_events::EventContent for AnyRedactedMessageEventContent {
        fn event_type(&self) -> &str {
            match self {
                Self::CallAnswer(content) => content.event_type(),
                Self::CallInvite(content) => content.event_type(),
                Self::CallHangup(content) => content.event_type(),
                Self::CallCandidates(content) => content.event_type(),
                Self::RoomEncrypted(content) => content.event_type(),
                Self::RoomMessage(content) => content.event_type(),
                Self::RoomMessageFeedback(content) => content.event_type(),
                Self::RoomRedaction(content) => content.event_type(),
                Self::Sticker(content) => content.event_type(),
                Self::Custom(content) => content.event_type(),
            }
        }
        fn from_parts(
            event_type: &str,
            input: Box<::serde_json::value::RawValue>,
        ) -> Result<Self, ::serde_json::Error> {
            match event_type {
                "m.call.answer" => {
                    let content = ::ruma_events::call::answer::AnswerEventContent::from_parts(
                        event_type, input,
                    )?;
                    Ok(Self::CallAnswer(content))
                }
                "m.call.invite" => {
                    let content = ::ruma_events::call::invite::InviteEventContent::from_parts(
                        event_type, input,
                    )?;
                    Ok(Self::CallInvite(content))
                }
                "m.call.hangup" => {
                    let content = ::ruma_events::call::hangup::HangupEventContent::from_parts(
                        event_type, input,
                    )?;
                    Ok(Self::CallHangup(content))
                }
                "m.call.candidates" => {
                    let content =
                        ::ruma_events::call::candidates::CandidatesEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::CallCandidates(content))
                }
                "m.room.encrypted" => {
                    let content =
                        ::ruma_events::room::encrypted::EncryptedEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::RoomEncrypted(content))
                }
                "m.room.message" => {
                    let content = ::ruma_events::room::message::MessageEventContent::from_parts(
                        event_type, input,
                    )?;
                    Ok(Self::RoomMessage(content))
                }
                "m.room.message.feedback" => {
                    let content =
                        ::ruma_events::room::message::feedback::FeedbackEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::RoomMessageFeedback(content))
                }
                "m.room.redaction" => {
                    let content =
                        ::ruma_events::room::redaction::RedactionEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::RoomRedaction(content))
                }
                "m.sticker" => {
                    let content =
                        ::ruma_events::sticker::StickerEventContent::from_parts(event_type, input)?;
                    Ok(Self::Sticker(content))
                }
                ev_type => {
                    let content =
                        ::ruma_events::custom::CustomEventContent::from_parts(ev_type, input)?;
                    Ok(Self::Custom(content))
                }
            }
        }
    }
    /// Any message event.
    #[allow(clippy::large_enum_variant)]
    pub enum AnyRedactedMessageEvent {
        ///m.call.answer
        CallAnswer(::ruma_events::call::answer::AnswerEvent),
        ///m.call.invite
        CallInvite(::ruma_events::call::invite::InviteEvent),
        ///m.call.hangup
        CallHangup(::ruma_events::call::hangup::HangupEvent),
        ///m.call.candidates
        CallCandidates(::ruma_events::call::candidates::CandidatesEvent),
        ///m.room.encrypted
        RoomEncrypted(::ruma_events::room::encrypted::EncryptedEvent),
        ///m.room.message
        RoomMessage(::ruma_events::room::message::MessageEvent),
        ///m.room.message.feedback
        RoomMessageFeedback(::ruma_events::room::message::feedback::FeedbackEvent),
        ///m.room.redaction
        RoomRedaction(::ruma_events::room::redaction::RedactionEvent),
        ///m.sticker
        Sticker(::ruma_events::sticker::StickerEvent),
        /// An event not defined by the Matrix specification
        Custom(::ruma_events::MessageEvent<::ruma_events::custom::CustomEventContent>),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::clone::Clone for AnyRedactedMessageEvent {
        #[inline]
        fn clone(&self) -> AnyRedactedMessageEvent {
            match (&*self,) {
                (&AnyRedactedMessageEvent::CallAnswer(ref __self_0),) => {
                    AnyRedactedMessageEvent::CallAnswer(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyRedactedMessageEvent::CallInvite(ref __self_0),) => {
                    AnyRedactedMessageEvent::CallInvite(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyRedactedMessageEvent::CallHangup(ref __self_0),) => {
                    AnyRedactedMessageEvent::CallHangup(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyRedactedMessageEvent::CallCandidates(ref __self_0),) => {
                    AnyRedactedMessageEvent::CallCandidates(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedMessageEvent::RoomEncrypted(ref __self_0),) => {
                    AnyRedactedMessageEvent::RoomEncrypted(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedMessageEvent::RoomMessage(ref __self_0),) => {
                    AnyRedactedMessageEvent::RoomMessage(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyRedactedMessageEvent::RoomMessageFeedback(ref __self_0),) => {
                    AnyRedactedMessageEvent::RoomMessageFeedback(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedMessageEvent::RoomRedaction(ref __self_0),) => {
                    AnyRedactedMessageEvent::RoomRedaction(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedMessageEvent::Sticker(ref __self_0),) => {
                    AnyRedactedMessageEvent::Sticker(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyRedactedMessageEvent::Custom(ref __self_0),) => {
                    AnyRedactedMessageEvent::Custom(::core::clone::Clone::clone(&(*__self_0)))
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::fmt::Debug for AnyRedactedMessageEvent {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&AnyRedactedMessageEvent::CallAnswer(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("CallAnswer");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedMessageEvent::CallInvite(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("CallInvite");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedMessageEvent::CallHangup(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("CallHangup");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedMessageEvent::CallCandidates(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("CallCandidates");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedMessageEvent::RoomEncrypted(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomEncrypted");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedMessageEvent::RoomMessage(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomMessage");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedMessageEvent::RoomMessageFeedback(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomMessageFeedback");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedMessageEvent::RoomRedaction(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomRedaction");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedMessageEvent::Sticker(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Sticker");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedMessageEvent::Custom(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Custom");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(rust_2018_idioms, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for AnyRedactedMessageEvent {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::export::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    AnyRedactedMessageEvent::CallAnswer(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedMessageEvent::CallInvite(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedMessageEvent::CallHangup(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedMessageEvent::CallCandidates(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedMessageEvent::RoomEncrypted(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedMessageEvent::RoomMessage(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedMessageEvent::RoomMessageFeedback(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedMessageEvent::RoomRedaction(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedMessageEvent::Sticker(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedMessageEvent::Custom(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                }
            }
        }
    };
    impl AnyRedactedMessageEvent {
        /// Returns the any content enum for this event.
        pub fn content(&self) -> AnyRedactedMessageEventContent {
            match self {
                Self::CallAnswer(event) => {
                    AnyRedactedMessageEventContent::CallAnswer(event.content.clone())
                }
                Self::CallInvite(event) => {
                    AnyRedactedMessageEventContent::CallInvite(event.content.clone())
                }
                Self::CallHangup(event) => {
                    AnyRedactedMessageEventContent::CallHangup(event.content.clone())
                }
                Self::CallCandidates(event) => {
                    AnyRedactedMessageEventContent::CallCandidates(event.content.clone())
                }
                Self::RoomEncrypted(event) => {
                    AnyRedactedMessageEventContent::RoomEncrypted(event.content.clone())
                }
                Self::RoomMessage(event) => {
                    AnyRedactedMessageEventContent::RoomMessage(event.content.clone())
                }
                Self::RoomMessageFeedback(event) => {
                    AnyRedactedMessageEventContent::RoomMessageFeedback(event.content.clone())
                }
                Self::RoomRedaction(event) => {
                    AnyRedactedMessageEventContent::RoomRedaction(event.content.clone())
                }
                Self::Sticker(event) => {
                    AnyRedactedMessageEventContent::Sticker(event.content.clone())
                }
                Self::Custom(event) => {
                    AnyRedactedMessageEventContent::Custom(event.content.clone())
                }
            }
        }
    }
    impl<'de> ::serde::de::Deserialize<'de> for AnyRedactedMessageEvent {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: ::serde::de::Deserializer<'de>,
        {
            use ::serde::de::Error as _;
            let json = Box::<::serde_json::value::RawValue>::deserialize(deserializer)?;
            let ::ruma_events::EventDeHelper { ev_type, .. } =
                ::ruma_events::from_raw_json_value(&json)?;
            match ev_type.as_str() {
                "m.call.answer" => {
                    let event = ::serde_json::from_str::<::ruma_events::call::answer::AnswerEvent>(
                        json.get(),
                    )
                    .map_err(D::Error::custom)?;
                    Ok(Self::CallAnswer(event))
                }
                "m.call.invite" => {
                    let event = ::serde_json::from_str::<::ruma_events::call::invite::InviteEvent>(
                        json.get(),
                    )
                    .map_err(D::Error::custom)?;
                    Ok(Self::CallInvite(event))
                }
                "m.call.hangup" => {
                    let event = ::serde_json::from_str::<::ruma_events::call::hangup::HangupEvent>(
                        json.get(),
                    )
                    .map_err(D::Error::custom)?;
                    Ok(Self::CallHangup(event))
                }
                "m.call.candidates" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::call::candidates::CandidatesEvent,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::CallCandidates(event))
                }
                "m.room.encrypted" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::room::encrypted::EncryptedEvent,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomEncrypted(event))
                }
                "m.room.message" => {
                    let event =
                        ::serde_json::from_str::<::ruma_events::room::message::MessageEvent>(
                            json.get(),
                        )
                        .map_err(D::Error::custom)?;
                    Ok(Self::RoomMessage(event))
                }
                "m.room.message.feedback" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::room::message::feedback::FeedbackEvent,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomMessageFeedback(event))
                }
                "m.room.redaction" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::room::redaction::RedactionEvent,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomRedaction(event))
                }
                "m.sticker" => {
                    let event =
                        ::serde_json::from_str::<::ruma_events::sticker::StickerEvent>(json.get())
                            .map_err(D::Error::custom)?;
                    Ok(Self::Sticker(event))
                }
                event => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::MessageEvent<::ruma_events::custom::CustomEventContent>,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::Custom(event))
                }
            }
        }
    }
    /// Any message event.
    #[allow(clippy::large_enum_variant)]
    pub enum AnyRedactedMessageEventStub {
        ///m.call.answer
        CallAnswer(
            ::ruma_events::MessageEventStub<::ruma_events::call::answer::AnswerEventContent>,
        ),
        ///m.call.invite
        CallInvite(
            ::ruma_events::MessageEventStub<::ruma_events::call::invite::InviteEventContent>,
        ),
        ///m.call.hangup
        CallHangup(
            ::ruma_events::MessageEventStub<::ruma_events::call::hangup::HangupEventContent>,
        ),
        ///m.call.candidates
        CallCandidates(
            ::ruma_events::MessageEventStub<
                ::ruma_events::call::candidates::CandidatesEventContent,
            >,
        ),
        ///m.room.encrypted
        RoomEncrypted(
            ::ruma_events::MessageEventStub<::ruma_events::room::encrypted::EncryptedEventContent>,
        ),
        ///m.room.message
        RoomMessage(
            ::ruma_events::MessageEventStub<::ruma_events::room::message::MessageEventContent>,
        ),
        ///m.room.message.feedback
        RoomMessageFeedback(
            ::ruma_events::MessageEventStub<
                ::ruma_events::room::message::feedback::FeedbackEventContent,
            >,
        ),
        ///m.room.redaction
        RoomRedaction(::ruma_events::room::redaction::RedactionEventStub),
        ///m.sticker
        Sticker(::ruma_events::MessageEventStub<::ruma_events::sticker::StickerEventContent>),
        /// An event not defined by the Matrix specification
        Custom(::ruma_events::MessageEventStub<::ruma_events::custom::CustomEventContent>),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::clone::Clone for AnyRedactedMessageEventStub {
        #[inline]
        fn clone(&self) -> AnyRedactedMessageEventStub {
            match (&*self,) {
                (&AnyRedactedMessageEventStub::CallAnswer(ref __self_0),) => {
                    AnyRedactedMessageEventStub::CallAnswer(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedMessageEventStub::CallInvite(ref __self_0),) => {
                    AnyRedactedMessageEventStub::CallInvite(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedMessageEventStub::CallHangup(ref __self_0),) => {
                    AnyRedactedMessageEventStub::CallHangup(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedMessageEventStub::CallCandidates(ref __self_0),) => {
                    AnyRedactedMessageEventStub::CallCandidates(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedMessageEventStub::RoomEncrypted(ref __self_0),) => {
                    AnyRedactedMessageEventStub::RoomEncrypted(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedMessageEventStub::RoomMessage(ref __self_0),) => {
                    AnyRedactedMessageEventStub::RoomMessage(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedMessageEventStub::RoomMessageFeedback(ref __self_0),) => {
                    AnyRedactedMessageEventStub::RoomMessageFeedback(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedMessageEventStub::RoomRedaction(ref __self_0),) => {
                    AnyRedactedMessageEventStub::RoomRedaction(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedMessageEventStub::Sticker(ref __self_0),) => {
                    AnyRedactedMessageEventStub::Sticker(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyRedactedMessageEventStub::Custom(ref __self_0),) => {
                    AnyRedactedMessageEventStub::Custom(::core::clone::Clone::clone(&(*__self_0)))
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::fmt::Debug for AnyRedactedMessageEventStub {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&AnyRedactedMessageEventStub::CallAnswer(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("CallAnswer");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedMessageEventStub::CallInvite(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("CallInvite");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedMessageEventStub::CallHangup(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("CallHangup");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedMessageEventStub::CallCandidates(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("CallCandidates");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedMessageEventStub::RoomEncrypted(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomEncrypted");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedMessageEventStub::RoomMessage(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomMessage");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedMessageEventStub::RoomMessageFeedback(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomMessageFeedback");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedMessageEventStub::RoomRedaction(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomRedaction");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedMessageEventStub::Sticker(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Sticker");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedMessageEventStub::Custom(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Custom");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(rust_2018_idioms, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for AnyRedactedMessageEventStub {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::export::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    AnyRedactedMessageEventStub::CallAnswer(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedMessageEventStub::CallInvite(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedMessageEventStub::CallHangup(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedMessageEventStub::CallCandidates(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedMessageEventStub::RoomEncrypted(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedMessageEventStub::RoomMessage(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedMessageEventStub::RoomMessageFeedback(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedMessageEventStub::RoomRedaction(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedMessageEventStub::Sticker(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedMessageEventStub::Custom(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                }
            }
        }
    };
    impl AnyRedactedMessageEventStub {
        /// Returns the any content enum for this event.
        pub fn content(&self) -> AnyRedactedMessageEventContent {
            match self {
                Self::CallAnswer(event) => {
                    AnyRedactedMessageEventContent::CallAnswer(event.content.clone())
                }
                Self::CallInvite(event) => {
                    AnyRedactedMessageEventContent::CallInvite(event.content.clone())
                }
                Self::CallHangup(event) => {
                    AnyRedactedMessageEventContent::CallHangup(event.content.clone())
                }
                Self::CallCandidates(event) => {
                    AnyRedactedMessageEventContent::CallCandidates(event.content.clone())
                }
                Self::RoomEncrypted(event) => {
                    AnyRedactedMessageEventContent::RoomEncrypted(event.content.clone())
                }
                Self::RoomMessage(event) => {
                    AnyRedactedMessageEventContent::RoomMessage(event.content.clone())
                }
                Self::RoomMessageFeedback(event) => {
                    AnyRedactedMessageEventContent::RoomMessageFeedback(event.content.clone())
                }
                Self::RoomRedaction(event) => {
                    AnyRedactedMessageEventContent::RoomRedaction(event.content.clone())
                }
                Self::Sticker(event) => {
                    AnyRedactedMessageEventContent::Sticker(event.content.clone())
                }
                Self::Custom(event) => {
                    AnyRedactedMessageEventContent::Custom(event.content.clone())
                }
            }
        }
    }
    impl<'de> ::serde::de::Deserialize<'de> for AnyRedactedMessageEventStub {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: ::serde::de::Deserializer<'de>,
        {
            use ::serde::de::Error as _;
            let json = Box::<::serde_json::value::RawValue>::deserialize(deserializer)?;
            let ::ruma_events::EventDeHelper { ev_type, .. } =
                ::ruma_events::from_raw_json_value(&json)?;
            match ev_type.as_str() {
                "m.call.answer" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::MessageEventStub<
                            ::ruma_events::call::answer::AnswerEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::CallAnswer(event))
                }
                "m.call.invite" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::MessageEventStub<
                            ::ruma_events::call::invite::InviteEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::CallInvite(event))
                }
                "m.call.hangup" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::MessageEventStub<
                            ::ruma_events::call::hangup::HangupEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::CallHangup(event))
                }
                "m.call.candidates" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::MessageEventStub<
                            ::ruma_events::call::candidates::CandidatesEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::CallCandidates(event))
                }
                "m.room.encrypted" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::MessageEventStub<
                            ::ruma_events::room::encrypted::EncryptedEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomEncrypted(event))
                }
                "m.room.message" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::MessageEventStub<
                            ::ruma_events::room::message::MessageEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomMessage(event))
                }
                "m.room.message.feedback" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::MessageEventStub<
                            ::ruma_events::room::message::feedback::FeedbackEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomMessageFeedback(event))
                }
                "m.room.redaction" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::room::redaction::RedactionEventStub,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomRedaction(event))
                }
                "m.sticker" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::MessageEventStub<
                            ::ruma_events::sticker::StickerEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::Sticker(event))
                }
                event => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::MessageEventStub<::ruma_events::custom::CustomEventContent>,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::Custom(event))
                }
            }
        }
    }
    /// Any message event.
    #[allow(clippy::large_enum_variant)]
    pub enum AnyMessageEventContent {
        ///m.call.answer
        CallAnswer(::ruma_events::call::answer::AnswerEventContent),
        ///m.call.invite
        CallInvite(::ruma_events::call::invite::InviteEventContent),
        ///m.call.hangup
        CallHangup(::ruma_events::call::hangup::HangupEventContent),
        ///m.call.candidates
        CallCandidates(::ruma_events::call::candidates::CandidatesEventContent),
        ///m.room.encrypted
        RoomEncrypted(::ruma_events::room::encrypted::EncryptedEventContent),
        ///m.room.message
        RoomMessage(::ruma_events::room::message::MessageEventContent),
        ///m.room.message.feedback
        RoomMessageFeedback(::ruma_events::room::message::feedback::FeedbackEventContent),
        ///m.room.redaction
        RoomRedaction(::ruma_events::room::redaction::RedactionEventContent),
        ///m.sticker
        Sticker(::ruma_events::sticker::StickerEventContent),
        /// Content of an event not defined by the Matrix specification.
        Custom(::ruma_events::custom::CustomEventContent),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::clone::Clone for AnyMessageEventContent {
        #[inline]
        fn clone(&self) -> AnyMessageEventContent {
            match (&*self,) {
                (&AnyMessageEventContent::CallAnswer(ref __self_0),) => {
                    AnyMessageEventContent::CallAnswer(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyMessageEventContent::CallInvite(ref __self_0),) => {
                    AnyMessageEventContent::CallInvite(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyMessageEventContent::CallHangup(ref __self_0),) => {
                    AnyMessageEventContent::CallHangup(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyMessageEventContent::CallCandidates(ref __self_0),) => {
                    AnyMessageEventContent::CallCandidates(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyMessageEventContent::RoomEncrypted(ref __self_0),) => {
                    AnyMessageEventContent::RoomEncrypted(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyMessageEventContent::RoomMessage(ref __self_0),) => {
                    AnyMessageEventContent::RoomMessage(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyMessageEventContent::RoomMessageFeedback(ref __self_0),) => {
                    AnyMessageEventContent::RoomMessageFeedback(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyMessageEventContent::RoomRedaction(ref __self_0),) => {
                    AnyMessageEventContent::RoomRedaction(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyMessageEventContent::Sticker(ref __self_0),) => {
                    AnyMessageEventContent::Sticker(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyMessageEventContent::Custom(ref __self_0),) => {
                    AnyMessageEventContent::Custom(::core::clone::Clone::clone(&(*__self_0)))
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::fmt::Debug for AnyMessageEventContent {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&AnyMessageEventContent::CallAnswer(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("CallAnswer");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyMessageEventContent::CallInvite(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("CallInvite");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyMessageEventContent::CallHangup(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("CallHangup");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyMessageEventContent::CallCandidates(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("CallCandidates");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyMessageEventContent::RoomEncrypted(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomEncrypted");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyMessageEventContent::RoomMessage(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomMessage");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyMessageEventContent::RoomMessageFeedback(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomMessageFeedback");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyMessageEventContent::RoomRedaction(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomRedaction");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyMessageEventContent::Sticker(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Sticker");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyMessageEventContent::Custom(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Custom");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(rust_2018_idioms, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for AnyMessageEventContent {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::export::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    AnyMessageEventContent::CallAnswer(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyMessageEventContent::CallInvite(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyMessageEventContent::CallHangup(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyMessageEventContent::CallCandidates(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyMessageEventContent::RoomEncrypted(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyMessageEventContent::RoomMessage(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyMessageEventContent::RoomMessageFeedback(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyMessageEventContent::RoomRedaction(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyMessageEventContent::Sticker(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyMessageEventContent::Custom(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                }
            }
        }
    };
    impl ::ruma_events::EventContent for AnyMessageEventContent {
        fn event_type(&self) -> &str {
            match self {
                Self::CallAnswer(content) => content.event_type(),
                Self::CallInvite(content) => content.event_type(),
                Self::CallHangup(content) => content.event_type(),
                Self::CallCandidates(content) => content.event_type(),
                Self::RoomEncrypted(content) => content.event_type(),
                Self::RoomMessage(content) => content.event_type(),
                Self::RoomMessageFeedback(content) => content.event_type(),
                Self::RoomRedaction(content) => content.event_type(),
                Self::Sticker(content) => content.event_type(),
                Self::Custom(content) => content.event_type(),
            }
        }
        fn from_parts(
            event_type: &str,
            input: Box<::serde_json::value::RawValue>,
        ) -> Result<Self, ::serde_json::Error> {
            match event_type {
                "m.call.answer" => {
                    let content = ::ruma_events::call::answer::AnswerEventContent::from_parts(
                        event_type, input,
                    )?;
                    Ok(Self::CallAnswer(content))
                }
                "m.call.invite" => {
                    let content = ::ruma_events::call::invite::InviteEventContent::from_parts(
                        event_type, input,
                    )?;
                    Ok(Self::CallInvite(content))
                }
                "m.call.hangup" => {
                    let content = ::ruma_events::call::hangup::HangupEventContent::from_parts(
                        event_type, input,
                    )?;
                    Ok(Self::CallHangup(content))
                }
                "m.call.candidates" => {
                    let content =
                        ::ruma_events::call::candidates::CandidatesEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::CallCandidates(content))
                }
                "m.room.encrypted" => {
                    let content =
                        ::ruma_events::room::encrypted::EncryptedEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::RoomEncrypted(content))
                }
                "m.room.message" => {
                    let content = ::ruma_events::room::message::MessageEventContent::from_parts(
                        event_type, input,
                    )?;
                    Ok(Self::RoomMessage(content))
                }
                "m.room.message.feedback" => {
                    let content =
                        ::ruma_events::room::message::feedback::FeedbackEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::RoomMessageFeedback(content))
                }
                "m.room.redaction" => {
                    let content =
                        ::ruma_events::room::redaction::RedactionEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::RoomRedaction(content))
                }
                "m.sticker" => {
                    let content =
                        ::ruma_events::sticker::StickerEventContent::from_parts(event_type, input)?;
                    Ok(Self::Sticker(content))
                }
                ev_type => {
                    let content =
                        ::ruma_events::custom::CustomEventContent::from_parts(ev_type, input)?;
                    Ok(Self::Custom(content))
                }
            }
        }
    }
    impl ::ruma_events::RoomEventContent for AnyMessageEventContent {}
    impl ::ruma_events::MessageEventContent for AnyMessageEventContent {}
    /// Any state event.
    #[allow(clippy::large_enum_variant)]
    pub enum AnyStateEvent {
        ///m.room.aliases
        RoomAliases(::ruma_events::room::aliases::AliasesEvent),
        ///m.room.avatar
        RoomAvatar(::ruma_events::room::avatar::AvatarEvent),
        ///m.room.canonical_alias
        RoomCanonicalAlias(::ruma_events::room::canonical_alias::CanonicalAliasEvent),
        ///m.room.create
        RoomCreate(::ruma_events::room::create::CreateEvent),
        ///m.room.encryption
        RoomEncryption(::ruma_events::room::encryption::EncryptionEvent),
        ///m.room.guest_access
        RoomGuestAccess(::ruma_events::room::guest_access::GuestAccessEvent),
        ///m.room.history_visibility
        RoomHistoryVisibility(::ruma_events::room::history_visibility::HistoryVisibilityEvent),
        ///m.room.join_rules
        RoomJoinRules(::ruma_events::room::join_rules::JoinRulesEvent),
        ///m.room.member
        RoomMember(::ruma_events::room::member::MemberEvent),
        ///m.room.name
        RoomName(::ruma_events::room::name::NameEvent),
        ///m.room.pinned_events
        RoomPinnedEvents(::ruma_events::room::pinned_events::PinnedEventsEvent),
        ///m.room.power_levels
        RoomPowerLevels(::ruma_events::room::power_levels::PowerLevelsEvent),
        ///m.room.server_acl
        RoomServerAcl(::ruma_events::room::server_acl::ServerAclEvent),
        ///m.room.third_party_invite
        RoomThirdPartyInvite(::ruma_events::room::third_party_invite::ThirdPartyInviteEvent),
        ///m.room.tombstone
        RoomTombstone(::ruma_events::room::tombstone::TombstoneEvent),
        ///m.room.topic
        RoomTopic(::ruma_events::room::topic::TopicEvent),
        /// An event not defined by the Matrix specification
        Custom(::ruma_events::StateEvent<::ruma_events::custom::CustomEventContent>),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::clone::Clone for AnyStateEvent {
        #[inline]
        fn clone(&self) -> AnyStateEvent {
            match (&*self,) {
                (&AnyStateEvent::RoomAliases(ref __self_0),) => {
                    AnyStateEvent::RoomAliases(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEvent::RoomAvatar(ref __self_0),) => {
                    AnyStateEvent::RoomAvatar(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEvent::RoomCanonicalAlias(ref __self_0),) => {
                    AnyStateEvent::RoomCanonicalAlias(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEvent::RoomCreate(ref __self_0),) => {
                    AnyStateEvent::RoomCreate(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEvent::RoomEncryption(ref __self_0),) => {
                    AnyStateEvent::RoomEncryption(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEvent::RoomGuestAccess(ref __self_0),) => {
                    AnyStateEvent::RoomGuestAccess(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEvent::RoomHistoryVisibility(ref __self_0),) => {
                    AnyStateEvent::RoomHistoryVisibility(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEvent::RoomJoinRules(ref __self_0),) => {
                    AnyStateEvent::RoomJoinRules(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEvent::RoomMember(ref __self_0),) => {
                    AnyStateEvent::RoomMember(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEvent::RoomName(ref __self_0),) => {
                    AnyStateEvent::RoomName(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEvent::RoomPinnedEvents(ref __self_0),) => {
                    AnyStateEvent::RoomPinnedEvents(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEvent::RoomPowerLevels(ref __self_0),) => {
                    AnyStateEvent::RoomPowerLevels(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEvent::RoomServerAcl(ref __self_0),) => {
                    AnyStateEvent::RoomServerAcl(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEvent::RoomThirdPartyInvite(ref __self_0),) => {
                    AnyStateEvent::RoomThirdPartyInvite(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEvent::RoomTombstone(ref __self_0),) => {
                    AnyStateEvent::RoomTombstone(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEvent::RoomTopic(ref __self_0),) => {
                    AnyStateEvent::RoomTopic(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEvent::Custom(ref __self_0),) => {
                    AnyStateEvent::Custom(::core::clone::Clone::clone(&(*__self_0)))
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::fmt::Debug for AnyStateEvent {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&AnyStateEvent::RoomAliases(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomAliases");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEvent::RoomAvatar(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomAvatar");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEvent::RoomCanonicalAlias(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomCanonicalAlias");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEvent::RoomCreate(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomCreate");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEvent::RoomEncryption(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomEncryption");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEvent::RoomGuestAccess(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomGuestAccess");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEvent::RoomHistoryVisibility(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomHistoryVisibility");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEvent::RoomJoinRules(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomJoinRules");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEvent::RoomMember(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomMember");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEvent::RoomName(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomName");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEvent::RoomPinnedEvents(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomPinnedEvents");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEvent::RoomPowerLevels(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomPowerLevels");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEvent::RoomServerAcl(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomServerAcl");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEvent::RoomThirdPartyInvite(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomThirdPartyInvite");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEvent::RoomTombstone(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomTombstone");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEvent::RoomTopic(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomTopic");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEvent::Custom(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Custom");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(rust_2018_idioms, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for AnyStateEvent {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::export::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    AnyStateEvent::RoomAliases(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEvent::RoomAvatar(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEvent::RoomCanonicalAlias(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEvent::RoomCreate(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEvent::RoomEncryption(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEvent::RoomGuestAccess(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEvent::RoomHistoryVisibility(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEvent::RoomJoinRules(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEvent::RoomMember(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEvent::RoomName(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEvent::RoomPinnedEvents(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEvent::RoomPowerLevels(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEvent::RoomServerAcl(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEvent::RoomThirdPartyInvite(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEvent::RoomTombstone(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEvent::RoomTopic(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEvent::Custom(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                }
            }
        }
    };
    impl AnyStateEvent {
        /// Returns the any content enum for this event.
        pub fn content(&self) -> AnyStateEventContent {
            match self {
                Self::RoomAliases(event) => {
                    AnyStateEventContent::RoomAliases(event.content.clone())
                }
                Self::RoomAvatar(event) => AnyStateEventContent::RoomAvatar(event.content.clone()),
                Self::RoomCanonicalAlias(event) => {
                    AnyStateEventContent::RoomCanonicalAlias(event.content.clone())
                }
                Self::RoomCreate(event) => AnyStateEventContent::RoomCreate(event.content.clone()),
                Self::RoomEncryption(event) => {
                    AnyStateEventContent::RoomEncryption(event.content.clone())
                }
                Self::RoomGuestAccess(event) => {
                    AnyStateEventContent::RoomGuestAccess(event.content.clone())
                }
                Self::RoomHistoryVisibility(event) => {
                    AnyStateEventContent::RoomHistoryVisibility(event.content.clone())
                }
                Self::RoomJoinRules(event) => {
                    AnyStateEventContent::RoomJoinRules(event.content.clone())
                }
                Self::RoomMember(event) => AnyStateEventContent::RoomMember(event.content.clone()),
                Self::RoomName(event) => AnyStateEventContent::RoomName(event.content.clone()),
                Self::RoomPinnedEvents(event) => {
                    AnyStateEventContent::RoomPinnedEvents(event.content.clone())
                }
                Self::RoomPowerLevels(event) => {
                    AnyStateEventContent::RoomPowerLevels(event.content.clone())
                }
                Self::RoomServerAcl(event) => {
                    AnyStateEventContent::RoomServerAcl(event.content.clone())
                }
                Self::RoomThirdPartyInvite(event) => {
                    AnyStateEventContent::RoomThirdPartyInvite(event.content.clone())
                }
                Self::RoomTombstone(event) => {
                    AnyStateEventContent::RoomTombstone(event.content.clone())
                }
                Self::RoomTopic(event) => AnyStateEventContent::RoomTopic(event.content.clone()),
                Self::Custom(event) => AnyStateEventContent::Custom(event.content.clone()),
            }
        }
        /// Returns the any content enum for this events prev_content.
        pub fn prev_content(&self) -> Option<AnyStateEventContent> {
            match self {
                Self::RoomAliases(event) => event
                    .prev_content
                    .as_ref()
                    .map(|c| AnyStateEventContent::RoomAliases(c.clone())),
                Self::RoomAvatar(event) => {
                    event.prev_content.as_ref().map(|c| AnyStateEventContent::RoomAvatar(c.clone()))
                }
                Self::RoomCanonicalAlias(event) => event
                    .prev_content
                    .as_ref()
                    .map(|c| AnyStateEventContent::RoomCanonicalAlias(c.clone())),
                Self::RoomCreate(event) => {
                    event.prev_content.as_ref().map(|c| AnyStateEventContent::RoomCreate(c.clone()))
                }
                Self::RoomEncryption(event) => event
                    .prev_content
                    .as_ref()
                    .map(|c| AnyStateEventContent::RoomEncryption(c.clone())),
                Self::RoomGuestAccess(event) => event
                    .prev_content
                    .as_ref()
                    .map(|c| AnyStateEventContent::RoomGuestAccess(c.clone())),
                Self::RoomHistoryVisibility(event) => event
                    .prev_content
                    .as_ref()
                    .map(|c| AnyStateEventContent::RoomHistoryVisibility(c.clone())),
                Self::RoomJoinRules(event) => event
                    .prev_content
                    .as_ref()
                    .map(|c| AnyStateEventContent::RoomJoinRules(c.clone())),
                Self::RoomMember(event) => {
                    event.prev_content.as_ref().map(|c| AnyStateEventContent::RoomMember(c.clone()))
                }
                Self::RoomName(event) => {
                    event.prev_content.as_ref().map(|c| AnyStateEventContent::RoomName(c.clone()))
                }
                Self::RoomPinnedEvents(event) => event
                    .prev_content
                    .as_ref()
                    .map(|c| AnyStateEventContent::RoomPinnedEvents(c.clone())),
                Self::RoomPowerLevels(event) => event
                    .prev_content
                    .as_ref()
                    .map(|c| AnyStateEventContent::RoomPowerLevels(c.clone())),
                Self::RoomServerAcl(event) => event
                    .prev_content
                    .as_ref()
                    .map(|c| AnyStateEventContent::RoomServerAcl(c.clone())),
                Self::RoomThirdPartyInvite(event) => event
                    .prev_content
                    .as_ref()
                    .map(|c| AnyStateEventContent::RoomThirdPartyInvite(c.clone())),
                Self::RoomTombstone(event) => event
                    .prev_content
                    .as_ref()
                    .map(|c| AnyStateEventContent::RoomTombstone(c.clone())),
                Self::RoomTopic(event) => {
                    event.prev_content.as_ref().map(|c| AnyStateEventContent::RoomTopic(c.clone()))
                }
                Self::Custom(event) => {
                    event.prev_content.as_ref().map(|c| AnyStateEventContent::Custom(c.clone()))
                }
            }
        }
        ///Returns this events origin_server_ts field.
        pub fn origin_server_ts(&self) -> &::std::time::SystemTime {
            match self {
                Self::RoomAliases(event) => &event.origin_server_ts,
                Self::RoomAvatar(event) => &event.origin_server_ts,
                Self::RoomCanonicalAlias(event) => &event.origin_server_ts,
                Self::RoomCreate(event) => &event.origin_server_ts,
                Self::RoomEncryption(event) => &event.origin_server_ts,
                Self::RoomGuestAccess(event) => &event.origin_server_ts,
                Self::RoomHistoryVisibility(event) => &event.origin_server_ts,
                Self::RoomJoinRules(event) => &event.origin_server_ts,
                Self::RoomMember(event) => &event.origin_server_ts,
                Self::RoomName(event) => &event.origin_server_ts,
                Self::RoomPinnedEvents(event) => &event.origin_server_ts,
                Self::RoomPowerLevels(event) => &event.origin_server_ts,
                Self::RoomServerAcl(event) => &event.origin_server_ts,
                Self::RoomThirdPartyInvite(event) => &event.origin_server_ts,
                Self::RoomTombstone(event) => &event.origin_server_ts,
                Self::RoomTopic(event) => &event.origin_server_ts,
                Self::Custom(event) => &event.origin_server_ts,
            }
        }
        ///Returns this events room_id field.
        pub fn room_id(&self) -> &::ruma_identifiers::RoomId {
            match self {
                Self::RoomAliases(event) => &event.room_id,
                Self::RoomAvatar(event) => &event.room_id,
                Self::RoomCanonicalAlias(event) => &event.room_id,
                Self::RoomCreate(event) => &event.room_id,
                Self::RoomEncryption(event) => &event.room_id,
                Self::RoomGuestAccess(event) => &event.room_id,
                Self::RoomHistoryVisibility(event) => &event.room_id,
                Self::RoomJoinRules(event) => &event.room_id,
                Self::RoomMember(event) => &event.room_id,
                Self::RoomName(event) => &event.room_id,
                Self::RoomPinnedEvents(event) => &event.room_id,
                Self::RoomPowerLevels(event) => &event.room_id,
                Self::RoomServerAcl(event) => &event.room_id,
                Self::RoomThirdPartyInvite(event) => &event.room_id,
                Self::RoomTombstone(event) => &event.room_id,
                Self::RoomTopic(event) => &event.room_id,
                Self::Custom(event) => &event.room_id,
            }
        }
        ///Returns this events event_id field.
        pub fn event_id(&self) -> &::ruma_identifiers::EventId {
            match self {
                Self::RoomAliases(event) => &event.event_id,
                Self::RoomAvatar(event) => &event.event_id,
                Self::RoomCanonicalAlias(event) => &event.event_id,
                Self::RoomCreate(event) => &event.event_id,
                Self::RoomEncryption(event) => &event.event_id,
                Self::RoomGuestAccess(event) => &event.event_id,
                Self::RoomHistoryVisibility(event) => &event.event_id,
                Self::RoomJoinRules(event) => &event.event_id,
                Self::RoomMember(event) => &event.event_id,
                Self::RoomName(event) => &event.event_id,
                Self::RoomPinnedEvents(event) => &event.event_id,
                Self::RoomPowerLevels(event) => &event.event_id,
                Self::RoomServerAcl(event) => &event.event_id,
                Self::RoomThirdPartyInvite(event) => &event.event_id,
                Self::RoomTombstone(event) => &event.event_id,
                Self::RoomTopic(event) => &event.event_id,
                Self::Custom(event) => &event.event_id,
            }
        }
        ///Returns this events sender field.
        pub fn sender(&self) -> &::ruma_identifiers::UserId {
            match self {
                Self::RoomAliases(event) => &event.sender,
                Self::RoomAvatar(event) => &event.sender,
                Self::RoomCanonicalAlias(event) => &event.sender,
                Self::RoomCreate(event) => &event.sender,
                Self::RoomEncryption(event) => &event.sender,
                Self::RoomGuestAccess(event) => &event.sender,
                Self::RoomHistoryVisibility(event) => &event.sender,
                Self::RoomJoinRules(event) => &event.sender,
                Self::RoomMember(event) => &event.sender,
                Self::RoomName(event) => &event.sender,
                Self::RoomPinnedEvents(event) => &event.sender,
                Self::RoomPowerLevels(event) => &event.sender,
                Self::RoomServerAcl(event) => &event.sender,
                Self::RoomThirdPartyInvite(event) => &event.sender,
                Self::RoomTombstone(event) => &event.sender,
                Self::RoomTopic(event) => &event.sender,
                Self::Custom(event) => &event.sender,
            }
        }
        ///Returns this events state_key field.
        pub fn state_key(&self) -> &str {
            match self {
                Self::RoomAliases(event) => &event.state_key,
                Self::RoomAvatar(event) => &event.state_key,
                Self::RoomCanonicalAlias(event) => &event.state_key,
                Self::RoomCreate(event) => &event.state_key,
                Self::RoomEncryption(event) => &event.state_key,
                Self::RoomGuestAccess(event) => &event.state_key,
                Self::RoomHistoryVisibility(event) => &event.state_key,
                Self::RoomJoinRules(event) => &event.state_key,
                Self::RoomMember(event) => &event.state_key,
                Self::RoomName(event) => &event.state_key,
                Self::RoomPinnedEvents(event) => &event.state_key,
                Self::RoomPowerLevels(event) => &event.state_key,
                Self::RoomServerAcl(event) => &event.state_key,
                Self::RoomThirdPartyInvite(event) => &event.state_key,
                Self::RoomTombstone(event) => &event.state_key,
                Self::RoomTopic(event) => &event.state_key,
                Self::Custom(event) => &event.state_key,
            }
        }
        ///Returns this events unsigned field.
        pub fn unsigned(&self) -> &::ruma_events::UnsignedData {
            match self {
                Self::RoomAliases(event) => &event.unsigned,
                Self::RoomAvatar(event) => &event.unsigned,
                Self::RoomCanonicalAlias(event) => &event.unsigned,
                Self::RoomCreate(event) => &event.unsigned,
                Self::RoomEncryption(event) => &event.unsigned,
                Self::RoomGuestAccess(event) => &event.unsigned,
                Self::RoomHistoryVisibility(event) => &event.unsigned,
                Self::RoomJoinRules(event) => &event.unsigned,
                Self::RoomMember(event) => &event.unsigned,
                Self::RoomName(event) => &event.unsigned,
                Self::RoomPinnedEvents(event) => &event.unsigned,
                Self::RoomPowerLevels(event) => &event.unsigned,
                Self::RoomServerAcl(event) => &event.unsigned,
                Self::RoomThirdPartyInvite(event) => &event.unsigned,
                Self::RoomTombstone(event) => &event.unsigned,
                Self::RoomTopic(event) => &event.unsigned,
                Self::Custom(event) => &event.unsigned,
            }
        }
    }
    impl<'de> ::serde::de::Deserialize<'de> for AnyStateEvent {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: ::serde::de::Deserializer<'de>,
        {
            use ::serde::de::Error as _;
            let json = Box::<::serde_json::value::RawValue>::deserialize(deserializer)?;
            let ::ruma_events::EventDeHelper { ev_type, .. } =
                ::ruma_events::from_raw_json_value(&json)?;
            match ev_type.as_str() {
                "m.room.aliases" => {
                    let event =
                        ::serde_json::from_str::<::ruma_events::room::aliases::AliasesEvent>(
                            json.get(),
                        )
                        .map_err(D::Error::custom)?;
                    Ok(Self::RoomAliases(event))
                }
                "m.room.avatar" => {
                    let event = ::serde_json::from_str::<::ruma_events::room::avatar::AvatarEvent>(
                        json.get(),
                    )
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomAvatar(event))
                }
                "m.room.canonical_alias" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::room::canonical_alias::CanonicalAliasEvent,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomCanonicalAlias(event))
                }
                "m.room.create" => {
                    let event = ::serde_json::from_str::<::ruma_events::room::create::CreateEvent>(
                        json.get(),
                    )
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomCreate(event))
                }
                "m.room.encryption" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::room::encryption::EncryptionEvent,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomEncryption(event))
                }
                "m.room.guest_access" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::room::guest_access::GuestAccessEvent,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomGuestAccess(event))
                }
                "m.room.history_visibility" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::room::history_visibility::HistoryVisibilityEvent,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomHistoryVisibility(event))
                }
                "m.room.join_rules" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::room::join_rules::JoinRulesEvent,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomJoinRules(event))
                }
                "m.room.member" => {
                    let event = ::serde_json::from_str::<::ruma_events::room::member::MemberEvent>(
                        json.get(),
                    )
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomMember(event))
                }
                "m.room.name" => {
                    let event =
                        ::serde_json::from_str::<::ruma_events::room::name::NameEvent>(json.get())
                            .map_err(D::Error::custom)?;
                    Ok(Self::RoomName(event))
                }
                "m.room.pinned_events" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::room::pinned_events::PinnedEventsEvent,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomPinnedEvents(event))
                }
                "m.room.power_levels" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::room::power_levels::PowerLevelsEvent,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomPowerLevels(event))
                }
                "m.room.server_acl" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::room::server_acl::ServerAclEvent,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomServerAcl(event))
                }
                "m.room.third_party_invite" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::room::third_party_invite::ThirdPartyInviteEvent,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomThirdPartyInvite(event))
                }
                "m.room.tombstone" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::room::tombstone::TombstoneEvent,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomTombstone(event))
                }
                "m.room.topic" => {
                    let event = ::serde_json::from_str::<::ruma_events::room::topic::TopicEvent>(
                        json.get(),
                    )
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomTopic(event))
                }
                event => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEvent<::ruma_events::custom::CustomEventContent>,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::Custom(event))
                }
            }
        }
    }
    /// Any state event.
    #[allow(clippy::large_enum_variant)]
    pub enum AnyStateEventStub {
        ///m.room.aliases
        RoomAliases(
            ::ruma_events::StateEventStub<::ruma_events::room::aliases::AliasesEventContent>,
        ),
        ///m.room.avatar
        RoomAvatar(::ruma_events::StateEventStub<::ruma_events::room::avatar::AvatarEventContent>),
        ///m.room.canonical_alias
        RoomCanonicalAlias(
            ::ruma_events::StateEventStub<
                ::ruma_events::room::canonical_alias::CanonicalAliasEventContent,
            >,
        ),
        ///m.room.create
        RoomCreate(::ruma_events::StateEventStub<::ruma_events::room::create::CreateEventContent>),
        ///m.room.encryption
        RoomEncryption(
            ::ruma_events::StateEventStub<::ruma_events::room::encryption::EncryptionEventContent>,
        ),
        ///m.room.guest_access
        RoomGuestAccess(
            ::ruma_events::StateEventStub<
                ::ruma_events::room::guest_access::GuestAccessEventContent,
            >,
        ),
        ///m.room.history_visibility
        RoomHistoryVisibility(
            ::ruma_events::StateEventStub<
                ::ruma_events::room::history_visibility::HistoryVisibilityEventContent,
            >,
        ),
        ///m.room.join_rules
        RoomJoinRules(
            ::ruma_events::StateEventStub<::ruma_events::room::join_rules::JoinRulesEventContent>,
        ),
        ///m.room.member
        RoomMember(::ruma_events::StateEventStub<::ruma_events::room::member::MemberEventContent>),
        ///m.room.name
        RoomName(::ruma_events::StateEventStub<::ruma_events::room::name::NameEventContent>),
        ///m.room.pinned_events
        RoomPinnedEvents(
            ::ruma_events::StateEventStub<
                ::ruma_events::room::pinned_events::PinnedEventsEventContent,
            >,
        ),
        ///m.room.power_levels
        RoomPowerLevels(
            ::ruma_events::StateEventStub<
                ::ruma_events::room::power_levels::PowerLevelsEventContent,
            >,
        ),
        ///m.room.server_acl
        RoomServerAcl(
            ::ruma_events::StateEventStub<::ruma_events::room::server_acl::ServerAclEventContent>,
        ),
        ///m.room.third_party_invite
        RoomThirdPartyInvite(
            ::ruma_events::StateEventStub<
                ::ruma_events::room::third_party_invite::ThirdPartyInviteEventContent,
            >,
        ),
        ///m.room.tombstone
        RoomTombstone(
            ::ruma_events::StateEventStub<::ruma_events::room::tombstone::TombstoneEventContent>,
        ),
        ///m.room.topic
        RoomTopic(::ruma_events::StateEventStub<::ruma_events::room::topic::TopicEventContent>),
        /// An event not defined by the Matrix specification
        Custom(::ruma_events::StateEventStub<::ruma_events::custom::CustomEventContent>),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::clone::Clone for AnyStateEventStub {
        #[inline]
        fn clone(&self) -> AnyStateEventStub {
            match (&*self,) {
                (&AnyStateEventStub::RoomAliases(ref __self_0),) => {
                    AnyStateEventStub::RoomAliases(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEventStub::RoomAvatar(ref __self_0),) => {
                    AnyStateEventStub::RoomAvatar(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEventStub::RoomCanonicalAlias(ref __self_0),) => {
                    AnyStateEventStub::RoomCanonicalAlias(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEventStub::RoomCreate(ref __self_0),) => {
                    AnyStateEventStub::RoomCreate(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEventStub::RoomEncryption(ref __self_0),) => {
                    AnyStateEventStub::RoomEncryption(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEventStub::RoomGuestAccess(ref __self_0),) => {
                    AnyStateEventStub::RoomGuestAccess(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEventStub::RoomHistoryVisibility(ref __self_0),) => {
                    AnyStateEventStub::RoomHistoryVisibility(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyStateEventStub::RoomJoinRules(ref __self_0),) => {
                    AnyStateEventStub::RoomJoinRules(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEventStub::RoomMember(ref __self_0),) => {
                    AnyStateEventStub::RoomMember(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEventStub::RoomName(ref __self_0),) => {
                    AnyStateEventStub::RoomName(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEventStub::RoomPinnedEvents(ref __self_0),) => {
                    AnyStateEventStub::RoomPinnedEvents(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEventStub::RoomPowerLevels(ref __self_0),) => {
                    AnyStateEventStub::RoomPowerLevels(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEventStub::RoomServerAcl(ref __self_0),) => {
                    AnyStateEventStub::RoomServerAcl(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEventStub::RoomThirdPartyInvite(ref __self_0),) => {
                    AnyStateEventStub::RoomThirdPartyInvite(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyStateEventStub::RoomTombstone(ref __self_0),) => {
                    AnyStateEventStub::RoomTombstone(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEventStub::RoomTopic(ref __self_0),) => {
                    AnyStateEventStub::RoomTopic(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEventStub::Custom(ref __self_0),) => {
                    AnyStateEventStub::Custom(::core::clone::Clone::clone(&(*__self_0)))
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::fmt::Debug for AnyStateEventStub {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&AnyStateEventStub::RoomAliases(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomAliases");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventStub::RoomAvatar(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomAvatar");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventStub::RoomCanonicalAlias(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomCanonicalAlias");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventStub::RoomCreate(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomCreate");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventStub::RoomEncryption(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomEncryption");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventStub::RoomGuestAccess(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomGuestAccess");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventStub::RoomHistoryVisibility(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomHistoryVisibility");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventStub::RoomJoinRules(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomJoinRules");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventStub::RoomMember(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomMember");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventStub::RoomName(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomName");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventStub::RoomPinnedEvents(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomPinnedEvents");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventStub::RoomPowerLevels(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomPowerLevels");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventStub::RoomServerAcl(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomServerAcl");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventStub::RoomThirdPartyInvite(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomThirdPartyInvite");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventStub::RoomTombstone(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomTombstone");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventStub::RoomTopic(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomTopic");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventStub::Custom(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Custom");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(rust_2018_idioms, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for AnyStateEventStub {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::export::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    AnyStateEventStub::RoomAliases(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventStub::RoomAvatar(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventStub::RoomCanonicalAlias(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventStub::RoomCreate(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventStub::RoomEncryption(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventStub::RoomGuestAccess(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventStub::RoomHistoryVisibility(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventStub::RoomJoinRules(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventStub::RoomMember(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventStub::RoomName(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventStub::RoomPinnedEvents(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventStub::RoomPowerLevels(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventStub::RoomServerAcl(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventStub::RoomThirdPartyInvite(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventStub::RoomTombstone(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventStub::RoomTopic(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventStub::Custom(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                }
            }
        }
    };
    impl AnyStateEventStub {
        /// Returns the any content enum for this event.
        pub fn content(&self) -> AnyStateEventContent {
            match self {
                Self::RoomAliases(event) => {
                    AnyStateEventContent::RoomAliases(event.content.clone())
                }
                Self::RoomAvatar(event) => AnyStateEventContent::RoomAvatar(event.content.clone()),
                Self::RoomCanonicalAlias(event) => {
                    AnyStateEventContent::RoomCanonicalAlias(event.content.clone())
                }
                Self::RoomCreate(event) => AnyStateEventContent::RoomCreate(event.content.clone()),
                Self::RoomEncryption(event) => {
                    AnyStateEventContent::RoomEncryption(event.content.clone())
                }
                Self::RoomGuestAccess(event) => {
                    AnyStateEventContent::RoomGuestAccess(event.content.clone())
                }
                Self::RoomHistoryVisibility(event) => {
                    AnyStateEventContent::RoomHistoryVisibility(event.content.clone())
                }
                Self::RoomJoinRules(event) => {
                    AnyStateEventContent::RoomJoinRules(event.content.clone())
                }
                Self::RoomMember(event) => AnyStateEventContent::RoomMember(event.content.clone()),
                Self::RoomName(event) => AnyStateEventContent::RoomName(event.content.clone()),
                Self::RoomPinnedEvents(event) => {
                    AnyStateEventContent::RoomPinnedEvents(event.content.clone())
                }
                Self::RoomPowerLevels(event) => {
                    AnyStateEventContent::RoomPowerLevels(event.content.clone())
                }
                Self::RoomServerAcl(event) => {
                    AnyStateEventContent::RoomServerAcl(event.content.clone())
                }
                Self::RoomThirdPartyInvite(event) => {
                    AnyStateEventContent::RoomThirdPartyInvite(event.content.clone())
                }
                Self::RoomTombstone(event) => {
                    AnyStateEventContent::RoomTombstone(event.content.clone())
                }
                Self::RoomTopic(event) => AnyStateEventContent::RoomTopic(event.content.clone()),
                Self::Custom(event) => AnyStateEventContent::Custom(event.content.clone()),
            }
        }
        /// Returns the any content enum for this events prev_content.
        pub fn prev_content(&self) -> Option<AnyStateEventContent> {
            match self {
                Self::RoomAliases(event) => event
                    .prev_content
                    .as_ref()
                    .map(|c| AnyStateEventContent::RoomAliases(c.clone())),
                Self::RoomAvatar(event) => {
                    event.prev_content.as_ref().map(|c| AnyStateEventContent::RoomAvatar(c.clone()))
                }
                Self::RoomCanonicalAlias(event) => event
                    .prev_content
                    .as_ref()
                    .map(|c| AnyStateEventContent::RoomCanonicalAlias(c.clone())),
                Self::RoomCreate(event) => {
                    event.prev_content.as_ref().map(|c| AnyStateEventContent::RoomCreate(c.clone()))
                }
                Self::RoomEncryption(event) => event
                    .prev_content
                    .as_ref()
                    .map(|c| AnyStateEventContent::RoomEncryption(c.clone())),
                Self::RoomGuestAccess(event) => event
                    .prev_content
                    .as_ref()
                    .map(|c| AnyStateEventContent::RoomGuestAccess(c.clone())),
                Self::RoomHistoryVisibility(event) => event
                    .prev_content
                    .as_ref()
                    .map(|c| AnyStateEventContent::RoomHistoryVisibility(c.clone())),
                Self::RoomJoinRules(event) => event
                    .prev_content
                    .as_ref()
                    .map(|c| AnyStateEventContent::RoomJoinRules(c.clone())),
                Self::RoomMember(event) => {
                    event.prev_content.as_ref().map(|c| AnyStateEventContent::RoomMember(c.clone()))
                }
                Self::RoomName(event) => {
                    event.prev_content.as_ref().map(|c| AnyStateEventContent::RoomName(c.clone()))
                }
                Self::RoomPinnedEvents(event) => event
                    .prev_content
                    .as_ref()
                    .map(|c| AnyStateEventContent::RoomPinnedEvents(c.clone())),
                Self::RoomPowerLevels(event) => event
                    .prev_content
                    .as_ref()
                    .map(|c| AnyStateEventContent::RoomPowerLevels(c.clone())),
                Self::RoomServerAcl(event) => event
                    .prev_content
                    .as_ref()
                    .map(|c| AnyStateEventContent::RoomServerAcl(c.clone())),
                Self::RoomThirdPartyInvite(event) => event
                    .prev_content
                    .as_ref()
                    .map(|c| AnyStateEventContent::RoomThirdPartyInvite(c.clone())),
                Self::RoomTombstone(event) => event
                    .prev_content
                    .as_ref()
                    .map(|c| AnyStateEventContent::RoomTombstone(c.clone())),
                Self::RoomTopic(event) => {
                    event.prev_content.as_ref().map(|c| AnyStateEventContent::RoomTopic(c.clone()))
                }
                Self::Custom(event) => {
                    event.prev_content.as_ref().map(|c| AnyStateEventContent::Custom(c.clone()))
                }
            }
        }
        ///Returns this events origin_server_ts field.
        pub fn origin_server_ts(&self) -> &::std::time::SystemTime {
            match self {
                Self::RoomAliases(event) => &event.origin_server_ts,
                Self::RoomAvatar(event) => &event.origin_server_ts,
                Self::RoomCanonicalAlias(event) => &event.origin_server_ts,
                Self::RoomCreate(event) => &event.origin_server_ts,
                Self::RoomEncryption(event) => &event.origin_server_ts,
                Self::RoomGuestAccess(event) => &event.origin_server_ts,
                Self::RoomHistoryVisibility(event) => &event.origin_server_ts,
                Self::RoomJoinRules(event) => &event.origin_server_ts,
                Self::RoomMember(event) => &event.origin_server_ts,
                Self::RoomName(event) => &event.origin_server_ts,
                Self::RoomPinnedEvents(event) => &event.origin_server_ts,
                Self::RoomPowerLevels(event) => &event.origin_server_ts,
                Self::RoomServerAcl(event) => &event.origin_server_ts,
                Self::RoomThirdPartyInvite(event) => &event.origin_server_ts,
                Self::RoomTombstone(event) => &event.origin_server_ts,
                Self::RoomTopic(event) => &event.origin_server_ts,
                Self::Custom(event) => &event.origin_server_ts,
            }
        }
        ///Returns this events event_id field.
        pub fn event_id(&self) -> &::ruma_identifiers::EventId {
            match self {
                Self::RoomAliases(event) => &event.event_id,
                Self::RoomAvatar(event) => &event.event_id,
                Self::RoomCanonicalAlias(event) => &event.event_id,
                Self::RoomCreate(event) => &event.event_id,
                Self::RoomEncryption(event) => &event.event_id,
                Self::RoomGuestAccess(event) => &event.event_id,
                Self::RoomHistoryVisibility(event) => &event.event_id,
                Self::RoomJoinRules(event) => &event.event_id,
                Self::RoomMember(event) => &event.event_id,
                Self::RoomName(event) => &event.event_id,
                Self::RoomPinnedEvents(event) => &event.event_id,
                Self::RoomPowerLevels(event) => &event.event_id,
                Self::RoomServerAcl(event) => &event.event_id,
                Self::RoomThirdPartyInvite(event) => &event.event_id,
                Self::RoomTombstone(event) => &event.event_id,
                Self::RoomTopic(event) => &event.event_id,
                Self::Custom(event) => &event.event_id,
            }
        }
        ///Returns this events sender field.
        pub fn sender(&self) -> &::ruma_identifiers::UserId {
            match self {
                Self::RoomAliases(event) => &event.sender,
                Self::RoomAvatar(event) => &event.sender,
                Self::RoomCanonicalAlias(event) => &event.sender,
                Self::RoomCreate(event) => &event.sender,
                Self::RoomEncryption(event) => &event.sender,
                Self::RoomGuestAccess(event) => &event.sender,
                Self::RoomHistoryVisibility(event) => &event.sender,
                Self::RoomJoinRules(event) => &event.sender,
                Self::RoomMember(event) => &event.sender,
                Self::RoomName(event) => &event.sender,
                Self::RoomPinnedEvents(event) => &event.sender,
                Self::RoomPowerLevels(event) => &event.sender,
                Self::RoomServerAcl(event) => &event.sender,
                Self::RoomThirdPartyInvite(event) => &event.sender,
                Self::RoomTombstone(event) => &event.sender,
                Self::RoomTopic(event) => &event.sender,
                Self::Custom(event) => &event.sender,
            }
        }
        ///Returns this events state_key field.
        pub fn state_key(&self) -> &str {
            match self {
                Self::RoomAliases(event) => &event.state_key,
                Self::RoomAvatar(event) => &event.state_key,
                Self::RoomCanonicalAlias(event) => &event.state_key,
                Self::RoomCreate(event) => &event.state_key,
                Self::RoomEncryption(event) => &event.state_key,
                Self::RoomGuestAccess(event) => &event.state_key,
                Self::RoomHistoryVisibility(event) => &event.state_key,
                Self::RoomJoinRules(event) => &event.state_key,
                Self::RoomMember(event) => &event.state_key,
                Self::RoomName(event) => &event.state_key,
                Self::RoomPinnedEvents(event) => &event.state_key,
                Self::RoomPowerLevels(event) => &event.state_key,
                Self::RoomServerAcl(event) => &event.state_key,
                Self::RoomThirdPartyInvite(event) => &event.state_key,
                Self::RoomTombstone(event) => &event.state_key,
                Self::RoomTopic(event) => &event.state_key,
                Self::Custom(event) => &event.state_key,
            }
        }
        ///Returns this events unsigned field.
        pub fn unsigned(&self) -> &::ruma_events::UnsignedData {
            match self {
                Self::RoomAliases(event) => &event.unsigned,
                Self::RoomAvatar(event) => &event.unsigned,
                Self::RoomCanonicalAlias(event) => &event.unsigned,
                Self::RoomCreate(event) => &event.unsigned,
                Self::RoomEncryption(event) => &event.unsigned,
                Self::RoomGuestAccess(event) => &event.unsigned,
                Self::RoomHistoryVisibility(event) => &event.unsigned,
                Self::RoomJoinRules(event) => &event.unsigned,
                Self::RoomMember(event) => &event.unsigned,
                Self::RoomName(event) => &event.unsigned,
                Self::RoomPinnedEvents(event) => &event.unsigned,
                Self::RoomPowerLevels(event) => &event.unsigned,
                Self::RoomServerAcl(event) => &event.unsigned,
                Self::RoomThirdPartyInvite(event) => &event.unsigned,
                Self::RoomTombstone(event) => &event.unsigned,
                Self::RoomTopic(event) => &event.unsigned,
                Self::Custom(event) => &event.unsigned,
            }
        }
    }
    impl<'de> ::serde::de::Deserialize<'de> for AnyStateEventStub {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: ::serde::de::Deserializer<'de>,
        {
            use ::serde::de::Error as _;
            let json = Box::<::serde_json::value::RawValue>::deserialize(deserializer)?;
            let ::ruma_events::EventDeHelper { ev_type, .. } =
                ::ruma_events::from_raw_json_value(&json)?;
            match ev_type.as_str() {
                "m.room.aliases" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::aliases::AliasesEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomAliases(event))
                }
                "m.room.avatar" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::avatar::AvatarEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomAvatar(event))
                }
                "m.room.canonical_alias" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::canonical_alias::CanonicalAliasEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomCanonicalAlias(event))
                }
                "m.room.create" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::create::CreateEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomCreate(event))
                }
                "m.room.encryption" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::encryption::EncryptionEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomEncryption(event))
                }
                "m.room.guest_access" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::guest_access::GuestAccessEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomGuestAccess(event))
                }
                "m.room.history_visibility" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::history_visibility::HistoryVisibilityEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomHistoryVisibility(event))
                }
                "m.room.join_rules" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::join_rules::JoinRulesEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomJoinRules(event))
                }
                "m.room.member" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::member::MemberEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomMember(event))
                }
                "m.room.name" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<::ruma_events::room::name::NameEventContent>,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomName(event))
                }
                "m.room.pinned_events" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::pinned_events::PinnedEventsEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomPinnedEvents(event))
                }
                "m.room.power_levels" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::power_levels::PowerLevelsEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomPowerLevels(event))
                }
                "m.room.server_acl" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::server_acl::ServerAclEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomServerAcl(event))
                }
                "m.room.third_party_invite" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::third_party_invite::ThirdPartyInviteEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomThirdPartyInvite(event))
                }
                "m.room.tombstone" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::tombstone::TombstoneEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomTombstone(event))
                }
                "m.room.topic" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::topic::TopicEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomTopic(event))
                }
                event => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<::ruma_events::custom::CustomEventContent>,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::Custom(event))
                }
            }
        }
    }
    /// Any state event.
    #[allow(clippy::large_enum_variant)]
    pub enum AnyStrippedStateEventStub {
        ///m.room.aliases
        RoomAliases(
            ::ruma_events::StrippedStateEventStub<
                ::ruma_events::room::aliases::AliasesEventContent,
            >,
        ),
        ///m.room.avatar
        RoomAvatar(
            ::ruma_events::StrippedStateEventStub<::ruma_events::room::avatar::AvatarEventContent>,
        ),
        ///m.room.canonical_alias
        RoomCanonicalAlias(
            ::ruma_events::StrippedStateEventStub<
                ::ruma_events::room::canonical_alias::CanonicalAliasEventContent,
            >,
        ),
        ///m.room.create
        RoomCreate(
            ::ruma_events::StrippedStateEventStub<::ruma_events::room::create::CreateEventContent>,
        ),
        ///m.room.encryption
        RoomEncryption(
            ::ruma_events::StrippedStateEventStub<
                ::ruma_events::room::encryption::EncryptionEventContent,
            >,
        ),
        ///m.room.guest_access
        RoomGuestAccess(
            ::ruma_events::StrippedStateEventStub<
                ::ruma_events::room::guest_access::GuestAccessEventContent,
            >,
        ),
        ///m.room.history_visibility
        RoomHistoryVisibility(
            ::ruma_events::StrippedStateEventStub<
                ::ruma_events::room::history_visibility::HistoryVisibilityEventContent,
            >,
        ),
        ///m.room.join_rules
        RoomJoinRules(
            ::ruma_events::StrippedStateEventStub<
                ::ruma_events::room::join_rules::JoinRulesEventContent,
            >,
        ),
        ///m.room.member
        RoomMember(
            ::ruma_events::StrippedStateEventStub<::ruma_events::room::member::MemberEventContent>,
        ),
        ///m.room.name
        RoomName(
            ::ruma_events::StrippedStateEventStub<::ruma_events::room::name::NameEventContent>,
        ),
        ///m.room.pinned_events
        RoomPinnedEvents(
            ::ruma_events::StrippedStateEventStub<
                ::ruma_events::room::pinned_events::PinnedEventsEventContent,
            >,
        ),
        ///m.room.power_levels
        RoomPowerLevels(
            ::ruma_events::StrippedStateEventStub<
                ::ruma_events::room::power_levels::PowerLevelsEventContent,
            >,
        ),
        ///m.room.server_acl
        RoomServerAcl(
            ::ruma_events::StrippedStateEventStub<
                ::ruma_events::room::server_acl::ServerAclEventContent,
            >,
        ),
        ///m.room.third_party_invite
        RoomThirdPartyInvite(
            ::ruma_events::StrippedStateEventStub<
                ::ruma_events::room::third_party_invite::ThirdPartyInviteEventContent,
            >,
        ),
        ///m.room.tombstone
        RoomTombstone(
            ::ruma_events::StrippedStateEventStub<
                ::ruma_events::room::tombstone::TombstoneEventContent,
            >,
        ),
        ///m.room.topic
        RoomTopic(
            ::ruma_events::StrippedStateEventStub<::ruma_events::room::topic::TopicEventContent>,
        ),
        /// An event not defined by the Matrix specification
        Custom(::ruma_events::StrippedStateEventStub<::ruma_events::custom::CustomEventContent>),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::clone::Clone for AnyStrippedStateEventStub {
        #[inline]
        fn clone(&self) -> AnyStrippedStateEventStub {
            match (&*self,) {
                (&AnyStrippedStateEventStub::RoomAliases(ref __self_0),) => {
                    AnyStrippedStateEventStub::RoomAliases(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyStrippedStateEventStub::RoomAvatar(ref __self_0),) => {
                    AnyStrippedStateEventStub::RoomAvatar(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStrippedStateEventStub::RoomCanonicalAlias(ref __self_0),) => {
                    AnyStrippedStateEventStub::RoomCanonicalAlias(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyStrippedStateEventStub::RoomCreate(ref __self_0),) => {
                    AnyStrippedStateEventStub::RoomCreate(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStrippedStateEventStub::RoomEncryption(ref __self_0),) => {
                    AnyStrippedStateEventStub::RoomEncryption(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyStrippedStateEventStub::RoomGuestAccess(ref __self_0),) => {
                    AnyStrippedStateEventStub::RoomGuestAccess(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyStrippedStateEventStub::RoomHistoryVisibility(ref __self_0),) => {
                    AnyStrippedStateEventStub::RoomHistoryVisibility(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyStrippedStateEventStub::RoomJoinRules(ref __self_0),) => {
                    AnyStrippedStateEventStub::RoomJoinRules(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyStrippedStateEventStub::RoomMember(ref __self_0),) => {
                    AnyStrippedStateEventStub::RoomMember(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStrippedStateEventStub::RoomName(ref __self_0),) => {
                    AnyStrippedStateEventStub::RoomName(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStrippedStateEventStub::RoomPinnedEvents(ref __self_0),) => {
                    AnyStrippedStateEventStub::RoomPinnedEvents(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyStrippedStateEventStub::RoomPowerLevels(ref __self_0),) => {
                    AnyStrippedStateEventStub::RoomPowerLevels(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyStrippedStateEventStub::RoomServerAcl(ref __self_0),) => {
                    AnyStrippedStateEventStub::RoomServerAcl(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyStrippedStateEventStub::RoomThirdPartyInvite(ref __self_0),) => {
                    AnyStrippedStateEventStub::RoomThirdPartyInvite(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyStrippedStateEventStub::RoomTombstone(ref __self_0),) => {
                    AnyStrippedStateEventStub::RoomTombstone(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyStrippedStateEventStub::RoomTopic(ref __self_0),) => {
                    AnyStrippedStateEventStub::RoomTopic(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStrippedStateEventStub::Custom(ref __self_0),) => {
                    AnyStrippedStateEventStub::Custom(::core::clone::Clone::clone(&(*__self_0)))
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::fmt::Debug for AnyStrippedStateEventStub {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&AnyStrippedStateEventStub::RoomAliases(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomAliases");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStrippedStateEventStub::RoomAvatar(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomAvatar");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStrippedStateEventStub::RoomCanonicalAlias(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomCanonicalAlias");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStrippedStateEventStub::RoomCreate(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomCreate");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStrippedStateEventStub::RoomEncryption(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomEncryption");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStrippedStateEventStub::RoomGuestAccess(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomGuestAccess");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStrippedStateEventStub::RoomHistoryVisibility(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomHistoryVisibility");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStrippedStateEventStub::RoomJoinRules(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomJoinRules");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStrippedStateEventStub::RoomMember(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomMember");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStrippedStateEventStub::RoomName(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomName");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStrippedStateEventStub::RoomPinnedEvents(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomPinnedEvents");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStrippedStateEventStub::RoomPowerLevels(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomPowerLevels");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStrippedStateEventStub::RoomServerAcl(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomServerAcl");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStrippedStateEventStub::RoomThirdPartyInvite(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomThirdPartyInvite");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStrippedStateEventStub::RoomTombstone(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomTombstone");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStrippedStateEventStub::RoomTopic(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomTopic");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStrippedStateEventStub::Custom(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Custom");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(rust_2018_idioms, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for AnyStrippedStateEventStub {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::export::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    AnyStrippedStateEventStub::RoomAliases(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStrippedStateEventStub::RoomAvatar(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStrippedStateEventStub::RoomCanonicalAlias(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStrippedStateEventStub::RoomCreate(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStrippedStateEventStub::RoomEncryption(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStrippedStateEventStub::RoomGuestAccess(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStrippedStateEventStub::RoomHistoryVisibility(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStrippedStateEventStub::RoomJoinRules(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStrippedStateEventStub::RoomMember(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStrippedStateEventStub::RoomName(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStrippedStateEventStub::RoomPinnedEvents(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStrippedStateEventStub::RoomPowerLevels(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStrippedStateEventStub::RoomServerAcl(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStrippedStateEventStub::RoomThirdPartyInvite(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStrippedStateEventStub::RoomTombstone(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStrippedStateEventStub::RoomTopic(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStrippedStateEventStub::Custom(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                }
            }
        }
    };
    impl AnyStrippedStateEventStub {
        /// Returns the any content enum for this event.
        pub fn content(&self) -> AnyStateEventContent {
            match self {
                Self::RoomAliases(event) => {
                    AnyStateEventContent::RoomAliases(event.content.clone())
                }
                Self::RoomAvatar(event) => AnyStateEventContent::RoomAvatar(event.content.clone()),
                Self::RoomCanonicalAlias(event) => {
                    AnyStateEventContent::RoomCanonicalAlias(event.content.clone())
                }
                Self::RoomCreate(event) => AnyStateEventContent::RoomCreate(event.content.clone()),
                Self::RoomEncryption(event) => {
                    AnyStateEventContent::RoomEncryption(event.content.clone())
                }
                Self::RoomGuestAccess(event) => {
                    AnyStateEventContent::RoomGuestAccess(event.content.clone())
                }
                Self::RoomHistoryVisibility(event) => {
                    AnyStateEventContent::RoomHistoryVisibility(event.content.clone())
                }
                Self::RoomJoinRules(event) => {
                    AnyStateEventContent::RoomJoinRules(event.content.clone())
                }
                Self::RoomMember(event) => AnyStateEventContent::RoomMember(event.content.clone()),
                Self::RoomName(event) => AnyStateEventContent::RoomName(event.content.clone()),
                Self::RoomPinnedEvents(event) => {
                    AnyStateEventContent::RoomPinnedEvents(event.content.clone())
                }
                Self::RoomPowerLevels(event) => {
                    AnyStateEventContent::RoomPowerLevels(event.content.clone())
                }
                Self::RoomServerAcl(event) => {
                    AnyStateEventContent::RoomServerAcl(event.content.clone())
                }
                Self::RoomThirdPartyInvite(event) => {
                    AnyStateEventContent::RoomThirdPartyInvite(event.content.clone())
                }
                Self::RoomTombstone(event) => {
                    AnyStateEventContent::RoomTombstone(event.content.clone())
                }
                Self::RoomTopic(event) => AnyStateEventContent::RoomTopic(event.content.clone()),
                Self::Custom(event) => AnyStateEventContent::Custom(event.content.clone()),
            }
        }
        ///Returns this events sender field.
        pub fn sender(&self) -> &::ruma_identifiers::UserId {
            match self {
                Self::RoomAliases(event) => &event.sender,
                Self::RoomAvatar(event) => &event.sender,
                Self::RoomCanonicalAlias(event) => &event.sender,
                Self::RoomCreate(event) => &event.sender,
                Self::RoomEncryption(event) => &event.sender,
                Self::RoomGuestAccess(event) => &event.sender,
                Self::RoomHistoryVisibility(event) => &event.sender,
                Self::RoomJoinRules(event) => &event.sender,
                Self::RoomMember(event) => &event.sender,
                Self::RoomName(event) => &event.sender,
                Self::RoomPinnedEvents(event) => &event.sender,
                Self::RoomPowerLevels(event) => &event.sender,
                Self::RoomServerAcl(event) => &event.sender,
                Self::RoomThirdPartyInvite(event) => &event.sender,
                Self::RoomTombstone(event) => &event.sender,
                Self::RoomTopic(event) => &event.sender,
                Self::Custom(event) => &event.sender,
            }
        }
        ///Returns this events state_key field.
        pub fn state_key(&self) -> &str {
            match self {
                Self::RoomAliases(event) => &event.state_key,
                Self::RoomAvatar(event) => &event.state_key,
                Self::RoomCanonicalAlias(event) => &event.state_key,
                Self::RoomCreate(event) => &event.state_key,
                Self::RoomEncryption(event) => &event.state_key,
                Self::RoomGuestAccess(event) => &event.state_key,
                Self::RoomHistoryVisibility(event) => &event.state_key,
                Self::RoomJoinRules(event) => &event.state_key,
                Self::RoomMember(event) => &event.state_key,
                Self::RoomName(event) => &event.state_key,
                Self::RoomPinnedEvents(event) => &event.state_key,
                Self::RoomPowerLevels(event) => &event.state_key,
                Self::RoomServerAcl(event) => &event.state_key,
                Self::RoomThirdPartyInvite(event) => &event.state_key,
                Self::RoomTombstone(event) => &event.state_key,
                Self::RoomTopic(event) => &event.state_key,
                Self::Custom(event) => &event.state_key,
            }
        }
    }
    impl<'de> ::serde::de::Deserialize<'de> for AnyStrippedStateEventStub {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: ::serde::de::Deserializer<'de>,
        {
            use ::serde::de::Error as _;
            let json = Box::<::serde_json::value::RawValue>::deserialize(deserializer)?;
            let ::ruma_events::EventDeHelper { ev_type, .. } =
                ::ruma_events::from_raw_json_value(&json)?;
            match ev_type.as_str() {
                "m.room.aliases" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::aliases::AliasesEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomAliases(event))
                }
                "m.room.avatar" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::avatar::AvatarEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomAvatar(event))
                }
                "m.room.canonical_alias" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::canonical_alias::CanonicalAliasEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomCanonicalAlias(event))
                }
                "m.room.create" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::create::CreateEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomCreate(event))
                }
                "m.room.encryption" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::encryption::EncryptionEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomEncryption(event))
                }
                "m.room.guest_access" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::guest_access::GuestAccessEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomGuestAccess(event))
                }
                "m.room.history_visibility" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::history_visibility::HistoryVisibilityEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomHistoryVisibility(event))
                }
                "m.room.join_rules" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::join_rules::JoinRulesEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomJoinRules(event))
                }
                "m.room.member" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::member::MemberEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomMember(event))
                }
                "m.room.name" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::name::NameEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomName(event))
                }
                "m.room.pinned_events" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::pinned_events::PinnedEventsEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomPinnedEvents(event))
                }
                "m.room.power_levels" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::power_levels::PowerLevelsEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomPowerLevels(event))
                }
                "m.room.server_acl" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::server_acl::ServerAclEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomServerAcl(event))
                }
                "m.room.third_party_invite" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::third_party_invite::ThirdPartyInviteEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomThirdPartyInvite(event))
                }
                "m.room.tombstone" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::tombstone::TombstoneEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomTombstone(event))
                }
                "m.room.topic" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::topic::TopicEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomTopic(event))
                }
                event => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::custom::CustomEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::Custom(event))
                }
            }
        }
    }
    /// Any state event.
    #[allow(clippy::large_enum_variant)]
    pub enum AnyRedactedStateEventContent {
        ///m.room.aliases
        RoomAliases(::ruma_events::room::aliases::AliasesEventContent),
        ///m.room.avatar
        RoomAvatar(::ruma_events::room::avatar::AvatarEventContent),
        ///m.room.canonical_alias
        RoomCanonicalAlias(::ruma_events::room::canonical_alias::CanonicalAliasEventContent),
        ///m.room.create
        RoomCreate(::ruma_events::room::create::CreateEventContent),
        ///m.room.encryption
        RoomEncryption(::ruma_events::room::encryption::EncryptionEventContent),
        ///m.room.guest_access
        RoomGuestAccess(::ruma_events::room::guest_access::GuestAccessEventContent),
        ///m.room.history_visibility
        RoomHistoryVisibility(
            ::ruma_events::room::history_visibility::HistoryVisibilityEventContent,
        ),
        ///m.room.join_rules
        RoomJoinRules(::ruma_events::room::join_rules::JoinRulesEventContent),
        ///m.room.member
        RoomMember(::ruma_events::room::member::MemberEventContent),
        ///m.room.name
        RoomName(::ruma_events::room::name::NameEventContent),
        ///m.room.pinned_events
        RoomPinnedEvents(::ruma_events::room::pinned_events::PinnedEventsEventContent),
        ///m.room.power_levels
        RoomPowerLevels(::ruma_events::room::power_levels::PowerLevelsEventContent),
        ///m.room.server_acl
        RoomServerAcl(::ruma_events::room::server_acl::ServerAclEventContent),
        ///m.room.third_party_invite
        RoomThirdPartyInvite(::ruma_events::room::third_party_invite::ThirdPartyInviteEventContent),
        ///m.room.tombstone
        RoomTombstone(::ruma_events::room::tombstone::TombstoneEventContent),
        ///m.room.topic
        RoomTopic(::ruma_events::room::topic::TopicEventContent),
        /// Content of an event not defined by the Matrix specification.
        Custom(::ruma_events::custom::CustomEventContent),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::clone::Clone for AnyRedactedStateEventContent {
        #[inline]
        fn clone(&self) -> AnyRedactedStateEventContent {
            match (&*self,) {
                (&AnyRedactedStateEventContent::RoomAliases(ref __self_0),) => {
                    AnyRedactedStateEventContent::RoomAliases(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEventContent::RoomAvatar(ref __self_0),) => {
                    AnyRedactedStateEventContent::RoomAvatar(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEventContent::RoomCanonicalAlias(ref __self_0),) => {
                    AnyRedactedStateEventContent::RoomCanonicalAlias(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEventContent::RoomCreate(ref __self_0),) => {
                    AnyRedactedStateEventContent::RoomCreate(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEventContent::RoomEncryption(ref __self_0),) => {
                    AnyRedactedStateEventContent::RoomEncryption(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEventContent::RoomGuestAccess(ref __self_0),) => {
                    AnyRedactedStateEventContent::RoomGuestAccess(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEventContent::RoomHistoryVisibility(ref __self_0),) => {
                    AnyRedactedStateEventContent::RoomHistoryVisibility(
                        ::core::clone::Clone::clone(&(*__self_0)),
                    )
                }
                (&AnyRedactedStateEventContent::RoomJoinRules(ref __self_0),) => {
                    AnyRedactedStateEventContent::RoomJoinRules(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEventContent::RoomMember(ref __self_0),) => {
                    AnyRedactedStateEventContent::RoomMember(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEventContent::RoomName(ref __self_0),) => {
                    AnyRedactedStateEventContent::RoomName(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEventContent::RoomPinnedEvents(ref __self_0),) => {
                    AnyRedactedStateEventContent::RoomPinnedEvents(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEventContent::RoomPowerLevels(ref __self_0),) => {
                    AnyRedactedStateEventContent::RoomPowerLevels(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEventContent::RoomServerAcl(ref __self_0),) => {
                    AnyRedactedStateEventContent::RoomServerAcl(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEventContent::RoomThirdPartyInvite(ref __self_0),) => {
                    AnyRedactedStateEventContent::RoomThirdPartyInvite(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEventContent::RoomTombstone(ref __self_0),) => {
                    AnyRedactedStateEventContent::RoomTombstone(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEventContent::RoomTopic(ref __self_0),) => {
                    AnyRedactedStateEventContent::RoomTopic(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEventContent::Custom(ref __self_0),) => {
                    AnyRedactedStateEventContent::Custom(::core::clone::Clone::clone(&(*__self_0)))
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::fmt::Debug for AnyRedactedStateEventContent {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&AnyRedactedStateEventContent::RoomAliases(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomAliases");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventContent::RoomAvatar(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomAvatar");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventContent::RoomCanonicalAlias(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomCanonicalAlias");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventContent::RoomCreate(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomCreate");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventContent::RoomEncryption(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomEncryption");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventContent::RoomGuestAccess(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomGuestAccess");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventContent::RoomHistoryVisibility(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomHistoryVisibility");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventContent::RoomJoinRules(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomJoinRules");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventContent::RoomMember(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomMember");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventContent::RoomName(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomName");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventContent::RoomPinnedEvents(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomPinnedEvents");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventContent::RoomPowerLevels(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomPowerLevels");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventContent::RoomServerAcl(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomServerAcl");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventContent::RoomThirdPartyInvite(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomThirdPartyInvite");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventContent::RoomTombstone(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomTombstone");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventContent::RoomTopic(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomTopic");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventContent::Custom(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Custom");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(rust_2018_idioms, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for AnyRedactedStateEventContent {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::export::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    AnyRedactedStateEventContent::RoomAliases(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventContent::RoomAvatar(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventContent::RoomCanonicalAlias(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventContent::RoomCreate(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventContent::RoomEncryption(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventContent::RoomGuestAccess(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventContent::RoomHistoryVisibility(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventContent::RoomJoinRules(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventContent::RoomMember(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventContent::RoomName(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventContent::RoomPinnedEvents(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventContent::RoomPowerLevels(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventContent::RoomServerAcl(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventContent::RoomThirdPartyInvite(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventContent::RoomTombstone(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventContent::RoomTopic(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventContent::Custom(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                }
            }
        }
    };
    impl ::ruma_events::EventContent for AnyRedactedStateEventContent {
        fn event_type(&self) -> &str {
            match self {
                Self::RoomAliases(content) => content.event_type(),
                Self::RoomAvatar(content) => content.event_type(),
                Self::RoomCanonicalAlias(content) => content.event_type(),
                Self::RoomCreate(content) => content.event_type(),
                Self::RoomEncryption(content) => content.event_type(),
                Self::RoomGuestAccess(content) => content.event_type(),
                Self::RoomHistoryVisibility(content) => content.event_type(),
                Self::RoomJoinRules(content) => content.event_type(),
                Self::RoomMember(content) => content.event_type(),
                Self::RoomName(content) => content.event_type(),
                Self::RoomPinnedEvents(content) => content.event_type(),
                Self::RoomPowerLevels(content) => content.event_type(),
                Self::RoomServerAcl(content) => content.event_type(),
                Self::RoomThirdPartyInvite(content) => content.event_type(),
                Self::RoomTombstone(content) => content.event_type(),
                Self::RoomTopic(content) => content.event_type(),
                Self::Custom(content) => content.event_type(),
            }
        }
        fn from_parts(
            event_type: &str,
            input: Box<::serde_json::value::RawValue>,
        ) -> Result<Self, ::serde_json::Error> {
            match event_type {
                "m.room.aliases" => {
                    let content = ::ruma_events::room::aliases::AliasesEventContent::from_parts(
                        event_type, input,
                    )?;
                    Ok(Self::RoomAliases(content))
                }
                "m.room.avatar" => {
                    let content = ::ruma_events::room::avatar::AvatarEventContent::from_parts(
                        event_type, input,
                    )?;
                    Ok(Self::RoomAvatar(content))
                }
                "m.room.canonical_alias" => {
                    let content = :: ruma_events :: room :: canonical_alias :: CanonicalAliasEventContent :: from_parts ( event_type , input ) ? ;
                    Ok(Self::RoomCanonicalAlias(content))
                }
                "m.room.create" => {
                    let content = ::ruma_events::room::create::CreateEventContent::from_parts(
                        event_type, input,
                    )?;
                    Ok(Self::RoomCreate(content))
                }
                "m.room.encryption" => {
                    let content =
                        ::ruma_events::room::encryption::EncryptionEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::RoomEncryption(content))
                }
                "m.room.guest_access" => {
                    let content =
                        ::ruma_events::room::guest_access::GuestAccessEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::RoomGuestAccess(content))
                }
                "m.room.history_visibility" => {
                    let content = :: ruma_events :: room :: history_visibility :: HistoryVisibilityEventContent :: from_parts ( event_type , input ) ? ;
                    Ok(Self::RoomHistoryVisibility(content))
                }
                "m.room.join_rules" => {
                    let content =
                        ::ruma_events::room::join_rules::JoinRulesEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::RoomJoinRules(content))
                }
                "m.room.member" => {
                    let content = ::ruma_events::room::member::MemberEventContent::from_parts(
                        event_type, input,
                    )?;
                    Ok(Self::RoomMember(content))
                }
                "m.room.name" => {
                    let content =
                        ::ruma_events::room::name::NameEventContent::from_parts(event_type, input)?;
                    Ok(Self::RoomName(content))
                }
                "m.room.pinned_events" => {
                    let content =
                        ::ruma_events::room::pinned_events::PinnedEventsEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::RoomPinnedEvents(content))
                }
                "m.room.power_levels" => {
                    let content =
                        ::ruma_events::room::power_levels::PowerLevelsEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::RoomPowerLevels(content))
                }
                "m.room.server_acl" => {
                    let content =
                        ::ruma_events::room::server_acl::ServerAclEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::RoomServerAcl(content))
                }
                "m.room.third_party_invite" => {
                    let content = :: ruma_events :: room :: third_party_invite :: ThirdPartyInviteEventContent :: from_parts ( event_type , input ) ? ;
                    Ok(Self::RoomThirdPartyInvite(content))
                }
                "m.room.tombstone" => {
                    let content =
                        ::ruma_events::room::tombstone::TombstoneEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::RoomTombstone(content))
                }
                "m.room.topic" => {
                    let content = ::ruma_events::room::topic::TopicEventContent::from_parts(
                        event_type, input,
                    )?;
                    Ok(Self::RoomTopic(content))
                }
                ev_type => {
                    let content =
                        ::ruma_events::custom::CustomEventContent::from_parts(ev_type, input)?;
                    Ok(Self::Custom(content))
                }
            }
        }
    }
    /// Any state event.
    #[allow(clippy::large_enum_variant)]
    pub enum AnyRedactedStateEvent {
        ///m.room.aliases
        RoomAliases(::ruma_events::room::aliases::AliasesEvent),
        ///m.room.avatar
        RoomAvatar(::ruma_events::room::avatar::AvatarEvent),
        ///m.room.canonical_alias
        RoomCanonicalAlias(::ruma_events::room::canonical_alias::CanonicalAliasEvent),
        ///m.room.create
        RoomCreate(::ruma_events::room::create::CreateEvent),
        ///m.room.encryption
        RoomEncryption(::ruma_events::room::encryption::EncryptionEvent),
        ///m.room.guest_access
        RoomGuestAccess(::ruma_events::room::guest_access::GuestAccessEvent),
        ///m.room.history_visibility
        RoomHistoryVisibility(::ruma_events::room::history_visibility::HistoryVisibilityEvent),
        ///m.room.join_rules
        RoomJoinRules(::ruma_events::room::join_rules::JoinRulesEvent),
        ///m.room.member
        RoomMember(::ruma_events::room::member::MemberEvent),
        ///m.room.name
        RoomName(::ruma_events::room::name::NameEvent),
        ///m.room.pinned_events
        RoomPinnedEvents(::ruma_events::room::pinned_events::PinnedEventsEvent),
        ///m.room.power_levels
        RoomPowerLevels(::ruma_events::room::power_levels::PowerLevelsEvent),
        ///m.room.server_acl
        RoomServerAcl(::ruma_events::room::server_acl::ServerAclEvent),
        ///m.room.third_party_invite
        RoomThirdPartyInvite(::ruma_events::room::third_party_invite::ThirdPartyInviteEvent),
        ///m.room.tombstone
        RoomTombstone(::ruma_events::room::tombstone::TombstoneEvent),
        ///m.room.topic
        RoomTopic(::ruma_events::room::topic::TopicEvent),
        /// An event not defined by the Matrix specification
        Custom(::ruma_events::StateEvent<::ruma_events::custom::CustomEventContent>),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::clone::Clone for AnyRedactedStateEvent {
        #[inline]
        fn clone(&self) -> AnyRedactedStateEvent {
            match (&*self,) {
                (&AnyRedactedStateEvent::RoomAliases(ref __self_0),) => {
                    AnyRedactedStateEvent::RoomAliases(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyRedactedStateEvent::RoomAvatar(ref __self_0),) => {
                    AnyRedactedStateEvent::RoomAvatar(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyRedactedStateEvent::RoomCanonicalAlias(ref __self_0),) => {
                    AnyRedactedStateEvent::RoomCanonicalAlias(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEvent::RoomCreate(ref __self_0),) => {
                    AnyRedactedStateEvent::RoomCreate(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyRedactedStateEvent::RoomEncryption(ref __self_0),) => {
                    AnyRedactedStateEvent::RoomEncryption(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyRedactedStateEvent::RoomGuestAccess(ref __self_0),) => {
                    AnyRedactedStateEvent::RoomGuestAccess(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEvent::RoomHistoryVisibility(ref __self_0),) => {
                    AnyRedactedStateEvent::RoomHistoryVisibility(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEvent::RoomJoinRules(ref __self_0),) => {
                    AnyRedactedStateEvent::RoomJoinRules(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyRedactedStateEvent::RoomMember(ref __self_0),) => {
                    AnyRedactedStateEvent::RoomMember(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyRedactedStateEvent::RoomName(ref __self_0),) => {
                    AnyRedactedStateEvent::RoomName(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyRedactedStateEvent::RoomPinnedEvents(ref __self_0),) => {
                    AnyRedactedStateEvent::RoomPinnedEvents(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEvent::RoomPowerLevels(ref __self_0),) => {
                    AnyRedactedStateEvent::RoomPowerLevels(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEvent::RoomServerAcl(ref __self_0),) => {
                    AnyRedactedStateEvent::RoomServerAcl(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyRedactedStateEvent::RoomThirdPartyInvite(ref __self_0),) => {
                    AnyRedactedStateEvent::RoomThirdPartyInvite(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEvent::RoomTombstone(ref __self_0),) => {
                    AnyRedactedStateEvent::RoomTombstone(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyRedactedStateEvent::RoomTopic(ref __self_0),) => {
                    AnyRedactedStateEvent::RoomTopic(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyRedactedStateEvent::Custom(ref __self_0),) => {
                    AnyRedactedStateEvent::Custom(::core::clone::Clone::clone(&(*__self_0)))
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::fmt::Debug for AnyRedactedStateEvent {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&AnyRedactedStateEvent::RoomAliases(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomAliases");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEvent::RoomAvatar(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomAvatar");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEvent::RoomCanonicalAlias(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomCanonicalAlias");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEvent::RoomCreate(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomCreate");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEvent::RoomEncryption(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomEncryption");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEvent::RoomGuestAccess(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomGuestAccess");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEvent::RoomHistoryVisibility(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomHistoryVisibility");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEvent::RoomJoinRules(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomJoinRules");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEvent::RoomMember(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomMember");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEvent::RoomName(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomName");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEvent::RoomPinnedEvents(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomPinnedEvents");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEvent::RoomPowerLevels(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomPowerLevels");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEvent::RoomServerAcl(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomServerAcl");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEvent::RoomThirdPartyInvite(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomThirdPartyInvite");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEvent::RoomTombstone(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomTombstone");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEvent::RoomTopic(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomTopic");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEvent::Custom(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Custom");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(rust_2018_idioms, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for AnyRedactedStateEvent {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::export::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    AnyRedactedStateEvent::RoomAliases(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEvent::RoomAvatar(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEvent::RoomCanonicalAlias(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEvent::RoomCreate(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEvent::RoomEncryption(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEvent::RoomGuestAccess(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEvent::RoomHistoryVisibility(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEvent::RoomJoinRules(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEvent::RoomMember(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEvent::RoomName(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEvent::RoomPinnedEvents(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEvent::RoomPowerLevels(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEvent::RoomServerAcl(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEvent::RoomThirdPartyInvite(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEvent::RoomTombstone(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEvent::RoomTopic(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEvent::Custom(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                }
            }
        }
    };
    impl AnyRedactedStateEvent {
        /// Returns the any content enum for this event.
        pub fn content(&self) -> AnyRedactedStateEventContent {
            match self {
                Self::RoomAliases(event) => {
                    AnyRedactedStateEventContent::RoomAliases(event.content.clone())
                }
                Self::RoomAvatar(event) => {
                    AnyRedactedStateEventContent::RoomAvatar(event.content.clone())
                }
                Self::RoomCanonicalAlias(event) => {
                    AnyRedactedStateEventContent::RoomCanonicalAlias(event.content.clone())
                }
                Self::RoomCreate(event) => {
                    AnyRedactedStateEventContent::RoomCreate(event.content.clone())
                }
                Self::RoomEncryption(event) => {
                    AnyRedactedStateEventContent::RoomEncryption(event.content.clone())
                }
                Self::RoomGuestAccess(event) => {
                    AnyRedactedStateEventContent::RoomGuestAccess(event.content.clone())
                }
                Self::RoomHistoryVisibility(event) => {
                    AnyRedactedStateEventContent::RoomHistoryVisibility(event.content.clone())
                }
                Self::RoomJoinRules(event) => {
                    AnyRedactedStateEventContent::RoomJoinRules(event.content.clone())
                }
                Self::RoomMember(event) => {
                    AnyRedactedStateEventContent::RoomMember(event.content.clone())
                }
                Self::RoomName(event) => {
                    AnyRedactedStateEventContent::RoomName(event.content.clone())
                }
                Self::RoomPinnedEvents(event) => {
                    AnyRedactedStateEventContent::RoomPinnedEvents(event.content.clone())
                }
                Self::RoomPowerLevels(event) => {
                    AnyRedactedStateEventContent::RoomPowerLevels(event.content.clone())
                }
                Self::RoomServerAcl(event) => {
                    AnyRedactedStateEventContent::RoomServerAcl(event.content.clone())
                }
                Self::RoomThirdPartyInvite(event) => {
                    AnyRedactedStateEventContent::RoomThirdPartyInvite(event.content.clone())
                }
                Self::RoomTombstone(event) => {
                    AnyRedactedStateEventContent::RoomTombstone(event.content.clone())
                }
                Self::RoomTopic(event) => {
                    AnyRedactedStateEventContent::RoomTopic(event.content.clone())
                }
                Self::Custom(event) => AnyRedactedStateEventContent::Custom(event.content.clone()),
            }
        }
    }
    impl<'de> ::serde::de::Deserialize<'de> for AnyRedactedStateEvent {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: ::serde::de::Deserializer<'de>,
        {
            use ::serde::de::Error as _;
            let json = Box::<::serde_json::value::RawValue>::deserialize(deserializer)?;
            let ::ruma_events::EventDeHelper { ev_type, .. } =
                ::ruma_events::from_raw_json_value(&json)?;
            match ev_type.as_str() {
                "m.room.aliases" => {
                    let event =
                        ::serde_json::from_str::<::ruma_events::room::aliases::AliasesEvent>(
                            json.get(),
                        )
                        .map_err(D::Error::custom)?;
                    Ok(Self::RoomAliases(event))
                }
                "m.room.avatar" => {
                    let event = ::serde_json::from_str::<::ruma_events::room::avatar::AvatarEvent>(
                        json.get(),
                    )
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomAvatar(event))
                }
                "m.room.canonical_alias" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::room::canonical_alias::CanonicalAliasEvent,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomCanonicalAlias(event))
                }
                "m.room.create" => {
                    let event = ::serde_json::from_str::<::ruma_events::room::create::CreateEvent>(
                        json.get(),
                    )
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomCreate(event))
                }
                "m.room.encryption" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::room::encryption::EncryptionEvent,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomEncryption(event))
                }
                "m.room.guest_access" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::room::guest_access::GuestAccessEvent,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomGuestAccess(event))
                }
                "m.room.history_visibility" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::room::history_visibility::HistoryVisibilityEvent,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomHistoryVisibility(event))
                }
                "m.room.join_rules" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::room::join_rules::JoinRulesEvent,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomJoinRules(event))
                }
                "m.room.member" => {
                    let event = ::serde_json::from_str::<::ruma_events::room::member::MemberEvent>(
                        json.get(),
                    )
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomMember(event))
                }
                "m.room.name" => {
                    let event =
                        ::serde_json::from_str::<::ruma_events::room::name::NameEvent>(json.get())
                            .map_err(D::Error::custom)?;
                    Ok(Self::RoomName(event))
                }
                "m.room.pinned_events" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::room::pinned_events::PinnedEventsEvent,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomPinnedEvents(event))
                }
                "m.room.power_levels" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::room::power_levels::PowerLevelsEvent,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomPowerLevels(event))
                }
                "m.room.server_acl" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::room::server_acl::ServerAclEvent,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomServerAcl(event))
                }
                "m.room.third_party_invite" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::room::third_party_invite::ThirdPartyInviteEvent,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomThirdPartyInvite(event))
                }
                "m.room.tombstone" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::room::tombstone::TombstoneEvent,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomTombstone(event))
                }
                "m.room.topic" => {
                    let event = ::serde_json::from_str::<::ruma_events::room::topic::TopicEvent>(
                        json.get(),
                    )
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomTopic(event))
                }
                event => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEvent<::ruma_events::custom::CustomEventContent>,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::Custom(event))
                }
            }
        }
    }
    /// Any state event.
    #[allow(clippy::large_enum_variant)]
    pub enum AnyRedactedStateEventStub {
        ///m.room.aliases
        RoomAliases(
            ::ruma_events::StateEventStub<::ruma_events::room::aliases::AliasesEventContent>,
        ),
        ///m.room.avatar
        RoomAvatar(::ruma_events::StateEventStub<::ruma_events::room::avatar::AvatarEventContent>),
        ///m.room.canonical_alias
        RoomCanonicalAlias(
            ::ruma_events::StateEventStub<
                ::ruma_events::room::canonical_alias::CanonicalAliasEventContent,
            >,
        ),
        ///m.room.create
        RoomCreate(::ruma_events::StateEventStub<::ruma_events::room::create::CreateEventContent>),
        ///m.room.encryption
        RoomEncryption(
            ::ruma_events::StateEventStub<::ruma_events::room::encryption::EncryptionEventContent>,
        ),
        ///m.room.guest_access
        RoomGuestAccess(
            ::ruma_events::StateEventStub<
                ::ruma_events::room::guest_access::GuestAccessEventContent,
            >,
        ),
        ///m.room.history_visibility
        RoomHistoryVisibility(
            ::ruma_events::StateEventStub<
                ::ruma_events::room::history_visibility::HistoryVisibilityEventContent,
            >,
        ),
        ///m.room.join_rules
        RoomJoinRules(
            ::ruma_events::StateEventStub<::ruma_events::room::join_rules::JoinRulesEventContent>,
        ),
        ///m.room.member
        RoomMember(::ruma_events::StateEventStub<::ruma_events::room::member::MemberEventContent>),
        ///m.room.name
        RoomName(::ruma_events::StateEventStub<::ruma_events::room::name::NameEventContent>),
        ///m.room.pinned_events
        RoomPinnedEvents(
            ::ruma_events::StateEventStub<
                ::ruma_events::room::pinned_events::PinnedEventsEventContent,
            >,
        ),
        ///m.room.power_levels
        RoomPowerLevels(
            ::ruma_events::StateEventStub<
                ::ruma_events::room::power_levels::PowerLevelsEventContent,
            >,
        ),
        ///m.room.server_acl
        RoomServerAcl(
            ::ruma_events::StateEventStub<::ruma_events::room::server_acl::ServerAclEventContent>,
        ),
        ///m.room.third_party_invite
        RoomThirdPartyInvite(
            ::ruma_events::StateEventStub<
                ::ruma_events::room::third_party_invite::ThirdPartyInviteEventContent,
            >,
        ),
        ///m.room.tombstone
        RoomTombstone(
            ::ruma_events::StateEventStub<::ruma_events::room::tombstone::TombstoneEventContent>,
        ),
        ///m.room.topic
        RoomTopic(::ruma_events::StateEventStub<::ruma_events::room::topic::TopicEventContent>),
        /// An event not defined by the Matrix specification
        Custom(::ruma_events::StateEventStub<::ruma_events::custom::CustomEventContent>),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::clone::Clone for AnyRedactedStateEventStub {
        #[inline]
        fn clone(&self) -> AnyRedactedStateEventStub {
            match (&*self,) {
                (&AnyRedactedStateEventStub::RoomAliases(ref __self_0),) => {
                    AnyRedactedStateEventStub::RoomAliases(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEventStub::RoomAvatar(ref __self_0),) => {
                    AnyRedactedStateEventStub::RoomAvatar(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyRedactedStateEventStub::RoomCanonicalAlias(ref __self_0),) => {
                    AnyRedactedStateEventStub::RoomCanonicalAlias(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEventStub::RoomCreate(ref __self_0),) => {
                    AnyRedactedStateEventStub::RoomCreate(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyRedactedStateEventStub::RoomEncryption(ref __self_0),) => {
                    AnyRedactedStateEventStub::RoomEncryption(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEventStub::RoomGuestAccess(ref __self_0),) => {
                    AnyRedactedStateEventStub::RoomGuestAccess(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEventStub::RoomHistoryVisibility(ref __self_0),) => {
                    AnyRedactedStateEventStub::RoomHistoryVisibility(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEventStub::RoomJoinRules(ref __self_0),) => {
                    AnyRedactedStateEventStub::RoomJoinRules(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEventStub::RoomMember(ref __self_0),) => {
                    AnyRedactedStateEventStub::RoomMember(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyRedactedStateEventStub::RoomName(ref __self_0),) => {
                    AnyRedactedStateEventStub::RoomName(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyRedactedStateEventStub::RoomPinnedEvents(ref __self_0),) => {
                    AnyRedactedStateEventStub::RoomPinnedEvents(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEventStub::RoomPowerLevels(ref __self_0),) => {
                    AnyRedactedStateEventStub::RoomPowerLevels(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEventStub::RoomServerAcl(ref __self_0),) => {
                    AnyRedactedStateEventStub::RoomServerAcl(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEventStub::RoomThirdPartyInvite(ref __self_0),) => {
                    AnyRedactedStateEventStub::RoomThirdPartyInvite(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEventStub::RoomTombstone(ref __self_0),) => {
                    AnyRedactedStateEventStub::RoomTombstone(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStateEventStub::RoomTopic(ref __self_0),) => {
                    AnyRedactedStateEventStub::RoomTopic(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyRedactedStateEventStub::Custom(ref __self_0),) => {
                    AnyRedactedStateEventStub::Custom(::core::clone::Clone::clone(&(*__self_0)))
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::fmt::Debug for AnyRedactedStateEventStub {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&AnyRedactedStateEventStub::RoomAliases(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomAliases");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventStub::RoomAvatar(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomAvatar");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventStub::RoomCanonicalAlias(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomCanonicalAlias");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventStub::RoomCreate(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomCreate");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventStub::RoomEncryption(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomEncryption");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventStub::RoomGuestAccess(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomGuestAccess");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventStub::RoomHistoryVisibility(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomHistoryVisibility");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventStub::RoomJoinRules(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomJoinRules");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventStub::RoomMember(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomMember");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventStub::RoomName(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomName");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventStub::RoomPinnedEvents(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomPinnedEvents");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventStub::RoomPowerLevels(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomPowerLevels");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventStub::RoomServerAcl(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomServerAcl");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventStub::RoomThirdPartyInvite(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomThirdPartyInvite");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventStub::RoomTombstone(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomTombstone");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventStub::RoomTopic(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomTopic");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStateEventStub::Custom(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Custom");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(rust_2018_idioms, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for AnyRedactedStateEventStub {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::export::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    AnyRedactedStateEventStub::RoomAliases(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventStub::RoomAvatar(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventStub::RoomCanonicalAlias(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventStub::RoomCreate(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventStub::RoomEncryption(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventStub::RoomGuestAccess(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventStub::RoomHistoryVisibility(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventStub::RoomJoinRules(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventStub::RoomMember(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventStub::RoomName(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventStub::RoomPinnedEvents(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventStub::RoomPowerLevels(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventStub::RoomServerAcl(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventStub::RoomThirdPartyInvite(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventStub::RoomTombstone(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventStub::RoomTopic(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStateEventStub::Custom(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                }
            }
        }
    };
    impl AnyRedactedStateEventStub {
        /// Returns the any content enum for this event.
        pub fn content(&self) -> AnyRedactedStateEventContent {
            match self {
                Self::RoomAliases(event) => {
                    AnyRedactedStateEventContent::RoomAliases(event.content.clone())
                }
                Self::RoomAvatar(event) => {
                    AnyRedactedStateEventContent::RoomAvatar(event.content.clone())
                }
                Self::RoomCanonicalAlias(event) => {
                    AnyRedactedStateEventContent::RoomCanonicalAlias(event.content.clone())
                }
                Self::RoomCreate(event) => {
                    AnyRedactedStateEventContent::RoomCreate(event.content.clone())
                }
                Self::RoomEncryption(event) => {
                    AnyRedactedStateEventContent::RoomEncryption(event.content.clone())
                }
                Self::RoomGuestAccess(event) => {
                    AnyRedactedStateEventContent::RoomGuestAccess(event.content.clone())
                }
                Self::RoomHistoryVisibility(event) => {
                    AnyRedactedStateEventContent::RoomHistoryVisibility(event.content.clone())
                }
                Self::RoomJoinRules(event) => {
                    AnyRedactedStateEventContent::RoomJoinRules(event.content.clone())
                }
                Self::RoomMember(event) => {
                    AnyRedactedStateEventContent::RoomMember(event.content.clone())
                }
                Self::RoomName(event) => {
                    AnyRedactedStateEventContent::RoomName(event.content.clone())
                }
                Self::RoomPinnedEvents(event) => {
                    AnyRedactedStateEventContent::RoomPinnedEvents(event.content.clone())
                }
                Self::RoomPowerLevels(event) => {
                    AnyRedactedStateEventContent::RoomPowerLevels(event.content.clone())
                }
                Self::RoomServerAcl(event) => {
                    AnyRedactedStateEventContent::RoomServerAcl(event.content.clone())
                }
                Self::RoomThirdPartyInvite(event) => {
                    AnyRedactedStateEventContent::RoomThirdPartyInvite(event.content.clone())
                }
                Self::RoomTombstone(event) => {
                    AnyRedactedStateEventContent::RoomTombstone(event.content.clone())
                }
                Self::RoomTopic(event) => {
                    AnyRedactedStateEventContent::RoomTopic(event.content.clone())
                }
                Self::Custom(event) => AnyRedactedStateEventContent::Custom(event.content.clone()),
            }
        }
    }
    impl<'de> ::serde::de::Deserialize<'de> for AnyRedactedStateEventStub {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: ::serde::de::Deserializer<'de>,
        {
            use ::serde::de::Error as _;
            let json = Box::<::serde_json::value::RawValue>::deserialize(deserializer)?;
            let ::ruma_events::EventDeHelper { ev_type, .. } =
                ::ruma_events::from_raw_json_value(&json)?;
            match ev_type.as_str() {
                "m.room.aliases" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::aliases::AliasesEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomAliases(event))
                }
                "m.room.avatar" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::avatar::AvatarEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomAvatar(event))
                }
                "m.room.canonical_alias" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::canonical_alias::CanonicalAliasEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomCanonicalAlias(event))
                }
                "m.room.create" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::create::CreateEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomCreate(event))
                }
                "m.room.encryption" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::encryption::EncryptionEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomEncryption(event))
                }
                "m.room.guest_access" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::guest_access::GuestAccessEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomGuestAccess(event))
                }
                "m.room.history_visibility" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::history_visibility::HistoryVisibilityEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomHistoryVisibility(event))
                }
                "m.room.join_rules" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::join_rules::JoinRulesEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomJoinRules(event))
                }
                "m.room.member" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::member::MemberEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomMember(event))
                }
                "m.room.name" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<::ruma_events::room::name::NameEventContent>,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomName(event))
                }
                "m.room.pinned_events" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::pinned_events::PinnedEventsEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomPinnedEvents(event))
                }
                "m.room.power_levels" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::power_levels::PowerLevelsEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomPowerLevels(event))
                }
                "m.room.server_acl" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::server_acl::ServerAclEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomServerAcl(event))
                }
                "m.room.third_party_invite" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::third_party_invite::ThirdPartyInviteEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomThirdPartyInvite(event))
                }
                "m.room.tombstone" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::tombstone::TombstoneEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomTombstone(event))
                }
                "m.room.topic" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<
                            ::ruma_events::room::topic::TopicEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomTopic(event))
                }
                event => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StateEventStub<::ruma_events::custom::CustomEventContent>,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::Custom(event))
                }
            }
        }
    }
    /// Any state event.
    #[allow(clippy::large_enum_variant)]
    pub enum AnyRedactedStrippedStateEventStub {
        ///m.room.aliases
        RoomAliases(
            ::ruma_events::StrippedStateEventStub<
                ::ruma_events::room::aliases::AliasesEventContent,
            >,
        ),
        ///m.room.avatar
        RoomAvatar(
            ::ruma_events::StrippedStateEventStub<::ruma_events::room::avatar::AvatarEventContent>,
        ),
        ///m.room.canonical_alias
        RoomCanonicalAlias(
            ::ruma_events::StrippedStateEventStub<
                ::ruma_events::room::canonical_alias::CanonicalAliasEventContent,
            >,
        ),
        ///m.room.create
        RoomCreate(
            ::ruma_events::StrippedStateEventStub<::ruma_events::room::create::CreateEventContent>,
        ),
        ///m.room.encryption
        RoomEncryption(
            ::ruma_events::StrippedStateEventStub<
                ::ruma_events::room::encryption::EncryptionEventContent,
            >,
        ),
        ///m.room.guest_access
        RoomGuestAccess(
            ::ruma_events::StrippedStateEventStub<
                ::ruma_events::room::guest_access::GuestAccessEventContent,
            >,
        ),
        ///m.room.history_visibility
        RoomHistoryVisibility(
            ::ruma_events::StrippedStateEventStub<
                ::ruma_events::room::history_visibility::HistoryVisibilityEventContent,
            >,
        ),
        ///m.room.join_rules
        RoomJoinRules(
            ::ruma_events::StrippedStateEventStub<
                ::ruma_events::room::join_rules::JoinRulesEventContent,
            >,
        ),
        ///m.room.member
        RoomMember(
            ::ruma_events::StrippedStateEventStub<::ruma_events::room::member::MemberEventContent>,
        ),
        ///m.room.name
        RoomName(
            ::ruma_events::StrippedStateEventStub<::ruma_events::room::name::NameEventContent>,
        ),
        ///m.room.pinned_events
        RoomPinnedEvents(
            ::ruma_events::StrippedStateEventStub<
                ::ruma_events::room::pinned_events::PinnedEventsEventContent,
            >,
        ),
        ///m.room.power_levels
        RoomPowerLevels(
            ::ruma_events::StrippedStateEventStub<
                ::ruma_events::room::power_levels::PowerLevelsEventContent,
            >,
        ),
        ///m.room.server_acl
        RoomServerAcl(
            ::ruma_events::StrippedStateEventStub<
                ::ruma_events::room::server_acl::ServerAclEventContent,
            >,
        ),
        ///m.room.third_party_invite
        RoomThirdPartyInvite(
            ::ruma_events::StrippedStateEventStub<
                ::ruma_events::room::third_party_invite::ThirdPartyInviteEventContent,
            >,
        ),
        ///m.room.tombstone
        RoomTombstone(
            ::ruma_events::StrippedStateEventStub<
                ::ruma_events::room::tombstone::TombstoneEventContent,
            >,
        ),
        ///m.room.topic
        RoomTopic(
            ::ruma_events::StrippedStateEventStub<::ruma_events::room::topic::TopicEventContent>,
        ),
        /// An event not defined by the Matrix specification
        Custom(::ruma_events::StrippedStateEventStub<::ruma_events::custom::CustomEventContent>),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::clone::Clone for AnyRedactedStrippedStateEventStub {
        #[inline]
        fn clone(&self) -> AnyRedactedStrippedStateEventStub {
            match (&*self,) {
                (&AnyRedactedStrippedStateEventStub::RoomAliases(ref __self_0),) => {
                    AnyRedactedStrippedStateEventStub::RoomAliases(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStrippedStateEventStub::RoomAvatar(ref __self_0),) => {
                    AnyRedactedStrippedStateEventStub::RoomAvatar(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStrippedStateEventStub::RoomCanonicalAlias(ref __self_0),) => {
                    AnyRedactedStrippedStateEventStub::RoomCanonicalAlias(
                        ::core::clone::Clone::clone(&(*__self_0)),
                    )
                }
                (&AnyRedactedStrippedStateEventStub::RoomCreate(ref __self_0),) => {
                    AnyRedactedStrippedStateEventStub::RoomCreate(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStrippedStateEventStub::RoomEncryption(ref __self_0),) => {
                    AnyRedactedStrippedStateEventStub::RoomEncryption(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStrippedStateEventStub::RoomGuestAccess(ref __self_0),) => {
                    AnyRedactedStrippedStateEventStub::RoomGuestAccess(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStrippedStateEventStub::RoomHistoryVisibility(ref __self_0),) => {
                    AnyRedactedStrippedStateEventStub::RoomHistoryVisibility(
                        ::core::clone::Clone::clone(&(*__self_0)),
                    )
                }
                (&AnyRedactedStrippedStateEventStub::RoomJoinRules(ref __self_0),) => {
                    AnyRedactedStrippedStateEventStub::RoomJoinRules(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStrippedStateEventStub::RoomMember(ref __self_0),) => {
                    AnyRedactedStrippedStateEventStub::RoomMember(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStrippedStateEventStub::RoomName(ref __self_0),) => {
                    AnyRedactedStrippedStateEventStub::RoomName(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStrippedStateEventStub::RoomPinnedEvents(ref __self_0),) => {
                    AnyRedactedStrippedStateEventStub::RoomPinnedEvents(
                        ::core::clone::Clone::clone(&(*__self_0)),
                    )
                }
                (&AnyRedactedStrippedStateEventStub::RoomPowerLevels(ref __self_0),) => {
                    AnyRedactedStrippedStateEventStub::RoomPowerLevels(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStrippedStateEventStub::RoomServerAcl(ref __self_0),) => {
                    AnyRedactedStrippedStateEventStub::RoomServerAcl(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStrippedStateEventStub::RoomThirdPartyInvite(ref __self_0),) => {
                    AnyRedactedStrippedStateEventStub::RoomThirdPartyInvite(
                        ::core::clone::Clone::clone(&(*__self_0)),
                    )
                }
                (&AnyRedactedStrippedStateEventStub::RoomTombstone(ref __self_0),) => {
                    AnyRedactedStrippedStateEventStub::RoomTombstone(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStrippedStateEventStub::RoomTopic(ref __self_0),) => {
                    AnyRedactedStrippedStateEventStub::RoomTopic(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyRedactedStrippedStateEventStub::Custom(ref __self_0),) => {
                    AnyRedactedStrippedStateEventStub::Custom(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::fmt::Debug for AnyRedactedStrippedStateEventStub {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&AnyRedactedStrippedStateEventStub::RoomAliases(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomAliases");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStrippedStateEventStub::RoomAvatar(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomAvatar");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStrippedStateEventStub::RoomCanonicalAlias(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomCanonicalAlias");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStrippedStateEventStub::RoomCreate(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomCreate");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStrippedStateEventStub::RoomEncryption(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomEncryption");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStrippedStateEventStub::RoomGuestAccess(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomGuestAccess");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStrippedStateEventStub::RoomHistoryVisibility(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomHistoryVisibility");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStrippedStateEventStub::RoomJoinRules(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomJoinRules");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStrippedStateEventStub::RoomMember(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomMember");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStrippedStateEventStub::RoomName(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomName");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStrippedStateEventStub::RoomPinnedEvents(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomPinnedEvents");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStrippedStateEventStub::RoomPowerLevels(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomPowerLevels");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStrippedStateEventStub::RoomServerAcl(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomServerAcl");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStrippedStateEventStub::RoomThirdPartyInvite(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomThirdPartyInvite");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStrippedStateEventStub::RoomTombstone(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomTombstone");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStrippedStateEventStub::RoomTopic(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomTopic");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRedactedStrippedStateEventStub::Custom(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Custom");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(rust_2018_idioms, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for AnyRedactedStrippedStateEventStub {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::export::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    AnyRedactedStrippedStateEventStub::RoomAliases(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStrippedStateEventStub::RoomAvatar(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStrippedStateEventStub::RoomCanonicalAlias(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStrippedStateEventStub::RoomCreate(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStrippedStateEventStub::RoomEncryption(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStrippedStateEventStub::RoomGuestAccess(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStrippedStateEventStub::RoomHistoryVisibility(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStrippedStateEventStub::RoomJoinRules(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStrippedStateEventStub::RoomMember(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStrippedStateEventStub::RoomName(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStrippedStateEventStub::RoomPinnedEvents(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStrippedStateEventStub::RoomPowerLevels(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStrippedStateEventStub::RoomServerAcl(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStrippedStateEventStub::RoomThirdPartyInvite(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStrippedStateEventStub::RoomTombstone(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStrippedStateEventStub::RoomTopic(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRedactedStrippedStateEventStub::Custom(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                }
            }
        }
    };
    impl AnyRedactedStrippedStateEventStub {
        /// Returns the any content enum for this event.
        pub fn content(&self) -> AnyRedactedStateEventContent {
            match self {
                Self::RoomAliases(event) => {
                    AnyRedactedStateEventContent::RoomAliases(event.content.clone())
                }
                Self::RoomAvatar(event) => {
                    AnyRedactedStateEventContent::RoomAvatar(event.content.clone())
                }
                Self::RoomCanonicalAlias(event) => {
                    AnyRedactedStateEventContent::RoomCanonicalAlias(event.content.clone())
                }
                Self::RoomCreate(event) => {
                    AnyRedactedStateEventContent::RoomCreate(event.content.clone())
                }
                Self::RoomEncryption(event) => {
                    AnyRedactedStateEventContent::RoomEncryption(event.content.clone())
                }
                Self::RoomGuestAccess(event) => {
                    AnyRedactedStateEventContent::RoomGuestAccess(event.content.clone())
                }
                Self::RoomHistoryVisibility(event) => {
                    AnyRedactedStateEventContent::RoomHistoryVisibility(event.content.clone())
                }
                Self::RoomJoinRules(event) => {
                    AnyRedactedStateEventContent::RoomJoinRules(event.content.clone())
                }
                Self::RoomMember(event) => {
                    AnyRedactedStateEventContent::RoomMember(event.content.clone())
                }
                Self::RoomName(event) => {
                    AnyRedactedStateEventContent::RoomName(event.content.clone())
                }
                Self::RoomPinnedEvents(event) => {
                    AnyRedactedStateEventContent::RoomPinnedEvents(event.content.clone())
                }
                Self::RoomPowerLevels(event) => {
                    AnyRedactedStateEventContent::RoomPowerLevels(event.content.clone())
                }
                Self::RoomServerAcl(event) => {
                    AnyRedactedStateEventContent::RoomServerAcl(event.content.clone())
                }
                Self::RoomThirdPartyInvite(event) => {
                    AnyRedactedStateEventContent::RoomThirdPartyInvite(event.content.clone())
                }
                Self::RoomTombstone(event) => {
                    AnyRedactedStateEventContent::RoomTombstone(event.content.clone())
                }
                Self::RoomTopic(event) => {
                    AnyRedactedStateEventContent::RoomTopic(event.content.clone())
                }
                Self::Custom(event) => AnyRedactedStateEventContent::Custom(event.content.clone()),
            }
        }
    }
    impl<'de> ::serde::de::Deserialize<'de> for AnyRedactedStrippedStateEventStub {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: ::serde::de::Deserializer<'de>,
        {
            use ::serde::de::Error as _;
            let json = Box::<::serde_json::value::RawValue>::deserialize(deserializer)?;
            let ::ruma_events::EventDeHelper { ev_type, .. } =
                ::ruma_events::from_raw_json_value(&json)?;
            match ev_type.as_str() {
                "m.room.aliases" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::aliases::AliasesEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomAliases(event))
                }
                "m.room.avatar" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::avatar::AvatarEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomAvatar(event))
                }
                "m.room.canonical_alias" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::canonical_alias::CanonicalAliasEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomCanonicalAlias(event))
                }
                "m.room.create" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::create::CreateEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomCreate(event))
                }
                "m.room.encryption" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::encryption::EncryptionEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomEncryption(event))
                }
                "m.room.guest_access" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::guest_access::GuestAccessEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomGuestAccess(event))
                }
                "m.room.history_visibility" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::history_visibility::HistoryVisibilityEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomHistoryVisibility(event))
                }
                "m.room.join_rules" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::join_rules::JoinRulesEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomJoinRules(event))
                }
                "m.room.member" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::member::MemberEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomMember(event))
                }
                "m.room.name" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::name::NameEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomName(event))
                }
                "m.room.pinned_events" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::pinned_events::PinnedEventsEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomPinnedEvents(event))
                }
                "m.room.power_levels" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::power_levels::PowerLevelsEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomPowerLevels(event))
                }
                "m.room.server_acl" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::server_acl::ServerAclEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomServerAcl(event))
                }
                "m.room.third_party_invite" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::third_party_invite::ThirdPartyInviteEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomThirdPartyInvite(event))
                }
                "m.room.tombstone" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::tombstone::TombstoneEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomTombstone(event))
                }
                "m.room.topic" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::room::topic::TopicEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomTopic(event))
                }
                event => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::StrippedStateEventStub<
                            ::ruma_events::custom::CustomEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::Custom(event))
                }
            }
        }
    }
    /// Any state event.
    #[allow(clippy::large_enum_variant)]
    pub enum AnyStateEventContent {
        ///m.room.aliases
        RoomAliases(::ruma_events::room::aliases::AliasesEventContent),
        ///m.room.avatar
        RoomAvatar(::ruma_events::room::avatar::AvatarEventContent),
        ///m.room.canonical_alias
        RoomCanonicalAlias(::ruma_events::room::canonical_alias::CanonicalAliasEventContent),
        ///m.room.create
        RoomCreate(::ruma_events::room::create::CreateEventContent),
        ///m.room.encryption
        RoomEncryption(::ruma_events::room::encryption::EncryptionEventContent),
        ///m.room.guest_access
        RoomGuestAccess(::ruma_events::room::guest_access::GuestAccessEventContent),
        ///m.room.history_visibility
        RoomHistoryVisibility(
            ::ruma_events::room::history_visibility::HistoryVisibilityEventContent,
        ),
        ///m.room.join_rules
        RoomJoinRules(::ruma_events::room::join_rules::JoinRulesEventContent),
        ///m.room.member
        RoomMember(::ruma_events::room::member::MemberEventContent),
        ///m.room.name
        RoomName(::ruma_events::room::name::NameEventContent),
        ///m.room.pinned_events
        RoomPinnedEvents(::ruma_events::room::pinned_events::PinnedEventsEventContent),
        ///m.room.power_levels
        RoomPowerLevels(::ruma_events::room::power_levels::PowerLevelsEventContent),
        ///m.room.server_acl
        RoomServerAcl(::ruma_events::room::server_acl::ServerAclEventContent),
        ///m.room.third_party_invite
        RoomThirdPartyInvite(::ruma_events::room::third_party_invite::ThirdPartyInviteEventContent),
        ///m.room.tombstone
        RoomTombstone(::ruma_events::room::tombstone::TombstoneEventContent),
        ///m.room.topic
        RoomTopic(::ruma_events::room::topic::TopicEventContent),
        /// Content of an event not defined by the Matrix specification.
        Custom(::ruma_events::custom::CustomEventContent),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::clone::Clone for AnyStateEventContent {
        #[inline]
        fn clone(&self) -> AnyStateEventContent {
            match (&*self,) {
                (&AnyStateEventContent::RoomAliases(ref __self_0),) => {
                    AnyStateEventContent::RoomAliases(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEventContent::RoomAvatar(ref __self_0),) => {
                    AnyStateEventContent::RoomAvatar(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEventContent::RoomCanonicalAlias(ref __self_0),) => {
                    AnyStateEventContent::RoomCanonicalAlias(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyStateEventContent::RoomCreate(ref __self_0),) => {
                    AnyStateEventContent::RoomCreate(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEventContent::RoomEncryption(ref __self_0),) => {
                    AnyStateEventContent::RoomEncryption(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEventContent::RoomGuestAccess(ref __self_0),) => {
                    AnyStateEventContent::RoomGuestAccess(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEventContent::RoomHistoryVisibility(ref __self_0),) => {
                    AnyStateEventContent::RoomHistoryVisibility(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyStateEventContent::RoomJoinRules(ref __self_0),) => {
                    AnyStateEventContent::RoomJoinRules(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEventContent::RoomMember(ref __self_0),) => {
                    AnyStateEventContent::RoomMember(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEventContent::RoomName(ref __self_0),) => {
                    AnyStateEventContent::RoomName(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEventContent::RoomPinnedEvents(ref __self_0),) => {
                    AnyStateEventContent::RoomPinnedEvents(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyStateEventContent::RoomPowerLevels(ref __self_0),) => {
                    AnyStateEventContent::RoomPowerLevels(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEventContent::RoomServerAcl(ref __self_0),) => {
                    AnyStateEventContent::RoomServerAcl(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEventContent::RoomThirdPartyInvite(ref __self_0),) => {
                    AnyStateEventContent::RoomThirdPartyInvite(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyStateEventContent::RoomTombstone(ref __self_0),) => {
                    AnyStateEventContent::RoomTombstone(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEventContent::RoomTopic(ref __self_0),) => {
                    AnyStateEventContent::RoomTopic(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyStateEventContent::Custom(ref __self_0),) => {
                    AnyStateEventContent::Custom(::core::clone::Clone::clone(&(*__self_0)))
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::fmt::Debug for AnyStateEventContent {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&AnyStateEventContent::RoomAliases(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomAliases");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventContent::RoomAvatar(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomAvatar");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventContent::RoomCanonicalAlias(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomCanonicalAlias");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventContent::RoomCreate(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomCreate");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventContent::RoomEncryption(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomEncryption");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventContent::RoomGuestAccess(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomGuestAccess");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventContent::RoomHistoryVisibility(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomHistoryVisibility");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventContent::RoomJoinRules(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomJoinRules");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventContent::RoomMember(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomMember");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventContent::RoomName(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomName");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventContent::RoomPinnedEvents(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomPinnedEvents");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventContent::RoomPowerLevels(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomPowerLevels");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventContent::RoomServerAcl(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomServerAcl");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventContent::RoomThirdPartyInvite(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomThirdPartyInvite");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventContent::RoomTombstone(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomTombstone");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventContent::RoomTopic(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomTopic");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyStateEventContent::Custom(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Custom");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(rust_2018_idioms, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for AnyStateEventContent {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::export::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    AnyStateEventContent::RoomAliases(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventContent::RoomAvatar(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventContent::RoomCanonicalAlias(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventContent::RoomCreate(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventContent::RoomEncryption(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventContent::RoomGuestAccess(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventContent::RoomHistoryVisibility(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventContent::RoomJoinRules(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventContent::RoomMember(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventContent::RoomName(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventContent::RoomPinnedEvents(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventContent::RoomPowerLevels(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventContent::RoomServerAcl(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventContent::RoomThirdPartyInvite(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventContent::RoomTombstone(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventContent::RoomTopic(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyStateEventContent::Custom(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                }
            }
        }
    };
    impl ::ruma_events::EventContent for AnyStateEventContent {
        fn event_type(&self) -> &str {
            match self {
                Self::RoomAliases(content) => content.event_type(),
                Self::RoomAvatar(content) => content.event_type(),
                Self::RoomCanonicalAlias(content) => content.event_type(),
                Self::RoomCreate(content) => content.event_type(),
                Self::RoomEncryption(content) => content.event_type(),
                Self::RoomGuestAccess(content) => content.event_type(),
                Self::RoomHistoryVisibility(content) => content.event_type(),
                Self::RoomJoinRules(content) => content.event_type(),
                Self::RoomMember(content) => content.event_type(),
                Self::RoomName(content) => content.event_type(),
                Self::RoomPinnedEvents(content) => content.event_type(),
                Self::RoomPowerLevels(content) => content.event_type(),
                Self::RoomServerAcl(content) => content.event_type(),
                Self::RoomThirdPartyInvite(content) => content.event_type(),
                Self::RoomTombstone(content) => content.event_type(),
                Self::RoomTopic(content) => content.event_type(),
                Self::Custom(content) => content.event_type(),
            }
        }
        fn from_parts(
            event_type: &str,
            input: Box<::serde_json::value::RawValue>,
        ) -> Result<Self, ::serde_json::Error> {
            match event_type {
                "m.room.aliases" => {
                    let content = ::ruma_events::room::aliases::AliasesEventContent::from_parts(
                        event_type, input,
                    )?;
                    Ok(Self::RoomAliases(content))
                }
                "m.room.avatar" => {
                    let content = ::ruma_events::room::avatar::AvatarEventContent::from_parts(
                        event_type, input,
                    )?;
                    Ok(Self::RoomAvatar(content))
                }
                "m.room.canonical_alias" => {
                    let content = :: ruma_events :: room :: canonical_alias :: CanonicalAliasEventContent :: from_parts ( event_type , input ) ? ;
                    Ok(Self::RoomCanonicalAlias(content))
                }
                "m.room.create" => {
                    let content = ::ruma_events::room::create::CreateEventContent::from_parts(
                        event_type, input,
                    )?;
                    Ok(Self::RoomCreate(content))
                }
                "m.room.encryption" => {
                    let content =
                        ::ruma_events::room::encryption::EncryptionEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::RoomEncryption(content))
                }
                "m.room.guest_access" => {
                    let content =
                        ::ruma_events::room::guest_access::GuestAccessEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::RoomGuestAccess(content))
                }
                "m.room.history_visibility" => {
                    let content = :: ruma_events :: room :: history_visibility :: HistoryVisibilityEventContent :: from_parts ( event_type , input ) ? ;
                    Ok(Self::RoomHistoryVisibility(content))
                }
                "m.room.join_rules" => {
                    let content =
                        ::ruma_events::room::join_rules::JoinRulesEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::RoomJoinRules(content))
                }
                "m.room.member" => {
                    let content = ::ruma_events::room::member::MemberEventContent::from_parts(
                        event_type, input,
                    )?;
                    Ok(Self::RoomMember(content))
                }
                "m.room.name" => {
                    let content =
                        ::ruma_events::room::name::NameEventContent::from_parts(event_type, input)?;
                    Ok(Self::RoomName(content))
                }
                "m.room.pinned_events" => {
                    let content =
                        ::ruma_events::room::pinned_events::PinnedEventsEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::RoomPinnedEvents(content))
                }
                "m.room.power_levels" => {
                    let content =
                        ::ruma_events::room::power_levels::PowerLevelsEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::RoomPowerLevels(content))
                }
                "m.room.server_acl" => {
                    let content =
                        ::ruma_events::room::server_acl::ServerAclEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::RoomServerAcl(content))
                }
                "m.room.third_party_invite" => {
                    let content = :: ruma_events :: room :: third_party_invite :: ThirdPartyInviteEventContent :: from_parts ( event_type , input ) ? ;
                    Ok(Self::RoomThirdPartyInvite(content))
                }
                "m.room.tombstone" => {
                    let content =
                        ::ruma_events::room::tombstone::TombstoneEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::RoomTombstone(content))
                }
                "m.room.topic" => {
                    let content = ::ruma_events::room::topic::TopicEventContent::from_parts(
                        event_type, input,
                    )?;
                    Ok(Self::RoomTopic(content))
                }
                ev_type => {
                    let content =
                        ::ruma_events::custom::CustomEventContent::from_parts(ev_type, input)?;
                    Ok(Self::Custom(content))
                }
            }
        }
    }
    impl ::ruma_events::RoomEventContent for AnyStateEventContent {}
    impl ::ruma_events::StateEventContent for AnyStateEventContent {}
    /// Any to-device event.
    #[allow(clippy::large_enum_variant)]
    pub enum AnyToDeviceEvent {
        ///m.dummy
        Dummy(::ruma_events::ToDeviceEvent<::ruma_events::dummy::DummyEventContent>),
        ///m.room_key
        RoomKey(::ruma_events::ToDeviceEvent<::ruma_events::room_key::RoomKeyEventContent>),
        ///m.room_key_request
        RoomKeyRequest(
            ::ruma_events::ToDeviceEvent<
                ::ruma_events::room_key_request::RoomKeyRequestEventContent,
            >,
        ),
        ///m.forwarded_room_key
        ForwardedRoomKey(
            ::ruma_events::ToDeviceEvent<
                ::ruma_events::forwarded_room_key::ForwardedRoomKeyEventContent,
            >,
        ),
        ///m.key.verification.request
        KeyVerificationRequest(
            ::ruma_events::ToDeviceEvent<
                ::ruma_events::key::verification::request::RequestEventContent,
            >,
        ),
        ///m.key.verification.start
        KeyVerificationStart(
            ::ruma_events::ToDeviceEvent<
                ::ruma_events::key::verification::start::StartEventContent,
            >,
        ),
        ///m.key.verification.cancel
        KeyVerificationCancel(
            ::ruma_events::ToDeviceEvent<
                ::ruma_events::key::verification::cancel::CancelEventContent,
            >,
        ),
        ///m.key.verification.accept
        KeyVerificationAccept(
            ::ruma_events::ToDeviceEvent<
                ::ruma_events::key::verification::accept::AcceptEventContent,
            >,
        ),
        ///m.key.verification.key
        KeyVerificationKey(
            ::ruma_events::ToDeviceEvent<::ruma_events::key::verification::key::KeyEventContent>,
        ),
        ///m.key.verification.mac
        KeyVerificationMac(
            ::ruma_events::ToDeviceEvent<::ruma_events::key::verification::mac::MacEventContent>,
        ),
        ///m.room.encrypted
        RoomEncrypted(
            ::ruma_events::ToDeviceEvent<::ruma_events::room::encrypted::EncryptedEventContent>,
        ),
        /// An event not defined by the Matrix specification
        Custom(::ruma_events::ToDeviceEvent<::ruma_events::custom::CustomEventContent>),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::clone::Clone for AnyToDeviceEvent {
        #[inline]
        fn clone(&self) -> AnyToDeviceEvent {
            match (&*self,) {
                (&AnyToDeviceEvent::Dummy(ref __self_0),) => {
                    AnyToDeviceEvent::Dummy(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyToDeviceEvent::RoomKey(ref __self_0),) => {
                    AnyToDeviceEvent::RoomKey(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyToDeviceEvent::RoomKeyRequest(ref __self_0),) => {
                    AnyToDeviceEvent::RoomKeyRequest(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyToDeviceEvent::ForwardedRoomKey(ref __self_0),) => {
                    AnyToDeviceEvent::ForwardedRoomKey(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyToDeviceEvent::KeyVerificationRequest(ref __self_0),) => {
                    AnyToDeviceEvent::KeyVerificationRequest(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyToDeviceEvent::KeyVerificationStart(ref __self_0),) => {
                    AnyToDeviceEvent::KeyVerificationStart(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyToDeviceEvent::KeyVerificationCancel(ref __self_0),) => {
                    AnyToDeviceEvent::KeyVerificationCancel(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyToDeviceEvent::KeyVerificationAccept(ref __self_0),) => {
                    AnyToDeviceEvent::KeyVerificationAccept(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyToDeviceEvent::KeyVerificationKey(ref __self_0),) => {
                    AnyToDeviceEvent::KeyVerificationKey(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyToDeviceEvent::KeyVerificationMac(ref __self_0),) => {
                    AnyToDeviceEvent::KeyVerificationMac(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyToDeviceEvent::RoomEncrypted(ref __self_0),) => {
                    AnyToDeviceEvent::RoomEncrypted(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyToDeviceEvent::Custom(ref __self_0),) => {
                    AnyToDeviceEvent::Custom(::core::clone::Clone::clone(&(*__self_0)))
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::fmt::Debug for AnyToDeviceEvent {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&AnyToDeviceEvent::Dummy(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Dummy");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyToDeviceEvent::RoomKey(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomKey");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyToDeviceEvent::RoomKeyRequest(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomKeyRequest");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyToDeviceEvent::ForwardedRoomKey(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("ForwardedRoomKey");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyToDeviceEvent::KeyVerificationRequest(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("KeyVerificationRequest");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyToDeviceEvent::KeyVerificationStart(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("KeyVerificationStart");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyToDeviceEvent::KeyVerificationCancel(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("KeyVerificationCancel");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyToDeviceEvent::KeyVerificationAccept(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("KeyVerificationAccept");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyToDeviceEvent::KeyVerificationKey(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("KeyVerificationKey");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyToDeviceEvent::KeyVerificationMac(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("KeyVerificationMac");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyToDeviceEvent::RoomEncrypted(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomEncrypted");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyToDeviceEvent::Custom(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Custom");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(rust_2018_idioms, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for AnyToDeviceEvent {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::export::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    AnyToDeviceEvent::Dummy(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyToDeviceEvent::RoomKey(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyToDeviceEvent::RoomKeyRequest(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyToDeviceEvent::ForwardedRoomKey(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyToDeviceEvent::KeyVerificationRequest(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyToDeviceEvent::KeyVerificationStart(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyToDeviceEvent::KeyVerificationCancel(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyToDeviceEvent::KeyVerificationAccept(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyToDeviceEvent::KeyVerificationKey(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyToDeviceEvent::KeyVerificationMac(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyToDeviceEvent::RoomEncrypted(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyToDeviceEvent::Custom(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                }
            }
        }
    };
    impl AnyToDeviceEvent {
        /// Returns the any content enum for this event.
        pub fn content(&self) -> AnyToDeviceEventContent {
            match self {
                Self::Dummy(event) => AnyToDeviceEventContent::Dummy(event.content.clone()),
                Self::RoomKey(event) => AnyToDeviceEventContent::RoomKey(event.content.clone()),
                Self::RoomKeyRequest(event) => {
                    AnyToDeviceEventContent::RoomKeyRequest(event.content.clone())
                }
                Self::ForwardedRoomKey(event) => {
                    AnyToDeviceEventContent::ForwardedRoomKey(event.content.clone())
                }
                Self::KeyVerificationRequest(event) => {
                    AnyToDeviceEventContent::KeyVerificationRequest(event.content.clone())
                }
                Self::KeyVerificationStart(event) => {
                    AnyToDeviceEventContent::KeyVerificationStart(event.content.clone())
                }
                Self::KeyVerificationCancel(event) => {
                    AnyToDeviceEventContent::KeyVerificationCancel(event.content.clone())
                }
                Self::KeyVerificationAccept(event) => {
                    AnyToDeviceEventContent::KeyVerificationAccept(event.content.clone())
                }
                Self::KeyVerificationKey(event) => {
                    AnyToDeviceEventContent::KeyVerificationKey(event.content.clone())
                }
                Self::KeyVerificationMac(event) => {
                    AnyToDeviceEventContent::KeyVerificationMac(event.content.clone())
                }
                Self::RoomEncrypted(event) => {
                    AnyToDeviceEventContent::RoomEncrypted(event.content.clone())
                }
                Self::Custom(event) => AnyToDeviceEventContent::Custom(event.content.clone()),
            }
        }
        ///Returns this events sender field.
        pub fn sender(&self) -> &::ruma_identifiers::UserId {
            match self {
                Self::Dummy(event) => &event.sender,
                Self::RoomKey(event) => &event.sender,
                Self::RoomKeyRequest(event) => &event.sender,
                Self::ForwardedRoomKey(event) => &event.sender,
                Self::KeyVerificationRequest(event) => &event.sender,
                Self::KeyVerificationStart(event) => &event.sender,
                Self::KeyVerificationCancel(event) => &event.sender,
                Self::KeyVerificationAccept(event) => &event.sender,
                Self::KeyVerificationKey(event) => &event.sender,
                Self::KeyVerificationMac(event) => &event.sender,
                Self::RoomEncrypted(event) => &event.sender,
                Self::Custom(event) => &event.sender,
            }
        }
    }
    impl<'de> ::serde::de::Deserialize<'de> for AnyToDeviceEvent {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: ::serde::de::Deserializer<'de>,
        {
            use ::serde::de::Error as _;
            let json = Box::<::serde_json::value::RawValue>::deserialize(deserializer)?;
            let ::ruma_events::EventDeHelper { ev_type, .. } =
                ::ruma_events::from_raw_json_value(&json)?;
            match ev_type.as_str() {
                "m.dummy" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::ToDeviceEvent<::ruma_events::dummy::DummyEventContent>,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::Dummy(event))
                }
                "m.room_key" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::ToDeviceEvent<::ruma_events::room_key::RoomKeyEventContent>,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomKey(event))
                }
                "m.room_key_request" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::ToDeviceEvent<
                            ::ruma_events::room_key_request::RoomKeyRequestEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomKeyRequest(event))
                }
                "m.forwarded_room_key" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::ToDeviceEvent<
                            ::ruma_events::forwarded_room_key::ForwardedRoomKeyEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::ForwardedRoomKey(event))
                }
                "m.key.verification.request" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::ToDeviceEvent<
                            ::ruma_events::key::verification::request::RequestEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::KeyVerificationRequest(event))
                }
                "m.key.verification.start" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::ToDeviceEvent<
                            ::ruma_events::key::verification::start::StartEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::KeyVerificationStart(event))
                }
                "m.key.verification.cancel" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::ToDeviceEvent<
                            ::ruma_events::key::verification::cancel::CancelEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::KeyVerificationCancel(event))
                }
                "m.key.verification.accept" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::ToDeviceEvent<
                            ::ruma_events::key::verification::accept::AcceptEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::KeyVerificationAccept(event))
                }
                "m.key.verification.key" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::ToDeviceEvent<
                            ::ruma_events::key::verification::key::KeyEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::KeyVerificationKey(event))
                }
                "m.key.verification.mac" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::ToDeviceEvent<
                            ::ruma_events::key::verification::mac::MacEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::KeyVerificationMac(event))
                }
                "m.room.encrypted" => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::ToDeviceEvent<
                            ::ruma_events::room::encrypted::EncryptedEventContent,
                        >,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::RoomEncrypted(event))
                }
                event => {
                    let event = ::serde_json::from_str::<
                        ::ruma_events::ToDeviceEvent<::ruma_events::custom::CustomEventContent>,
                    >(json.get())
                    .map_err(D::Error::custom)?;
                    Ok(Self::Custom(event))
                }
            }
        }
    }
    /// Any to-device event.
    #[allow(clippy::large_enum_variant)]
    pub enum AnyToDeviceEventContent {
        ///m.dummy
        Dummy(::ruma_events::dummy::DummyEventContent),
        ///m.room_key
        RoomKey(::ruma_events::room_key::RoomKeyEventContent),
        ///m.room_key_request
        RoomKeyRequest(::ruma_events::room_key_request::RoomKeyRequestEventContent),
        ///m.forwarded_room_key
        ForwardedRoomKey(::ruma_events::forwarded_room_key::ForwardedRoomKeyEventContent),
        ///m.key.verification.request
        KeyVerificationRequest(::ruma_events::key::verification::request::RequestEventContent),
        ///m.key.verification.start
        KeyVerificationStart(::ruma_events::key::verification::start::StartEventContent),
        ///m.key.verification.cancel
        KeyVerificationCancel(::ruma_events::key::verification::cancel::CancelEventContent),
        ///m.key.verification.accept
        KeyVerificationAccept(::ruma_events::key::verification::accept::AcceptEventContent),
        ///m.key.verification.key
        KeyVerificationKey(::ruma_events::key::verification::key::KeyEventContent),
        ///m.key.verification.mac
        KeyVerificationMac(::ruma_events::key::verification::mac::MacEventContent),
        ///m.room.encrypted
        RoomEncrypted(::ruma_events::room::encrypted::EncryptedEventContent),
        /// Content of an event not defined by the Matrix specification.
        Custom(::ruma_events::custom::CustomEventContent),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::clone::Clone for AnyToDeviceEventContent {
        #[inline]
        fn clone(&self) -> AnyToDeviceEventContent {
            match (&*self,) {
                (&AnyToDeviceEventContent::Dummy(ref __self_0),) => {
                    AnyToDeviceEventContent::Dummy(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyToDeviceEventContent::RoomKey(ref __self_0),) => {
                    AnyToDeviceEventContent::RoomKey(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyToDeviceEventContent::RoomKeyRequest(ref __self_0),) => {
                    AnyToDeviceEventContent::RoomKeyRequest(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyToDeviceEventContent::ForwardedRoomKey(ref __self_0),) => {
                    AnyToDeviceEventContent::ForwardedRoomKey(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyToDeviceEventContent::KeyVerificationRequest(ref __self_0),) => {
                    AnyToDeviceEventContent::KeyVerificationRequest(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyToDeviceEventContent::KeyVerificationStart(ref __self_0),) => {
                    AnyToDeviceEventContent::KeyVerificationStart(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyToDeviceEventContent::KeyVerificationCancel(ref __self_0),) => {
                    AnyToDeviceEventContent::KeyVerificationCancel(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyToDeviceEventContent::KeyVerificationAccept(ref __self_0),) => {
                    AnyToDeviceEventContent::KeyVerificationAccept(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyToDeviceEventContent::KeyVerificationKey(ref __self_0),) => {
                    AnyToDeviceEventContent::KeyVerificationKey(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyToDeviceEventContent::KeyVerificationMac(ref __self_0),) => {
                    AnyToDeviceEventContent::KeyVerificationMac(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyToDeviceEventContent::RoomEncrypted(ref __self_0),) => {
                    AnyToDeviceEventContent::RoomEncrypted(::core::clone::Clone::clone(
                        &(*__self_0),
                    ))
                }
                (&AnyToDeviceEventContent::Custom(ref __self_0),) => {
                    AnyToDeviceEventContent::Custom(::core::clone::Clone::clone(&(*__self_0)))
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    #[allow(clippy::large_enum_variant)]
    impl ::core::fmt::Debug for AnyToDeviceEventContent {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&AnyToDeviceEventContent::Dummy(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Dummy");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyToDeviceEventContent::RoomKey(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomKey");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyToDeviceEventContent::RoomKeyRequest(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomKeyRequest");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyToDeviceEventContent::ForwardedRoomKey(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("ForwardedRoomKey");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyToDeviceEventContent::KeyVerificationRequest(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("KeyVerificationRequest");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyToDeviceEventContent::KeyVerificationStart(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("KeyVerificationStart");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyToDeviceEventContent::KeyVerificationCancel(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("KeyVerificationCancel");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyToDeviceEventContent::KeyVerificationAccept(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("KeyVerificationAccept");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyToDeviceEventContent::KeyVerificationKey(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("KeyVerificationKey");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyToDeviceEventContent::KeyVerificationMac(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("KeyVerificationMac");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyToDeviceEventContent::RoomEncrypted(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RoomEncrypted");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyToDeviceEventContent::Custom(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Custom");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(rust_2018_idioms, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for AnyToDeviceEventContent {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::export::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    AnyToDeviceEventContent::Dummy(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyToDeviceEventContent::RoomKey(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyToDeviceEventContent::RoomKeyRequest(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyToDeviceEventContent::ForwardedRoomKey(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyToDeviceEventContent::KeyVerificationRequest(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyToDeviceEventContent::KeyVerificationStart(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyToDeviceEventContent::KeyVerificationCancel(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyToDeviceEventContent::KeyVerificationAccept(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyToDeviceEventContent::KeyVerificationKey(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyToDeviceEventContent::KeyVerificationMac(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyToDeviceEventContent::RoomEncrypted(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyToDeviceEventContent::Custom(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                }
            }
        }
    };
    impl ::ruma_events::EventContent for AnyToDeviceEventContent {
        fn event_type(&self) -> &str {
            match self {
                Self::Dummy(content) => content.event_type(),
                Self::RoomKey(content) => content.event_type(),
                Self::RoomKeyRequest(content) => content.event_type(),
                Self::ForwardedRoomKey(content) => content.event_type(),
                Self::KeyVerificationRequest(content) => content.event_type(),
                Self::KeyVerificationStart(content) => content.event_type(),
                Self::KeyVerificationCancel(content) => content.event_type(),
                Self::KeyVerificationAccept(content) => content.event_type(),
                Self::KeyVerificationKey(content) => content.event_type(),
                Self::KeyVerificationMac(content) => content.event_type(),
                Self::RoomEncrypted(content) => content.event_type(),
                Self::Custom(content) => content.event_type(),
            }
        }
        fn from_parts(
            event_type: &str,
            input: Box<::serde_json::value::RawValue>,
        ) -> Result<Self, ::serde_json::Error> {
            match event_type {
                "m.dummy" => {
                    let content =
                        ::ruma_events::dummy::DummyEventContent::from_parts(event_type, input)?;
                    Ok(Self::Dummy(content))
                }
                "m.room_key" => {
                    let content = ::ruma_events::room_key::RoomKeyEventContent::from_parts(
                        event_type, input,
                    )?;
                    Ok(Self::RoomKey(content))
                }
                "m.room_key_request" => {
                    let content =
                        ::ruma_events::room_key_request::RoomKeyRequestEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::RoomKeyRequest(content))
                }
                "m.forwarded_room_key" => {
                    let content = :: ruma_events :: forwarded_room_key :: ForwardedRoomKeyEventContent :: from_parts ( event_type , input ) ? ;
                    Ok(Self::ForwardedRoomKey(content))
                }
                "m.key.verification.request" => {
                    let content =
                        ::ruma_events::key::verification::request::RequestEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::KeyVerificationRequest(content))
                }
                "m.key.verification.start" => {
                    let content =
                        ::ruma_events::key::verification::start::StartEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::KeyVerificationStart(content))
                }
                "m.key.verification.cancel" => {
                    let content =
                        ::ruma_events::key::verification::cancel::CancelEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::KeyVerificationCancel(content))
                }
                "m.key.verification.accept" => {
                    let content =
                        ::ruma_events::key::verification::accept::AcceptEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::KeyVerificationAccept(content))
                }
                "m.key.verification.key" => {
                    let content =
                        ::ruma_events::key::verification::key::KeyEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::KeyVerificationKey(content))
                }
                "m.key.verification.mac" => {
                    let content =
                        ::ruma_events::key::verification::mac::MacEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::KeyVerificationMac(content))
                }
                "m.room.encrypted" => {
                    let content =
                        ::ruma_events::room::encrypted::EncryptedEventContent::from_parts(
                            event_type, input,
                        )?;
                    Ok(Self::RoomEncrypted(content))
                }
                ev_type => {
                    let content =
                        ::ruma_events::custom::CustomEventContent::from_parts(ev_type, input)?;
                    Ok(Self::Custom(content))
                }
            }
        }
    }
    /// Any event.
    pub enum AnyEvent {
        /// Any basic event.
        Basic(AnyBasicEvent),
        /// Any ephemeral room event.
        Ephemeral(AnyEphemeralRoomEvent),
        /// Any message event.
        Message(AnyMessageEvent),
        /// Any state event.
        State(AnyStateEvent),
        /// Any message event that has been redacted.
        RedactedMessage(AnyRedactedMessageEvent),
        /// Any state event that has been redacted.
        RedactedState(AnyRedactedStateEvent),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for AnyEvent {
        #[inline]
        fn clone(&self) -> AnyEvent {
            match (&*self,) {
                (&AnyEvent::Basic(ref __self_0),) => {
                    AnyEvent::Basic(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyEvent::Ephemeral(ref __self_0),) => {
                    AnyEvent::Ephemeral(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyEvent::Message(ref __self_0),) => {
                    AnyEvent::Message(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyEvent::State(ref __self_0),) => {
                    AnyEvent::State(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyEvent::RedactedMessage(ref __self_0),) => {
                    AnyEvent::RedactedMessage(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyEvent::RedactedState(ref __self_0),) => {
                    AnyEvent::RedactedState(::core::clone::Clone::clone(&(*__self_0)))
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for AnyEvent {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&AnyEvent::Basic(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Basic");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyEvent::Ephemeral(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Ephemeral");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyEvent::Message(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Message");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyEvent::State(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("State");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyEvent::RedactedMessage(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RedactedMessage");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyEvent::RedactedState(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RedactedState");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(rust_2018_idioms, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for AnyEvent {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::export::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    AnyEvent::Basic(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyEvent::Ephemeral(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyEvent::Message(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyEvent::State(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyEvent::RedactedMessage(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyEvent::RedactedState(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                }
            }
        }
    };
    /// Any room event.
    pub enum AnyRoomEvent {
        /// Any message event.
        Message(AnyMessageEvent),
        /// Any state event.
        State(AnyStateEvent),
        /// Any message event that has been redacted.
        RedactedMessage(AnyRedactedMessageEvent),
        /// Any state event that has been redacted.
        RedactedState(AnyRedactedStateEvent),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for AnyRoomEvent {
        #[inline]
        fn clone(&self) -> AnyRoomEvent {
            match (&*self,) {
                (&AnyRoomEvent::Message(ref __self_0),) => {
                    AnyRoomEvent::Message(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyRoomEvent::State(ref __self_0),) => {
                    AnyRoomEvent::State(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyRoomEvent::RedactedMessage(ref __self_0),) => {
                    AnyRoomEvent::RedactedMessage(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyRoomEvent::RedactedState(ref __self_0),) => {
                    AnyRoomEvent::RedactedState(::core::clone::Clone::clone(&(*__self_0)))
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for AnyRoomEvent {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&AnyRoomEvent::Message(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Message");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRoomEvent::State(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("State");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRoomEvent::RedactedMessage(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RedactedMessage");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRoomEvent::RedactedState(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RedactedState");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(rust_2018_idioms, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for AnyRoomEvent {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::export::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    AnyRoomEvent::Message(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRoomEvent::State(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRoomEvent::RedactedMessage(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRoomEvent::RedactedState(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                }
            }
        }
    };
    /// Any room event stub (room event without a `room_id`, as returned in `/sync` responses)
    pub enum AnyRoomEventStub {
        /// Any message event stub
        Message(AnyMessageEventStub),
        /// Any state event stub
        State(AnyStateEventStub),
        /// Any message event stub that has been redacted.
        RedactedMessage(AnyRedactedMessageEventStub),
        /// Any state event stub that has been redacted.
        RedactedState(AnyRedactedStateEventStub),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for AnyRoomEventStub {
        #[inline]
        fn clone(&self) -> AnyRoomEventStub {
            match (&*self,) {
                (&AnyRoomEventStub::Message(ref __self_0),) => {
                    AnyRoomEventStub::Message(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyRoomEventStub::State(ref __self_0),) => {
                    AnyRoomEventStub::State(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyRoomEventStub::RedactedMessage(ref __self_0),) => {
                    AnyRoomEventStub::RedactedMessage(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&AnyRoomEventStub::RedactedState(ref __self_0),) => {
                    AnyRoomEventStub::RedactedState(::core::clone::Clone::clone(&(*__self_0)))
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for AnyRoomEventStub {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&AnyRoomEventStub::Message(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Message");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRoomEventStub::State(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("State");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRoomEventStub::RedactedMessage(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RedactedMessage");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&AnyRoomEventStub::RedactedState(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("RedactedState");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(rust_2018_idioms, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for AnyRoomEventStub {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::export::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    AnyRoomEventStub::Message(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRoomEventStub::State(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRoomEventStub::RedactedMessage(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                    AnyRoomEventStub::RedactedState(ref __field0) => {
                        _serde::Serialize::serialize(__field0, __serializer)
                    }
                }
            }
        }
    };
    impl<'de> de::Deserialize<'de> for AnyEvent {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            let json = Box::<RawJsonValue>::deserialize(deserializer)?;
            let EventDeHelper { state_key, event_id, room_id, unsigned, .. } =
                from_raw_json_value(&json)?;
            if state_key.is_some() {
                if let Some(unsigned) = unsigned {
                    if unsigned.redacted_because.is_some() {
                        return Ok(AnyEvent::RedactedState(from_raw_json_value(&json)?));
                    }
                }
                Ok(AnyEvent::State(from_raw_json_value(&json)?))
            } else if event_id.is_some() {
                if let Some(unsigned) = unsigned {
                    if unsigned.redacted_because.is_some() {
                        return Ok(AnyEvent::RedactedMessage(from_raw_json_value(&json)?));
                    }
                }
                Ok(AnyEvent::Message(from_raw_json_value(&json)?))
            } else if room_id.is_some() {
                Ok(AnyEvent::Ephemeral(from_raw_json_value(&json)?))
            } else {
                Ok(AnyEvent::Basic(from_raw_json_value(&json)?))
            }
        }
    }
    impl<'de> de::Deserialize<'de> for AnyRoomEvent {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            let json = Box::<RawJsonValue>::deserialize(deserializer)?;
            let EventDeHelper { state_key, unsigned, .. } = from_raw_json_value(&json)?;
            if state_key.is_some() {
                if let Some(unsigned) = unsigned {
                    if unsigned.redacted_because.is_some() {
                        return Ok(AnyRoomEvent::RedactedState(from_raw_json_value(&json)?));
                    }
                }
                Ok(AnyRoomEvent::State(from_raw_json_value(&json)?))
            } else {
                if let Some(unsigned) = unsigned {
                    if unsigned.redacted_because.is_some() {
                        return Ok(AnyRoomEvent::RedactedMessage(from_raw_json_value(&json)?));
                    }
                }
                Ok(AnyRoomEvent::Message(from_raw_json_value(&json)?))
            }
        }
    }
    impl<'de> de::Deserialize<'de> for AnyRoomEventStub {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            let json = Box::<RawJsonValue>::deserialize(deserializer)?;
            let EventDeHelper { state_key, unsigned, .. } = from_raw_json_value(&json)?;
            if state_key.is_some() {
                if let Some(unsigned) = unsigned {
                    if unsigned.redacted_because.is_some() {
                        return Ok(AnyRoomEventStub::RedactedState(from_raw_json_value(&json)?));
                    }
                }
                Ok(AnyRoomEventStub::State(from_raw_json_value(&json)?))
            } else {
                if let Some(unsigned) = unsigned {
                    if unsigned.redacted_because.is_some() {
                        return Ok(AnyRoomEventStub::RedactedMessage(from_raw_json_value(&json)?));
                    }
                }
                Ok(AnyRoomEventStub::Message(from_raw_json_value(&json)?))
            }
        }
    }
}
