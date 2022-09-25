use assert_matches::assert_matches;
use http::Method;
use ruma_common::api::{
    error::IntoHttpError,
    select_path,
    MatrixVersion::{self, V1_0, V1_1, V1_2, V1_3},
    Metadata, PathData, VersionHistory,
};

const BASE: Metadata = Metadata {
    description: "",
    method: Method::GET,
    name: "test_endpoint",
    rate_limited: false,
    authentication: ruma_common::api::AuthScheme::None,

    history: VersionHistory {
        unstable_paths: &[],
        stable_paths: &[],
        deprecated: None,
        removed: None,
    },
};

const UNSTABLE_PATH: PathData = PathData { canon: "/unstable/path", parts: &["/unstable/path"] };
const V1_0_PATH: (MatrixVersion, PathData) =
    (V1_0, PathData { canon: "/r0/path", parts: &["/r0/path"] });
const V1_1_PATH: (MatrixVersion, PathData) =
    (V1_1, PathData { canon: "/stable/path", parts: &["/stable/path"] });

// TODO add test that can hook into tracing and verify the deprecation warning is emitted

#[test]
fn select_latest_stable() {
    const META: Metadata = Metadata {
        history: VersionHistory {
            unstable_paths: &[UNSTABLE_PATH],
            stable_paths: &[V1_0_PATH, V1_1_PATH],
            ..BASE.history
        },
        ..BASE
    };

    let res = select_path(&[V1_0, V1_1], &META).unwrap();

    assert_eq!(res, &V1_1_PATH.1);
}

#[test]
fn select_fallback_unstable() {
    const META: Metadata = Metadata {
        history: VersionHistory {
            unstable_paths: &[UNSTABLE_PATH],
            stable_paths: &[V1_1_PATH],
            ..BASE.history
        },
        ..BASE
    };

    let res = select_path(&[V1_0], &META).unwrap();

    assert_eq!(res, &UNSTABLE_PATH);
}

#[test]
fn select_constrained_stable() {
    const META: Metadata = Metadata {
        history: VersionHistory {
            unstable_paths: &[UNSTABLE_PATH],
            stable_paths: &[V1_0_PATH, V1_1_PATH],
            ..BASE.history
        },
        ..BASE
    };

    let res = select_path(&[V1_0], &META).unwrap();

    assert_eq!(res, &V1_0_PATH.1);
}

#[test]
fn select_removed_err() {
    const META: Metadata = Metadata {
        history: VersionHistory {
            unstable_paths: &[UNSTABLE_PATH],
            stable_paths: &[V1_0_PATH, V1_1_PATH],
            deprecated: Some(V1_2),
            removed: Some(V1_3),
        },
        ..BASE
    };

    let res = select_path(&[V1_3], &META).unwrap_err();

    assert_matches!(res, IntoHttpError::EndpointRemoved(V1_3));
}

#[test]
fn partially_deprecated_but_stable() {
    const META: Metadata = Metadata {
        history: VersionHistory {
            unstable_paths: &[UNSTABLE_PATH],
            stable_paths: &[V1_0_PATH],
            deprecated: Some(V1_1),
            removed: Some(V1_2),
        },
        ..BASE
    };

    let res = select_path(&[V1_1], &META).unwrap();

    assert_eq!(res, &V1_0_PATH.1);
}

#[test]
fn no_unstable() {
    const META: Metadata =
        Metadata { history: VersionHistory { stable_paths: &[V1_1_PATH], ..BASE.history }, ..BASE };

    let res = select_path(&[V1_0], &META).unwrap_err();

    assert_matches!(res, IntoHttpError::NoUnstablePath);
}
