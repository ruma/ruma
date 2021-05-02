#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    // rustc overflows when compiling this see:
    // https://github.com/rust-lang/rust/issues/55779
    // there is a workaround in the file.
    t.pass("tests/ui/04-event-sanity-check.rs");
    t.compile_fail("tests/ui/05-named-fields.rs");
    t.compile_fail("tests/ui/06-no-content-field.rs");
}
