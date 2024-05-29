use std::{env, process};

fn main() {
    // Prevent unnecessary rerunning of this build script
    println!("cargo:rerun-if-changed=build.rs");

    let tls_features = [
        ("tls-native", env::var_os("CARGO_FEATURE_TLS_NATIVE").is_some()),
        ("tls-rustls-native-roots", env::var_os("CARGO_FEATURE_TLS_RUSTLS_NATIVE_ROOTS").is_some()),
        ("tls-rustls-webpki-roots", env::var_os("CARGO_FEATURE_TLS_RUSTLS_WEBPKI_ROOTS").is_some()),
    ];

    if tls_features.iter().filter(|(_, a)| *a).count() > 1 {
        eprintln!("error: Only one tls features can be enabled.");

        for (f, a) in &tls_features {
            eprintln!("  {f}: {}", if *a { "enabled" } else { "disabled" });
        }

        process::exit(1);
    }
}
