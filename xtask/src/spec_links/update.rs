use std::{fs, io::ErrorKind, path::Path};

use super::{HasDuplicates, PagesCache, SpecLink};
use crate::Result;

/// Task to update spec links.
pub(super) struct SpecLinksUpdateTask {
    /// The webpages cache.
    pages_cache: PagesCache,

    /// The number of links that were updated.
    updated_count: u32,

    /// The spec links that were replaced that the user should check because the fragment has
    /// duplicates and it may point to the wrong part of the spec.
    links_to_check: Vec<SpecLink>,

    /// The number of errors that were encountered.
    error_count: u32,
}

impl SpecLinksUpdateTask {
    /// Construct a new `SpecLinksUpdateTask`.
    pub(super) fn new() -> Self {
        Self {
            pages_cache: PagesCache::default(),
            updated_count: 0,
            links_to_check: Vec::new(),
            error_count: 0,
        }
    }

    /// Run the `SpecLinksUpdateTask`.
    pub(crate) fn run(mut self, path: &Path) -> Result<()> {
        self.update_path(path);

        if self.updated_count > 0 {
            println!("\nUpdated {} spec links.", self.updated_count);
        }

        if !self.links_to_check.is_empty() {
            println!(
                "\nCheck that the following links still point to the correct part of the spec:"
            );

            for link in self.links_to_check {
                println!();
                link.print();
            }
        }

        if self.error_count > 0 {
            // Visual aid to separate the error message.
            println!();
            return Err(format!("Encountered {} errors.", self.error_count).into());
        }

        Ok(())
    }

    /// Walk the given path recursively an attempt to update the encountered spec links.
    fn update_path(&mut self, path: &Path) {
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

                self.update_path(&entry.path());
            }
        } else {
            self.update_file(path);
        }
    }

    /// Update the spec links in the given file, if any.
    fn update_file(&mut self, path: &Path) {
        let mut content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(error) => {
                if error.kind() == ErrorKind::InvalidData {
                    // The content is not UTF-8 text, skip the file.
                } else {
                    println!("Could not read file `{}`: {error}", path.display());
                    self.error_count += 1;
                }

                return;
            }
        };

        let mut replacements = Vec::new();

        // We assume that there will be at most one spec link per line and the a link will never
        // break to another line, so we read the file line by line.
        for (line, string) in content.lines().enumerate() {
            let Ok(link) = SpecLink::parse_line(string, path.to_owned(), line as u16 + 1) else {
                self.error_count += 1;
                continue;
            };

            if let Some(link) = link {
                replacements.extend(self.maybe_replace_link(link));
            }
        }

        if !replacements.is_empty() {
            let replacement_count = replacements.len() as u32;

            for replacement in replacements {
                content = content
                    .replace(replacement.original.url.as_str(), replacement.new.url.as_str());
            }

            match fs::write(path, content) {
                Ok(()) => {
                    self.updated_count += replacement_count;
                }
                Err(error) => {
                    println!("Could not write to file `{}`: {error}", path.display());
                    self.error_count += 1;
                }
            }
        }
    }

    /// Check whether the given spec link needs to be replaced.
    ///
    /// Returns a `Replacement` if the link needs to be replaced.
    fn maybe_replace_link(&mut self, original: SpecLink) -> Option<Replacement> {
        // If the URL is allowed, no need to update it.
        if original.is_allowed() {
            return None;
        }

        // If there is no version or it already uses the preferred version, no need to update it.
        let from_version = original
            .version()
            .filter(|from_version| *from_version != SpecLink::PREFERRED_VERSION)?;

        // Build the new URL.
        let mut new = original.clone();
        new.url.set_path(&new.url.path().replacen(from_version, SpecLink::PREFERRED_VERSION, 1));

        // Check if the new URL exists.
        let page_ids = self
            .pages_cache
            .get_or_fetch_page_ids(&new)
            .as_ref()
            .inspect_err(|error| {
                original
                    .print_err(format_args!("Can't update spec link to `{}`: {error}", new.url));
                self.error_count += 1;
            })
            .ok()?;

        if let Some(fragment) = new.url.fragment() {
            let Some(has_duplicates) = page_ids.get(fragment) else {
                original.print_err(format_args!(
                    "Can't update spec link to `{}`: fragment doesn't exist",
                    new.url
                ));
                self.error_count += 1;

                return None;
            };

            if matches!(has_duplicates, HasDuplicates::Yes) {
                // The user will need to check manually that the link still uses the correct
                // fragment.
                self.links_to_check.push(new.clone());
            }
        }

        Some(Replacement { original, new })
    }
}

/// Data to replace a spec link.
struct Replacement {
    /// The original spec link.
    original: SpecLink,

    /// The new spec link.
    new: SpecLink,
}
