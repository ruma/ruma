#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/it/identifiers/ui/01-valid-id-macros.rs");
    t.compile_fail("tests/it/identifiers/ui/02-invalid-id-macros.rs");
    t.compile_fail("tests/it/identifiers/ui/03-invalid-new-id-macros.rs");
}
