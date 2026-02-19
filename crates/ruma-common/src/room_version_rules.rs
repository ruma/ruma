//! Types for the rules applied to the different [room versions].
//!
//! [room versions]: https://spec.matrix.org/latest/rooms/

#[allow(clippy::disallowed_types)]
use std::collections::HashSet;

use as_variant::as_variant;

use crate::UserId;

/// The rules applied to a [room version].
///
/// This type can be constructed from one of its constants (like [`RoomVersionRules::V1`]), or from
/// [`RoomVersionId::rules()`].
///
/// [room version]: https://spec.matrix.org/latest/rooms/
/// [`RoomVersionId::rules()`]: crate::RoomVersionId::rules
#[derive(Debug, Clone)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct RoomVersionRules {
    /// The stability of the room version.
    pub disposition: RoomVersionDisposition,

    /// The format of event IDs.
    pub event_id_format: EventIdFormatVersion,

    /// The format of room IDs.
    pub room_id_format: RoomIdFormatVersion,

    /// The format of arrays referencing events in PDUs.
    pub events_reference_format: EventsReferenceFormatVersion,

    /// The state resolution algorithm used.
    pub state_res: StateResolutionVersion,

    /// Whether to enforce the key validity period when verifying signatures ([spec]), introduced
    /// in room version 5.
    ///
    /// [spec]: https://spec.matrix.org/latest/rooms/v5/#signing-key-validity-period
    pub enforce_key_validity: bool,

    /// The tweaks in the authorization rules.
    pub authorization: AuthorizationRules,

    /// The tweaks in the redaction algorithm.
    pub redaction: RedactionRules,

    /// The tweaks for verifying signatures.
    pub signatures: SignaturesRules,

    /// The tweaks for verifying the event format.
    pub event_format: EventFormatRules,
}

impl RoomVersionRules {
    /// Rules for [room version 1].
    ///
    /// [room version 1]: https://spec.matrix.org/latest/rooms/v1/
    pub const V1: Self = Self {
        disposition: RoomVersionDisposition::Stable,
        event_id_format: EventIdFormatVersion::V1,
        room_id_format: RoomIdFormatVersion::V1,
        events_reference_format: EventsReferenceFormatVersion::V1,
        state_res: StateResolutionVersion::V1,
        enforce_key_validity: false,
        authorization: AuthorizationRules::V1,
        redaction: RedactionRules::V1,
        signatures: SignaturesRules::V1,
        event_format: EventFormatRules::V1,
    };

    /// Rules for [room version 2].
    ///
    /// [room version 2]: https://spec.matrix.org/latest/rooms/v2/
    pub const V2: Self =
        Self { state_res: StateResolutionVersion::V2(StateResolutionV2Rules::V2_0), ..Self::V1 };

    /// Rules for [room version 3].
    ///
    /// [room version 3]: https://spec.matrix.org/latest/rooms/v3/
    pub const V3: Self = Self {
        event_id_format: EventIdFormatVersion::V2,
        events_reference_format: EventsReferenceFormatVersion::V2,
        authorization: AuthorizationRules::V3,
        signatures: SignaturesRules::V3,
        event_format: EventFormatRules::V3,
        ..Self::V2
    };

    /// Rules for [room version 4].
    ///
    /// [room version 4]: https://spec.matrix.org/latest/rooms/v4/
    pub const V4: Self = Self { event_id_format: EventIdFormatVersion::V3, ..Self::V3 };

    /// Rules for [room version 5].
    ///
    /// [room version 5]: https://spec.matrix.org/latest/rooms/v5/
    pub const V5: Self = Self { enforce_key_validity: true, ..Self::V4 };

    /// Rules for [room version 6].
    ///
    /// [room version 6]: https://spec.matrix.org/latest/rooms/v6/
    pub const V6: Self =
        Self { authorization: AuthorizationRules::V6, redaction: RedactionRules::V6, ..Self::V5 };

    /// Rules for [room version 7].
    ///
    /// [room version 7]: https://spec.matrix.org/latest/rooms/v7/
    pub const V7: Self = Self { authorization: AuthorizationRules::V7, ..Self::V6 };

    /// Rules for [room version 8].
    ///
    /// [room version 8]: https://spec.matrix.org/latest/rooms/v8/
    pub const V8: Self = Self {
        authorization: AuthorizationRules::V8,
        redaction: RedactionRules::V8,
        signatures: SignaturesRules::V8,
        ..Self::V7
    };

    /// Rules for [room version 9].
    ///
    /// [room version 9]: https://spec.matrix.org/latest/rooms/v9/
    pub const V9: Self = Self { redaction: RedactionRules::V9, ..Self::V8 };

    /// Rules for [room version 10].
    ///
    /// [room version 10]: https://spec.matrix.org/latest/rooms/v10/
    pub const V10: Self = Self { authorization: AuthorizationRules::V10, ..Self::V9 };

    /// Rules for [room version 11].
    ///
    /// [room version 11]: https://spec.matrix.org/latest/rooms/v11/
    pub const V11: Self = Self {
        authorization: AuthorizationRules::V11,
        redaction: RedactionRules::V11,
        ..Self::V10
    };

    /// Rules for room version 12.
    pub const V12: Self = Self {
        room_id_format: RoomIdFormatVersion::V2,
        authorization: AuthorizationRules::V12,
        event_format: EventFormatRules::V12,
        state_res: StateResolutionVersion::V2(StateResolutionV2Rules::V2_1),
        ..Self::V11
    };

    /// Rules for room version `org.matrix.msc2870` ([MSC2870]).
    ///
    /// [MSC2870]: https://github.com/matrix-org/matrix-spec-proposals/pull/2870
    #[cfg(feature = "unstable-msc2870")]
    pub const MSC2870: Self = Self {
        disposition: RoomVersionDisposition::Unstable,
        redaction: RedactionRules::MSC2870,
        ..Self::V11
    };
}

/// The stability of a room version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::exhaustive_enums)]
pub enum RoomVersionDisposition {
    /// A room version that has a stable specification.
    Stable,

    /// A room version that is not yet fully specified.
    Unstable,
}

/// The format of [event IDs] for a room version.
///
/// [event IDs]: https://spec.matrix.org/latest/appendices/#event-ids
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub enum EventIdFormatVersion {
    /// `$id:server` format ([spec]), introduced in room version 1.
    ///
    /// [spec]: https://spec.matrix.org/latest/rooms/v1/#event-ids
    V1,

    /// `$hash` format using standard unpadded base64 ([spec]), introduced in room version 3.
    ///
    /// [spec]: https://spec.matrix.org/latest/rooms/v3/#event-ids
    V2,

    /// `$hash` format using URL-safe unpadded base64 ([spec]), introduced in room version 4.
    ///
    /// [spec]: https://spec.matrix.org/latest/rooms/v4/#event-ids
    V3,
}

/// The format of [room IDs] for a room version.
///
/// [room IDs]: https://spec.matrix.org/latest/appendices/#room-ids
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub enum RoomIdFormatVersion {
    /// `!id:server` format, introduced in room version 1.
    V1,

    /// `!hash` format using the reference hash of the `m.room.create` event of the room,
    /// introduced in room version 12.
    V2,
}

/// The format of [PDU] `auth_events` and `prev_events` for a room version.
///
/// [PDU]: https://spec.matrix.org/latest/server-server-api/#pdus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub enum EventsReferenceFormatVersion {
    /// `[["$id:server", {"sha256": "hash"}]]` format ([spec]), introduced in room version 1.
    ///
    /// [spec]: https://spec.matrix.org/latest/rooms/v1/#event-format
    V1,

    /// `["$hash"]` format ([spec]), introduced in room version 3.
    ///
    /// [spec]: https://spec.matrix.org/latest/rooms/v3/#event-format
    V2,
}

/// The version of [state resolution] for a room version.
///
/// [state resolution]: https://spec.matrix.org/latest/server-server-api/#room-state-resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub enum StateResolutionVersion {
    /// First version of the state resolution algorithm ([spec]), introduced in room version 1.
    ///
    /// [spec]: https://spec.matrix.org/latest/rooms/v1/#state-resolution
    V1,

    /// Second version of the state resolution algorithm ([spec]), introduced in room version 2.
    ///
    /// [spec]: https://spec.matrix.org/latest/rooms/v2/#state-resolution
    V2(StateResolutionV2Rules),
}

impl StateResolutionVersion {
    /// Gets the `StateResolutionV2Rules` for the room version, if it uses the second version of
    /// the state resolution algorithm.
    pub fn v2_rules(&self) -> Option<&StateResolutionV2Rules> {
        as_variant!(self, StateResolutionVersion::V2)
    }
}

/// The tweaks in the [state resolution v2 algorithm] for a room version.
///
/// This type can be constructed from one of its constants (like [`StateResolutionV2Rules::V2_0`]),
/// or by constructing a [`RoomVersionRules`] first and using the `state_res` field (if the room
/// version uses version 2 of the state resolution algorithm).
///
/// [state resolution v2 algorithm]: https://spec.matrix.org/latest/rooms/v2/#state-resolution
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateResolutionV2Rules {
    /// Whether to begin the first phase of iterative auth checks with an empty state map, as
    /// opposed to one containing the unconflicted state, enabled since room version 12.
    pub begin_iterative_auth_checks_with_empty_state_map: bool,

    /// Whether to include the conflicted state subgraph in the full conflicted state, enabled
    /// since room version 12.
    pub consider_conflicted_state_subgraph: bool,
}

impl StateResolutionV2Rules {
    /// The first version of the second iteration of the state resolution algorithm, introduced in
    /// room version 2.
    pub const V2_0: Self = Self {
        begin_iterative_auth_checks_with_empty_state_map: false,
        consider_conflicted_state_subgraph: false,
    };

    /// The second version of the second iteration of the state resolution algorithm, introduced in
    /// room version 12.
    pub const V2_1: Self = Self {
        begin_iterative_auth_checks_with_empty_state_map: true,
        consider_conflicted_state_subgraph: true,
        ..Self::V2_0
    };
}

/// The tweaks in the [authorization rules] for a room version.
///
/// This type can be constructed from one of its constants (like [`AuthorizationRules::V1`]), or by
/// constructing a [`RoomVersionRules`] first and using the `authorization` field.
///
/// [authorization rules]: https://spec.matrix.org/latest/server-server-api/#authorization-rules
#[derive(Debug, Clone)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct AuthorizationRules {
    /// Whether to apply special authorization rules for `m.room.redaction` events ([spec]),
    /// disabled since room version 3.
    ///
    /// [spec]: https://spec.matrix.org/latest/rooms/v3/#handling-redactions
    pub special_case_room_redaction: bool,

    /// Whether to apply special authorization rules for `m.room.aliases` events ([spec]), disabled
    /// since room version 6.
    ///
    /// [spec]: https://spec.matrix.org/latest/rooms/v6/#authorization-rules
    pub special_case_room_aliases: bool,

    /// Whether to strictly enforce [canonical JSON] ([spec]), introduced in room version 6.
    ///
    /// Numbers in Canonical JSON must be integers in the range [-2<sup>53</sup> + 1,
    /// 2<sup>53</sup> - 1], represented without exponents or decimal places, and negative zero
    /// (`-0`) MUST NOT appear.
    ///
    /// [canonical JSON]: https://spec.matrix.org/latest/appendices/#canonical-json
    /// [spec]: https://spec.matrix.org/latest/rooms/v6/#canonical-json
    pub strict_canonical_json: bool,

    /// Whether to check the `notifications` field when checking `m.room.power_levels` events
    /// ([spec]), introduced in room version 6.
    ///
    /// [spec]: https://spec.matrix.org/latest/rooms/v6/#authorization-rules
    pub limit_notifications_power_levels: bool,

    /// Whether to allow the `knock` membership for `m.room.member` events and the `knock` join
    /// rule for `m.room.join_rules` events ([spec]), introduced in room version 7.
    ///
    /// [spec]: https://spec.matrix.org/latest/rooms/v7/#authorization-rules
    pub knocking: bool,

    /// Whether to allow the `restricted` join rule for `m.room.join_rules` events ([spec]),
    /// introduced in room version 8.
    ///
    /// [spec]: https://spec.matrix.org/latest/rooms/v8/#authorization-rules
    pub restricted_join_rule: bool,

    /// Whether to allow the `knock_restricted` join rule for `m.room.join_rules` events ([spec]),
    /// introduced in room version 10.
    ///
    /// [spec]: https://spec.matrix.org/latest/rooms/v10/#authorization-rules
    pub knock_restricted_join_rule: bool,

    /// Whether to enforce that power levels values in `m.room.power_levels` events be integers
    /// ([spec]), introduced in room version 10.
    ///
    /// [spec]: https://spec.matrix.org/latest/rooms/v10/#values-in-mroompower_levels-events-must-be-integers
    pub integer_power_levels: bool,

    /// Whether the room creator should be determined using the `m.room.create` event's `sender`,
    /// instead of the event content's `creator` field ([spec]), introduced in room version 11.
    ///
    /// [spec]: https://spec.matrix.org/v1.17/rooms/v11/#event-format
    pub use_room_create_sender: bool,

    /// Whether room creators should always be considered to have "infinite" power level,
    /// introduced in room version 12.
    pub explicitly_privilege_room_creators: bool,

    /// Whether additional room creators can be set with the `content.additional_creators` field of
    /// an `m.room.create` event, introduced in room version 12.
    pub additional_room_creators: bool,

    /// Whether to use the event ID of the `m.room.create` event of the room as the room ID,
    /// introduced in room version 12.
    pub room_create_event_id_as_room_id: bool,
}

impl AuthorizationRules {
    /// Authorization rules as introduced in room version 1 ([spec]).
    ///
    /// [spec]: https://spec.matrix.org/latest/rooms/v1/#authorization-rules
    pub const V1: Self = Self {
        special_case_room_redaction: true,
        special_case_room_aliases: true,
        strict_canonical_json: false,
        limit_notifications_power_levels: false,
        knocking: false,
        restricted_join_rule: false,
        knock_restricted_join_rule: false,
        integer_power_levels: false,
        use_room_create_sender: false,
        explicitly_privilege_room_creators: false,
        additional_room_creators: false,
        room_create_event_id_as_room_id: false,
    };

    /// Authorization rules with tweaks introduced in room version 3 ([spec]).
    ///
    /// [spec]: https://spec.matrix.org/latest/rooms/v3/#authorization-rules
    pub const V3: Self = Self { special_case_room_redaction: false, ..Self::V1 };

    /// Authorization rules with tweaks introduced in room version 6 ([spec]).
    ///
    /// [spec]: https://spec.matrix.org/latest/rooms/v6/#authorization-rules
    pub const V6: Self = Self {
        special_case_room_aliases: false,
        strict_canonical_json: true,
        limit_notifications_power_levels: true,
        ..Self::V3
    };

    /// Authorization rules with tweaks introduced in room version 7 ([spec]).
    ///
    /// [spec]: https://spec.matrix.org/latest/rooms/v7/#authorization-rules
    pub const V7: Self = Self { knocking: true, ..Self::V6 };

    /// Authorization rules with tweaks introduced in room version 8 ([spec]).
    ///
    /// [spec]: https://spec.matrix.org/latest/rooms/v8/#authorization-rules
    pub const V8: Self = Self { restricted_join_rule: true, ..Self::V7 };

    /// Authorization rules with tweaks introduced in room version 10 ([spec]).
    ///
    /// [spec]: https://spec.matrix.org/latest/rooms/v10/#authorization-rules
    pub const V10: Self =
        Self { knock_restricted_join_rule: true, integer_power_levels: true, ..Self::V8 };

    /// Authorization rules with tweaks introduced in room version 11 ([spec]).
    ///
    /// [spec]: https://spec.matrix.org/latest/rooms/v11/#authorization-rules
    pub const V11: Self = Self { use_room_create_sender: true, ..Self::V10 };

    /// Authorization rules with tweaks introduced in room version 12.
    pub const V12: Self = Self {
        explicitly_privilege_room_creators: true,
        additional_room_creators: true,
        room_create_event_id_as_room_id: true,
        ..Self::V11
    };
}

/// The tweaks in the [redaction] algorithm for a room version.
///
/// This type can be constructed from one of its constants (like [`RedactionRules::V1`]), or by
/// constructing a [`RoomVersionRules`] first and using the `redaction` field.
///
/// [redaction]: https://spec.matrix.org/latest/client-server-api/#redactions
#[derive(Debug, Clone)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct RedactionRules {
    /// Whether to keep the `aliases` field in the `content` of `m.room.aliases` events ([spec]),
    /// disabled since room version 6.
    ///
    /// [spec]: https://spec.matrix.org/v1.17/rooms/v6/#redactions
    pub keep_room_aliases_aliases: bool,

    /// Whether to keep the `allow` field in the `content` of `m.room.join_rules` events ([spec]),
    /// introduced in room version 8.
    ///
    /// [spec]: https://spec.matrix.org/v1.17/rooms/v8/#redactions
    pub keep_room_join_rules_allow: bool,

    /// Whether to keep the `join_authorised_via_users_server` field in the `content` of
    /// `m.room.member` events ([spec]), introduced in room version 9.
    ///
    /// [spec]: https://spec.matrix.org/v1.17/rooms/v9/#redactions
    pub keep_room_member_join_authorised_via_users_server: bool,

    /// Whether to keep the `origin`, `membership` and `prev_state` fields a the top-level of all
    /// events ([spec]), disabled since room version 11.
    ///
    /// [spec]: https://spec.matrix.org/v1.17/rooms/v11/#redactions
    pub keep_origin_membership_prev_state: bool,

    /// Whether to keep the entire `content` of `m.room.create` events ([spec]), introduced in room
    /// version 11.
    ///
    /// [spec]: https://spec.matrix.org/v1.17/rooms/v11/#redactions
    pub keep_room_create_content: bool,

    /// Whether to keep the `redacts` field in the `content` of `m.room.redaction` events ([spec]),
    /// introduced in room version 11.
    ///
    /// [spec]: https://spec.matrix.org/v1.17/rooms/v11/#redactions
    pub keep_room_redaction_redacts: bool,

    /// Whether to keep the `invite` field in the `content` of `m.room.power_levels` events
    /// ([spec]), introduced in room version 11.
    ///
    /// [spec]: https://spec.matrix.org/v1.17/rooms/v11/#redactions
    pub keep_room_power_levels_invite: bool,

    /// Whether to keep the `signed` field in `third_party_invite` of the `content` of
    /// `m.room.member` events ([spec]), introduced in room version 11.
    ///
    /// [spec]: https://spec.matrix.org/v1.17/rooms/v11/#redactions
    pub keep_room_member_third_party_invite_signed: bool,

    /// Whether the `content.redacts` field should be used to determine the event an event
    /// redacts, as opposed to the top-level `redacts` field ([spec]), introduced in room version
    /// 11.
    ///
    /// [spec]: https://spec.matrix.org/v1.17/rooms/v11/#redactions
    pub content_field_redacts: bool,

    /// Whether to keep the `allow`, `deny` and `allow_ip_literals` in the `content` of
    /// `m.room.server_acl` events ([MSC2870]).
    ///
    /// [MSC2870]: https://github.com/matrix-org/matrix-spec-proposals/pull/2870
    #[cfg(feature = "unstable-msc2870")]
    pub keep_room_server_acl_allow_deny_allow_ip_literals: bool,
}

impl RedactionRules {
    /// Redaction rules as introduced in room version 1 ([spec]).
    ///
    /// [spec]: https://spec.matrix.org/v1.17/rooms/v1/#redactions
    pub const V1: Self = Self {
        keep_room_aliases_aliases: true,
        keep_room_join_rules_allow: false,
        keep_room_member_join_authorised_via_users_server: false,
        keep_origin_membership_prev_state: true,
        keep_room_create_content: false,
        keep_room_redaction_redacts: false,
        keep_room_power_levels_invite: false,
        keep_room_member_third_party_invite_signed: false,
        content_field_redacts: false,
        #[cfg(feature = "unstable-msc2870")]
        keep_room_server_acl_allow_deny_allow_ip_literals: false,
    };

    /// Redaction rules with tweaks introduced in room version 6 ([spec]).
    ///
    /// [spec]: https://spec.matrix.org/v1.17/rooms/v6/#redactions
    pub const V6: Self = Self { keep_room_aliases_aliases: false, ..Self::V1 };

    /// Redaction rules with tweaks introduced in room version 8 ([spec]).
    ///
    /// [spec]: https://spec.matrix.org/v1.17/rooms/v8/#redactions
    pub const V8: Self = Self { keep_room_join_rules_allow: true, ..Self::V6 };

    /// Redaction rules with tweaks introduced in room version 9 ([spec]).
    ///
    /// [spec]: https://spec.matrix.org/v1.17/rooms/v9/#redactions
    pub const V9: Self =
        Self { keep_room_member_join_authorised_via_users_server: true, ..Self::V8 };

    /// Redaction rules with tweaks introduced in room version 11 ([spec]).
    ///
    /// [spec]: https://spec.matrix.org/v1.17/rooms/v11/#redactions
    pub const V11: Self = Self {
        keep_origin_membership_prev_state: false,
        keep_room_create_content: true,
        keep_room_redaction_redacts: true,
        keep_room_power_levels_invite: true,
        keep_room_member_third_party_invite_signed: true,
        content_field_redacts: true,
        ..Self::V9
    };

    /// Redaction rules with tweaks introduced in [MSC2870].
    ///
    /// [MSC2870]: https://github.com/matrix-org/matrix-spec-proposals/pull/2870
    #[cfg(feature = "unstable-msc2870")]
    pub const MSC2870: Self =
        Self { keep_room_server_acl_allow_deny_allow_ip_literals: true, ..Self::V11 };
}

/// The tweaks for [verifying the signatures] for a room version.
///
/// This type can be constructed from one of its constants (like [`SignaturesRules::V1`]), or by
/// constructing a [`RoomVersionRules`] first and using the `signatures` field.
///
/// [verifying the signatures]: https://spec.matrix.org/latest/server-server-api/#validating-hashes-and-signatures-on-received-events
#[derive(Debug, Clone)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct SignaturesRules {
    /// Whether to check the server of the event ID, disabled since room version 3.
    pub check_event_id_server: bool,

    /// Whether to check the server of the `join_authorised_via_users_server` field in the
    /// `content` of `m.room.member` events ([spec]), introduced in room version 8.
    ///
    /// [spec]: https://spec.matrix.org/latest/rooms/v8/#authorization-rules
    pub check_join_authorised_via_users_server: bool,
}

impl SignaturesRules {
    /// Signatures verification rules as introduced in room version 1.
    pub const V1: Self =
        Self { check_event_id_server: true, check_join_authorised_via_users_server: false };

    /// Signatures verification rules with tweaks introduced in room version 3.
    pub const V3: Self = Self { check_event_id_server: false, ..Self::V1 };

    /// Signatures verification rules with tweaks introduced in room version 8.
    pub const V8: Self = Self { check_join_authorised_via_users_server: true, ..Self::V3 };
}

/// The tweaks for verifying the event format for a room version.
///
/// This type can be constructed from one of its constants (like [`EventFormatRules::V1`]), or by
/// constructing a [`RoomVersionRules`] first and using the `event_format` field.
#[derive(Debug, Clone)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct EventFormatRules {
    /// Whether the `event_id` field is required, disabled since room version 3.
    pub require_event_id: bool,

    /// Whether the `room_id` field is required on the `m.room.create` event, disabled since room
    /// version 12.
    pub require_room_create_room_id: bool,

    /// Whether the `m.room.create` event is allowed to be in the `auth_events`, disabled since
    /// room version 12.
    pub allow_room_create_in_auth_events: bool,
}

impl EventFormatRules {
    /// Event format rules as introduced in room version 1.
    pub const V1: Self = Self {
        require_event_id: true,
        require_room_create_room_id: true,
        allow_room_create_in_auth_events: true,
    };

    /// Event format rules with tweaks introduced in room version 3.
    pub const V3: Self = Self { require_event_id: false, ..Self::V1 };

    /// Event format rules with tweaks introduced in room version 12.
    pub const V12: Self = Self {
        require_room_create_room_id: false,
        allow_room_create_in_auth_events: false,
        ..Self::V3
    };
}

/// The tweaks for determining the power level of a user.
#[derive(Clone, Debug)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct RoomPowerLevelsRules {
    /// The creator(s) of the room which are considered to have "infinite" power level, introduced
    /// in room version 12.
    #[allow(clippy::disallowed_types)]
    pub privileged_creators: Option<HashSet<UserId>>,
}

impl RoomPowerLevelsRules {
    /// Creates a new `RoomPowerLevelsRules` from the given parameters, using the `creators` if
    /// the rule `explicitly_privilege_room_creators` is `true`.
    pub fn new(rules: &AuthorizationRules, creators: impl IntoIterator<Item = UserId>) -> Self {
        Self {
            privileged_creators: rules
                .explicitly_privilege_room_creators
                .then(|| creators.into_iter().collect()),
        }
    }
}
