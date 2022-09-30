use std::{
    fmt::{self, Display, Write},
    str::FromStr,
};

use http::Method;
use percent_encoding::utf8_percent_encode;
use tracing::warn;

use super::{
    error::{IntoHttpError, UnknownVersionError},
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

    /// The unstable path of this endpoint's URL, often `None`, used for developmental
    /// purposes.
    pub unstable_path: Option<&'static str>,

    /// The pre-v1.1 version of this endpoint's URL, `None` for post-v1.1 endpoints,
    /// supplemental to `stable_path`.
    pub r0_path: Option<&'static str>,

    /// The path of this endpoint's URL, with variable names where path parameters should be
    /// filled in during a request.
    pub stable_path: Option<&'static str>,

    /// Whether or not this endpoint is rate limited by the server.
    pub rate_limited: bool,

    /// What authentication scheme the server uses for this endpoint.
    pub authentication: AuthScheme,

    /// The matrix version that this endpoint was added in.
    ///
    /// Is None when this endpoint is unstable/unreleased.
    pub added: Option<MatrixVersion>,

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
        if self.removed.map(greater_or_equal_all).unwrap_or(false) {
            return VersioningDecision::Removed;
        }

        // Check if *any* version marks this endpoint as stable.
        if self.added.map(greater_or_equal_any).unwrap_or(false) {
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

    /// Generate the endpoint URL for this endpoint.
    pub fn make_endpoint_url(
        &self,
        versions: &[MatrixVersion],
        base_url: &str,
        path_args: &[&dyn Display],
        query_string: Option<&str>,
    ) -> Result<String, IntoHttpError> {
        let path_with_placeholders = self.select_path(versions)?;

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
                let arg = utf8_percent_encode(&arg, percent_encoding::NON_ALPHANUMERIC);

                write!(res, "/{arg}").expect("writing to a String using fmt::Write can't fail");
            } else {
                res.reserve(segment.len() + 1);
                res.push('/');
                res.push_str(segment);
            }
        }

        if let Some(query) = query_string {
            res.push('?');
            res.push_str(query);
        }

        Ok(res)
    }

    // This function helps picks the right path (or an error) from a set of matrix versions.
    fn select_path(&self, versions: &[MatrixVersion]) -> Result<&str, IntoHttpError> {
        match self.versioning_decision_for(versions) {
            VersioningDecision::Removed => Err(IntoHttpError::EndpointRemoved(
                self.removed.expect("VersioningDecision::Removed implies metadata.removed"),
            )),
            VersioningDecision::Stable { any_deprecated, all_deprecated, any_removed } => {
                if any_removed {
                    if all_deprecated {
                        warn!(
                            "endpoint {} is removed in some (and deprecated in ALL) \
                             of the following versions: {:?}",
                            self.name, versions
                        );
                    } else if any_deprecated {
                        warn!(
                            "endpoint {} is removed (and deprecated) in some of the \
                             following versions: {:?}",
                            self.name, versions
                        );
                    } else {
                        unreachable!("any_removed implies *_deprecated");
                    }
                } else if all_deprecated {
                    warn!(
                        "endpoint {} is deprecated in ALL of the following versions: {:?}",
                        self.name, versions
                    );
                } else if any_deprecated {
                    warn!(
                        "endpoint {} is deprecated in some of the following versions: {:?}",
                        self.name, versions
                    );
                }

                if let Some(r0) = self.r0_path {
                    if versions.iter().all(|&v| v == MatrixVersion::V1_0) {
                        // Endpoint was added in 1.0, we return the r0 variant.
                        return Ok(r0);
                    }
                }

                Ok(self.stable_path.expect("metadata.added enforces the stable path to exist"))
            }
            VersioningDecision::Unstable => self.unstable_path.ok_or(IntoHttpError::NoUnstablePath),
        }
    }
}

/// A versioning "decision" derived from a set of matrix versions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(clippy::exhaustive_enums)]
pub enum VersioningDecision {
    /// The unstable endpoint should be used.
    Unstable,
    /// The stable endpoint should be used.
    ///
    /// Note, in the special case that all versions note [v1.0](MatrixVersion::V1_0), and the
    /// [`r0_path`](Metadata::r0_path) is not `None`, that path should be used.
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

    /// Version 1.4 of the Matrix specification, released in Q3 2022.
    ///
    /// See <https://spec.matrix.org/v1.4/>.
    V1_4,
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
            "v1.4" => V1_4,
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
            MatrixVersion::V1_4 => (1, 4),
        }
    }

    /// Try to turn a pair of (major, minor) version components back into a `MatrixVersion`.
    pub fn from_parts(major: u8, minor: u8) -> Result<Self, UnknownVersionError> {
        match (major, minor) {
            (1, 0) => Ok(MatrixVersion::V1_0),
            (1, 1) => Ok(MatrixVersion::V1_1),
            (1, 2) => Ok(MatrixVersion::V1_2),
            (1, 3) => Ok(MatrixVersion::V1_3),
            (1, 4) => Ok(MatrixVersion::V1_4),
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
            MatrixVersion::V1_3
            // <https://spec.matrix.org/v1.4/rooms/#complete-list-of-room-versions>
            | MatrixVersion::V1_4 => RoomVersionId::V9,
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
    use assert_matches::assert_matches;
    use http::Method;

    use super::{
        AuthScheme,
        MatrixVersion::{V1_0, V1_1, V1_2},
        Metadata,
    };
    use crate::api::error::IntoHttpError;

    const BASE: Metadata = Metadata {
        description: "",
        method: Method::GET,
        name: "test_endpoint",
        unstable_path: None,
        r0_path: None,
        stable_path: None,
        rate_limited: false,
        authentication: AuthScheme::None,
        added: None,
        deprecated: None,
        removed: None,
    };

    // TODO add test that can hook into tracing and verify the deprecation warning is emitted

    #[test]
    fn make_simple_endpoint_url() {
        let meta = Metadata { added: Some(V1_0), stable_path: Some("/s"), ..BASE };
        let url = meta.make_endpoint_url(&[V1_0], "https://example.org", &[], None).unwrap();
        assert_eq!(url, "https://example.org/s");
    }

    #[test]
    fn make_endpoint_url_with_path_args() {
        let meta = Metadata { added: Some(V1_0), stable_path: Some("/s/:x"), ..BASE };
        let url = meta.make_endpoint_url(&[V1_0], "https://example.org", &[&"123"], None).unwrap();
        assert_eq!(url, "https://example.org/s/123");
    }

    #[test]
    fn make_endpoint_url_with_query() {
        let meta = Metadata { added: Some(V1_0), stable_path: Some("/s/"), ..BASE };
        let url =
            meta.make_endpoint_url(&[V1_0], "https://example.org", &[], Some("foo=bar")).unwrap();
        assert_eq!(url, "https://example.org/s/?foo=bar");
    }

    #[test]
    #[should_panic]
    fn make_endpoint_url_wrong_num_path_args() {
        let meta = Metadata { added: Some(V1_0), stable_path: Some("/s/:x"), ..BASE };
        _ = meta.make_endpoint_url(&[V1_0], "https://example.org", &[], None);
    }

    #[test]
    fn select_stable() {
        let meta = Metadata { added: Some(V1_1), stable_path: Some("s"), ..BASE };
        assert_matches!(meta.select_path(&[V1_0, V1_1]), Ok("s"));
    }

    #[test]
    fn select_unstable() {
        let meta = Metadata { unstable_path: Some("u"), ..BASE };
        assert_matches!(meta.select_path(&[V1_0]), Ok("u"));
    }

    #[test]
    fn select_r0() {
        let meta = Metadata { added: Some(V1_0), r0_path: Some("r"), ..BASE };
        assert_matches!(meta.select_path(&[V1_0]), Ok("r"));
    }

    #[test]
    fn select_removed_err() {
        let meta = Metadata {
            added: Some(V1_0),
            deprecated: Some(V1_1),
            removed: Some(V1_2),
            unstable_path: Some("u"),
            r0_path: Some("r"),
            stable_path: Some("s"),
            ..BASE
        };
        assert_matches!(meta.select_path(&[V1_2]), Err(IntoHttpError::EndpointRemoved(V1_2)));
    }

    #[test]
    fn partially_removed_but_stable() {
        let meta = Metadata {
            added: Some(V1_0),
            deprecated: Some(V1_1),
            removed: Some(V1_2),
            r0_path: Some("r"),
            stable_path: Some("s"),
            ..BASE
        };
        assert_matches!(meta.select_path(&[V1_1]), Ok("s"));
    }

    #[test]
    fn no_unstable() {
        let meta =
            Metadata { added: Some(V1_1), r0_path: Some("r"), stable_path: Some("s"), ..BASE };
        assert_matches!(meta.select_path(&[V1_0]), Err(IntoHttpError::NoUnstablePath));
    }
}
