//! `GET /_matrix/client/*/retention/configuration`
//!
//! Get the configuration for the message retention policy.

pub mod unstable {
    //! `msc1763` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/1763

    use std::{collections::BTreeMap, time::Duration};

    use js_int::UInt;
    use ruma_common::{
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };
    use ruma_events::room::retention::RoomRetentionEventContent;
    use serde::Deserialize;

    use crate::retention::RoomIdOrAllRooms;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc1763/retention/configuration",
        }
    }

    /// Request type for the `GET` `retention/configuration` endpoint.
    #[request]
    pub struct Request {}

    /// Response type for the `GET` `retention/configuration` endpoint.
    #[response]
    pub struct Response {
        /// Map between a Room ID and their respective room retention policy.
        pub policies: BTreeMap<RoomIdOrAllRooms, RoomRetentionEventContent>,
        /// Map between a Room ID and their respective room retention policy.
        pub limits: BTreeMap<RoomIdOrAllRooms, RoomRetentionEventContent>,
    }

    /// Limits to apply to policies defined by `m.room.retention` state events.
    #[derive(Debug, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct RetentionLimits {
        /// Limits to apply to the maximum lifetime of `m.room.retention` limits.
        pub max_lifetime: Option<LifetimeLimits>,
        /// Limits to apply to the minimum lifetime of `m.room.retention` limits.
        pub min_lifetime: Option<LifetimeLimits>,
    }

    impl RetentionLimits {
        /// Create a new [`RetentionLimits`] object with the given maximum and minimum limits.
        pub fn new(
            min_lifetime: Option<LifetimeLimits>,
            max_lifetime: Option<LifetimeLimits>,
        ) -> Self {
            Self { min_lifetime, max_lifetime }
        }
    }

    /// Global limits for the per-room retention policy lifetimes.
    #[derive(Debug, Deserialize)]
    pub struct LifetimeLimits {
        min: Option<UInt>,
        max: Option<UInt>,
    }

    impl LifetimeLimits {
        /// Create a new [`LifetimeLimits`] object with the given maximum and minimum limits.
        ///
        /// This will return `None` if the duration of one of the limites, expressed as
        /// miliseconnds, doesn't fall into the [0, (2^53)-1] range.
        pub fn new(min: Option<Duration>, max: Option<Duration>) -> Option<Self> {
            let max = max.map(|l| UInt::try_from(l.as_millis())).transpose().ok()?;
            let min = min.map(|l| UInt::try_from(l.as_millis())).transpose().ok()?;

            Some(Self { min, max })
        }

        /// Get the minimum accepted value of this limit.
        pub fn min(&self) -> Option<Duration> {
            self.min.map(|l| Duration::from_millis(l.into()))
        }

        /// Get the maximum accepted value of this limit.
        pub fn max(&self) -> Option<Duration> {
            self.max.map(|l| Duration::from_millis(l.into()))
        }
    }
}
