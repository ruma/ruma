use ruma::identifiers::RoomVersionId;

pub enum RoomDisposition {
    /// A room version that has a stable specification.
    Stable,
    /// A room version that is not yet fully specified.
    #[allow(dead_code)]
    Unstable,
}

pub enum EventFormatVersion {
    /// $id:server event id format
    V1,
    /// MSC1659-style $hash event id format: introduced for room v3
    V2,
    /// MSC1884-style $hash format: introduced for room v4
    V3,
}

pub enum StateResolutionVersion {
    /// State resolution for rooms at version 1.
    V1,
    /// State resolution for room at version 2 or later.
    V2,
}

pub struct RoomVersion {
    /// The version this room is set to.
    pub version: RoomVersionId,
    /// The stability of this room.
    pub disposition: RoomDisposition,
    /// The format of the EventId.
    pub event_format: EventFormatVersion,
    /// Which state resolution algorithm is used.
    pub state_res: StateResolutionVersion,
    /// not sure
    pub enforce_key_validity: bool,

    // bool: before MSC2261/MSC2432, m.room.aliases had special auth rules and redaction rules
    pub special_case_aliases_auth: bool,
    // Strictly enforce canonicaljson, do not allow:
    // * Integers outside the range of [-2 ^ 53 + 1, 2 ^ 53 - 1]
    // * Floats
    // * NaN, Infinity, -Infinity
    pub strict_canonicaljson: bool,
    // bool: MSC2209: Check 'notifications' key while verifying
    // m.room.power_levels auth rules.
    pub limit_notifications_power_levels: bool,
}

impl RoomVersion {
    pub fn new(version: &RoomVersionId) -> Self {
        if version.is_version_1() {
            Self::version_1()
        } else if version.is_version_2() {
            Self::version_2()
        } else if version.is_version_3() {
            Self::version_3()
        } else if version.is_version_4() {
            Self::version_4()
        } else if version.is_version_5() {
            Self::version_5()
        } else if version.is_version_6() {
            Self::version_6()
        } else {
            panic!("this crate needs to be updated with ruma")
        }
    }

    fn version_1() -> Self {
        Self {
            version: RoomVersionId::version_1(),
            disposition: RoomDisposition::Stable,
            event_format: EventFormatVersion::V1,
            state_res: StateResolutionVersion::V1,
            enforce_key_validity: false,
            special_case_aliases_auth: true,
            strict_canonicaljson: false,
            limit_notifications_power_levels: false,
        }
    }

    fn version_2() -> Self {
        Self {
            version: RoomVersionId::version_2(),
            disposition: RoomDisposition::Stable,
            event_format: EventFormatVersion::V1,
            state_res: StateResolutionVersion::V2,
            enforce_key_validity: false,
            special_case_aliases_auth: true,
            strict_canonicaljson: false,
            limit_notifications_power_levels: false,
        }
    }

    fn version_3() -> Self {
        Self {
            version: RoomVersionId::version_3(),
            disposition: RoomDisposition::Stable,
            event_format: EventFormatVersion::V2,
            state_res: StateResolutionVersion::V2,
            enforce_key_validity: false,
            special_case_aliases_auth: true,
            strict_canonicaljson: false,
            limit_notifications_power_levels: false,
        }
    }

    fn version_4() -> Self {
        Self {
            version: RoomVersionId::version_4(),
            disposition: RoomDisposition::Stable,
            event_format: EventFormatVersion::V3,
            state_res: StateResolutionVersion::V2,
            enforce_key_validity: false,
            special_case_aliases_auth: true,
            strict_canonicaljson: false,
            limit_notifications_power_levels: false,
        }
    }

    fn version_5() -> Self {
        Self {
            version: RoomVersionId::version_5(),
            disposition: RoomDisposition::Stable,
            event_format: EventFormatVersion::V3,
            state_res: StateResolutionVersion::V2,
            enforce_key_validity: true,
            special_case_aliases_auth: true,
            strict_canonicaljson: false,
            limit_notifications_power_levels: false,
        }
    }

    fn version_6() -> Self {
        Self {
            version: RoomVersionId::version_6(),
            disposition: RoomDisposition::Stable,
            event_format: EventFormatVersion::V3,
            state_res: StateResolutionVersion::V2,
            enforce_key_validity: true,
            special_case_aliases_auth: false,
            strict_canonicaljson: true,
            limit_notifications_power_levels: true,
        }
    }
}
