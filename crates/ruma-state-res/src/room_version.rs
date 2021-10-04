use ruma_identifiers::RoomVersionId;

use crate::{Error, Result};

#[derive(Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum RoomDisposition {
    /// A room version that has a stable specification.
    Stable,
    /// A room version that is not yet fully specified.
    #[allow(dead_code)]
    Unstable,
}

#[derive(Debug)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum EventFormatVersion {
    /// $id:server event id format
    V1,
    /// MSC1659-style $hash event id format: introduced for room v3
    V2,
    /// MSC1884-style $hash format: introduced for room v4
    V3,
}

#[derive(Debug)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum StateResolutionVersion {
    /// State resolution for rooms at version 1.
    V1,
    /// State resolution for room at version 2 or later.
    V2,
}

#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RoomVersion {
    /// The version this room is set to.
    pub version: RoomVersionId,
    /// The stability of this room.
    pub disposition: RoomDisposition,
    /// The format of the EventId.
    pub event_format: EventFormatVersion,
    /// Which state resolution algorithm is used.
    pub state_res: StateResolutionVersion,
    // FIXME: not sure what this one means?
    pub enforce_key_validity: bool,

    /// `m.room.aliases` had special auth rules and redaction rules
    /// before room version 6.
    ///
    /// before MSC2261/MSC2432,
    pub special_case_aliases_auth: bool,
    /// Strictly enforce canonical json, do not allow:
    /// * Integers outside the range of [-2 ^ 53 + 1, 2 ^ 53 - 1]
    /// * Floats
    /// * NaN, Infinity, -Infinity
    pub strict_canonicaljson: bool,
    /// Verify notifications key while checking m.room.power_levels.
    ///
    /// bool: MSC2209: Check 'notifications'
    pub limit_notifications_power_levels: bool,
    /// Extra rules when verifying redaction events.
    pub extra_redaction_checks: bool,
    /// Allow knocking in event authentication.
    pub allow_knocking: bool,
    /// Allow knocking in event authentication.
    pub restricted_join_rules: bool,
}

impl RoomVersion {
    pub fn new(version: &RoomVersionId) -> Result<Self> {
        Ok(match version {
            RoomVersionId::Version1 => Self::version_1(),
            RoomVersionId::Version2 => Self::version_2(),
            RoomVersionId::Version3 => Self::version_3(),
            RoomVersionId::Version4 => Self::version_4(),
            RoomVersionId::Version5 => Self::version_5(),
            RoomVersionId::Version6 => Self::version_6(),
            #[cfg(feature = "unstable-pre-spec")]
            RoomVersionId::Version7 => Self::version_7(),
            #[cfg(feature = "unstable-pre-spec")]
            RoomVersionId::Version8 => Self::version_8(),
            #[cfg(feature = "unstable-pre-spec")]
            RoomVersionId::Version9 => Self::version_9(),
            ver => return Err(Error::Unsupported(format!("found version `{}`", ver.as_str()))),
        })
    }

    pub fn version_1() -> Self {
        Self {
            version: RoomVersionId::Version1,
            disposition: RoomDisposition::Stable,
            event_format: EventFormatVersion::V1,
            state_res: StateResolutionVersion::V1,
            enforce_key_validity: false,
            special_case_aliases_auth: true,
            strict_canonicaljson: false,
            limit_notifications_power_levels: false,
            extra_redaction_checks: false,
            allow_knocking: false,
            restricted_join_rules: false,
        }
    }

    pub fn version_2() -> Self {
        Self {
            version: RoomVersionId::Version2,
            disposition: RoomDisposition::Stable,
            event_format: EventFormatVersion::V1,
            state_res: StateResolutionVersion::V2,
            enforce_key_validity: false,
            special_case_aliases_auth: true,
            strict_canonicaljson: false,
            limit_notifications_power_levels: false,
            extra_redaction_checks: false,
            allow_knocking: false,
            restricted_join_rules: false,
        }
    }

    pub fn version_3() -> Self {
        Self {
            version: RoomVersionId::Version3,
            disposition: RoomDisposition::Stable,
            event_format: EventFormatVersion::V2,
            state_res: StateResolutionVersion::V2,
            enforce_key_validity: false,
            special_case_aliases_auth: true,
            strict_canonicaljson: false,
            limit_notifications_power_levels: false,
            extra_redaction_checks: true,
            allow_knocking: false,
            restricted_join_rules: false,
        }
    }

    pub fn version_4() -> Self {
        Self {
            version: RoomVersionId::Version4,
            disposition: RoomDisposition::Stable,
            event_format: EventFormatVersion::V3,
            state_res: StateResolutionVersion::V2,
            enforce_key_validity: false,
            special_case_aliases_auth: true,
            strict_canonicaljson: false,
            limit_notifications_power_levels: false,
            extra_redaction_checks: true,
            allow_knocking: false,
            restricted_join_rules: false,
        }
    }

    pub fn version_5() -> Self {
        Self {
            version: RoomVersionId::Version5,
            disposition: RoomDisposition::Stable,
            event_format: EventFormatVersion::V3,
            state_res: StateResolutionVersion::V2,
            enforce_key_validity: true,
            special_case_aliases_auth: true,
            strict_canonicaljson: false,
            limit_notifications_power_levels: false,
            extra_redaction_checks: true,
            allow_knocking: false,
            restricted_join_rules: false,
        }
    }

    pub fn version_6() -> Self {
        Self {
            version: RoomVersionId::Version6,
            disposition: RoomDisposition::Stable,
            event_format: EventFormatVersion::V3,
            state_res: StateResolutionVersion::V2,
            enforce_key_validity: true,
            special_case_aliases_auth: false,
            strict_canonicaljson: true,
            limit_notifications_power_levels: true,
            extra_redaction_checks: true,
            allow_knocking: false,
            restricted_join_rules: false,
        }
    }

    #[cfg(feature = "unstable-pre-spec")]
    pub fn version_7() -> Self {
        Self {
            version: RoomVersionId::Version7,
            disposition: RoomDisposition::Stable,
            event_format: EventFormatVersion::V3,
            state_res: StateResolutionVersion::V2,
            enforce_key_validity: true,
            special_case_aliases_auth: false,
            strict_canonicaljson: true,
            limit_notifications_power_levels: true,
            extra_redaction_checks: true,
            allow_knocking: true,
            restricted_join_rules: false,
        }
    }

    #[cfg(feature = "unstable-pre-spec")]
    pub fn version_8() -> Self {
        Self {
            version: RoomVersionId::Version8,
            disposition: RoomDisposition::Stable,
            event_format: EventFormatVersion::V3,
            state_res: StateResolutionVersion::V2,
            enforce_key_validity: true,
            special_case_aliases_auth: false,
            strict_canonicaljson: true,
            limit_notifications_power_levels: true,
            extra_redaction_checks: true,
            // Wait so should this be false?
            // because: https://github.com/matrix-org/matrix-doc/pull/3289#issuecomment-884165177
            allow_knocking: false,
            restricted_join_rules: true,
        }
    }

    #[cfg(feature = "unstable-pre-spec")]
    pub fn version_9() -> Self {
        Self {
            version: RoomVersionId::Version9,
            disposition: RoomDisposition::Stable,
            event_format: EventFormatVersion::V3,
            state_res: StateResolutionVersion::V2,
            enforce_key_validity: true,
            special_case_aliases_auth: false,
            strict_canonicaljson: true,
            limit_notifications_power_levels: true,
            extra_redaction_checks: true,
            allow_knocking: true,
            restricted_join_rules: true,
        }
    }
}
