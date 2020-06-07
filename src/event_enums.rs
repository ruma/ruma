use serde::{Deserialize, Serialize};

use crate::{
    room::{
        aliases::AliasesEventContent, avatar::AvatarEventContent,
        canonical_alias::CanonicalAliasEventContent, create::CreateEventContent,
        encryption::EncryptionEventContent, guest_access::GuestAccessEventContent,
        history_visibility::HistoryVisibilityEventContent, join_rules::JoinRulesEventContent,
        member::MemberEventContent, name::NameEventContent,
        pinned_events::PinnedEventsEventContent, power_levels::PowerLevelsEventContent,
        server_acl::ServerAclEventContent, third_party_invite::ThirdPartyInviteEventContent,
        tombstone::TombstoneEventContent, topic::TopicEventContent,
    },
    StateEvent,
};

// TODO: Optimize `Deserialize` implementations.
// It should be possible to first deserialize into CatchAllEvent<AnyWhateverEventContent> and
// transform that, which should have much more consistent performance than trying all variants
// in order.

/// Any state event.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum AnyStateEvent {
    /// An `m.room.aliases` event.
    RoomAliases(StateEvent<AliasesEventContent>),
    /// An `m.room.avatar` event.
    RoomAvatar(StateEvent<AvatarEventContent>),
    /// An `m.room.canonical_alias` event.
    RoomCanonicalAlias(StateEvent<CanonicalAliasEventContent>),
    /// An `m.room.create` event.
    RoomCreate(StateEvent<CreateEventContent>),
    /// An `m.room.encryption` event.
    RoomEncryption(StateEvent<EncryptionEventContent>),
    /// An `m.room.guest_access` event.
    RoomGuestAccess(StateEvent<GuestAccessEventContent>),
    /// An `m.room.history_visibility` event.
    RoomHistoryVisibility(StateEvent<HistoryVisibilityEventContent>),
    /// An `m.room.join_rules` event.
    RoomJoinRules(StateEvent<JoinRulesEventContent>),
    /// An `m.room.member` event.
    RoomMember(StateEvent<MemberEventContent>),
    /// An `m.room.name` event.
    RoomName(StateEvent<NameEventContent>),
    /// An `m.room.pinned_events` event.
    RoomPinnedEvents(StateEvent<PinnedEventsEventContent>),
    /// An `m.room.power_levels` event.
    RoomPowerLevels(StateEvent<PowerLevelsEventContent>),
    /// An `m.room.server_acl` event.
    RoomServerAcl(StateEvent<ServerAclEventContent>),
    /// An `m.room.third_party_invite` event.
    RoomThirdPartyInvite(StateEvent<ThirdPartyInviteEventContent>),
    /// An `m.room.tombstone` event.
    RoomTombstone(StateEvent<TombstoneEventContent>),
    /// An `m.room.topic` event.
    RoomTopic(StateEvent<TopicEventContent>),
}
