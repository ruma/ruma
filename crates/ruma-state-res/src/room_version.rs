use ruma_identifiers::RoomVersionId;

use crate::{Error, Result};

#[derive(Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum RoomDisposition {
    /// A room version that has a stable specification.
    Stable,
    /// A room version that is not yet fully specified.
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
    ///
    /// See: https://spec.matrix.org/unstable/rooms/v7/ for more information.
    pub allow_knocking: bool,
    /// Adds support for the restricted join rule.
    ///
    /// See: https://github.com/matrix-org/matrix-doc/pull/3289 for more information.
    pub restricted_join_rules: bool,
}

impl RoomVersion {
    pub const VERSION1: Self = Self {
        version: RoomVersionId::V1,
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
    };

    pub const VERSION2: Self = Self {
        version: RoomVersionId::V2,
        state_res: StateResolutionVersion::V2,
        ..Self::VERSION1
    };

    pub const VERSION3: Self = Self {
        version: RoomVersionId::V3,
        event_format: EventFormatVersion::V2,
        extra_redaction_checks: true,
        ..Self::VERSION2
    };

    pub const VERSION4: Self =
        Self { version: RoomVersionId::V4, event_format: EventFormatVersion::V3, ..Self::VERSION3 };

    pub const VERSION5: Self =
        Self { version: RoomVersionId::V5, enforce_key_validity: true, ..Self::VERSION4 };

    pub const VERSION6: Self = Self {
        version: RoomVersionId::V5,
        special_case_aliases_auth: false,
        strict_canonicaljson: true,
        limit_notifications_power_levels: true,
        ..Self::VERSION5
    };

    #[cfg(feature = "unstable-pre-spec")]
    pub const VERSION7: Self = Self {
        version: RoomVersionId::V7,
        // FIXME: once room version 7 is stabilized move this to version 8
        disposition: RoomDisposition::Unstable,
        allow_knocking: true,
        ..Self::VERSION6
    };

    #[cfg(feature = "unstable-pre-spec")]
    pub const VERSION8: Self =
        Self { version: RoomVersionId::V8, restricted_join_rules: true, ..Self::VERSION7 };

    #[cfg(feature = "unstable-pre-spec")]
    pub const VERSION9: Self = Self { version: RoomVersionId::V9, ..Self::VERSION8 };

    pub fn new(version: &RoomVersionId) -> Result<Self> {
        Ok(match version {
            RoomVersionId::V1 => Self::VERSION1,
            RoomVersionId::V2 => Self::VERSION2,
            RoomVersionId::V3 => Self::VERSION3,
            RoomVersionId::V4 => Self::VERSION4,
            RoomVersionId::V5 => Self::VERSION5,
            RoomVersionId::V6 => Self::VERSION6,
            #[cfg(feature = "unstable-pre-spec")]
            RoomVersionId::V7 => Self::VERSION7,
            #[cfg(feature = "unstable-pre-spec")]
            RoomVersionId::V8 => Self::VERSION8,
            #[cfg(feature = "unstable-pre-spec")]
            RoomVersionId::V9 => Self::VERSION9,
            ver => return Err(Error::Unsupported(format!("found version `{}`", ver.as_str()))),
        })
    }
}
