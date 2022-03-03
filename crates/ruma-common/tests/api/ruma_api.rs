#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/01-api-sanity-check.rs");
    t.compile_fail("tests/ui/02-invalid-path.rs");
    t.pass("tests/ui/03-move-value.rs");
    t.compile_fail("tests/ui/04-attributes.rs");
    t.pass("tests/ui/05-request-only.rs");
    t.pass("tests/ui/06-response-only.rs");
    t.compile_fail("tests/ui/07-error-type-attribute.rs");
    t.compile_fail("tests/ui/08-deprecated-without-added.rs");
    t.compile_fail("tests/ui/09-removed-without-deprecated.rs");
}
