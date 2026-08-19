//! `GET /_matrix/client/*/retention/configuration`
//!
//! Get the configuration for the message retention policy.

pub mod unstable {
    //! `msc1763` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/1763

    use std::{collections::BTreeMap, ops::RangeBounds, time::Duration};

    use js_int::UInt;
    use ruma_common::{
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };
    use ruma_events::room::retention::{RoomRetentionEventContent, is_valid_lifetime_combination};
    use serde::{Deserialize, Serialize};

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
    #[derive(Default)]
    pub struct Request {}

    impl Request {
        /// Creates an empty `Request`.
        pub fn new() -> Self {
            Self {}
        }
    }

    /// Response type for the `GET` `retention/configuration` endpoint.
    #[response]
    pub struct Response {
        /// Map between a Room ID and their respective room retention policy.
        pub policies: BTreeMap<RoomIdOrAllRooms, RoomRetentionEventContent>,

        /// Limits to apply to policies defined by m.room.retention state events.
        pub limits: RetentionLimits,
    }

    impl Response {
        /// Creates a new `Response` with the given policies and limits.
        pub fn new(
            policies: BTreeMap<RoomIdOrAllRooms, RoomRetentionEventContent>,
            limits: RetentionLimits,
        ) -> Self {
            Self { policies, limits }
        }
    }

    /// Struct describing limits to apply to policies defined by `m.room.retention` state events.
    #[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct RetentionLimits {
        /// Limits to apply to the maximum lifetime of `m.room.retention` limits.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub max_lifetime: Option<LifetimeLimits>,

        /// Limits to apply to the minimum lifetime of `m.room.retention` limits.
        #[serde(skip_serializing_if = "Option::is_none")]
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
    #[derive(Clone, Copy, Debug, Default, Serialize)]
    pub struct LifetimeLimits {
        /// The minimum accepted value for this limit.
        min: Option<UInt>,

        /// The maximum accepted value for this limit.
        max: Option<UInt>,
    }

    impl LifetimeLimits {
        /// Create a new [`LifetimeLimits`] object with no limits set.
        ///
        /// This method can be combined with the [`LifetimeLimits::at_least`] and
        /// [`LifetimeLimits::at_most`] methods to configure the individual limits.
        ///
        /// # Examples
        ///
        /// ```
        /// # use std::time::Duration;
        /// # use ruma_client_api::retention::get_retention_configuration::unstable::LifetimeLimits;
        /// # fn doctest() -> Option<()> {
        /// let content = LifetimeLimits::new()
        ///     .at_least(Duration::from_hours(24))?
        ///     .at_most(Duration::from_hours(24 * 10))?;
        /// # None
        /// # }
        /// ```
        pub fn new() -> Self {
            Self::default()
        }

        /// Create a new [`LifetimeLimits`] object with the given maximum and minimum limits.
        ///
        /// This will return `None` if the duration of one of the limits, expressed as
        /// milliseconds, doesn't fall into the [0, (2^53)-1] range.
        fn new_impl(min: Option<Duration>, max: Option<Duration>) -> Option<Self> {
            let max = max.map(|l| UInt::try_from(l.as_millis())).transpose().ok()?;
            let min = min.map(|l| UInt::try_from(l.as_millis())).transpose().ok()?;

            if is_valid_lifetime_combination(min, max) { Some(Self { min, max }) } else { None }
        }

        /// Create a new [`LifetimeLimits`] object from a range.
        ///
        /// Returns `None` if the duration of one of the limits, expressed as milliseconds, doesn't
        /// fall into the [0, (2^53)-1] range, or if the lower bound of the range is bigger than the
        /// upper bound, i.e. `10..0`.
        ///
        /// # Examples
        ///
        /// ```
        /// # use std::time::Duration;
        /// # use ruma_client_api::retention::get_retention_configuration::unstable::LifetimeLimits;
        /// # fn doctest() -> Option<()> {
        /// let content =
        ///     LifetimeLimits::from_range(Duration::from_hours(24)..Duration::from_hours(24 * 10))?;
        /// # None
        /// # }
        /// ```
        pub fn from_range(lifetime_range: impl RangeBounds<Duration>) -> Option<Self> {
            let min_lifetime = match lifetime_range.start_bound() {
                std::ops::Bound::Included(v) => Some(*v),
                std::ops::Bound::Excluded(v) => Some(v.saturating_add(Duration::from_millis(1))),
                std::ops::Bound::Unbounded => None,
            };

            let max_lifetime = match lifetime_range.end_bound() {
                std::ops::Bound::Included(v) => Some(*v),
                std::ops::Bound::Excluded(v) => Some(v.saturating_sub(Duration::from_millis(1))),
                std::ops::Bound::Unbounded => None,
            };

            Self::new_impl(min_lifetime, max_lifetime)
        }

        /// Sets the maximum value that a retention policy limit is allowed to have.
        ///
        /// Returns `None` if the given limit, expressed as milliseconds, doesn't fall into the [0,
        /// (2^53)-1] range, or if the limits don't adhere to the `max` < `min` constraint.
        pub fn at_most(self, max: Duration) -> Option<Self> {
            let min = self.min();
            Self::new_impl(min, Some(max))
        }

        /// Sets the minimum value that a retention policy limit is allowed to have.
        ///
        /// Returns `None` if the given limit, expressed as milliseconds, doesn't fall into the [0,
        /// (2^53)-1] range, or if the limits don't adhere to the `max` < `min` constraint.
        pub fn at_least(self, min: Duration) -> Option<Self> {
            let max = self.max();
            Self::new_impl(Some(min), max)
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

    impl<'de> Deserialize<'de> for LifetimeLimits {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize)]
            struct Helper {
                min: Option<UInt>,
                max: Option<UInt>,
            }

            let Helper { min, max } = Helper::deserialize(deserializer)?;

            if is_valid_lifetime_combination(min, max) {
                Ok(Self { min, max })
            } else {
                Err(serde::de::Error::custom(
                "Invalid lifetime limits, the max limit must always be higher or equal to the min limit."
                    .to_owned(),
            ))
            }
        }
    }
}
