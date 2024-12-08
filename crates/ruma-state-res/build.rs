use std::env;

fn main() {
    // Set the `ruma_unstable_exhaustive_types` configuration from an environment variable.
    if env::var("RUMA_UNSTABLE_EXHAUSTIVE_TYPES").is_ok() {
        println!("cargo:rustc-cfg=ruma_unstable_exhaustive_types");
    }

    println!("cargo:rerun-if-env-changed=RUMA_UNSTABLE_EXHAUSTIVE_TYPES");
}
