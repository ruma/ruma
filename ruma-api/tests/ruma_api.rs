#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/01-api-sanity-check.rs");
    t.compile_fail("tests/ui/02-invalid-path.rs");
}
