#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/09-any-deserialize-sanity-check.rs");
    t.compile_fail("tests/ui/10-invalid-variant.rs");
}
