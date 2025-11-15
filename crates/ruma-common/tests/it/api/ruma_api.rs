#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/it/api/ui/api-single-path-check.rs");
    t.pass("tests/it/api/ui/api-version-history-check.rs");
    t.pass("tests/it/api/ui/move-value.rs");
    t.pass("tests/it/api/ui/request-only.rs");
    t.pass("tests/it/api/ui/response-only.rs");
    t.compile_fail("tests/it/api/ui/serde-flatten-request-body.rs");
    t.compile_fail("tests/it/api/ui/serde-flatten-response-body.rs");
    t.compile_fail("tests/it/api/ui/invalid-path.rs");
    t.compile_fail("tests/it/api/ui/invalid-version-history.rs");
}
