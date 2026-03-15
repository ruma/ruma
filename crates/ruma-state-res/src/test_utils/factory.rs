use std::collections::{HashMap, HashSet};

use js_int::{UInt, uint};
use ruma_common::{
    EventId, MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedRoomId, OwnedUserId, RoomVersionId,
    owned_event_id, owned_user_id,
    room::JoinRule,
    room_version_rules::{AuthorizationRules, RoomVersionRules},
    serde::JsonObject,
};
use ruma_events::{StateEventType, TimelineEventType};
use serde_json::{json, to_value as to_json_value};

use super::{Pdu, default_room_id};
use crate::{StateMap, auth_types_for_event, events::RoomCreateEvent};

/// A helper type to build a room timeline.
///
/// This keeps track of the order of events and the room state at the end of the timeline.
///
/// In the most common case where you want a room that is already set up, this should be constructed
/// with [`with_public_chat_preset()`](Self::with_public_chat_preset). It is also possible to only
/// populate the `m.room.create` event with [`RoomCreatePduBuilder`] and create the factory with
/// [`.build_factory()`](RoomCreatePduBuilder::build_factory).
///
/// Creating PDUs with this factory will create the proper content for the event type with good
/// defaults, and populate the `prev_events` and `auth_events` according to the room history.
///
/// Every PDU in the factory can be modified after creation if needed.
pub struct RoomTimelineFactory {
    /// The ID of the room.
    room_id: OwnedRoomId,

    /// The rules for the current room version.
    rules: RoomVersionRules,

    /// The current server timestamp.
    ///
    /// This is incremented for every event.
    server_ts: UInt,

    /// The PDUs in the room.
    pdus: HashMap<OwnedEventId, Pdu>,

    /// The ordered list of PDUs in the timeline.
    ///
    /// Following the `prev_events` of PDUs should give that order.
    timeline: Vec<OwnedEventId>,

    /// The current state at the end of the timeline.
    ///
    /// Map of `(type, state_key)` to `event_id`.
    state: StateMap<OwnedEventId>,
}

impl RoomTimelineFactory {
    /// Construct a `RoomTimelineFactory` with the default timeline for the `public_chat` preset for
    /// the given room version.
    ///
    /// Panics if the room version is not supported, i.e. it is not possible to get the room version
    /// rules, or it doesn't enforce canonical JSON.
    pub fn with_public_chat_preset(room_version: RoomVersionId) -> Self {
        let mut factory = RoomCreatePduBuilder::new(room_version).build_factory();
        let alice_id = UserFactory::Alice.user_id();

        // Alice, the creator, joins the room.
        factory.add_room_member(
            PublicChatInitialPdu::RoomMemberAliceJoin.event_id(),
            alice_id.clone(),
            RoomMemberPduContent::Join,
        );

        // Initial power levels with Alice as an admin or creator, depending on the room version.
        factory.add_room_power_levels(
            PublicChatInitialPdu::RoomPowerLevels.event_id(),
            alice_id.clone(),
            RoomPowerLevelsPduContent::Default,
        );

        // `public` join rule for the `public_chat` preset.
        factory.add_room_join_rules(
            PublicChatInitialPdu::RoomJoinRules.event_id(),
            alice_id,
            JoinRule::Public,
        );

        // Bob joins the room.
        factory.add_room_member(
            PublicChatInitialPdu::RoomMemberBobJoin.event_id(),
            UserFactory::Bob.user_id(),
            RoomMemberPduContent::Join,
        );

        factory
    }

    /// Get a reference to map of PDUs.
    pub fn pdus(&self) -> &HashMap<OwnedEventId, Pdu> {
        &self.pdus
    }

    /// Get a reference to the PDU with the given event ID.
    pub fn get(&self, event_id: &EventId) -> Option<&Pdu> {
        self.pdus.get(event_id)
    }

    /// A function to get a reference to the PDU with the given event ID
    pub fn get_fn<'a>(&'a self) -> impl Fn(&EventId) -> Option<&'a Pdu> + Copy {
        |event_id: &EventId| self.get(event_id)
    }

    /// Get a mutable reference to the PDU with the given event ID.
    pub fn get_mut(&mut self, event_id: &EventId) -> Option<&mut Pdu> {
        self.pdus.get_mut(event_id)
    }

    /// Get the `m.room.create` PDU for this room.
    pub fn room_create_pdu(&self) -> RoomCreateEvent<&Pdu> {
        RoomCreateEvent::new(
            self.pdus
                .get(&self.timeline[0])
                .expect("A RoomTimelineFactory should have an `m.room.create` event"),
        )
    }

    /// Remove the PDU with the given event ID from the map of PDUs.
    ///
    /// Its event ID will still appear in the state map, the timeline, and possibly the
    /// `prev_events` and `auth_events` of other PDUs.
    pub fn remove(&mut self, event_id: &EventId) {
        self.pdus.remove(event_id);
    }

    /// Get a reference to the state map.
    pub fn state(&self) -> &StateMap<OwnedEventId> {
        &self.state
    }

    /// Get the event ID of the PDU for the given `type` and `state_key` in the current state.
    pub fn state_event_id(
        &self,
        event_type: &StateEventType,
        state_key: &str,
    ) -> Option<&OwnedEventId> {
        self.state.get(&(event_type.clone(), state_key.to_owned()))
    }

    /// Get the PDU for the given `type` and `state_key` in the current state.
    pub fn state_event(&self, event_type: &StateEventType, state_key: &str) -> Option<&Pdu> {
        let event_id = self.state_event_id(event_type, state_key)?;
        self.pdus.get(event_id)
    }

    /// A function to get the state event for the given `type` and `state_key` in the current state.
    pub fn state_event_fn<'a>(
        &'a self,
    ) -> impl Fn(&StateEventType, &str) -> Option<&'a Pdu> + Copy {
        |event_type: &StateEventType, state_key: &str| self.state_event(event_type, state_key)
    }

    /// Get the full auth chain for the given state map.
    ///
    /// Panics if an event in the auth chain is missing from the map of PDUs.
    pub fn full_auth_chain(&self, state_map: &StateMap<OwnedEventId>) -> HashSet<OwnedEventId> {
        let mut auth_chain = HashSet::new();
        let mut stack = state_map.values().cloned().collect::<Vec<_>>();

        while let Some(event_id) = stack.pop() {
            let pdu = self.pdus.get(&event_id).expect("PDU should be in map");

            stack.extend(
                pdu.auth_events
                    .iter()
                    .filter(|auth_event_id| !auth_chain.contains(*auth_event_id))
                    .cloned(),
            );

            auth_chain.insert(event_id);
        }

        auth_chain
    }

    /// Get the next server timestamp.
    fn next_server_timestamp(&mut self) -> MilliSecondsSinceUnixEpoch {
        self.server_ts += uint!(1);
        MilliSecondsSinceUnixEpoch(self.server_ts)
    }

    /// Prepare the PDU before adding it to the end of the timeline.
    ///
    /// The following fields are populated according to the current state:
    ///
    /// * `room_id`
    /// * `origin_server_ts`
    /// * `prev_events`
    /// * `auth_events`
    pub fn prepare_to_add_pdu(&mut self, pdu: &mut Pdu) {
        pdu.room_id = Some(self.room_id.clone());
        pdu.origin_server_ts = self.next_server_timestamp();
        pdu.prev_events.extend(self.timeline.last().cloned());

        // Populate the auth events with the algorithm from the spec.
        let auth_types = auth_types_for_event(
            &pdu.event_type,
            &pdu.sender,
            pdu.state_key.as_deref(),
            &pdu.content,
            &self.rules.authorization,
        )
        .unwrap();
        pdu.auth_events.extend(auth_types.iter().flat_map(|(event_type, state_key)| {
            self.state_event_id(event_type, state_key).cloned()
        }));
    }

    /// Add the given PDU to the end of the timeline.
    ///
    /// When a PDU is created directly with the constructors of [`Pdu`],
    /// [`RoomTimelineFactory::prepare_to_add_pdu()`] must be used before this method.
    ///
    /// Returns a mutable reference to the added PDU.
    pub fn add_pdu(&mut self, pdu: Pdu) -> &mut Pdu {
        let event_id = pdu.event_id.clone();

        if let Some(state_key) = pdu.state_key.clone() {
            self.state.insert((pdu.event_type.to_string().into(), state_key), event_id.clone());
        }

        self.timeline.push(event_id.clone());

        self.pdus.entry(event_id).insert_entry(pdu).into_mut()
    }

    /// Create an `m.room.member` event prepared to be added to the timeline.
    ///
    /// `target` will always be used as the `state_key`, and will also be used as the `sender` if
    /// the membership is not allowed from another sender.
    ///
    /// Returns the newly created PDU.
    pub fn create_room_member(
        &mut self,
        event_id: OwnedEventId,
        target: OwnedUserId,
        content: RoomMemberPduContent,
    ) -> Pdu {
        let (sender, content) = content.into_parts(&target);

        let mut pdu = Pdu::with_minimal_state_fields(
            event_id,
            sender,
            TimelineEventType::RoomMember,
            target.into(),
            content,
        );
        self.prepare_to_add_pdu(&mut pdu);

        pdu
    }

    /// Create an `m.room.member` event and add it to the end of the timeline.
    ///
    /// `target` will always be used as the `state_key`, and will also be used as the `sender` if
    /// the change is not allowed from another sender.
    ///
    /// Returns a mutable reference to the added PDU.
    pub fn add_room_member(
        &mut self,
        event_id: OwnedEventId,
        target: OwnedUserId,
        content: RoomMemberPduContent,
    ) -> &mut Pdu {
        let pdu = self.create_room_member(event_id, target, content);
        self.add_pdu(pdu)
    }

    /// Create an `m.room.power_levels` event prepared to be added to the timeline.
    ///
    /// Returns the newly created PDU.
    pub fn create_room_power_levels(
        &mut self,
        event_id: OwnedEventId,
        sender: OwnedUserId,
        content: RoomPowerLevelsPduContent,
    ) -> Pdu {
        let mut pdu = Pdu::with_minimal_state_fields(
            event_id,
            sender,
            TimelineEventType::RoomPowerLevels,
            String::new(),
            content.into_json(&self.rules.authorization),
        );
        self.prepare_to_add_pdu(&mut pdu);

        pdu
    }

    /// Create an `m.room.power_levels` event and add it to the end of the timeline.
    ///
    /// Returns a mutable reference to the added PDU.
    pub fn add_room_power_levels(
        &mut self,
        event_id: OwnedEventId,
        sender: OwnedUserId,
        content: RoomPowerLevelsPduContent,
    ) -> &mut Pdu {
        let pdu = self.create_room_power_levels(event_id, sender, content);
        self.add_pdu(pdu)
    }

    /// Create an `m.room.join_rules` event prepared to be added to the timeline.
    ///
    /// Returns the newly created PDU.
    pub fn create_room_join_rules(
        &mut self,
        event_id: OwnedEventId,
        sender: OwnedUserId,
        join_rule: JoinRule,
    ) -> Pdu {
        let mut pdu = Pdu::with_minimal_state_fields(
            event_id,
            sender,
            TimelineEventType::RoomJoinRules,
            String::new(),
            join_rule,
        );
        self.prepare_to_add_pdu(&mut pdu);

        pdu
    }

    /// Create an `m.room.join_rules` event and add it to the end of the timeline.
    ///
    /// Returns a mutable reference to the added PDU.
    pub fn add_room_join_rules(
        &mut self,
        event_id: OwnedEventId,
        sender: OwnedUserId,
        join_rule: JoinRule,
    ) -> &mut Pdu {
        let pdu = self.create_room_join_rules(event_id, sender, join_rule);
        self.add_pdu(pdu)
    }

    /// Create an `m.room.redaction` event prepared to be added to the timeline.
    ///
    /// Returns the newly created PDU.
    pub fn create_room_redaction(
        &mut self,
        event_id: OwnedEventId,
        sender: OwnedUserId,
        redacts: OwnedEventId,
    ) -> Pdu {
        let mut content = JsonObject::new();

        if self.rules.redaction.content_field_redacts {
            content.insert("redacts".to_owned(), redacts.to_string().into());
        }

        let mut pdu =
            Pdu::with_minimal_fields(event_id, sender, TimelineEventType::RoomRedaction, content);
        pdu.redacts = Some(redacts);
        self.prepare_to_add_pdu(&mut pdu);

        pdu
    }

    /// Create an `m.room.redaction` event and add it to the end of the timeline.
    ///
    /// Returns a mutable reference to the added PDU.
    pub fn add_room_redaction(
        &mut self,
        event_id: OwnedEventId,
        sender: OwnedUserId,
        redacts: OwnedEventId,
    ) -> &mut Pdu {
        let pdu = self.create_room_redaction(event_id, sender, redacts);
        self.add_pdu(pdu)
    }

    /// Create a textual `m.room.message` event prepared to be added to the timeline.
    ///
    /// Returns the newly created PDU.
    pub fn create_text_message(
        &mut self,
        event_id: OwnedEventId,
        sender: OwnedUserId,
        text: impl Into<String>,
    ) -> Pdu {
        let mut pdu = Pdu::with_minimal_fields(
            event_id,
            sender,
            TimelineEventType::RoomMessage,
            json!({
                "msgtype": "m.text",
                "body": text.into(),
            }),
        );
        self.prepare_to_add_pdu(&mut pdu);

        pdu
    }

    /// Create an `m.room.third_party_invite` event prepared to be added to the timeline.
    ///
    /// The sender is [`Bob`](UserFactory::Bob). This is hardcoded to match
    /// [`create_room_member_third_party_invite()`](Self::create_room_member_third_party_invite).
    ///
    /// Returns the newly created PDU.
    pub fn create_room_third_party_invite(&mut self) -> Pdu {
        let content = json!({
            "display_name": "z...@o...",
            "key_validity_url": "https://identity.local/_matrix/identity/v2/pubkey/isvalid",
            "public_key": "Gb9ECWmEzf6FQbrBZ9w7lshQhqowtrbLDFw4rXAxZuE",
            "public_keys": [
                {
                    "key_validity_url": "https://identity.local/_matrix/identity/v2/pubkey/isvalid",
                    "public_key": "Gb9ECWmEzf6FQbrBZ9w7lshQhqowtrbLDFw4rXAxZuE"
                },
                {
                    "key_validity_url": "https://identity.local/_matrix/identity/v2/pubkey/ephemeral/isvalid",
                    "public_key": "VmTAw9B8j/gpkrXl1R7N2NNAyfWuwJnRf5YWHnO3rW4"
                }
            ]
        });

        let mut pdu = Pdu::with_minimal_state_fields(
            owned_event_id!("$room-third-party-invite-zara"),
            UserFactory::Bob.user_id(),
            TimelineEventType::RoomThirdPartyInvite,
            "uniquetoken".to_owned(),
            content,
        );
        self.prepare_to_add_pdu(&mut pdu);

        pdu
    }

    /// Create an `m.room.third_party_invite` event and add it to the end of the timeline.
    ///
    /// The sender is [`Bob`](UserFactory::Bob). This is hardcoded to match
    /// [`create_room_member_third_party_invite()`](Self::create_room_member_third_party_invite).
    ///
    /// Returns a mutable reference to the PDU.
    pub fn add_room_third_party_invite(&mut self) -> &mut Pdu {
        let pdu = self.create_room_third_party_invite();
        self.add_pdu(pdu)
    }

    /// Create an `m.room.member` event with an `invite` membership and a `third_party_invite`.
    ///
    /// The invite is for [`Zara`](UserFactory::Zara) and the sender is [`Bob`](UserFactory::Bob).
    /// They are hardcoded because this event contains a hardcoded signature as well.
    ///
    /// To have a valid state, [`add_room_third_party_invite()`](Self::add_room_third_party_invite)
    ///
    /// Returns the newly created PDU.
    pub fn create_room_member_third_party_invite(&mut self) -> Pdu {
        let zara_id = UserFactory::Zara.user_id();
        let bob_id = UserFactory::Bob.user_id();

        let content = json!({
            "membership": "invite",
            "third_party_invite": {
                "display_name": "z...@o...",
                "signed": {
                    "mxid": zara_id,
                    "sender": bob_id,
                    "token": "uniquetoken",
                    "signatures": {
                        "identity.local": {
                            // This signature will be ignored because the algorithm is unsupported.
                            "unknown:0": "SomeSignature",
                            // This signature will fail the verification.
                            "ed25519:0": "ClearlyWrongSignature",
                            // This signature will pass verification!
                            "ed25519:1": "GTXhO9a3ysW0GWd79vTvzAPi2F2YjZNJDHkwFCyCfYaF0g6tEBjajamAkIgwhXNAp/85PVpzjY6j9oDci8DqDA",
                        }
                    },
                }
            }
        });

        let mut pdu = Pdu::with_minimal_state_fields(
            owned_event_id!("$room-member-zara-invite"),
            bob_id,
            TimelineEventType::RoomMember,
            zara_id.into(),
            content,
        );
        self.prepare_to_add_pdu(&mut pdu);

        pdu
    }
}

/// A type representing the different PDUs available in the initial state of
/// [`RoomTimelineFactory::with_public_chat_preset()`].
///
/// The variants use the same order as the events in the timeline.
///
/// The event ID is named after the variant, i.e. the PDU named `FooBar` will have an event ID of
/// `$foo-bar`.
///
/// The `type`, `state_key` and `event_id` can be obtained with accessors.
#[derive(Debug, Clone, Copy)]
#[allow(clippy::enum_variant_names)]
pub enum PublicChatInitialPdu {
    /// The `m.room.create` event, the first PDU in the room.
    ///
    /// By default the following fields are set in the `content`:
    ///
    /// * `room_version` - Set to the version provided when the factory was constructed.
    /// * `creator` - Set for room versions before 11, the same as the sender.
    ///
    /// The event ID is `$room-create` and [`Alice`](UserFactory::Alice) is the sender.
    RoomCreate,

    /// The `m.room.member` event making [`Alice`](UserFactory::Alice), the creator, join the
    /// room.
    ///
    /// The event ID is `$room-member-alice-join`.
    RoomMemberAliceJoin,

    /// The initial `m.room.power_levels` event.
    ///
    /// By default the following fields are set in the `content`:
    ///
    /// * `users` - For room versions before 11, the creator has a power level of `100`.
    ///
    /// The event ID is `$room-power-levels` and [`Alice`](UserFactory::Alice) is the
    /// sender.
    RoomPowerLevels,

    /// The initial `m.room.join_rules` event.
    ///
    /// By default the following fields are set in the `content`:
    ///
    /// * `join_rule` - Set to `public`.
    ///
    /// The event ID is `$room-join-rules` and [`Alice`](UserFactory::Alice) is the
    /// sender.
    RoomJoinRules,

    /// The `m.room.member` event making [`Bob`](UserFactory::Bob) join the room.
    ///
    /// The event ID is `$room-member-bob-join`.
    RoomMemberBobJoin,
}

impl PublicChatInitialPdu {
    /// The `type` and `state_key` tuple used as a key for the PDU in the state map.
    pub fn type_and_state_key(self) -> (StateEventType, String) {
        match self {
            Self::RoomCreate => (StateEventType::RoomCreate, String::new()),
            Self::RoomMemberAliceJoin => {
                (StateEventType::RoomMember, UserFactory::Alice.user_id().into())
            }
            Self::RoomPowerLevels => (StateEventType::RoomPowerLevels, String::new()),
            Self::RoomJoinRules => (StateEventType::RoomJoinRules, String::new()),
            Self::RoomMemberBobJoin => {
                (StateEventType::RoomMember, UserFactory::Bob.user_id().into())
            }
        }
    }

    /// The default `event_id` of the PDU.
    pub fn event_id(self) -> OwnedEventId {
        match self {
            Self::RoomCreate => owned_event_id!("$room-create"),
            Self::RoomMemberAliceJoin => owned_event_id!("$room-member-alice-join"),
            Self::RoomPowerLevels => owned_event_id!("$room-power-levels"),
            Self::RoomJoinRules => owned_event_id!("$room-join-rules"),
            Self::RoomMemberBobJoin => owned_event_id!("$room-member-bob-join"),
        }
    }
}

/// A type representing common users used in tests.
#[derive(Debug, Clone, Copy)]
pub enum UserFactory {
    /// `@alice:matrix.local`
    ///
    /// When using `RoomCreatePduBuilder::new().build()` or
    /// [`RoomTimelineFactory::with_public_chat_preset()`] this is the creator of the room.
    ///
    /// When using [`RoomTimelineFactory::with_public_chat_preset()`] with a room with a version
    /// before 12, this user has a power level of `100`.
    Alice,

    /// `@bob:matrix.local`
    ///
    /// When using the default settings of [`RoomTimelineFactory`], this is a member of the room.
    Bob,

    /// `@charlie:matrix.local`
    ///
    /// A user on the same homeserver as the room creator.
    Charlie,

    /// `@zara:other.local`
    ///
    /// A user on a different homeserver than the room creator.
    Zara,
}

impl UserFactory {
    /// Get the ID of this user.
    pub fn user_id(self) -> OwnedUserId {
        match self {
            Self::Alice => owned_user_id!("@alice:matrix.local"),
            Self::Bob => owned_user_id!("@bob:matrix.local"),
            Self::Charlie => owned_user_id!("@charlie:matrix.local"),
            Self::Zara => owned_user_id!("@zara:other.local"),
        }
    }
}

/// A helper to construct a valid `m.room.create` [`Pdu`].
///
/// This type purposefully doesn't allow to construct an invalid `m.room.create` event. The PDU can
/// be modified after construction to make it invalid.
pub struct RoomCreatePduBuilder {
    /// The version of the room.
    room_version: RoomVersionId,

    /// The rules for the current room version.
    rules: RoomVersionRules,

    /// The value of the `additional_creators` field in the content.
    additional_creators: Vec<OwnedUserId>,
}

impl RoomCreatePduBuilder {
    /// Construct a `RoomCreatePduBuilder` with the given room version.
    ///
    /// Panics if the room version is not supported, i.e. it is not possible to get the room version
    /// rules, or it doesn't enforce canonical JSON.
    pub fn new(room_version: RoomVersionId) -> Self {
        let rules = room_version.rules().expect("room version should be supported");

        if !rules.authorization.strict_canonical_json {
            panic!("Only room versions that enforce canonical JSON are properly supported");
        }

        Self { room_version, rules, additional_creators: Vec::new() }
    }

    /// Set the value of the `additional_creators` field in the content.
    ///
    /// The field is only set if the list is not empty.
    ///
    /// Defaults to an empty list.
    pub fn additional_creators(mut self, additional_creators: Vec<OwnedUserId>) -> Self {
        self.additional_creators = additional_creators;
        self
    }

    /// Build the `m.room.create` PDU.
    fn build_inner(&self) -> Pdu {
        let sender = UserFactory::Alice.user_id();

        let mut content = JsonObject::new();
        content.insert("room_version".to_owned(), self.room_version.to_string().into());

        if !self.rules.authorization.use_room_create_sender {
            content.insert("creator".to_owned(), sender.to_string().into());
        }

        if !self.additional_creators.is_empty() {
            content.insert(
                "additional_creators".to_owned(),
                to_json_value(&self.additional_creators).unwrap(),
            );
        }

        let mut pdu = Pdu::with_minimal_state_fields(
            PublicChatInitialPdu::RoomCreate.event_id(),
            sender,
            TimelineEventType::RoomCreate,
            String::new(),
            content,
        );
        pdu.room_id = self
            .rules
            .event_format
            .require_room_create_room_id
            .then(|| default_room_id(&self.rules.room_id_format));

        pdu
    }

    /// Consume this builder and return the `m.room.create` PDU.
    pub fn build(self) -> Pdu {
        self.build_inner()
    }

    /// Consume this builder and return a [`RoomTimelineFactory`] initialized with the
    /// `m.room.create` PDU.
    pub fn build_factory(self) -> RoomTimelineFactory {
        let room_create = self.build_inner();
        let room_create_event_id = room_create.event_id.clone();
        let room_id = room_create
            .room_id
            .clone()
            .unwrap_or_else(|| default_room_id(&self.rules.room_id_format));

        RoomTimelineFactory {
            room_id,
            rules: self.rules,
            server_ts: UInt::MIN,
            pdus: [(room_create_event_id.clone(), room_create)].into(),
            timeline: vec![room_create_event_id.clone()],
            state: [((StateEventType::RoomCreate, String::new()), room_create_event_id)].into(),
        }
    }
}

/// Supported predefined contents of an `m.room.member` [`Pdu`].
pub enum RoomMemberPduContent {
    /// The target user joins the room.
    Join,

    /// The target user joins a restricted room, authorized by the server of the given member of the
    /// room.
    JoinAuthorized {
        /// A member of the room that can invite the target user.
        via_users_server: OwnedUserId,
    },

    /// The target user updates their display name.
    DisplayName {
        /// The display name of the user.
        displayname: String,
    },

    /// The target user is banned from the room.
    Ban {
        /// The user that banned the target user.
        sender: OwnedUserId,
    },

    /// The target user is invited to the room.
    Invite {
        /// The user that invited the target user.
        sender: OwnedUserId,
    },

    /// The target user knocked on the room.
    Knock,

    /// The target user left the room.
    Leave,

    /// The target user was kicked from the room.
    Kick {
        /// The user that kicked the target user.
        sender: OwnedUserId,
    },
}

impl RoomMemberPduContent {
    /// Get the sender and content for this membership.
    pub fn into_parts(self, target: &OwnedUserId) -> (OwnedUserId, JsonObject) {
        let mut content = JsonObject::new();

        let (sender, membership) = match self {
            Self::Join => (target.clone(), "join"),
            Self::JoinAuthorized { via_users_server } => {
                content.insert(
                    "join_authorised_via_users_server".to_owned(),
                    String::from(via_users_server).into(),
                );

                (target.clone(), "join")
            }
            Self::DisplayName { displayname } => {
                content.insert("displayname".to_owned(), displayname.into());

                (target.clone(), "join")
            }
            Self::Ban { sender } => (sender, "ban"),
            Self::Invite { sender } => (sender, "invite"),
            Self::Knock => (target.clone(), "knock"),
            Self::Leave => (target.clone(), "leave"),
            Self::Kick { sender } => (sender, "leave"),
        };

        content.insert("membership".to_owned(), membership.into());

        (sender, content)
    }
}

/// Supported predefined contents of an `m.room.member` [`Pdu`].
pub enum RoomPowerLevelsPduContent {
    /// The minimal content share with other variants.
    ///
    /// In room versions 1 through 11, [`Alice`](UserFactory::Alice) has a power level of `100`.
    Default,

    /// The power level required to send invites is changed to the given value.
    Invite {
        /// The required power level.
        value: i32,
    },

    /// The power level required to send the given event types is changed to the given value.
    Events {
        /// The event types.
        event_types: Vec<TimelineEventType>,

        /// The required power level.
        value: i32,
    },

    /// The power level of the given user is changed to the given value.
    User {
        /// The user.
        user_id: OwnedUserId,

        /// The new power level.
        value: i32,
    },
}

impl RoomPowerLevelsPduContent {
    /// Construct the JSON object for this content for the given authorization rules.
    fn into_json(self, authorization_rules: &AuthorizationRules) -> JsonObject {
        let mut content = JsonObject::new();

        if !authorization_rules.explicitly_privilege_room_creators {
            let users = json!({
                UserFactory::Alice.user_id(): 100,
            });
            content.insert("users".to_owned(), users);
        }

        match self {
            Self::Default => {}
            Self::Invite { value } => {
                content.insert("invite".to_owned(), value.into());
            }
            Self::Events { event_types, value } => {
                let events = JsonObject::from_iter(
                    event_types
                        .into_iter()
                        .map(|event_type| (event_type.to_string(), value.into())),
                );
                content.insert("events".to_owned(), events.into());
            }
            Self::User { user_id, value } => {
                content
                    .entry("users".to_owned())
                    .or_insert_with(|| JsonObject::new().into())
                    .as_object_mut()
                    .unwrap()
                    .insert(user_id.into(), value.into());
            }
        }

        content
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use js_int::int;
    use ruma_common::{
        RoomVersionId, owned_event_id, room::JoinRuleKind, room_version_rules::AuthorizationRules,
        user_id,
    };
    use ruma_events::{StateEventType, room::member::MembershipState};

    use super::RoomTimelineFactory;
    use crate::events::{
        RoomCreateEvent, RoomJoinRulesEvent, RoomMemberEvent, RoomPowerLevelsEvent,
    };

    #[test]
    fn public_chat_preset_v10() {
        // Check that the PDUs are the ones that we expect and are correct.
        let factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V10);

        assert_eq!(factory.pdus.len(), 5);
        assert_eq!(factory.timeline.len(), 5);
        assert_eq!(factory.state.values().count(), 5);
        assert_eq!(factory.room_id, "!room:matrix.local");

        // `m.room.create`.
        let room_create_event_id = owned_event_id!("$room-create");
        assert_eq!(factory.timeline[0], room_create_event_id);
        assert_eq!(
            *factory.state_event_id(&StateEventType::RoomCreate, "").unwrap(),
            room_create_event_id
        );

        let pdu = RoomCreateEvent::new(factory.get(&room_create_event_id).unwrap());
        assert_eq!(pdu.event_id, room_create_event_id);
        // For room version 10, the room ID should be set.
        assert_eq!(pdu.room_id.as_ref().unwrap(), "!room:matrix.local");
        assert_eq!(pdu.room_version(), Ok(RoomVersionId::V10));
        // For room version 10, the creator field should be set in the content.
        assert_eq!(
            pdu.creator(&AuthorizationRules::V10).as_deref(),
            Ok(user_id!("@alice:matrix.local"))
        );
        assert!(pdu.prev_events.is_empty());
        assert!(pdu.auth_events.is_empty());

        // `m.room.member` for Alice.
        let room_member_alice_join_event_id = owned_event_id!("$room-member-alice-join");
        assert_eq!(factory.timeline[1], room_member_alice_join_event_id);
        assert_eq!(
            *factory.state_event_id(&StateEventType::RoomMember, "@alice:matrix.local").unwrap(),
            room_member_alice_join_event_id
        );

        let pdu = RoomMemberEvent::new(factory.get(&room_member_alice_join_event_id).unwrap());
        assert_eq!(pdu.event_id, room_member_alice_join_event_id);
        assert_eq!(pdu.state_key.as_deref(), Some("@alice:matrix.local"));
        assert_eq!(pdu.membership(), Ok(MembershipState::Join));
        assert_eq!(pdu.prev_events, [room_create_event_id.clone()].into());
        assert_eq!(pdu.auth_events, [room_create_event_id.clone()].into());

        // `m.room.power_levels`.
        let room_power_levels_event_id = owned_event_id!("$room-power-levels");
        assert_eq!(factory.timeline[2], room_power_levels_event_id);
        assert_eq!(
            *factory.state_event_id(&StateEventType::RoomPowerLevels, "").unwrap(),
            room_power_levels_event_id
        );

        let pdu = RoomPowerLevelsEvent::new(factory.get(&room_power_levels_event_id).unwrap());
        assert_eq!(pdu.event_id, room_power_levels_event_id);
        // For room version 10, Alice should appear in the users.
        let users = pdu.users(&AuthorizationRules::V10).unwrap().unwrap();
        assert_eq!(users.get(user_id!("@alice:matrix.local")), Some(&int!(100)));
        assert_eq!(pdu.prev_events, [room_member_alice_join_event_id.clone()].into());
        assert_eq!(
            pdu.auth_events,
            [room_member_alice_join_event_id.clone(), room_create_event_id.clone()].into()
        );

        // `m.room.join_rules`.
        let room_join_rules_event_id = owned_event_id!("$room-join-rules");
        assert_eq!(factory.timeline[3], room_join_rules_event_id);
        assert_eq!(
            *factory.state_event_id(&StateEventType::RoomJoinRules, "").unwrap(),
            room_join_rules_event_id
        );

        let pdu = RoomJoinRulesEvent::new(factory.get(&room_join_rules_event_id).unwrap());
        assert_eq!(pdu.event_id, room_join_rules_event_id);
        assert_eq!(pdu.join_rule(), Ok(JoinRuleKind::Public));
        assert_eq!(pdu.prev_events, [room_power_levels_event_id.clone()].into());
        assert_eq!(
            pdu.auth_events,
            [
                room_power_levels_event_id.clone(),
                room_member_alice_join_event_id.clone(),
                room_create_event_id.clone()
            ]
            .into()
        );

        // `m.room.member` for Bob.
        let room_member_bob_join_event_id = owned_event_id!("$room-member-bob-join");
        assert_eq!(factory.timeline[4], room_member_bob_join_event_id);
        assert_eq!(
            *factory.state_event_id(&StateEventType::RoomMember, "@bob:matrix.local").unwrap(),
            room_member_bob_join_event_id
        );

        let pdu = RoomMemberEvent::new(factory.get(&room_member_bob_join_event_id).unwrap());
        assert_eq!(pdu.event_id, room_member_bob_join_event_id);
        assert_eq!(pdu.state_key.as_deref(), Some("@bob:matrix.local"));
        assert_eq!(pdu.membership(), Ok(MembershipState::Join));
        assert_eq!(pdu.prev_events, [room_join_rules_event_id.clone()].into());
        assert_eq!(
            pdu.auth_events,
            [
                room_power_levels_event_id.clone(),
                room_create_event_id.clone(),
                room_join_rules_event_id.clone()
            ]
            .into()
        );
    }

    #[test]
    fn public_chat_preset_v11() {
        // Check that the PDUs are the ones that we expect and are correct.
        let factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V11);

        assert_eq!(factory.pdus.len(), 5);
        assert_eq!(factory.timeline.len(), 5);
        assert_eq!(factory.state.values().count(), 5);
        assert_eq!(factory.room_id, "!room:matrix.local");

        // `m.room.create`.
        let room_create_event_id = owned_event_id!("$room-create");
        assert_eq!(factory.timeline[0], room_create_event_id);
        assert_eq!(
            *factory.state_event_id(&StateEventType::RoomCreate, "").unwrap(),
            room_create_event_id
        );

        let pdu = RoomCreateEvent::new(factory.get(&room_create_event_id).unwrap());
        assert_eq!(pdu.event_id, room_create_event_id);
        // For room version 11, the room ID should be set.
        assert_eq!(pdu.room_id.as_ref().unwrap(), "!room:matrix.local");
        assert_eq!(pdu.room_version(), Ok(RoomVersionId::V11));
        // For room version 11, the creator field should not be set in the content.
        assert_matches!(pdu.creator(&AuthorizationRules::V10), Err(_));
        assert!(pdu.prev_events.is_empty());
        assert!(pdu.auth_events.is_empty());

        // `m.room.member` for Alice.
        let room_member_alice_join_event_id = owned_event_id!("$room-member-alice-join");
        assert_eq!(factory.timeline[1], room_member_alice_join_event_id);
        assert_eq!(
            *factory.state_event_id(&StateEventType::RoomMember, "@alice:matrix.local").unwrap(),
            room_member_alice_join_event_id
        );

        let pdu = RoomMemberEvent::new(factory.get(&room_member_alice_join_event_id).unwrap());
        assert_eq!(pdu.event_id, room_member_alice_join_event_id);
        assert_eq!(pdu.state_key.as_deref(), Some("@alice:matrix.local"));
        assert_eq!(pdu.membership(), Ok(MembershipState::Join));
        assert_eq!(pdu.prev_events, [room_create_event_id.clone()].into());
        assert_eq!(pdu.auth_events, [room_create_event_id.clone()].into());

        // `m.room.power_levels`.
        let room_power_levels_event_id = owned_event_id!("$room-power-levels");
        assert_eq!(factory.timeline[2], room_power_levels_event_id);
        assert_eq!(
            *factory.state_event_id(&StateEventType::RoomPowerLevels, "").unwrap(),
            room_power_levels_event_id
        );

        let pdu = RoomPowerLevelsEvent::new(factory.get(&room_power_levels_event_id).unwrap());
        assert_eq!(pdu.event_id, room_power_levels_event_id);
        // For room version 11, Alice should appear in the users.
        let users = pdu.users(&AuthorizationRules::V11).unwrap().unwrap();
        assert_eq!(users.get(user_id!("@alice:matrix.local")), Some(&int!(100)));
        assert_eq!(pdu.prev_events, [room_member_alice_join_event_id.clone()].into());
        assert_eq!(
            pdu.auth_events,
            [room_member_alice_join_event_id.clone(), room_create_event_id.clone()].into()
        );

        // `m.room.join_rules`.
        let room_join_rules_event_id = owned_event_id!("$room-join-rules");
        assert_eq!(factory.timeline[3], room_join_rules_event_id);
        assert_eq!(
            *factory.state_event_id(&StateEventType::RoomJoinRules, "").unwrap(),
            room_join_rules_event_id
        );

        let pdu = RoomJoinRulesEvent::new(factory.get(&room_join_rules_event_id).unwrap());
        assert_eq!(pdu.event_id, room_join_rules_event_id);
        assert_eq!(pdu.join_rule(), Ok(JoinRuleKind::Public));
        assert_eq!(pdu.prev_events, [room_power_levels_event_id.clone()].into());
        assert_eq!(
            pdu.auth_events,
            [
                room_power_levels_event_id.clone(),
                room_member_alice_join_event_id.clone(),
                room_create_event_id.clone()
            ]
            .into()
        );

        // `m.room.member` for Bob.
        let room_member_bob_join_event_id = owned_event_id!("$room-member-bob-join");
        assert_eq!(factory.timeline[4], room_member_bob_join_event_id);
        assert_eq!(
            *factory.state_event_id(&StateEventType::RoomMember, "@bob:matrix.local").unwrap(),
            room_member_bob_join_event_id
        );

        let pdu = RoomMemberEvent::new(factory.get(&room_member_bob_join_event_id).unwrap());
        assert_eq!(pdu.event_id, room_member_bob_join_event_id);
        assert_eq!(pdu.state_key.as_deref(), Some("@bob:matrix.local"));
        assert_eq!(pdu.membership(), Ok(MembershipState::Join));
        assert_eq!(pdu.prev_events, [room_join_rules_event_id.clone()].into());
        assert_eq!(
            pdu.auth_events,
            [
                room_power_levels_event_id.clone(),
                room_create_event_id.clone(),
                room_join_rules_event_id.clone()
            ]
            .into()
        );
    }

    #[test]
    fn public_chat_preset_v12() {
        // Check that the PDUs are the ones that we expect and are correct.
        // For room version 12, the room create event should not appear in auth events.
        let factory = RoomTimelineFactory::with_public_chat_preset(RoomVersionId::V12);

        assert_eq!(factory.pdus.len(), 5);
        assert_eq!(factory.timeline.len(), 5);
        assert_eq!(factory.state.values().count(), 5);
        assert_eq!(factory.room_id, "!room-create");

        // `m.room.create`.
        let room_create_event_id = owned_event_id!("$room-create");
        assert_eq!(factory.timeline[0], room_create_event_id);
        assert_eq!(
            *factory.state_event_id(&StateEventType::RoomCreate, "").unwrap(),
            room_create_event_id
        );

        let pdu = RoomCreateEvent::new(factory.get(&room_create_event_id).unwrap());
        assert_eq!(pdu.event_id, room_create_event_id);
        // For room version 12, the room ID should not be set.
        assert_eq!(pdu.room_id, None);
        assert_eq!(pdu.room_version(), Ok(RoomVersionId::V12));
        // For room version 12, the creator field should not be set in the content.
        assert_matches!(pdu.creator(&AuthorizationRules::V10), Err(_));
        assert!(pdu.prev_events.is_empty());
        assert!(pdu.auth_events.is_empty());

        // `m.room.member` for Alice.
        let room_member_alice_join_event_id = owned_event_id!("$room-member-alice-join");
        assert_eq!(factory.timeline[1], room_member_alice_join_event_id);
        assert_eq!(
            *factory.state_event_id(&StateEventType::RoomMember, "@alice:matrix.local").unwrap(),
            room_member_alice_join_event_id
        );

        let pdu = RoomMemberEvent::new(factory.get(&room_member_alice_join_event_id).unwrap());
        assert_eq!(pdu.event_id, room_member_alice_join_event_id);
        assert_eq!(pdu.state_key.as_deref(), Some("@alice:matrix.local"));
        assert_eq!(pdu.membership(), Ok(MembershipState::Join));
        assert_eq!(pdu.prev_events, [room_create_event_id.clone()].into());
        assert!(pdu.auth_events.is_empty());

        // `m.room.power_levels`.
        let room_power_levels_event_id = owned_event_id!("$room-power-levels");
        assert_eq!(factory.timeline[2], room_power_levels_event_id);
        assert_eq!(
            *factory.state_event_id(&StateEventType::RoomPowerLevels, "").unwrap(),
            room_power_levels_event_id
        );

        let pdu = RoomPowerLevelsEvent::new(factory.get(&room_power_levels_event_id).unwrap());
        assert_eq!(pdu.event_id, room_power_levels_event_id);
        // For room version 12, the users map should not be set.
        assert_matches!(pdu.users(&AuthorizationRules::V12), Ok(None));
        assert_eq!(pdu.prev_events, [room_member_alice_join_event_id.clone()].into());
        assert_eq!(pdu.auth_events, [room_member_alice_join_event_id.clone(),].into());

        // `m.room.join_rules`.
        let room_join_rules_event_id = owned_event_id!("$room-join-rules");
        assert_eq!(factory.timeline[3], room_join_rules_event_id);
        assert_eq!(
            *factory.state_event_id(&StateEventType::RoomJoinRules, "").unwrap(),
            room_join_rules_event_id
        );

        let pdu = RoomJoinRulesEvent::new(factory.get(&room_join_rules_event_id).unwrap());
        assert_eq!(pdu.event_id, room_join_rules_event_id);
        assert_eq!(pdu.join_rule(), Ok(JoinRuleKind::Public));
        assert_eq!(pdu.prev_events, [room_power_levels_event_id.clone()].into());
        assert_eq!(
            pdu.auth_events,
            [room_power_levels_event_id.clone(), room_member_alice_join_event_id.clone(),].into()
        );

        // `m.room.member` for Bob.
        let room_member_bob_join_event_id = owned_event_id!("$room-member-bob-join");
        assert_eq!(factory.timeline[4], room_member_bob_join_event_id);
        assert_eq!(
            *factory.state_event_id(&StateEventType::RoomMember, "@bob:matrix.local").unwrap(),
            room_member_bob_join_event_id
        );

        let pdu = RoomMemberEvent::new(factory.get(&room_member_bob_join_event_id).unwrap());
        assert_eq!(pdu.event_id, room_member_bob_join_event_id);
        assert_eq!(pdu.state_key.as_deref(), Some("@bob:matrix.local"));
        assert_eq!(pdu.membership(), Ok(MembershipState::Join));
        assert_eq!(pdu.prev_events, [room_join_rules_event_id.clone()].into());
        assert_eq!(
            pdu.auth_events,
            [room_power_levels_event_id.clone(), room_join_rules_event_id.clone()].into()
        );
    }
}
