use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::Path,
};

use crate::Result;

pub(crate) fn check_spec_links(path: &Path) -> Result<()> {
    println!("Checking Matrix Spec links are up-to-date...");
    walk_dirs(path, "https://matrix.org/docs/spec/", |_| false)?;
    walk_dirs(path, "https://spec.matrix.org/", |s| {
        s.starts_with("v1.1") || s.starts_with("v1.2") || s.starts_with("unstable")
    })?;
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
