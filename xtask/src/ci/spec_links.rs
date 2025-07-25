#![allow(clippy::disallowed_types)]

use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufRead, BufReader, ErrorKind},
    path::{Path, PathBuf},
};

use html5gum::{Token, Tokenizer};

use crate::Result;

/// Authorized URLs pointing to the old specs.
const OLD_URL_WHITELIST: &[&str] =
    &["https://spec.matrix.org/historical/index.html#complete-list-of-room-versions"];

/// Authorized versions in URLs pointing to the new specs.
const NEW_VERSION_WHITELIST: &[&str] = &[
    "v1.1", "v1.2", "v1.3", "v1.4", "v1.5", "v1.6", "v1.7", "v1.8", "v1.9", "v1.10", "v1.11",
    "v1.12", "v1.13", "v1.14", "v1.15",
    "latest",
    // This should only be enabled if a legitimate use case is found.
    // "unstable",
];

/// The version of URLs pointing to the old spec.
const OLD_VERSION: &str = "historical";

/// The start of the URLs pointing to the spec.
const URL_PREFIX: &str = "https://spec.matrix.org/";

/// A link to the spec.
struct SpecLink {
    /// The URL of the link.
    url: String,

    /// The path of the file containing the link.
    path: PathBuf,

    /// The line in the file containing the link.
    line: u16,
}

impl SpecLink {
    /// Create a new `SpecLink`.
    fn new(url: String, path: PathBuf, line: u16) -> Self {
        Self { url, path, line }
    }
}

/// Whether an ID has duplicates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HasDuplicates {
    /// The ID has duplicates.
    Yes,
    /// The ID doesn't have duplicates.
    No,
}

pub(crate) fn check_spec_links(path: &Path) -> Result<()> {
    println!("Checking Matrix spec links are up-to-date...");
    let links = collect_links(path)?;

    check_whitelist(&links)?;
    check_targets(&links)?;

    Ok(())
}

/// Collect the spec links under `path` starting with one of the URL starts.
///
/// Returns a list of tuples with the link and the file that contains it.
fn collect_links(path: &Path) -> Result<Vec<SpecLink>> {
    let mut links = Vec::new();

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            links.extend(collect_links(&entry.path())?);
        }
    } else {
        let mut buf = String::new();
        let mut line: u16 = 0;
        let mut content = BufReader::new(File::open(path)?);

        // We can assume a spec link will never overflow to another line.
        loop {
            match content.read_line(&mut buf) {
                Ok(read) => {
                    if read == 0 {
                        break;
                    }
                }
                Err(err) => {
                    if err.kind() == ErrorKind::InvalidData {
                        // The content is not UTF-8 text, skip.
                        break;
                    } else {
                        return Err(err.into());
                    }
                }
            }

            line += 1;

            // If for some reason a line has 2 spec links.
            for (start_idx, _) in buf.match_indices(URL_PREFIX) {
                links.push(SpecLink::new(get_full_link(&buf[start_idx..]), path.to_owned(), line));
            }

            buf.clear();
        }
    }

    Ok(links)
}

/// Get the full link starting at beginning of the given string.
fn get_full_link(s: &str) -> String {
    // We can assume a link will end either:
    // - With a `)` (Markdown link syntax)
    // - With a `>` (<link> syntax)
    // - With a whitespace
    // - At the end of the line if none of the above is met
    if let Some(end_idx) = s.find(|c: char| matches!(c, ')' | '>') || c.is_ascii_whitespace()) {
        s[..end_idx].to_owned()
    } else {
        s.to_owned()
    }
}

/// Check if links are in the whitelists.
///
/// Don't stop at the first error to be able to fix all the wrong links once.
fn check_whitelist(links: &[SpecLink]) -> Result<()> {
    let mut err_nb: u16 = 0;

    for link in links {
        let url_without_prefix = &link.url[URL_PREFIX.len()..];

        if url_without_prefix.starts_with(&format!("{OLD_VERSION}/")) {
            // Only old spec links in the whitelist are allowed.
            if !OLD_URL_WHITELIST.contains(&link.url.as_str()) {
                err_nb += 1;
                print_link_err("Old spec link not in whitelist", link);
            }
        } else if !NEW_VERSION_WHITELIST
            .iter()
            .any(|version| url_without_prefix.starts_with(&format!("{version}/")))
        {
            err_nb += 1;
            print_link_err("New spec link with wrong version", link);
        }
    }

    if err_nb > 0 {
        // Visual aid to separate the end error message.
        println!();
        return Err(format!("Found {err_nb} invalid spec links.").into());
    }

    Ok(())
}

/// Check if the URL points to a valid page and a valid fragment.
fn check_targets(links: &[SpecLink]) -> Result<()> {
    // Map page URL => IDs in the page.
    let mut page_cache: HashMap<String, Result<HashMap<String, HasDuplicates>>> = HashMap::new();
    let mut err_nb: u16 = 0;

    for link in links {
        let (url, fragment) =
            link.url.split_once('#').map(|(u, f)| (u, Some(f))).unwrap_or((&link.url, None));

        match page_cache.entry(url.to_owned()).or_insert_with(|| get_page_ids(url)) {
            Ok(ids) => {
                if let Some(fragment) = fragment {
                    if let Some(has_duplicates) = ids.get(fragment) {
                        // Don't allow links to the latest spec with duplicate IDs, they might point
                        // to another part of the spec in a new version.
                        if *has_duplicates == HasDuplicates::Yes
                            && link.url[URL_PREFIX.len()..].starts_with("latest/")
                        {
                            err_nb += 1;
                            print_link_err("Spec link to latest version with non-unique ID", link);
                        }
                    } else {
                        err_nb += 1;
                        print_link_err("Spec link with wrong fragment", link);
                    }
                }
            }
            Err(err) => {
                err_nb += 1;
                print_link_err(&format!("Spec link with wrong URL: {err}"), link);
            }
        }
    }

    if err_nb > 0 {
        // Visual aid to separate the end error message.
        println!();
        return Err(format!("Found {err_nb} invalid spec links.").into());
    }

    Ok(())
}

/// Get the IDs in the given webpage.
///
/// Returns an error if the URL points to an invalid HTML page.
fn get_page_ids(url: &str) -> Result<HashMap<String, HasDuplicates>> {
    let page = reqwest::blocking::get(url)?;

    let html = page.text()?;
    let mut ids = HashMap::new();

    for Ok(token) in Tokenizer::new(&html) {
        let Token::StartTag(tag) = token else {
            continue;
        };

        // For the URLs using the "latest" version, log the actual version we got.
        if url[URL_PREFIX.len()..].starts_with("latest/") {
            // Let's use the `meta` element with the `og:url` property, it contains the original
            // relative URL of the page.
            if tag.name.0 == b"meta"
                && tag
                    .attributes
                    .get(b"property".as_slice())
                    .is_some_and(|value| value.0 == b"og:url")
            {
                match tag.attributes.get(b"content".as_slice()) {
                    Some(value) => {
                        println!(
                            "Original URL for latest spec page: {}",
                            String::from_utf8_lossy(value)
                        );
                    }
                    None => println!(
                        "Could not get original URL for latest spec page: /{}",
                        &url[URL_PREFIX.len()..]
                    ),
                }
            }
        }

        let Some(id) =
            tag.attributes.get(b"id".as_slice()).and_then(|s| String::from_utf8(s.0.clone()).ok())
        else {
            continue;
        };

        let has_duplicates = heading_id_has_duplicates(&id, &mut ids);

        ids.insert(id, has_duplicates);
    }

    Ok(ids)
}

/// Check whether the given heading ID has duplicates in the given map.
///
/// This check is necessary because duplicates IDs have a number depending on their occurrence in a
/// HTML page. If a duplicate ID is added, moved or removed from the spec, its number might change
/// from one version to the next.
fn heading_id_has_duplicates(
    id: &str,
    unique_ids: &mut HashMap<String, HasDuplicates>,
) -> HasDuplicates {
    // IDs that should be duplicates end with `-{number}`.
    let Some((start, _end)) =
        id.rsplit_once('-').filter(|(_start, end)| end.chars().all(|c| c.is_ascii_digit()))
    else {
        return HasDuplicates::No;
    };

    // Update the first duplicate ID, because it doesn't end with a number.
    if let Some(other_id_has_dup) = unique_ids.get_mut(start) {
        *other_id_has_dup = HasDuplicates::Yes;
    }

    HasDuplicates::Yes
}

fn print_link_err(error: &str, link: &SpecLink) {
    println!(
        "\n{error}\n  file: {}:{}\n  link: {}",
        link.path.display(),
        link.line,
        link.url.get(..80).unwrap_or(&link.url),
    );
}
