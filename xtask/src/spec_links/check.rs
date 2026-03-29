use std::{
    fs::{self, File},
    io::{BufRead, BufReader, ErrorKind},
    path::Path,
};

use super::{PagesCache, SpecLink};
use crate::Result;

/// Task to check that spec links are valid.
#[derive(Default)]
pub(crate) struct SpecLinksCheckTask {
    /// The spec links that were found.
    links: Vec<SpecLink>,

    /// The number of errors that were encountered.
    error_count: u32,
}

impl SpecLinksCheckTask {
    /// Construct a new `SpecLinksCheckTask`.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Run this task at the given path.
    pub(crate) fn run(mut self, path: &Path) -> Result<()> {
        println!("Checking Matrix spec links are up-to-date...");
        self.collect_links(path);

        self.check_allowlists();
        self.check_fragments();

        if self.error_count > 0 {
            // Visual aid to separate the error message.
            println!();
            return Err(format!("Encountered {} errors.", self.error_count).into());
        }

        Ok(())
    }

    /// Collect the spec links under `path`.
    fn collect_links(&mut self, path: &Path) {
        if path.is_dir() {
            let read_dir = match fs::read_dir(path) {
                Ok(read_dir) => read_dir,
                Err(error) => {
                    println!("Could not read directory `{}`: {error}", path.display());
                    self.error_count += 1;
                    return;
                }
            };

            for entry in read_dir {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(error) => {
                        println!("Could not read entry in directory `{}`: {error}", path.display());
                        self.error_count += 1;
                        continue;
                    }
                };

                self.collect_links(&entry.path());
            }
        } else {
            self.collect_file_links(path);
        }
    }

    /// Collect the spec links in the file at `path`.
    fn collect_file_links(&mut self, path: &Path) {
        let mut content = match File::open(path) {
            Ok(file) => BufReader::new(file),
            Err(error) => {
                println!("Could not open file `{}`: {error}", path.display());
                self.error_count += 1;
                return;
            }
        };

        let mut buf = String::new();
        let mut line: u16 = 0;

        // We assume that there will be at most one spec link per line and the a link will never
        // break to another line, so we read the file line by line.
        loop {
            match content.read_line(&mut buf) {
                Ok(read) => {
                    if read == 0 {
                        // We are at the end of the file.
                        break;
                    }
                }
                Err(error) => {
                    // InvalidData means that the content is not UTF-8 so we can just ignore the
                    // file.
                    if error.kind() != ErrorKind::InvalidData {
                        println!("Could not read file `{}`: {error}", path.display());
                        self.error_count += 1;
                    }

                    break;
                }
            }

            line += 1;

            match SpecLink::parse_line(&buf, path.to_owned(), line) {
                Ok(link) => {
                    self.links.extend(link);
                }
                Err(_) => {
                    self.error_count += 1;
                }
            }

            buf.clear();
        }
    }

    /// Check if links are in the allowlists.
    ///
    /// Don't stop at the first error to be able to fix all the wrong links once.
    fn check_allowlists(&mut self) {
        for link in &self.links {
            if link.is_allowed() {
                continue;
            }

            if link.version().is_some_and(|version| version != SpecLink::PREFERRED_VERSION) {
                link.print_err("Spec link with unauthorized URL");
                self.error_count += 1;
            }
        }
    }

    /// Check if the spec links point to valid pages and fragments.
    fn check_fragments(&mut self) {
        let mut pages_cache = PagesCache::default();

        for link in &self.links {
            match pages_cache.get_or_fetch_page_ids(link) {
                Ok(ids) => {
                    if link.url.fragment().is_some_and(|fragment| !ids.contains_key(fragment)) {
                        link.print_err("Spec link with invalid fragment");
                        self.error_count += 1;
                    }
                }
                Err(err) => {
                    link.print_err(format_args!("Spec link with invalid URL: {err}"));
                    self.error_count += 1;
                }
            }
        }
    }
}
