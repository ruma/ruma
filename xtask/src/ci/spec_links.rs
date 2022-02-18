use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::Path,
};

use crate::Result;

type VersionFn = fn(&str) -> bool;
const SPLITS: &[(&str, VersionFn)] = &[
    ("https://matrix.org/docs/spec/client_server/", |_| false),
    ("https://matrix.org/docs/spec/server_server/", |_| false),
    ("https://matrix.org/docs/spec/application_service/", |_| false),
    ("https://matrix.org/docs/spec/identity_service/", |_| false),
    ("https://matrix.org/docs/spec/push_gateway/", |_| false),
    ("https://spec.matrix.org/", |s| {
        s.starts_with("v1.1") || s.starts_with("v1.2") || s.starts_with("unstable")
    }),
];

pub(crate) fn check_spec_links(path: &Path) -> Result<()> {
    println!("Checking all Matrix Spec links point to same version...");
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
                let mut buf = String::new();
                let mut content = BufReader::new(File::open(&path)?);

                // We can assume a spec link will never overflow to another line
                while content.read_line(&mut buf)? > 0 {
                    // If for some reason a line has 2 spec links
                    for (idx, _) in buf.match_indices(split) {
                        if !version_match(&buf[idx + split.len()..]) {
                            return err(&path, &buf);
                        }
                    }
                    buf.clear();
                }
            }
        }
    }
    Ok(())
}

fn err(path: &Path, snippet: &str) -> Result<()> {
    Err(format!(
        "error: spec URL with wrong version number\nfile: {}\n\nsnippet:\n{}",
        path.display(),
        &snippet[0..25]
    )
    .into())
}
