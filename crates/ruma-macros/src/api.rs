//! Methods and types for generating API endpoints.

use std::{env, fs, path::Path};

use once_cell::sync::Lazy;
use proc_macro2::Span;
use serde::{de::IgnoredAny, Deserialize};

mod attribute;
mod auth_scheme;
pub mod request;
pub mod response;

mod kw {
    syn::custom_keyword!(error);
}

// Returns an error with a helpful error if the crate the request or response macro is used from
// doesn't declare both a `client` and a `server` feature.
fn ensure_feature_presence() -> Option<&'static syn::Error> {
    #[derive(Deserialize)]
    struct CargoToml {
        features: Features,
    }

    #[derive(Deserialize)]
    struct Features {
        client: Option<IgnoredAny>,
        server: Option<IgnoredAny>,
    }

    static RESULT: Lazy<Result<(), syn::Error>> = Lazy::new(|| {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR")
            .map_err(|_| syn::Error::new(Span::call_site(), "Failed to read CARGO_MANIFEST_DIR"))?;

        let manifest_file = Path::new(&manifest_dir).join("Cargo.toml");
        let manifest_bytes = fs::read_to_string(manifest_file)
            .map_err(|_| syn::Error::new(Span::call_site(), "Failed to read Cargo.toml"))?;

        let manifest_parsed: CargoToml = toml::from_str(&manifest_bytes)
            .map_err(|_| syn::Error::new(Span::call_site(), "Failed to parse Cargo.toml"))?;

        if manifest_parsed.features.client.is_none() {
            return Err(syn::Error::new(
                Span::call_site(),
                "This crate doesn't define a `client` feature in its `Cargo.toml`.\n\
                 Please add a `client` feature such that generated `OutgoingRequest` and \
                 `IncomingResponse` implementations can be enabled.",
            ));
        }

        if manifest_parsed.features.server.is_none() {
            return Err(syn::Error::new(
                Span::call_site(),
                "This crate doesn't define a `server` feature in its `Cargo.toml`.\n\
                 Please add a `server` feature such that generated `IncomingRequest` and \
                 `OutgoingResponse` implementations can be enabled.",
            ));
        }

        Ok(())
    });

    RESULT.as_ref().err()
}
