use std::{fs, path::Path};

use crate::Result;

type VersionFn = fn(&str) -> bool;
const SPLITS: &[(&str, VersionFn)] = &[
    ("https://matrix.org/docs/spec/client_server/", |s| {
        // We cannot include the `#` because for every lib.rs file with spec docs the
        // URL is `../rx.x.x.html`
        s.starts_with("r0.6.1") || s.starts_with("unstable#")
    }),
    ("https://matrix.org/docs/spec/server_server/", |s| {
        s.starts_with("r0.1.4") || s.starts_with("unstable#")
    }),
    ("https://matrix.org/docs/spec/application_service/", |s| {
        s.starts_with("r0.1.2") || s.starts_with("unstable#")
    }),
    ("https://matrix.org/docs/spec/identity_service/", |s| {
        s.starts_with("r0.3.0") || s.starts_with("unstable#")
    }),
    ("https://matrix.org/docs/spec/push_gateway/", |s| {
        s.starts_with("r0.1.1") || s.starts_with("unstable#")
    }),
];

pub(crate) fn check_spec_links(path: &Path) -> Result<()> {
    // This is WAY overkill but since there are a few mixed in ruma-common
    // and this would catch any wrong version anywhere it's probably ok
    for (split, version_fn) in SPLITS {
        walk_dirs(path, split, *version_fn)?;
    }
    Ok(())
}

fn walk_dirs(path: &Path, split: &str, version_match: fn(&str) -> bool) -> Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                walk_dirs(&path, split, version_match)?;
            } else {
                let content = fs::read_to_string(&path)?;
                let split = content.split(split).collect::<Vec<_>>();

                match split.as_slice() {
                    // No spec link was found
                    // `[]` is only to satisfy exhaustiveness checking
                    [] | [_] => {}
                    [pre, post, rest @ ..] => {
                        if pre.lines().rev().next().map_or(false, |l| l.starts_with("//!"))
                            && !version_match(post)
                        {
                            return Err(format!(
                                "error: spec URL with no version number\nfrom: {}\n{}",
                                path.display(),
                                &post[0..25]
                            )
                            .into());
                        }

                        check_rest(rest, &path, version_match)?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn check_rest(rest: &[&str], path: &Path, version_match: fn(&str) -> bool) -> Result<()> {
    match rest {
        [] => Ok(()),
        [rest] => {
            if !version_match(rest) {
                Err(format!(
                    "error: spec URL with no version number\nfrom: {}\n{}",
                    path.display(),
                    &rest[0..25]
                )
                .into())
            } else {
                Ok(())
            }
        }
        [pre, post, rest @ ..] => {
            if pre.lines().rev().next().map_or(false, |l| l.starts_with("//!"))
                && !version_match(post)
            {
                return Err(format!(
                    "error: spec URL with no version number\nfrom: {}\n{}",
                    path.display(),
                    &post[0..25]
                )
                .into());
            }
            if !rest.is_empty() {
                check_rest(rest, path, version_match)?;
            }
            Ok(())
        }
    }
}
