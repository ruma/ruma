use std::{
    cmp::Ordering,
    fmt::{self, Display, Write},
    str::FromStr,
};

use bytes::BufMut;
use http::{
    header::{self, HeaderName, HeaderValue},
    Method,
};
use percent_encoding::utf8_percent_encode;
use tracing::warn;

use super::{
    error::{IntoHttpError, UnknownVersionError},
    AuthScheme, SendAccessToken,
};
use crate::{percent_encode::PATH_PERCENT_ENCODE_SET, serde::slice_to_buf, RoomVersionId};

/// Metadata about an API endpoint.
#[derive(Clone, Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct Metadata {
    /// The HTTP method used by this endpoint.
    pub method: Method,

    /// Whether or not this endpoint is rate limited by the server.
    pub rate_limited: bool,

    /// What authentication scheme the server uses for this endpoint.
    pub authentication: AuthScheme,

    /// All info pertaining to an endpoint's (historic) paths, deprecation version, and removal.
    pub history: VersionHistory,
}

impl Metadata {
    /// Returns an empty request body for this Matrix request.
    ///
    /// For `GET` requests, it returns an entirely empty buffer, for others it returns an empty JSON
    /// object (`{}`).
    pub fn empty_request_body<B>(&self) -> B
    where
        B: Default + BufMut,
    {
        if self.method == Method::GET {
            Default::default()
        } else {
            slice_to_buf(b"{}")
        }
    }

    /// Transform the `SendAccessToken` into an access token if the endpoint requires it, or if it
    /// is `SendAccessToken::Force`.
    ///
    /// Fails if the endpoint requires an access token but the parameter is `SendAccessToken::None`,
    /// or if the access token can't be converted to a [`HeaderValue`].
    pub fn authorization_header(
        &self,
        access_token: SendAccessToken<'_>,
    ) -> Result<Option<(HeaderName, HeaderValue)>, IntoHttpError> {
        Ok(match self.authentication {
            AuthScheme::None => match access_token.get_not_required_for_endpoint() {
                Some(token) => Some((header::AUTHORIZATION, format!("Bearer {token}").try_into()?)),
                None => None,
            },

            AuthScheme::AccessToken => {
                let token = access_token
                    .get_required_for_endpoint()
                    .ok_or(IntoHttpError::NeedsAuthentication)?;

                Some((header::AUTHORIZATION, format!("Bearer {token}").try_into()?))
            }

            AuthScheme::AccessTokenOptional => match access_token.get_required_for_endpoint() {
                Some(token) => Some((header::AUTHORIZATION, format!("Bearer {token}").try_into()?)),
                None => None,
            },

            AuthScheme::AppserviceToken => match access_token.get_required_for_appservice() {
                Some(token) => Some((header::AUTHORIZATION, format!("Bearer {token}").try_into()?)),
                None => None,
            },

            AuthScheme::ServerSignatures => None,
        })
    }

    /// Generate the endpoint URL for this endpoint.
    pub fn make_endpoint_url(
        &self,
        versions: &[MatrixVersion],
        base_url: &str,
        path_args: &[&dyn Display],
        query_string: &str,
    ) -> Result<String, IntoHttpError> {
        let path_with_placeholders = self.history.select_path(versions)?;

        let mut res = base_url.strip_suffix('/').unwrap_or(base_url).to_owned();
        let mut segments = path_with_placeholders.split('/');
        let mut path_args = path_args.iter();

        let first_segment = segments.next().expect("split iterator is never empty");
        assert!(first_segment.is_empty(), "endpoint paths must start with '/'");

        for segment in segments {
            if segment.starts_with(':') {
                let arg = path_args
                    .next()
                    .expect("number of placeholders must match number of arguments")
                    .to_string();
                let arg = utf8_percent_encode(&arg, PATH_PERCENT_ENCODE_SET);

                write!(res, "/{arg}").expect("writing to a String using fmt::Write can't fail");
            } else {
                res.reserve(segment.len() + 1);
                res.push('/');
                res.push_str(segment);
            }
        }

        if !query_string.is_empty() {
            res.push('?');
            res.push_str(query_string);
        }

        Ok(res)
    }

    // Used for generated `#[test]`s
    #[doc(hidden)]
    pub fn _path_parameters(&self) -> Vec<&'static str> {
        let path = self.history.all_paths().next().unwrap();
        path.split('/').filter_map(|segment| segment.strip_prefix(':')).collect()
    }
}

/// The complete history of this endpoint as far as Ruma knows, together with all variants on
/// versions stable and unstable.
///
/// The amount and positioning of path variables are the same over all path variants.
#[derive(Clone, Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct VersionHistory {
    /// A list of unstable paths over this endpoint's history.
    ///
    /// For endpoint querying purposes, the last item will be used.
    unstable_paths: &'static [&'static str],

    /// A list of path versions, mapped to Matrix versions.
    ///
    /// Sorted (ascending) by Matrix version, will not mix major versions.
    stable_paths: &'static [(MatrixVersion, &'static str)],

    /// The Matrix version that deprecated this endpoint.
    ///
    /// Deprecation often precedes one Matrix version before removal.
    ///
    /// This will make [`try_into_http_request`](super::OutgoingRequest::try_into_http_request)
    /// emit a warning, see the corresponding documentation for more information.
    deprecated: Option<MatrixVersion>,

    /// The Matrix version that removed this endpoint.
    ///
    /// This will make [`try_into_http_request`](super::OutgoingRequest::try_into_http_request)
    /// emit an error, see the corresponding documentation for more information.
    removed: Option<MatrixVersion>,
}

impl VersionHistory {
    /// Constructs an instance of [`VersionHistory`], erroring on compilation if it does not pass
    /// invariants.
    ///
    /// Specifically, this checks the following invariants:
    /// - Path Arguments are equal (in order, amount, and argument name) in all path strings
    /// - In stable_paths:
    ///   - matrix versions are in ascending order
    ///   - no matrix version is referenced twice
    /// - deprecated's version comes after the latest version mentioned in stable_paths, except for
    ///   version 1.0, and only if any stable path is defined
    /// - removed comes after deprecated, or after the latest referenced stable_paths, like
    ///   deprecated
    pub const fn new(
        unstable_paths: &'static [&'static str],
        stable_paths: &'static [(MatrixVersion, &'static str)],
        deprecated: Option<MatrixVersion>,
        removed: Option<MatrixVersion>,
    ) -> Self {
        use konst::{iter, slice, string};

        const fn check_path_is_valid(path: &'static str) {
            iter::for_each!(path_b in slice::iter(path.as_bytes()) => {
                match *path_b {
                    0x21..=0x7E => {},
                    _ => panic!("path contains invalid (non-ascii or whitespace) characters")
                }
            });
        }

        const fn check_path_args_equal(first: &'static str, second: &'static str) {
            let mut second_iter = string::split(second, "/").next();

            iter::for_each!(first_s in string::split(first, "/") => {
                if let Some(first_arg) = string::strip_prefix(first_s, ":") {
                    let second_next_arg: Option<&'static str> = loop {
                        let (second_s, second_n_iter) = match second_iter {
                            Some(tuple) => tuple,
                            None => break None,
                        };

                        let maybe_second_arg = string::strip_prefix(second_s, ":");

                        second_iter = second_n_iter.next();

                        if let Some(second_arg) = maybe_second_arg {
                            break Some(second_arg);
                        }
                    };

                    if let Some(second_next_arg) = second_next_arg {
                        if !string::eq_str(second_next_arg, first_arg) {
                            panic!("Path Arguments do not match");
                        }
                    } else {
                        panic!("Amount of Path Arguments do not match");
                    }
                }
            });

            // If second iterator still has some values, empty first.
            while let Some((second_s, second_n_iter)) = second_iter {
                if string::starts_with(second_s, ":") {
                    panic!("Amount of Path Arguments do not match");
                }
                second_iter = second_n_iter.next();
            }
        }

        // The path we're going to use to compare all other paths with
        let ref_path: &str = if let Some(s) = unstable_paths.first() {
            s
        } else if let Some((_, s)) = stable_paths.first() {
            s
        } else {
            panic!("No paths supplied")
        };

        iter::for_each!(unstable_path in slice::iter(unstable_paths) => {
            check_path_is_valid(unstable_path);
            check_path_args_equal(ref_path, unstable_path);
        });

        let mut prev_seen_version: Option<MatrixVersion> = None;

        iter::for_each!(stable_path in slice::iter(stable_paths) => {
            check_path_is_valid(stable_path.1);
            check_path_args_equal(ref_path, stable_path.1);

            let current_version = stable_path.0;

            if let Some(prev_seen_version) = prev_seen_version {
                let cmp_result = current_version.const_ord(&prev_seen_version);

                if cmp_result.is_eq() {
                    // Found a duplicate, current == previous
                    panic!("Duplicate matrix version in stable_paths")
                } else if cmp_result.is_lt() {
                    // Found an older version, current < previous
                    panic!("No ascending order in stable_paths")
                }
            }

            prev_seen_version = Some(current_version);
        });

        if let Some(deprecated) = deprecated {
            if let Some(prev_seen_version) = prev_seen_version {
                let ord_result = prev_seen_version.const_ord(&deprecated);
                if !deprecated.is_legacy() && ord_result.is_eq() {
                    // prev_seen_version == deprecated, except for 1.0.
                    // It is possible that an endpoint was both made stable and deprecated in the
                    // legacy versions.
                    panic!("deprecated version is equal to latest stable path version")
                } else if ord_result.is_gt() {
                    // prev_seen_version > deprecated
                    panic!("deprecated version is older than latest stable path version")
                }
            } else {
                panic!("Defined deprecated version while no stable path exists")
            }
        }

        if let Some(removed) = removed {
            if let Some(deprecated) = deprecated {
                let ord_result = deprecated.const_ord(&removed);
                if ord_result.is_eq() {
                    // deprecated == removed
                    panic!("removed version is equal to deprecated version")
                } else if ord_result.is_gt() {
                    // deprecated > removed
                    panic!("removed version is older than deprecated version")
                }
            } else {
                panic!("Defined removed version while no deprecated version exists")
            }
        }

        VersionHistory { unstable_paths, stable_paths, deprecated, removed }
    }

    // This function helps picks the right path (or an error) from a set of Matrix versions.
    fn select_path(&self, versions: &[MatrixVersion]) -> Result<&'static str, IntoHttpError> {
        match self.versioning_decision_for(versions) {
            VersioningDecision::Removed => Err(IntoHttpError::EndpointRemoved(
                self.removed.expect("VersioningDecision::Removed implies metadata.removed"),
            )),
            VersioningDecision::Stable { any_deprecated, all_deprecated, any_removed } => {
                if any_removed {
                    if all_deprecated {
                        warn!(
                            "endpoint is removed in some (and deprecated in ALL) \
                             of the following versions: {versions:?}",
                        );
                    } else if any_deprecated {
                        warn!(
                            "endpoint is removed (and deprecated) in some of the \
                             following versions: {versions:?}",
                        );
                    } else {
                        unreachable!("any_removed implies *_deprecated");
                    }
                } else if all_deprecated {
                    warn!(
                        "endpoint is deprecated in ALL of the following versions: \
                         {versions:?}",
                    );
                } else if any_deprecated {
                    warn!(
                        "endpoint is deprecated in some of the following versions: \
                         {versions:?}",
                    );
                }

                Ok(self
                    .stable_endpoint_for(versions)
                    .expect("VersioningDecision::Stable implies that a stable path exists"))
            }
            VersioningDecision::Unstable => self.unstable().ok_or(IntoHttpError::NoUnstablePath),
        }
    }

    /// Will decide how a particular set of Matrix versions sees an endpoint.
    ///
    /// It will only return `Deprecated` or `Removed` if all versions denote it.
    ///
    /// In other words, if in any version it tells it supports the endpoint in a stable fashion,
    /// this will return `Stable`, even if some versions in this set will denote deprecation or
    /// removal.
    ///
    /// If resulting [`VersioningDecision`] is `Stable`, it will also detail if any version denoted
    /// deprecation or removal.
    pub fn versioning_decision_for(&self, versions: &[MatrixVersion]) -> VersioningDecision {
        let greater_or_equal_any =
            |version: MatrixVersion| versions.iter().any(|v| v.is_superset_of(version));
        let greater_or_equal_all =
            |version: MatrixVersion| versions.iter().all(|v| v.is_superset_of(version));

        // Check if all versions removed this endpoint.
        if self.removed.is_some_and(greater_or_equal_all) {
            return VersioningDecision::Removed;
        }

        // Check if *any* version marks this endpoint as stable.
        if self.added_in().is_some_and(greater_or_equal_any) {
            let all_deprecated = self.deprecated.is_some_and(greater_or_equal_all);

            return VersioningDecision::Stable {
                any_deprecated: all_deprecated || self.deprecated.is_some_and(greater_or_equal_any),
                all_deprecated,
                any_removed: self.removed.is_some_and(greater_or_equal_any),
            };
        }

        VersioningDecision::Unstable
    }

    /// Returns the *first* version this endpoint was added in.
    ///
    /// Is `None` when this endpoint is unstable/unreleased.
    pub fn added_in(&self) -> Option<MatrixVersion> {
        self.stable_paths.first().map(|(v, _)| *v)
    }

    /// Returns the Matrix version that deprecated this endpoint, if any.
    pub fn deprecated_in(&self) -> Option<MatrixVersion> {
        self.deprecated
    }

    /// Returns the Matrix version that removed this endpoint, if any.
    pub fn removed_in(&self) -> Option<MatrixVersion> {
        self.removed
    }

    /// Picks the last unstable path, if it exists.
    pub fn unstable(&self) -> Option<&'static str> {
        self.unstable_paths.last().copied()
    }

    /// Returns all path variants in canon form, for use in server routers.
    pub fn all_paths(&self) -> impl Iterator<Item = &'static str> {
        self.unstable_paths().chain(self.stable_paths().map(|(_, path)| path))
    }

    /// Returns all unstable path variants in canon form.
    pub fn unstable_paths(&self) -> impl Iterator<Item = &'static str> {
        self.unstable_paths.iter().copied()
    }

    /// Returns all stable path variants in canon form, with corresponding Matrix version.
    pub fn stable_paths(&self) -> impl Iterator<Item = (MatrixVersion, &'static str)> {
        self.stable_paths.iter().map(|(version, data)| (*version, *data))
    }

    /// The path that should be used to query the endpoint, given a series of versions.
    ///
    /// This will pick the latest path that the version accepts.
    ///
    /// This will return an endpoint in the following format;
    /// - `/_matrix/client/versions`
    /// - `/_matrix/client/hello/:world` (`:world` is a path replacement parameter)
    ///
    /// Note: This will not keep in mind endpoint removals, check with
    /// [`versioning_decision_for`](VersionHistory::versioning_decision_for) to see if this endpoint
    /// is still available.
    pub fn stable_endpoint_for(&self, versions: &[MatrixVersion]) -> Option<&'static str> {
        // Go reverse, to check the "latest" version first.
        for (ver, path) in self.stable_paths.iter().rev() {
            // Check if any of the versions are equal or greater than the version the path needs.
            if versions.iter().any(|v| v.is_superset_of(*ver)) {
                return Some(path);
            }
        }

        None
    }
}

/// A versioning "decision" derived from a set of Matrix versions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(clippy::exhaustive_enums)]
pub enum VersioningDecision {
    /// The unstable endpoint should be used.
    Unstable,

    /// The stable endpoint should be used.
    Stable {
        /// If any version denoted deprecation.
        any_deprecated: bool,

        /// If *all* versions denoted deprecation.
        all_deprecated: bool,

        /// If any version denoted removal.
        any_removed: bool,
    },

    /// This endpoint was removed in all versions, it should not be used.
    Removed,
}

/// The Matrix versions Ruma currently understands to exist.
///
/// Matrix, since fall 2021, has a quarterly release schedule, using a global `vX.Y` versioning
/// scheme.
///
/// Every new minor version denotes stable support for endpoints in a *relatively*
/// backwards-compatible manner.
///
/// Matrix has a deprecation policy, read more about it here: <https://spec.matrix.org/latest/#deprecation-policy>.
///
/// Ruma keeps track of when endpoints are added, deprecated, and removed. It'll automatically
/// select the right endpoint stability variation to use depending on which Matrix versions you
/// pass to [`try_into_http_request`](super::OutgoingRequest::try_into_http_request), see its
/// respective documentation for more information.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum MatrixVersion {
    /// Version 1.0 of the Matrix specification.
    ///
    /// Retroactively defined as <https://spec.matrix.org/latest/#legacy-versioning>.
    V1_0,

    /// Version 1.1 of the Matrix specification, released in Q4 2021.
    ///
    /// See <https://spec.matrix.org/v1.1/>.
    V1_1,

    /// Version 1.2 of the Matrix specification, released in Q1 2022.
    ///
    /// See <https://spec.matrix.org/v1.2/>.
    V1_2,

    /// Version 1.3 of the Matrix specification, released in Q2 2022.
    ///
    /// See <https://spec.matrix.org/v1.3/>.
    V1_3,

    /// Version 1.4 of the Matrix specification, released in Q3 2022.
    ///
    /// See <https://spec.matrix.org/v1.4/>.
    V1_4,

    /// Version 1.5 of the Matrix specification, released in Q4 2022.
    ///
    /// See <https://spec.matrix.org/v1.5/>.
    V1_5,

    /// Version 1.6 of the Matrix specification, released in Q1 2023.
    ///
    /// See <https://spec.matrix.org/v1.6/>.
    V1_6,

    /// Version 1.7 of the Matrix specification, released in Q2 2023.
    ///
    /// See <https://spec.matrix.org/v1.7/>.
    V1_7,

    /// Version 1.8 of the Matrix specification, released in Q3 2023.
    ///
    /// See <https://spec.matrix.org/v1.8/>.
    V1_8,

    /// Version 1.9 of the Matrix specification, released in Q4 2023.
    ///
    /// See <https://spec.matrix.org/v1.9/>.
    V1_9,

    /// Version 1.10 of the Matrix specification, released in Q1 2024.
    ///
    /// See <https://spec.matrix.org/v1.10/>.
    V1_10,
}

impl TryFrom<&str> for MatrixVersion {
    type Error = UnknownVersionError;

    fn try_from(value: &str) -> Result<MatrixVersion, Self::Error> {
        use MatrixVersion::*;

        Ok(match value {
            // FIXME: these are likely not entirely correct; https://github.com/ruma/ruma/issues/852
            "v1.0" |
            // Additional definitions according to https://spec.matrix.org/latest/#legacy-versioning
            "r0.5.0" | "r0.6.0" | "r0.6.1" => V1_0,
            "v1.1" => V1_1,
            "v1.2" => V1_2,
            "v1.3" => V1_3,
            "v1.4" => V1_4,
            "v1.5" => V1_5,
            "v1.6" => V1_6,
            "v1.7" => V1_7,
            "v1.8" => V1_8,
            "v1.9" => V1_9,
            "v1.10" => V1_10,
            _ => return Err(UnknownVersionError),
        })
    }
}

impl FromStr for MatrixVersion {
    type Err = UnknownVersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl MatrixVersion {
    /// Checks whether a version is compatible with another.
    ///
    /// A is compatible with B as long as B is equal or less, so long as A and B have the same
    /// major versions.
    ///
    /// For example, v1.2 is compatible with v1.1, as it is likely only some additions of
    /// endpoints on top of v1.1, but v1.1 would not be compatible with v1.2, as v1.1
    /// cannot represent all of v1.2, in a manner similar to set theory.
    ///
    /// Warning: Matrix has a deprecation policy, and Matrix versioning is not as
    /// straight-forward as this function makes it out to be. This function only exists
    /// to prune major version differences, and versions too new for `self`.
    ///
    /// This (considering if major versions are the same) is equivalent to a `self >= other`
    /// check.
    pub fn is_superset_of(self, other: Self) -> bool {
        let (major_l, minor_l) = self.into_parts();
        let (major_r, minor_r) = other.into_parts();
        major_l == major_r && minor_l >= minor_r
    }

    /// Decompose the Matrix version into its major and minor number.
    pub const fn into_parts(self) -> (u8, u8) {
        match self {
            MatrixVersion::V1_0 => (1, 0),
            MatrixVersion::V1_1 => (1, 1),
            MatrixVersion::V1_2 => (1, 2),
            MatrixVersion::V1_3 => (1, 3),
            MatrixVersion::V1_4 => (1, 4),
            MatrixVersion::V1_5 => (1, 5),
            MatrixVersion::V1_6 => (1, 6),
            MatrixVersion::V1_7 => (1, 7),
            MatrixVersion::V1_8 => (1, 8),
            MatrixVersion::V1_9 => (1, 9),
            MatrixVersion::V1_10 => (1, 10),
        }
    }

    /// Try to turn a pair of (major, minor) version components back into a `MatrixVersion`.
    pub const fn from_parts(major: u8, minor: u8) -> Result<Self, UnknownVersionError> {
        match (major, minor) {
            (1, 0) => Ok(MatrixVersion::V1_0),
            (1, 1) => Ok(MatrixVersion::V1_1),
            (1, 2) => Ok(MatrixVersion::V1_2),
            (1, 3) => Ok(MatrixVersion::V1_3),
            (1, 4) => Ok(MatrixVersion::V1_4),
            (1, 5) => Ok(MatrixVersion::V1_5),
            (1, 6) => Ok(MatrixVersion::V1_6),
            (1, 7) => Ok(MatrixVersion::V1_7),
            (1, 8) => Ok(MatrixVersion::V1_8),
            (1, 9) => Ok(MatrixVersion::V1_9),
            (1, 10) => Ok(MatrixVersion::V1_10),
            _ => Err(UnknownVersionError),
        }
    }

    /// Constructor for use by the `metadata!` macro.
    ///
    /// Accepts string literals and parses them.
    #[doc(hidden)]
    pub const fn from_lit(lit: &'static str) -> Self {
        use konst::{option, primitive::parse_u8, result, string};

        let major: u8;
        let minor: u8;

        let mut lit_iter = string::split(lit, ".").next();

        {
            let (checked_first, checked_split) = option::unwrap!(lit_iter); // First iteration always succeeds

            major = result::unwrap_or_else!(parse_u8(checked_first), |_| panic!(
                "major version is not a valid number"
            ));

            lit_iter = checked_split.next();
        }

        match lit_iter {
            Some((checked_second, checked_split)) => {
                minor = result::unwrap_or_else!(parse_u8(checked_second), |_| panic!(
                    "minor version is not a valid number"
                ));

                lit_iter = checked_split.next();
            }
            None => panic!("could not find dot to denote second number"),
        }

        if lit_iter.is_some() {
            panic!("version literal contains more than one dot")
        }

        result::unwrap_or_else!(Self::from_parts(major, minor), |_| panic!(
            "not a valid version literal"
        ))
    }

    // Internal function to do ordering in const-fn contexts
    const fn const_ord(&self, other: &Self) -> Ordering {
        let self_parts = self.into_parts();
        let other_parts = other.into_parts();

        use konst::primitive::cmp::cmp_u8;

        let major_ord = cmp_u8(self_parts.0, other_parts.0);
        if major_ord.is_ne() {
            major_ord
        } else {
            cmp_u8(self_parts.1, other_parts.1)
        }
    }

    // Internal function to check if this version is the legacy (v1.0) version in const-fn contexts
    const fn is_legacy(&self) -> bool {
        let self_parts = self.into_parts();

        use konst::primitive::cmp::cmp_u8;

        cmp_u8(self_parts.0, 1).is_eq() && cmp_u8(self_parts.1, 0).is_eq()
    }

    /// Get the default [`RoomVersionId`] for this `MatrixVersion`.
    pub fn default_room_version(&self) -> RoomVersionId {
        match self {
            // <https://spec.matrix.org/historical/index.html#complete-list-of-room-versions>
            MatrixVersion::V1_0
            // <https://spec.matrix.org/v1.1/rooms/#complete-list-of-room-versions>
            | MatrixVersion::V1_1
            // <https://spec.matrix.org/v1.2/rooms/#complete-list-of-room-versions>
            | MatrixVersion::V1_2 => RoomVersionId::V6,
            // <https://spec.matrix.org/v1.3/rooms/#complete-list-of-room-versions>
            MatrixVersion::V1_3
            // <https://spec.matrix.org/v1.4/rooms/#complete-list-of-room-versions>
            | MatrixVersion::V1_4
            // <https://spec.matrix.org/v1.5/rooms/#complete-list-of-room-versions>
            | MatrixVersion::V1_5 => RoomVersionId::V9,
            // <https://spec.matrix.org/v1.6/rooms/#complete-list-of-room-versions>
            MatrixVersion::V1_6
            // <https://spec.matrix.org/v1.7/rooms/#complete-list-of-room-versions>
            | MatrixVersion::V1_7
            // <https://spec.matrix.org/v1.8/rooms/#complete-list-of-room-versions>
            | MatrixVersion::V1_8
            // <https://spec.matrix.org/v1.9/rooms/#complete-list-of-room-versions>
            | MatrixVersion::V1_9
            // <https://spec.matrix.org/v1.10/rooms/#complete-list-of-room-versions>
            | MatrixVersion::V1_10 => RoomVersionId::V10,
        }
    }
}

impl Display for MatrixVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (major, minor) = self.into_parts();
        f.write_str(&format!("v{major}.{minor}"))
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use http::Method;

    use super::{
        AuthScheme,
        MatrixVersion::{self, V1_0, V1_1, V1_2, V1_3},
        Metadata, VersionHistory,
    };
    use crate::api::error::IntoHttpError;

    fn stable_only_metadata(stable_paths: &'static [(MatrixVersion, &'static str)]) -> Metadata {
        Metadata {
            method: Method::GET,
            rate_limited: false,
            authentication: AuthScheme::None,
            history: VersionHistory {
                unstable_paths: &[],
                stable_paths,
                deprecated: None,
                removed: None,
            },
        }
    }

    // TODO add test that can hook into tracing and verify the deprecation warning is emitted

    #[test]
    fn make_simple_endpoint_url() {
        let meta = stable_only_metadata(&[(V1_0, "/s")]);
        let url = meta.make_endpoint_url(&[V1_0], "https://example.org", &[], "").unwrap();
        assert_eq!(url, "https://example.org/s");
    }

    #[test]
    fn make_endpoint_url_with_path_args() {
        let meta = stable_only_metadata(&[(V1_0, "/s/:x")]);
        let url = meta.make_endpoint_url(&[V1_0], "https://example.org", &[&"123"], "").unwrap();
        assert_eq!(url, "https://example.org/s/123");
    }

    #[test]
    fn make_endpoint_url_with_path_args_with_dash() {
        let meta = stable_only_metadata(&[(V1_0, "/s/:x")]);
        let url =
            meta.make_endpoint_url(&[V1_0], "https://example.org", &[&"my-path"], "").unwrap();
        assert_eq!(url, "https://example.org/s/my-path");
    }

    #[test]
    fn make_endpoint_url_with_path_args_with_reserved_char() {
        let meta = stable_only_metadata(&[(V1_0, "/s/:x")]);
        let url = meta.make_endpoint_url(&[V1_0], "https://example.org", &[&"#path"], "").unwrap();
        assert_eq!(url, "https://example.org/s/%23path");
    }

    #[test]
    fn make_endpoint_url_with_query() {
        let meta = stable_only_metadata(&[(V1_0, "/s/")]);
        let url = meta.make_endpoint_url(&[V1_0], "https://example.org", &[], "foo=bar").unwrap();
        assert_eq!(url, "https://example.org/s/?foo=bar");
    }

    #[test]
    #[should_panic]
    fn make_endpoint_url_wrong_num_path_args() {
        let meta = stable_only_metadata(&[(V1_0, "/s/:x")]);
        _ = meta.make_endpoint_url(&[V1_0], "https://example.org", &[], "");
    }

    const EMPTY: VersionHistory =
        VersionHistory { unstable_paths: &[], stable_paths: &[], deprecated: None, removed: None };

    #[test]
    fn select_latest_stable() {
        let hist = VersionHistory { stable_paths: &[(V1_1, "/s")], ..EMPTY };
        assert_matches!(hist.select_path(&[V1_0, V1_1]), Ok("/s"));
    }

    #[test]
    fn select_unstable() {
        let hist = VersionHistory { unstable_paths: &["/u"], ..EMPTY };
        assert_matches!(hist.select_path(&[V1_0]), Ok("/u"));
    }

    #[test]
    fn select_r0() {
        let hist = VersionHistory { stable_paths: &[(V1_0, "/r")], ..EMPTY };
        assert_matches!(hist.select_path(&[V1_0]), Ok("/r"));
    }

    #[test]
    fn select_removed_err() {
        let hist = VersionHistory {
            stable_paths: &[(V1_0, "/r"), (V1_1, "/s")],
            unstable_paths: &["/u"],
            deprecated: Some(V1_2),
            removed: Some(V1_3),
        };
        assert_matches!(hist.select_path(&[V1_3]), Err(IntoHttpError::EndpointRemoved(V1_3)));
    }

    #[test]
    fn partially_removed_but_stable() {
        let hist = VersionHistory {
            stable_paths: &[(V1_0, "/r"), (V1_1, "/s")],
            unstable_paths: &[],
            deprecated: Some(V1_2),
            removed: Some(V1_3),
        };
        assert_matches!(hist.select_path(&[V1_2]), Ok("/s"));
    }

    #[test]
    fn no_unstable() {
        let hist = VersionHistory { stable_paths: &[(V1_1, "/s")], ..EMPTY };
        assert_matches!(hist.select_path(&[V1_0]), Err(IntoHttpError::NoUnstablePath));
    }

    #[test]
    fn version_literal() {
        const LIT: MatrixVersion = MatrixVersion::from_lit("1.0");

        assert_eq!(LIT, V1_0);
    }
}
