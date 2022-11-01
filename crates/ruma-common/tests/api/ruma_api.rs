#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/api/ui/api-sanity-check.rs");
    t.pass("tests/api/ui/move-value.rs");
    t.pass("tests/api/ui/request-only.rs");
    t.pass("tests/api/ui/response-only.rs");
    t.compile_fail("tests/api/ui/deprecated-without-added.rs");
    t.compile_fail("tests/api/ui/removed-without-deprecated.rs");
}
