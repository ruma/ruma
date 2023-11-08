fn main() {
    // Prevent unnecessary rerunning of this build script
    println!("cargo:rerun-if-changed=build.rs");
    // Prevent nightly CI from erroring on docsrs attributes
    println!("cargo:rustc-check-cfg=cfg(docsrs)");
}
