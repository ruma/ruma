use std::{env, process};

fn main() {
    let tls_native_active = env::var_os("CARGO_FEATURE_TLS_NATIVE").is_some();
    let tls_rustls_active = env::var_os("CARGO_FEATURE_TLS_RUSTLS").is_some();

    if tls_native_active && tls_rustls_active {
        eprintln!(
            "error: The tls-native and tls-rustls features can't be activated at the same time."
        );
        process::exit(1);
    }
}
