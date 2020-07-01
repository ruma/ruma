#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/01-api-sanity-check.rs");
}
