#![allow(clippy::disallowed_types)]

use std::{
    collections::HashMap,
    fmt,
    ops::Deref,
    path::{Path, PathBuf},
};

use clap::{Args, Subcommand};
use html5gum::{Token, Tokenizer};
use reqwest::Url;
use xshell::Shell;

mod check;
mod update;

pub(crate) use self::check::SpecLinksCheckTask;
use self::update::SpecLinksUpdateTask;
use crate::{Metadata, Result};

#[derive(Args)]
pub(crate) struct SpecLinksArgs {
    #[clap(subcommand)]
    pub(crate) cmd: SpecLinksCmd,
}

#[derive(Subcommand)]
pub(crate) enum SpecLinksCmd {
    /// Check that the spec links use allowed URLs.
    Check,

    /// Update the spec links to the preferred spec version.
    ///
    /// The preferred spec version is defined by the `SpecLink::PREFERRED_VERSION` constant.
    Update,
}

/// Task to run CI tests.
pub(crate) struct SpecLinksTask {
    /// Which command to run.
    cmd: SpecLinksCmd,

    /// The metadata of the workspace.
    project_metadata: Metadata,
}

impl SpecLinksTask {
    pub(crate) fn new(cmd: SpecLinksCmd) -> Result<Self> {
        let sh = Shell::new()?;
        let project_metadata = Metadata::load(&sh)?;

        Ok(Self { cmd, project_metadata })
    }

    pub(crate) fn run(self) -> Result<()> {
        let crates_path = self.project_metadata.crates_path();

        match self.cmd {
            SpecLinksCmd::Check => SpecLinksCheckTask::new().run(&crates_path),
            SpecLinksCmd::Update => SpecLinksUpdateTask::new().run(&crates_path),
        }
    }
}

/// A link to the spec.
#[derive(Clone)]
struct SpecLink {
    /// The URL of the link.
    url: Url,

    /// The path of the file containing the link.
    path: PathBuf,

    /// The line in the file containing the link.
    line: u16,
}

impl SpecLink {
    /// The start of the URLs pointing to the spec.
    const URL_PREFIX: &str = "https://spec.matrix.org/";

    /// The version that spec links should use.
    const PREFERRED_VERSION: &str = "v1.18";

    /// URLs that are allowed to use a different spec version than
    /// [`PREFERRED_VERSION`](Self::PREFERRED_VERSION).
    ///
    /// This is either for URLs that are specifically attached to a Matrix spec version, or that
    /// were removed in recent spec versions.
    ///
    /// In this list `/*/` can be used as a wildcard to accept any content in a single path
    /// fragment.
    const URL_ALLOWLIST: &[&str] = &[
        // Links to specific Matrix versions.
        "https://spec.matrix.org/*/",
        "https://spec.matrix.org/historical/index.html#complete-list-of-room-versions",
        "https://spec.matrix.org/*/rooms/#complete-list-of-room-versions",
        // Links to removed sections.
        "https://spec.matrix.org/v1.15/client-server-api/#get_matrixclientv3profileuseridavatar_url",
        "https://spec.matrix.org/v1.15/client-server-api/#get_matrixclientv3profileuseriddisplayname",
        "https://spec.matrix.org/v1.15/client-server-api/#put_matrixclientv3profileuseridavatar_url",
        "https://spec.matrix.org/v1.15/client-server-api/#put_matrixclientv3profileuseriddisplayname",
        "https://spec.matrix.org/v1.17/server-server-api/#put_matrixfederationv1send_joinroomideventid",
        "https://spec.matrix.org/v1.17/server-server-api/#put_matrixfederationv1send_leaveroomideventid",
    ];

    /// Parse the given line for a spec link.
    ///
    /// Returns `Ok(None)` if no spec link was found, and `Err(_)` if a spec link was found but the
    /// URL could not be parsed.
    pub(crate) fn parse_line(string: &str, path: PathBuf, line: u16) -> Result<Option<Self>> {
        let Some(start_idx) = string.find(SpecLink::URL_PREFIX) else {
            return Ok(None);
        };

        let string = &string[start_idx..];

        // We can assume a link will end either:
        // - With a `)` (Markdown link syntax)
        // - With a `>` (<link> syntax)
        // - With a whitespace
        // - At the end of the line if none of the above is met
        let url = if let Some(end_idx) =
            string.find(|c: char| matches!(c, ')' | '>') || c.is_ascii_whitespace())
        {
            &string[..end_idx]
        } else {
            string
        };

        let url = Url::parse(url).inspect_err(|error| {
            Self::print_inner(url, &path, line, Some(&error));
        })?;

        let link = Self { url, path, line };

        if link.url.query().is_some() {
            let error = "Spec link can't contain a query component";
            link.print_err(error);
            return Err(error.into());
        }

        Ok(Some(link))
    }

    /// Print this link.
    fn print(&self) {
        Self::print_inner(self.url.as_str(), &self.path, self.line, None::<&str>);
    }

    /// Print an error with this link.
    fn print_err(&self, error: impl fmt::Display) {
        Self::print_inner(self.url.as_str(), &self.path, self.line, Some(error));
    }

    /// Print the given `SpecLink` fields with the given message before.
    fn print_inner(url: &str, path: &Path, line: u16, message: Option<impl fmt::Display>) {
        if let Some(message) = message {
            println!("{message}");
        }

        println!("  file: {}:{}\n  link: {}", path.display(), line, url.get(..120).unwrap_or(url));
    }

    /// Get the spec version of the link, if any.
    fn version(&self) -> Option<&str> {
        // The spec version is the first segment of the path.
        self.url.path_segments()?.next()
    }

    /// Whether this spec link is allowed.
    fn is_allowed(&self) -> bool {
        for allowed_url in Self::URL_ALLOWLIST {
            if allowed_url.contains('*') {
                if self.matches_wildcard_url(allowed_url) {
                    return true;
                }
            } else if self.url.as_str() == *allowed_url {
                return true;
            }
        }

        false
    }

    /// Check whether this spec link matches the given wildcard URL.
    fn matches_wildcard_url(&self, wildcard_url: &str) -> bool {
        let url = &self.url;
        let wildcard_url =
            Url::parse(wildcard_url).expect("URL in allowlist should be a valid URL");

        // We don't check the scheme and host since they have a fixed value, and we don't
        // check the query because there shouldn't be any.
        //
        // We start by the fragment because it's easier to check than the path and it is
        // more likely to differ between spec URLs.
        if url.fragment() != wildcard_url.fragment() {
            return false;
        }

        // Finally we check the path segments.
        let Some(path_segments) = url.path_segments() else {
            // If we have a wildcard URL, it should have path segments, so if this one doesn't they
            // don't match.
            return false;
        };

        let mut wildcard_path_segments =
            wildcard_url.path_segments().expect("a URL with a wildcard should have path segments");

        for segment in path_segments {
            if let Some(maybe_wildcard_segment) = wildcard_path_segments.next() {
                if maybe_wildcard_segment != "*" && segment != maybe_wildcard_segment {
                    // The path segments differ.
                    return false;
                }
            } else {
                // The spec link has more path segments than the wildcard URL.
                return false;
            }
        }

        // They have the same number of path segments and we already checked that they are
        // identical.
        wildcard_path_segments.next().is_none()
    }
}

/// A cache of spec page details.
#[derive(Default)]
struct PagesCache(HashMap<Url, Result<PageIds>>);

impl PagesCache {
    /// Get the IDs for the webpage matching the given [`SpecLink`].
    fn get_or_fetch_page_ids(&mut self, link: &SpecLink) -> &Result<PageIds> {
        // Remove the fragment so we only fetch the IDs once per page.
        let mut url = link.url.clone();
        url.set_fragment(None);

        self.0.entry(url).or_insert_with_key(|url| PageIds::fetch(url.clone()))
    }
}

/// The IDs in a spec page.
#[derive(Default)]
struct PageIds(HashMap<String, HasDuplicates>);

impl PageIds {
    /// Get the IDs in the given webpage.
    ///
    /// Returns an error if the URL points to an invalid HTML page.
    fn fetch(url: Url) -> Result<Self> {
        let page = reqwest::blocking::get(url)?;
        let html = page.text()?;

        let mut ids = Self::default();

        for Ok(token) in Tokenizer::new(&html) {
            if let Token::StartTag(tag) = token
                && let Some(id) = tag
                    .attributes
                    .get(b"id".as_slice())
                    .and_then(|s| String::from_utf8(s.0.clone()).ok())
            {
                ids.insert(id);
            }
        }

        Ok(ids)
    }

    /// Insert the given ID in this map while checking whether it's a duplicate.
    ///
    /// This check is necessary because duplicates IDs have a number depending on their occurrence
    /// in a HTML page. If a duplicate ID is added, moved or removed from the spec, its number might
    /// change from one version to the next.
    fn insert(&mut self, id: String) {
        // IDs that should be duplicates end with `-{number}`.
        let has_duplicates = if let Some(base_id) = id
            .rsplit_once('-')
            .and_then(|(start, end)| end.chars().all(|c| c.is_ascii_digit()).then_some(start))
        {
            // Update the first ID with the same base, because it doesn't end with a number.
            if let Some(base_id_value) = self.0.get_mut(base_id) {
                *base_id_value = HasDuplicates::Yes;
            }

            HasDuplicates::Yes
        } else {
            HasDuplicates::No
        };

        self.0.insert(id, has_duplicates);
    }
}

impl Deref for PageIds {
    type Target = HashMap<String, HasDuplicates>;

    fn deref(&self) -> &Self::Target {
        &self.0
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
