#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/01-valid-id-macros.rs");
    t.compile_fail("tests/ui/02-invalid-id-macros.rs");
}
