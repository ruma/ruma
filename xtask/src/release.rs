use std::path::Path;

use itertools::Itertools;
use serde::Deserialize;
use serde_json::json;
use toml::from_str as from_toml_str;
use xshell::{cmd, pushd, read_file};

use crate::{config, flags, project_root, Result};

const GITHUB_API_RELEASES: &str = "https://api.github.com/repos/ruma/ruma/releases";

impl flags::Release {
    /// Run the release command to effectively create a release.
    pub(crate) fn run(self) -> Result<()> {
        let project_root = &project_root()?;
        let _dir = pushd(project_root.join(&self.name))?;

        let remote = &self.get_remote()?;

        if !cmd!("git status -s -uno").read()?.is_empty() {
            return Err("This git repository contains untracked files".into());
        }

        let version = &self.get_version(project_root)?;
        println!("Making release for {} {}…", self.name, version);

        cmd!("cargo publish").run()?;

        let credentials = &config()?.github.credentials();

        let changes = &self.get_changes(project_root, &version)?;

        let tag = &format!("{}-{}", self.name, version);
        let name = &format!("{} {}", self.name, version);

        cmd!("git tag -s {tag} -m {name} -m {changes}").secret(true).run()?;

        cmd!("git push {remote} {tag}").run()?;

        let request_body = &json!({
            "tag_name": tag,
            "name": name,
            "body": changes.trim_softbreaks(),
        })
        .to_string();

        cmd!(
            "curl -u {credentials} -X POST -H 'Accept: application/vnd.github.v3+json'
            {GITHUB_API_RELEASES} -d {request_body}"
        )
        .secret(true)
        .run()?;

        Ok(())
    }

    /// Get the changes of the given version from the changelog.
    fn get_changes(&self, project_root: &Path, version: &str) -> Result<String> {
        let changelog = read_file(project_root.join(&self.name).join("CHANGELOG.md"))?;
        let lines_nb = changelog.lines().count();
        let mut lines = changelog.lines();

        let start = match lines.position(|l| l.starts_with(&format!("# {}", version))) {
            Some(p) => p + 1,
            None => {
                return Err("Could not find version title in changelog".into());
            }
        };

        let length = match lines.position(|l| l.starts_with("# ")) {
            Some(p) => p,
            None => lines_nb,
        };

        let changes = changelog.lines().skip(start).take(length).join("\n");

        Ok(changes.trim().to_owned())
    }

    /// Load the GitHub config from the config file.
    fn get_remote(&self) -> Result<String> {
        let branch = cmd!("git rev-parse --abbrev-ref HEAD").read()?;
        let remote = cmd!("git config branch.{branch}.remote").read()?;

        if remote.is_empty() {
            return Err("Could not get current git remote".into());
        }

        Ok(remote)
    }

    /// Get the current version of the crate from the manifest.
    fn get_version(&self, project_root: &Path) -> Result<String> {
        let manifest_toml = read_file(project_root.join(&self.name).join("Cargo.toml"))?;
        let manifest: CargoManifest = from_toml_str(&manifest_toml)?;

        Ok(manifest.package.version)
    }
}

/// The required cargo manifest data of a crate.
#[derive(Debug, Deserialize)]
struct CargoManifest {
    /// The package information.
    package: CargoPackage,
}

/// The required package information from a crate's cargo manifest.
#[derive(Debug, Deserialize)]
struct CargoPackage {
    /// The package version.
    version: String,
}

/// String manipulations for crate release.
trait StrExt {
    /// Remove soft line breaks as defined in CommonMark spec.
    fn trim_softbreaks(&self) -> String;
}

impl StrExt for str {
    fn trim_softbreaks(&self) -> String {
        let mut string = String::new();
        let mut s = self;

        while let Some(pos) = s.find('\n') {
            string.push_str(&s[..pos]);
            let pos_s = &s[pos..];

            if pos_s.starts_with("\n\n") {
                // Keep new paragraphs (multiple `\n`s).
                let next = pos_s.find(|c: char| c != '\n').unwrap_or(0);
                let (push, next_s) = pos_s.split_at(next);

                string.push_str(push);
                s = next_s;
            } else if s[..pos].ends_with("  ") || s[..pos].ends_with('\\') {
                // Keep hard line breaks (two spaces or a backslash before the line break).
                string.push('\n');
                s = &pos_s[1..];
            } else if let Some(p) = pos_s.find(|c: char| !c.is_ascii_whitespace()) {
                // Keep line break before list items (`\n` + whitespaces + `*` + whitespaces).
                // Remove line break and keep one space otherwise.
                let mut chars = pos_s.char_indices();
                let (_, char) = chars.find(|(i, _)| *i == p).unwrap();

                if char == '*' || char == '-' {
                    match chars.next() {
                        Some((_, next_char)) if next_char.is_ascii_whitespace() => {
                            string.push('\n');
                            s = &pos_s[1..];
                            continue;
                        }
                        _ => {}
                    }
                }

                string.push(' ');
                s = &pos_s[p..];
            }
        }

        string + s
    }
}
