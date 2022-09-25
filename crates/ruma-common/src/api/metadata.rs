use std::{
    fmt::{self, Display},
    str::FromStr,
};

use http::Method;

use super::{
    error::{IncorrectArgumentCount, UnknownVersionError},
    AuthScheme,
};
use crate::RoomVersionId;

/// Metadata about an API endpoint.
#[derive(Clone, Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct Metadata {
    /// A human-readable description of the endpoint.
    pub description: &'static str,

    /// The HTTP method used by this endpoint.
    pub method: Method,

    /// A unique identifier for this endpoint.
    pub name: &'static str,

    /// Whether or not this endpoint is rate limited by the server.
    pub rate_limited: bool,

    /// What authentication scheme the server uses for this endpoint.
    pub authentication: AuthScheme,

    /// All info pertaining to an endpoint's (historic) paths, deprecation version, and removal.
    pub history: VersionHistory,
}

impl Metadata {
    /// Will decide how a particular set of matrix versions sees an endpoint.
    ///
    /// It will pick `Stable` over `R0` and `Unstable`. It'll return `Deprecated` or `Removed` only
    /// if all versions denote it.
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
        if self.history.removed.map(greater_or_equal_all).unwrap_or(false) {
            return VersioningDecision::Removed;
        }

        // Check if *any* version marks this endpoint as stable.
        if self.history.added_version().map(greater_or_equal_any).unwrap_or(false) {
            let all_deprecated = self.history.deprecated.map(greater_or_equal_all).unwrap_or(false);

            return VersioningDecision::Stable {
                any_deprecated: all_deprecated
                    || self.history.deprecated.map(greater_or_equal_any).unwrap_or(false),
                all_deprecated,
                any_removed: self.history.removed.map(greater_or_equal_any).unwrap_or(false),
            };
        }

        VersioningDecision::Unstable
    }
}

/// Data for a single endpoint path variant, together with convenience methods.
#[derive(Clone, Copy, Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct PathData {
    /// The "Canonical" path, formatted with axum-ready :-prefixed path argument names.
    pub canon: &'static str,

    /// The parts of the path, meant for positional arguments to be interleaved with.
    ///
    /// For example, a `"/hello/:there"` path would produce a `&["/hello/", ""]` parts slice.
    pub parts: &'static [&'static str],
}

impl PathData {
    /// The amount of arguments this path variant expects to receive.
    pub const fn arg_count(&self) -> usize {
        self.parts.len() - 1
    }

    /// Formatting this path with a slice of positional arguments.
    ///
    /// Note: An incorrect amount of arguments will result in an error.
    pub fn format(&self, args: &'_ [&dyn Display]) -> Result<String, IncorrectArgumentCount> {
        if self.arg_count() != args.len() {
            return Err(IncorrectArgumentCount { expected: self.arg_count(), got: args.len() });
        };

        let mut f_str = String::new();

        for (i, arg) in args.iter().enumerate() {
            // This is checked through comparing arg count above,
            // self.parts is always one element more than args.
            f_str.push_str(self.parts[i]);

            f_str.push_str(&arg.to_string());
        }

        f_str.push_str(self.parts.last().expect("path parts has at least 1 element"));

        Ok(f_str)
    }

    /// Return the "canonical" form of this endpoint as a static string.
    ///
    /// The canonical form is axum-ready, with the path arguments :-prefixed.
    pub fn as_str(&self) -> &'static str {
        self.canon
    }
}

#[derive(Clone, Copy)]
pub struct PathParts();

impl PathParts {}

// type EndpointPath = &'static str;

/// The complete history of this endpoint as far ruma knows, together with all variants on versions
/// stable and unstable.
///
/// The amount and positioning of path variables are the same over all path variants.
#[derive(Clone, Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct VersionHistory {
    /// A list of unstable paths over this endpoint's history.
    ///
    /// For endpoint querying purposes, the last item will be used.
    pub unstable_paths: &'static [PathData],

    /// A list of path versions, mapped to matrix versions.
    ///
    /// Sorted (ascending) by matrix version, will not mix major versions.
    pub path_versions: &'static [(MatrixVersion, PathData)],

    /// The matrix version that deprecated this endpoint.
    ///
    /// Deprecation often precedes one matrix version before removal.
    ///
    /// This will make [`try_into_http_request`](super::OutgoingRequest::try_into_http_request)
    /// emit a warning, see the corresponding documentation for more information.
    pub deprecated: Option<MatrixVersion>,

    /// The matrix version that removed this endpoint.
    ///
    /// This will make [`try_into_http_request`](super::OutgoingRequest::try_into_http_request)
    /// emit an error, see the corresponding documentation for more information.
    pub removed: Option<MatrixVersion>,
}

impl VersionHistory {
    /// Will return the *first* version this path was added in.
    ///
    /// Is None when this endpoint is unstable/unreleased.
    pub fn added_version(&self) -> Option<MatrixVersion> {
        self.path_versions.first().map(|(x, _)| *x)
    }

    /// Picks the last unstable path, if it exists.
    pub fn unstable(&self) -> Option<PathData> {
        self.unstable_paths.last().copied()
    }

    /// Enumerates all raw paths, for use in server URL routers.
    pub fn all_raw(&self) -> Vec<&'static str> {
        let mut v = Vec::new();

        v.extend(self.unstable_paths.iter().map(|p| p.as_str()));

        v.extend(self.path_versions.iter().map(|(_, y)| y.as_str()));

        v
    }

    /// Will decide how a particular set of matrix versions sees an endpoint.
    ///
    /// It will pick `Removed` over `Stable`, and `Stable` over `Unstable`.
    ///
    /// In other words, if in any version it tells it supports the endpoint in a stable fashion,
    /// this will return `Stable`, even if *some* versions in this set will denote deprecation or
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
        if self.removed.map(greater_or_equal_all).unwrap_or(false) {
            return VersioningDecision::Removed;
        }

        // Check if *any* version marks this endpoint as stable.
        if self.added_version().map(greater_or_equal_any).unwrap_or(false) {
            let all_deprecated = self.deprecated.map(greater_or_equal_all).unwrap_or(false);

            return VersioningDecision::Stable {
                any_deprecated: all_deprecated
                    || self.deprecated.map(greater_or_equal_any).unwrap_or(false),
                all_deprecated,
                any_removed: self.removed.map(greater_or_equal_any).unwrap_or(false),
            };
        }

        VersioningDecision::Unstable
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
    pub fn stable_endpoint_for(&self, versions: &[MatrixVersion]) -> Option<PathData> {
        // Go reverse, to check the "latest" version first.
        for (ver, path) in self.path_versions.iter().rev() {
            // Check if any of the versions are equal or greater than the version the path needs.
            if versions.iter().any(|v| v.is_superset_of(*ver)) {
                return Some(*path);
            }
        }

        None
    }
}

/// A versioning "decision" derived from a set of matrix versions.
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
/// Matrix has a deprecation policy, read more about it here: <https://spec.matrix.org/v1.2/#deprecation-policy>.
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
    /// Retroactively defined as <https://spec.matrix.org/v1.2/#legacy-versioning>.
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
}

impl TryFrom<&str> for MatrixVersion {
    type Error = UnknownVersionError;

    fn try_from(value: &str) -> Result<MatrixVersion, Self::Error> {
        use MatrixVersion::*;

        Ok(match value {
            // FIXME: these are likely not entirely correct; https://github.com/ruma/ruma/issues/852
            "v1.0" |
            // Additional definitions according to https://spec.matrix.org/v1.2/#legacy-versioning
            "r0.5.0" | "r0.6.0" | "r0.6.1" => V1_0,
            "v1.1" => V1_1,
            "v1.2" => V1_2,
            "v1.3" => V1_3,
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
    pub fn into_parts(self) -> (u8, u8) {
        match self {
            MatrixVersion::V1_0 => (1, 0),
            MatrixVersion::V1_1 => (1, 1),
            MatrixVersion::V1_2 => (1, 2),
            MatrixVersion::V1_3 => (1, 3),
        }
    }

    /// Try to turn a pair of (major, minor) version components back into a `MatrixVersion`.
    pub fn from_parts(major: u8, minor: u8) -> Result<Self, UnknownVersionError> {
        match (major, minor) {
            (1, 0) => Ok(MatrixVersion::V1_0),
            (1, 1) => Ok(MatrixVersion::V1_1),
            (1, 2) => Ok(MatrixVersion::V1_2),
            (1, 3) => Ok(MatrixVersion::V1_3),
            _ => Err(UnknownVersionError),
        }
    }

    /// Get the default [`RoomVersionId`] for this `MatrixVersion`.
    pub fn default_room_version(&self) -> RoomVersionId {
        match self {
            // <https://matrix.org/docs/spec/index.html#complete-list-of-room-versions>
            MatrixVersion::V1_0
            // <https://spec.matrix.org/v1.1/rooms/#complete-list-of-room-versions>
            | MatrixVersion::V1_1
            // <https://spec.matrix.org/v1.2/rooms/#complete-list-of-room-versions>
            | MatrixVersion::V1_2 => RoomVersionId::V6,
            // <https://spec.matrix.org/v1.3/rooms/#complete-list-of-room-versions>
            MatrixVersion::V1_3 => RoomVersionId::V9,
        }
    }
}

impl Display for MatrixVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (major, minor) = self.into_parts();
        f.write_str(&format!("v{major}.{minor}"))
    }
}
