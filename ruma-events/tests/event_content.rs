#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/01-content-sanity-check.rs");
    t.compile_fail("tests/ui/02-no-event-type.rs");
    t.compile_fail("tests/ui/03-invalid-event-type.rs");
}
