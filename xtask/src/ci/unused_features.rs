use xshell::Shell;

use crate::{cmd, Metadata, Result};

/// List of features that trigger false positives.
const ALLOW_LIST: &[&str] = &[
    // Convenience feature to enable several MSC features at once.
    "unstable-extensible-events",
    // Features from the API macros, they are mostly found in macro-generated code.
    "client",
    "server",
];

/// Check that all cargo features are used.
///
/// We attempt a simplistic approach here so we don't need to work on generated code.
///
/// A cargo feature is deemed used in one of those cases:
///
/// - If it is an `unstable-*` cargo feature, it enables the same cargo feature in a `ruma-*`
///   dependency, because the feature will be checked in that crate. Unstable features have a
///   stricter rule because we assume that they are always used to trigger a different behavior
///   *inside* this workspace. This criterion can be relaxed if there is an actual use for a
///   different behavior *outside* this workspace.
/// - If it is another cargo feature, it enables a dependency or a cargo feature in a dependency.
/// - We can find the cargo feature name in a Rust file in the `src` directory of the crate with a
///   string search.
///
/// Requires `grep` to be available on the machine.
pub(crate) fn check_unused_features(sh: &Shell, metadata: &Metadata) -> Result<()> {
    println!("Checking all unused cargo features…");
    let mut n_errors = 0;

    for package in &metadata.packages {
        println!("Checking unused features of {}…", package.name);

        // Exclude xtask.
        if !package.name.starts_with("ruma") {
            continue;
        }

        // Set the shell in the current crate's directory.
        let _push_dir =
            sh.push_dir(package.manifest_path.parent().expect("manifest path is in a directory"));

        for (feature_name, dep_features) in &package.features {
            if ALLOW_LIST.contains(&feature_name.as_str()) {
                continue;
            }

            let is_unstable = feature_name.starts_with("unstable-");

            // If the feature is unstable and it enables a cargo feature of the same name in a
            // `ruma-*` dependency, we will check the feature in that dependency.
            if is_unstable
                && dep_features.iter().any(|dep_feature| {
                    dep_feature.starts_with("ruma-") && dep_feature.ends_with(feature_name)
                })
            {
                continue;
            }

            // If it enables a dependency or a feature in a dependency, we assume that it's ok.
            if !is_unstable && !dep_features.is_empty() {
                continue;
            }

            // Try to find uses of the feature with a string search, recursively inside Rust files
            // in the `src` directory of the crate. If none is found, grep will return an exit code
            // 1.
            let res = cmd!(sh, "grep --quiet --recursive --include '*.rs' --fixed-strings")
                // Do the string substitution with format! because for some reason it does not work
                // with cmd!.
                .args([&format!(r#"feature = "{feature_name}""#), "src"])
                .quiet()
                .run();

            if res.is_err() {
                println!("Feature {feature_name} is unused");
                n_errors += 1;
            }
        }
    }

    if n_errors > 0 {
        // Visual aid to separate the end error message.
        println!();
        return Err(format!("Found {n_errors} unused features").into());
    }

    Ok(())
}
