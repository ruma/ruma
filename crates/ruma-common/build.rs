use std::env;

fn main() {
    // Set the `ruma_identifiers_storage` configuration from an environment variable.
    if let Ok(value) = env::var("RUMA_IDENTIFIERS_STORAGE") {
        println!("cargo:rustc-cfg=ruma_identifiers_storage={value}");
    }

    println!("cargo:rerun-if-env-changed=RUMA_IDENTIFIERS_STORAGE");
}
