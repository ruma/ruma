#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/07-enum-sanity-check.rs");
    t.compile_fail("tests/ui/08-enum-invalid-path.rs");
}
