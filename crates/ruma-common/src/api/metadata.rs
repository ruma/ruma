use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    str::FromStr,
};

use bytes::BufMut;
use http::Method;
use ruma_macros::StringEnum;

use super::{auth_scheme::AuthScheme, error::UnknownVersionError, path_builder::PathBuilder};
use crate::{PrivOwnedStr, RoomVersionId, api::error::IntoHttpError, serde::slice_to_buf};

/// Convenient constructor for [`Metadata`] implementation.
///
/// ## Definition
///
/// By default, `Metadata` is implemented on a type named `Request` that is in scope. This can be
/// overridden by adding `@for MyType` at the beginning of the declaration.
///
/// The rest of the definition of the macro is made to look like a struct, with the following
/// fields:
///
/// * `method` - The HTTP method to use for the endpoint. Its value must be one of the associated
///   constants of [`http::Method`]. In most cases it should be one of `GET`, `POST`, `PUT` or
///   `DELETE`.
/// * `rate_limited` - Whether the endpoint should be rate-limited, according to the specification.
///   Its value must be a `bool`.
/// * `authentication` - The type of authentication that is required for the endpoint, according to
///   the specification. The type must be in scope and implement [`AuthScheme`].
///
/// And either of the following fields to define the path(s) of the endpoint.
///
/// * `history` - The history of the paths of the endpoint. This should be used for endpoints from
///   Matrix APIs that have a `/versions` endpoint that returns a list a [`MatrixVersion`]s and
///   possibly features, like the Client-Server API or the Identity Service API. However, a few
///   endpoints from those APIs shouldn't use this field because they cannot be versioned, like the
///   `/versions` or the `/.well-known` endpoints.
///
///   Its definition is made to look like match arms and must include at least one arm. The match
///   arms accept the following syntax:
///
///   * `unstable => "unstable/endpoint/path/{variable}"` - An unstable version of the endpoint as
///     defined in the MSC that adds it, if the MSC does **NOT** define an unstable feature in the
///     `unstable_features` field of the client-server API's `/versions` endpoint.
///   * `unstable("org.bar.unstable_feature") => "unstable/endpoint/path/{variable}"` - An unstable
///     version of the endpoint as defined in the MSC that adds it, if the MSC defines an unstable
///     feature in the `unstable_features` field of the client-server API's `/versions` endpoint.
///   * `1.0 | stable("org.bar.feature.stable") => "stable/endpoint/path/{variable}"` - A stable
///     version of the endpoint as defined in an MSC or the Matrix specification. The match arm can
///     be a Matrix version, a stable feature, or both separated by `|`.
///
///     A stable feature can be defined in an MSC alongside an unstable feature, and can be found in
///     the `unstable_features` field of the client-server API's `/versions` endpoint. It is meant
///     to be used by homeservers if they want to declare stable support for a feature before they
///     can declare support for a whole Matrix version that supports it.
///
///   * `1.2 => deprecated` - The Matrix version that deprecated the endpoint, if any. It must be
///     preceded by a match arm with a stable path and a different Matrix version.
///   * `1.3 => removed` - The Matrix version that removed the endpoint, if any. It must be preceded
///     by a match arm with a deprecation and a different Matrix version.
///
///   A Matrix version is a `float` representation of the version that looks like `major.minor`.
///   It must match one of the variants of [`MatrixVersion`]. For example `1.0` matches
///   [`MatrixVersion::V1_0`], `1.1` matches [`MatrixVersion::V1_1`], etc.
///
///   It is expected that the match arms are ordered by descending age. Usually the older unstable
///   paths would be before the newer unstable paths, then we would find the stable paths, and
///   finally the deprecation and removal.
///
///   The following checks occur at compile time:
///
///   * All unstable and stable paths contain the same variables (or lack thereof).
///   * Matrix versions in match arms are all different and in ascending order.
///
///   This field is represented as the [`VersionHistory`](super::path_builder::VersionHistory) type
///   in the generated implementation.
/// * `path` - The only path of the endpoint. This should be used for endpoints from Matrix APIs
///   that do NOT have a `/versions` endpoint that returns a list a [`MatrixVersion`]s, like the
///   Server-Server API or the Appservice API. It should also be used for endpoints that cannot be
///   versioned, like the `/versions` or the `/.well-known` endpoints.
///
///   Its value must be a static string representing the path, like `"endpoint/path/{variable}"`.
///
///   This field is represented as the [`SinglePath`](super::path_builder::SinglePath) type in the
///   generated implementation.
///
/// ## Example
///
/// ```
/// use ruma_common::{
///     api::auth_scheme::{AccessToken, NoAuthentication},
///     metadata,
/// };
///
/// /// A Request with a path version history.
/// pub struct Request {
///     body: Vec<u8>,
/// }
///
/// metadata! {
///     method: GET,
///     rate_limited: true,
///     authentication: AccessToken,
///
///     history: {
///         unstable => "/_matrix/unstable/org.bar.msc9000/baz",
///         unstable("org.bar.msc9000.v1") => "/_matrix/unstable/org.bar.msc9000.v1/qux",
///         1.0 | stable("org.bar.msc9000.stable") => "/_matrix/media/r0/qux",
///         1.1 => "/_matrix/media/v3/qux",
///         1.2 => deprecated,
///         1.3 => removed,
///     }
/// };
///
/// /// A request with a single path.
/// pub struct MySinglePathRequest {
///     body: Vec<u8>,
/// }
///
/// metadata! {
///     @for MySinglePathRequest,
///
///     method: GET,
///     rate_limited: false,
///     authentication: NoAuthentication,
///     path: "/_matrix/key/query",
/// };
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! metadata {
    ( @for $request_type:ty, $( $field:ident: $rhs:tt ),+ $(,)? ) => {
        #[allow(deprecated)]
        impl $crate::api::Metadata for $request_type {
            $( $crate::metadata!(@field $field: $rhs); )+
        }
    };

    ( $( $field:ident: $rhs:tt ),+ $(,)? ) => {
        $crate::metadata!{ @for Request, $( $field: $rhs),+ }
    };

    ( @field method: $method:ident ) => {
        const METHOD: $crate::exports::http::Method = $crate::exports::http::Method::$method;
    };

    ( @field rate_limited: $rate_limited:literal ) => { const RATE_LIMITED: bool = $rate_limited; };

    ( @field authentication: $scheme:path ) => {
        type Authentication = $scheme;
    };

    ( @field path: $path:literal ) => {
        type PathBuilder = $crate::api::path_builder::SinglePath;
        const PATH_BUILDER: $crate::api::path_builder::SinglePath = $crate::api::path_builder::SinglePath::new($path);
    };

    ( @field history: {
        $( unstable $(($unstable_feature:literal))? => $unstable_path:literal, )*
        $( stable ($stable_feature_only:literal) => $stable_feature_path:literal, )*
        $( $version:literal $(| stable ($stable_feature:literal))? => $stable_rhs:tt, )*
    } ) => {
        $crate::metadata! {
            @history_impl
            $( unstable $( ($unstable_feature) )? => $unstable_path, )*
            $( stable ($stable_feature_only) => $stable_feature_path, )*
            // Flip left and right to avoid macro parsing ambiguities
            $( $stable_rhs = $version $( | stable ($stable_feature) )?, )*
        }
    };

    ( @history_impl
        $( unstable $(($unstable_feature:literal))? => $unstable_path:literal, )*
        $( stable ($stable_feature_only:literal) => $stable_feature_path:literal, )*
        $( $stable_path:literal = $version:literal $(| stable ($stable_feature:literal))?, )*
        $( deprecated = $deprecated_version:literal, )?
        $( removed = $removed_version:literal, )?
    ) => {
        type PathBuilder = $crate::api::path_builder::VersionHistory;
        const PATH_BUILDER: $crate::api::path_builder::VersionHistory = $crate::api::path_builder::VersionHistory::new(
            &[ $(($crate::metadata!(@optional_feature $($unstable_feature)?), $unstable_path)),* ],
            &[
                $((
                    $crate::metadata!(@stable_path_selector stable($stable_feature_only)),
                    $stable_feature_path
                ),)*
                $((
                    $crate::metadata!(@stable_path_selector $version $( | stable($stable_feature) )?),
                    $stable_path
                ),)*
            ],
            $crate::metadata!(@optional_version $( $deprecated_version )?),
            $crate::metadata!(@optional_version $( $removed_version )?),
        );
    };

    ( @optional_feature ) => { None };
    ( @optional_feature $feature:literal ) => { Some($feature) };
    ( @stable_path_selector stable($feature:literal)) => {
        $crate::api::path_builder::StablePathSelector::Feature($feature)
    };
    ( @stable_path_selector $version:literal | stable($feature:literal)) => {
        $crate::api::path_builder::StablePathSelector::FeatureAndVersion {
            feature: $feature,
            version: $crate::api::MatrixVersion::from_lit(stringify!($version)),
        }
    };
    ( @stable_path_selector $version:literal) => {
        $crate::api::path_builder::StablePathSelector::Version(
            $crate::api::MatrixVersion::from_lit(stringify!($version))
        )
    };
    ( @optional_version ) => { None };
    ( @optional_version $version:literal ) => { Some($crate::api::MatrixVersion::from_lit(stringify!($version))) }
}

/// Metadata about an API endpoint.
pub trait Metadata: Sized {
    /// The HTTP method used by this endpoint.
    const METHOD: Method;

    /// Whether or not this endpoint is rate limited by the server.
    const RATE_LIMITED: bool;

    /// What authentication scheme the server uses for this endpoint.
    type Authentication: AuthScheme;

    /// The type used to build an endpoint's path.
    type PathBuilder: PathBuilder;

    /// All info pertaining to an endpoint's path.
    const PATH_BUILDER: Self::PathBuilder;

    /// Returns an empty request body for this Matrix request.
    ///
    /// For `GET` requests, it returns an entirely empty buffer, for others it returns an empty JSON
    /// object (`{}`).
    fn empty_request_body<B>() -> B
    where
        B: Default + BufMut,
    {
        if Self::METHOD == Method::GET { Default::default() } else { slice_to_buf(b"{}") }
    }

    /// Generate the endpoint URL for this endpoint.
    fn make_endpoint_url(
        path_builder_input: <Self::PathBuilder as PathBuilder>::Input<'_>,
        base_url: &str,
        path_args: &[&dyn Display],
        query_string: &str,
    ) -> Result<String, IntoHttpError> {
        Self::PATH_BUILDER.make_endpoint_url(path_builder_input, base_url, path_args, query_string)
    }

    /// The list of path parameters in the metadata.
    ///
    /// Used for `#[test]`s generated by the API macros.
    #[doc(hidden)]
    fn _path_parameters() -> Vec<&'static str> {
        Self::PATH_BUILDER._path_parameters()
    }
}

/// The Matrix versions Ruma currently understands to exist.
///
/// Matrix, since fall 2021, has a quarterly release schedule, using a global `vX.Y` versioning
/// scheme. Usually `Y` is bumped for new backwards compatible changes, but `X` can be bumped
/// instead when a large number of `Y` changes feel deserving of a major version increase.
///
/// Every new version denotes stable support for endpoints in a *relatively* backwards-compatible
/// manner.
///
/// Matrix has a deprecation policy, read more about it here: <https://spec.matrix.org/latest/#deprecation-policy>.
///
/// Ruma keeps track of when endpoints are added, deprecated, and removed. It'll automatically
/// select the right endpoint stability variation to use depending on which Matrix versions you
/// pass to [`try_into_http_request`](super::OutgoingRequest::try_into_http_request), see its
/// respective documentation for more information.
///
/// The `PartialOrd` and `Ord` implementations of this type sort the variants by release date. A
/// newer release is greater than an older release.
///
/// `MatrixVersion::is_superset_of()` is used to keep track of compatibility between versions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub enum MatrixVersion {
    /// Matrix 1.0 was a release prior to the global versioning system and does not correspond to a
    /// version of the Matrix specification.
    ///
    /// It matches the following per-API versions:
    ///
    /// * Client-Server API: r0.5.0 to r0.6.1
    /// * Identity Service API: r0.2.0 to r0.3.0
    ///
    /// The other APIs are not supported because they do not have a `GET /versions` endpoint.
    ///
    /// See <https://spec.matrix.org/latest/#legacy-versioning>.
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

    /// Version 1.11 of the Matrix specification, released in Q2 2024.
    ///
    /// See <https://spec.matrix.org/v1.11/>.
    V1_11,

    /// Version 1.12 of the Matrix specification, released in Q3 2024.
    ///
    /// See <https://spec.matrix.org/v1.12/>.
    V1_12,

    /// Version 1.13 of the Matrix specification, released in Q4 2024.
    ///
    /// See <https://spec.matrix.org/v1.13/>.
    V1_13,

    /// Version 1.14 of the Matrix specification, released in Q1 2025.
    ///
    /// See <https://spec.matrix.org/v1.14/>.
    V1_14,

    /// Version 1.15 of the Matrix specification, released in Q2 2025.
    ///
    /// See <https://spec.matrix.org/v1.15/>.
    V1_15,

    /// Version 1.16 of the Matrix specification, released in Q3 2025.
    ///
    /// See <https://spec.matrix.org/v1.16/>.
    V1_16,
}

impl TryFrom<&str> for MatrixVersion {
    type Error = UnknownVersionError;

    fn try_from(value: &str) -> Result<MatrixVersion, Self::Error> {
        use MatrixVersion::*;

        Ok(match value {
            // Identity service API versions between Matrix 1.0 and 1.1.
            // They might match older client-server API versions but that should not be a problem in practice.
            "r0.2.0" | "r0.2.1" | "r0.3.0" |
            // Client-server API versions between Matrix 1.0 and 1.1.
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
            "v1.11" => V1_11,
            "v1.12" => V1_12,
            "v1.13" => V1_13,
            "v1.14" => V1_14,
            "v1.15" => V1_15,
            "v1.16" => V1_16,
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
    /// Currently, all versions of Matrix are considered backwards compatible with all the previous
    /// versions, so this is equivalent to `self >= other`. This behaviour may change in the future,
    /// if a new release is considered to be breaking compatibility with the previous ones.
    ///
    /// > ⚠ Matrix has a deprecation policy, and Matrix versioning is not as straightforward as this
    /// > function makes it out to be. This function only exists to prune breaking changes between
    /// > versions, and versions too new for `self`.
    pub fn is_superset_of(self, other: Self) -> bool {
        self >= other
    }

    /// Get a string representation of this Matrix version.
    ///
    /// This is the string that can be found in the response to one of the `GET /versions`
    /// endpoints. Parsing this string will give the same variant.
    ///
    /// Returns `None` for [`MatrixVersion::V1_0`] because it can match several per-API versions.
    pub const fn as_str(self) -> Option<&'static str> {
        let string = match self {
            MatrixVersion::V1_0 => return None,
            MatrixVersion::V1_1 => "v1.1",
            MatrixVersion::V1_2 => "v1.2",
            MatrixVersion::V1_3 => "v1.3",
            MatrixVersion::V1_4 => "v1.4",
            MatrixVersion::V1_5 => "v1.5",
            MatrixVersion::V1_6 => "v1.6",
            MatrixVersion::V1_7 => "v1.7",
            MatrixVersion::V1_8 => "v1.8",
            MatrixVersion::V1_9 => "v1.9",
            MatrixVersion::V1_10 => "v1.10",
            MatrixVersion::V1_11 => "v1.11",
            MatrixVersion::V1_12 => "v1.12",
            MatrixVersion::V1_13 => "v1.13",
            MatrixVersion::V1_14 => "v1.14",
            MatrixVersion::V1_15 => "v1.15",
            MatrixVersion::V1_16 => "v1.16",
        };

        Some(string)
    }

    /// Decompose the Matrix version into its major and minor number.
    const fn into_parts(self) -> (u8, u8) {
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
            MatrixVersion::V1_11 => (1, 11),
            MatrixVersion::V1_12 => (1, 12),
            MatrixVersion::V1_13 => (1, 13),
            MatrixVersion::V1_14 => (1, 14),
            MatrixVersion::V1_15 => (1, 15),
            MatrixVersion::V1_16 => (1, 16),
        }
    }

    /// Try to turn a pair of (major, minor) version components back into a `MatrixVersion`.
    const fn from_parts(major: u8, minor: u8) -> Result<Self, UnknownVersionError> {
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
            (1, 11) => Ok(MatrixVersion::V1_11),
            (1, 12) => Ok(MatrixVersion::V1_12),
            (1, 13) => Ok(MatrixVersion::V1_13),
            (1, 14) => Ok(MatrixVersion::V1_14),
            (1, 15) => Ok(MatrixVersion::V1_15),
            (1, 16) => Ok(MatrixVersion::V1_16),
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
    pub(super) const fn const_ord(&self, other: &Self) -> Ordering {
        let self_parts = self.into_parts();
        let other_parts = other.into_parts();

        use konst::primitive::cmp::cmp_u8;

        let major_ord = cmp_u8(self_parts.0, other_parts.0);
        if major_ord.is_ne() { major_ord } else { cmp_u8(self_parts.1, other_parts.1) }
    }

    // Internal function to check if this version is the legacy (v1.0) version in const-fn contexts
    pub(super) const fn is_legacy(&self) -> bool {
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
            | MatrixVersion::V1_10
            // <https://spec.matrix.org/v1.11/rooms/#complete-list-of-room-versions>
            | MatrixVersion::V1_11
            // <https://spec.matrix.org/v1.12/rooms/#complete-list-of-room-versions>
            | MatrixVersion::V1_12
            // <https://spec.matrix.org/v1.13/rooms/#complete-list-of-room-versions>
            | MatrixVersion::V1_13 => RoomVersionId::V10,
            // <https://spec.matrix.org/v1.14/rooms/#complete-list-of-room-versions>
            | MatrixVersion::V1_14
            // <https://spec.matrix.org/v1.15/rooms/#complete-list-of-room-versions>
            | MatrixVersion::V1_15 => RoomVersionId::V11,
            // <https://spec.matrix.org/v1.16/rooms/#complete-list-of-room-versions>
            MatrixVersion::V1_16 => RoomVersionId::V12,
        }
    }
}

/// The list of Matrix versions and features supported by a homeserver.
#[derive(Debug, Clone)]
#[allow(clippy::exhaustive_structs)]
pub struct SupportedVersions {
    /// The Matrix versions that are supported by the homeserver.
    ///
    /// This set contains only known versions.
    pub versions: BTreeSet<MatrixVersion>,

    /// The features that are supported by the homeserver.
    ///
    /// This matches the `unstable_features` field of the `/versions` endpoint, without the boolean
    /// value.
    pub features: BTreeSet<FeatureFlag>,
}

impl SupportedVersions {
    /// Construct a `SupportedVersions` from the parts of a `/versions` response.
    ///
    /// Matrix versions that can't be parsed to a `MatrixVersion`, and features with the boolean
    /// value set to `false` are discarded.
    pub fn from_parts(versions: &[String], unstable_features: &BTreeMap<String, bool>) -> Self {
        Self {
            versions: versions.iter().flat_map(|s| s.parse::<MatrixVersion>()).collect(),
            features: unstable_features
                .iter()
                .filter(|(_, enabled)| **enabled)
                .map(|(feature, _)| feature.as_str().into())
                .collect(),
        }
    }
}

/// The Matrix features supported by Ruma.
///
/// Features that are not behind a cargo feature are features that are part of the Matrix
/// specification and that Ruma still supports, like the unstable version of an endpoint or a stable
/// feature. Features behind a cargo feature are only supported when this feature is enabled.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, StringEnum, Hash)]
#[non_exhaustive]
pub enum FeatureFlag {
    /// `fi.mau.msc2246` ([MSC])
    ///
    /// Asynchronous media uploads.
    ///
    /// [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/2246
    #[ruma_enum(rename = "fi.mau.msc2246")]
    Msc2246,

    /// `org.matrix.msc2432` ([MSC])
    ///
    /// Updated semantics for publishing room aliases.
    ///
    /// [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/2432
    #[ruma_enum(rename = "org.matrix.msc2432")]
    Msc2432,

    /// `fi.mau.msc2659` ([MSC])
    ///
    /// Application service ping endpoint.
    ///
    /// [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/2659
    #[ruma_enum(rename = "fi.mau.msc2659")]
    Msc2659,

    /// `fi.mau.msc2659` ([MSC])
    ///
    /// Stable version of the application service ping endpoint.
    ///
    /// [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/2659
    #[ruma_enum(rename = "fi.mau.msc2659.stable")]
    Msc2659Stable,

    /// `uk.half-shot.msc2666.query_mutual_rooms` ([MSC])
    ///
    /// Get rooms in common with another user.
    ///
    /// [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/2666
    #[cfg(feature = "unstable-msc2666")]
    #[ruma_enum(rename = "uk.half-shot.msc2666.query_mutual_rooms")]
    Msc2666,

    /// `org.matrix.msc3030` ([MSC])
    ///
    /// Jump to date API endpoint.
    ///
    /// [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/3030
    #[ruma_enum(rename = "org.matrix.msc3030")]
    Msc3030,

    /// `org.matrix.msc3882` ([MSC])
    ///
    /// Allow an existing session to sign in a new session.
    ///
    /// [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/3882
    #[ruma_enum(rename = "org.matrix.msc3882")]
    Msc3882,

    /// `org.matrix.msc3916` ([MSC])
    ///
    /// Authentication for media.
    ///
    /// [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/3916
    #[ruma_enum(rename = "org.matrix.msc3916")]
    Msc3916,

    /// `org.matrix.msc3916.stable` ([MSC])
    ///
    /// Stable version of authentication for media.
    ///
    /// [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/3916
    #[ruma_enum(rename = "org.matrix.msc3916.stable")]
    Msc3916Stable,

    /// `org.matrix.msc4108` ([MSC])
    ///
    /// Mechanism to allow OIDC sign in and E2EE set up via QR code.
    ///
    /// [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4108
    #[cfg(feature = "unstable-msc4108")]
    #[ruma_enum(rename = "org.matrix.msc4108")]
    Msc4108,

    /// `org.matrix.msc4140` ([MSC])
    ///
    /// Delayed events.
    ///
    /// [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4140
    #[cfg(feature = "unstable-msc4140")]
    #[ruma_enum(rename = "org.matrix.msc4140")]
    Msc4140,

    /// `org.matrix.simplified_msc3575` ([MSC])
    ///
    /// Simplified Sliding Sync.
    ///
    /// [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4186
    #[cfg(feature = "unstable-msc4186")]
    #[ruma_enum(rename = "org.matrix.simplified_msc3575")]
    Msc4186,

    /// `org.matrix.msc4380_invite_permission_config` ([MSC])
    ///
    /// Invite Blocking.
    ///
    /// [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4380
    #[cfg(feature = "unstable-msc4380")]
    #[ruma_enum(rename = "org.matrix.msc4380")]
    Msc4380,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}
