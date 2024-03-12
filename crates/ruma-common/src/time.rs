use std::{
    fmt,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use js_int::{uint, UInt};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// A timestamp represented as the number of milliseconds since the unix epoch.
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[allow(clippy::exhaustive_structs)]
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

    /// The current system time in milliseconds since the unix epoch.
    pub fn now() -> Self {
        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown", feature = "js")))]
        return Self::from_system_time(SystemTime::now()).expect("date out of range");

        #[cfg(all(target_arch = "wasm32", target_os = "unknown", feature = "js"))]
        return Self(f64_to_uint(js_sys::Date::now()));
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

impl fmt::Debug for MilliSecondsSinceUnixEpoch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match OffsetDateTime::from_unix_timestamp(i64::from(self.0) / 1000) {
            Ok(date) => {
                let date = date + Duration::from_millis(u64::from(self.0) % 1000);

                let (year, month, day) = date.to_calendar_date();
                let month = month as u8;
                let (hours, minutes, seconds, milliseconds) = date.to_hms_milli();

                write!(
                    f,
                    "{year}-{month:02}-{day:02}T\
                     {hours:02}:{minutes:02}:{seconds:02}.{milliseconds:03}"
                )
            }
            // Probably dead code..
            Err(_) => {
                // The default Debug impl would put the inner value on its own
                // line if the formatter's alternate mode is enabled, which
                // bloats debug strings unnecessarily
                write!(f, "MilliSecondsSinceUnixEpoch({})", self.0)
            }
        }
    }
}

/// A timestamp represented as the number of seconds since the unix epoch.
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[allow(clippy::exhaustive_structs)]
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

    /// The current system-time as seconds since the unix epoch.
    pub fn now() -> Self {
        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown", feature = "js")))]
        return Self::from_system_time(SystemTime::now()).expect("date out of range");

        #[cfg(all(target_arch = "wasm32", target_os = "unknown", feature = "js"))]
        return Self(f64_to_uint(js_sys::Date::now() / 1000.0));
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

impl fmt::Debug for SecondsSinceUnixEpoch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match OffsetDateTime::from_unix_timestamp(i64::from(self.0)) {
            Ok(date) => {
                let (year, month, day) = date.to_calendar_date();
                let month = month as u8;
                let (hours, minutes, seconds) = date.to_hms();

                write!(
                    f,
                    "{year}-{month:02}-{day:02}T\
                     {hours:02}:{minutes:02}:{seconds:02}"
                )
            }
            // Probably dead code..
            Err(_) => {
                // The default Debug impl would put the inner value on its own
                // line if the formatter's alternate mode is enabled, which
                // bloats debug strings unnecessarily
                write!(f, "SecondsSinceUnixEpoch({})", self.0)
            }
        }
    }
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown", feature = "js"))]
fn f64_to_uint(val: f64) -> UInt {
    // UInt::MAX milliseconds is ~285 616 years, we do not account for that
    // (or for dates before the unix epoch which would have to be negative)
    UInt::try_from(val as u64).expect("date out of range")
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use js_int::uint;
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

        let time = serde_json::from_value::<SystemTimeTest>(json).unwrap();
        assert_eq!(time.millis.to_system_time(), Some(UNIX_EPOCH + Duration::from_millis(3000)));
        assert_eq!(time.secs.to_system_time(), Some(UNIX_EPOCH + Duration::from_secs(60)));
    }

    #[test]
    fn serialize() {
        let request = SystemTimeTest {
            millis: MilliSecondsSinceUnixEpoch::from_system_time(UNIX_EPOCH + Duration::new(2, 0))
                .unwrap(),
            secs: SecondsSinceUnixEpoch(uint!(0)),
        };

        assert_eq!(serde_json::to_value(request).unwrap(), json!({ "millis": 2000, "secs": 0 }));
    }

    #[test]
    fn debug_s() {
        let seconds = SecondsSinceUnixEpoch(uint!(0));
        assert_eq!(format!("{seconds:?}"), "1970-01-01T00:00:00");
    }

    #[test]
    fn debug_ms() {
        let seconds = MilliSecondsSinceUnixEpoch(uint!(0));
        assert_eq!(format!("{seconds:?}"), "1970-01-01T00:00:00.000");
    }
}
