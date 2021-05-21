use js_int::{uint, UInt};
use serde::{Deserialize, Serialize};
use std::{
    convert::TryInto,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/// A timestamp represented as the number of milliseconds since the unix epoch.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(transparent)]
pub struct MilliSecondsSinceUnixEpoch(pub UInt);

impl MilliSecondsSinceUnixEpoch {
    /// Creates a new `MilliSecondsSinceUnixEpoch` from the given `SystemTime`, if it is not before
    /// the unix epoch, or too large to be represented.
    pub fn from_system_time(time: SystemTime) -> Option<Self> {
        let duration = time.duration_since(UNIX_EPOCH).ok()?;
        let millis = duration.as_millis().try_into().ok()?;
        Some(Self(millis))
    }

    /// The current milliseconds since the unix epoch.
    pub fn now() -> Self {
        Self::from_system_time(SystemTime::now()).unwrap()
    }

    /// Creates a new `SystemTime` from `self`, if it can be represented.
    pub fn to_system_time(self) -> Option<SystemTime> {
        UNIX_EPOCH.checked_add(Duration::from_millis(self.0.into()))
    }

    /// Get the time since the unix epoch in milliseconds.
    pub fn get(&self) -> UInt {
        self.0
    }

    /// Get time since the unix epoch in seconds.
    pub fn as_secs(&self) -> UInt {
        self.0 / uint!(1000)
    }
}

/// A timestamp represented as the number of seconds since the unix epoch.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(transparent)]
pub struct SecondsSinceUnixEpoch(pub UInt);

impl SecondsSinceUnixEpoch {
    /// Creates a new `MilliSecondsSinceUnixEpoch` from the given `SystemTime`, if it is not before
    /// the unix epoch, or too large to be represented.
    pub fn from_system_time(time: SystemTime) -> Option<Self> {
        let duration = time.duration_since(UNIX_EPOCH).ok()?;
        let millis = duration.as_secs().try_into().ok()?;
        Some(Self(millis))
    }

    /// Creates a new `SystemTime` from `self`, if it can be represented.
    pub fn to_system_time(self) -> Option<SystemTime> {
        UNIX_EPOCH.checked_add(Duration::from_secs(self.0.into()))
    }

    /// Get time since the unix epoch in seconds.
    pub fn get(&self) -> UInt {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use js_int::uint;
    use matches::assert_matches;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    use super::{MilliSecondsSinceUnixEpoch, SecondsSinceUnixEpoch};

    #[derive(Clone, Debug, Deserialize, Serialize)]
    struct SystemTimeTest {
        millis: MilliSecondsSinceUnixEpoch,
        secs: SecondsSinceUnixEpoch,
    }

    #[test]
    fn deserialize() {
        let json = json!({ "millis": 3000, "secs": 60 });

        assert_matches!(
            serde_json::from_value::<SystemTimeTest>(json),
            Ok(SystemTimeTest { millis, secs })
            if millis.to_system_time() == Some(UNIX_EPOCH + Duration::from_millis(3000))
                && secs.to_system_time() == Some(UNIX_EPOCH + Duration::from_secs(60))
        );
    }

    #[test]
    fn serialize() {
        let request = SystemTimeTest {
            millis: MilliSecondsSinceUnixEpoch::from_system_time(UNIX_EPOCH + Duration::new(2, 0))
                .unwrap(),
            secs: SecondsSinceUnixEpoch(uint!(0)),
        };
        assert_matches!(
            serde_json::to_value(&request),
            Ok(value) if value == json!({ "millis": 2000, "secs": 0 })
        );
    }
}
