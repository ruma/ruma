#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/it/api/ui/api-sanity-check.rs");
    t.pass("tests/it/api/ui/move-value.rs");
    t.pass("tests/it/api/ui/request-only.rs");
    t.pass("tests/it/api/ui/response-only.rs");
    t.compile_fail("tests/it/api/ui/deprecated-without-added.rs");
    t.compile_fail("tests/it/api/ui/removed-without-deprecated.rs");
}
