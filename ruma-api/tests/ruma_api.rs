#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/01-api-sanity-check.rs");
    t.compile_fail("tests/ui/02-invalid-path.rs");
    t.pass("tests/ui/03-move-value.rs");
    t.compile_fail("tests/ui/04-attributes.rs");
}
