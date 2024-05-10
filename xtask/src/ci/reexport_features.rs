use crate::{Metadata, Result};

/// Check that the ruma crate allows to enable all the features of the other ruma-* crates.
///
/// For simplicity, this function assumes that:
///
/// - Those dependencies are not renamed.
/// - ruma does not use `default-features = false` on those dependencies.
///
/// This does not check if all features are re-exported individually, as that is not always wanted.
pub(crate) fn check_reexport_features(metadata: &Metadata) -> Result<()> {
    println!("Checking all features can be enabled from ruma…");
    let mut n_errors = 0;

    let Some(ruma) = metadata.find_package("ruma") else {
        return Err("ruma package not found in workspace".into());
    };

    for package in ruma.dependencies.iter().filter_map(|dep| metadata.find_package(&dep.name)) {
        println!("Checking features of {}…", package.name);

        // Exclude ruma and xtask.
        if !package.name.starts_with("ruma-") {
            continue;
        }

        // Filter features that are enabled by other features of the same package.
        let features = package.features.keys().filter(|feature_name| {
            !package.features.values().flatten().any(|activated_feature| {
                activated_feature.trim_start_matches("dep:") == *feature_name
            })
        });

        for feature_name in features {
            // Let's assume that ruma never has `default-features = false`.
            if feature_name == "default" {
                continue;
            }

            if !ruma.can_enable_feature(&package.name, feature_name) {
                println!(r#"  Missing feature "{}/{feature_name}""#, package.name);
                n_errors += 1;
            }
        }
    }

    if n_errors > 0 {
        // Visual aid to separate the end error message.
        println!();
        return Err(format!("Found {n_errors} missing features").into());
    }

    Ok(())
}
