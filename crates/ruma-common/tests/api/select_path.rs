use http::Method;
use matches::assert_matches;
use ruma_common::api::{
    error::IntoHttpError,
    select_path,
    MatrixVersion::{V1_0, V1_1, V1_2},
    Metadata,
};

const BASE: Metadata = Metadata {
    description: "",
    method: Method::GET,
    name: "test_endpoint",
    unstable_path: Some("/unstable/path"),
    r0_path: Some("/r0/path"),
    stable_path: Some("/stable/path"),
    rate_limited: false,
    authentication: ruma_common::api::AuthScheme::None,
    added: None,
    deprecated: None,
    removed: None,
};

const U: &str = "u";
const S: &str = "s";
const R: &str = "r";

// TODO add test that can hook into tracing and verify the deprecation warning is emitted

#[test]
fn select_stable() {
    let meta = Metadata { added: Some(V1_1), ..BASE };

    let res = select_path(&[V1_0, V1_1], &meta, None, None, Some(format_args!("{}", S)))
        .unwrap()
        .to_string();

    assert_eq!(res, S);
}

#[test]
fn select_unstable() {
    let meta = BASE;

    let res =
        select_path(&[V1_0], &meta, Some(format_args!("{}", U)), None, None).unwrap().to_string();

    assert_eq!(res, U);
}

#[test]
fn select_r0() {
    let meta = Metadata { added: Some(V1_0), ..BASE };

    let res =
        select_path(&[V1_0], &meta, None, Some(format_args!("{}", R)), Some(format_args!("{}", S)))
            .unwrap()
            .to_string();

    assert_eq!(res, R);
}

#[test]
fn select_removed_err() {
    let meta = Metadata { added: Some(V1_0), deprecated: Some(V1_1), removed: Some(V1_2), ..BASE };

    let res = select_path(
        &[V1_2],
        &meta,
        Some(format_args!("{}", U)),
        Some(format_args!("{}", R)),
        Some(format_args!("{}", S)),
    )
    .unwrap_err();

    assert_matches!(res, IntoHttpError::EndpointRemoved(V1_2));
}

#[test]
fn partially_removed_but_stable() {
    let meta = Metadata { added: Some(V1_0), deprecated: Some(V1_1), removed: Some(V1_2), ..BASE };

    let res =
        select_path(&[V1_1], &meta, None, Some(format_args!("{}", R)), Some(format_args!("{}", S)))
            .unwrap()
            .to_string();

    assert_eq!(res, S);
}

#[test]
fn no_unstable() {
    let meta = Metadata { added: Some(V1_1), ..BASE };

    let res =
        select_path(&[V1_0], &meta, None, Some(format_args!("{}", R)), Some(format_args!("{}", S)))
            .unwrap_err();

    assert_matches!(res, IntoHttpError::NoUnstablePath);
}
