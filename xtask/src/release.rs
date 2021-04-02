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
        let _dir = pushd(project_root()?.join(&self.name))?;

        let remote = &self.get_remote()?;

        if !cmd!("git status -s -uno").read()?.is_empty() {
            Err("This git repository contains untracked files")?
        }

        let version = &self.get_version()?;
        println!("Making release for {} {}…", self.name, version);

        cmd!("cargo publish").run()?;

        let credentials = &config()?.github.credentials();

        let changes = &self.get_changes(&version)?;

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
    fn get_changes(&self, version: &str) -> Result<String> {
        let changelog = read_file(project_root()?.join(&self.name).join("CHANGELOG.md"))?;
        let lines_nb = changelog.lines().count();
        let mut lines = changelog.lines();

        let start = match lines.position(|l| l.starts_with(&format!("# {}", version))) {
            Some(p) => p + 1,
            None => {
                return Err("Could not find version title in changelog")?;
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
            Err("Could not get current git remote")?
        }

        Ok(remote)
    }

    /// Get the current version of the crate from the manifest.
    fn get_version(&self) -> Result<String> {
        let manifest_toml = read_file(project_root()?.join(&self.name).join("Cargo.toml"))?;
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
    /// Remove soft line breaks.
    fn trim_softbreaks(&self) -> String;
}

impl StrExt for str {
    fn trim_softbreaks(&self) -> String {
        let mut string = "".to_owned();
        let mut current = 0;
        let mut chars = self.char_indices();

        while let Some(p) = self[current..].find('\n') {
            let pos = current + p;
            string.push_str(&self[current..pos]);
            // Keep hard line breaks (multiple `\n`s).
            if self[pos..].starts_with("\n\n") {
                let next = self[pos..].find(|c: char| c != '\n').unwrap_or(0);
                string.push_str(&self[pos..(pos + next)]);
                current = pos + next;
            } else if let Some(p) = self[pos..].find(|c: char| !c.is_ascii_whitespace()) {
                // Keep line break before list items (`\n` + whitespaces + `*` + whitespaces).
                // Remove line break and keep one whitespace otherwise.
                let (_, char) = chars.find(|(i, _)| *i == pos + p).unwrap();
                if char == '*' {
                    match chars.next() {
                        Some((_, next_char)) if next_char.is_ascii_whitespace() => {
                            string.push('\n');
                            current = pos + 1;
                            continue;
                        }
                        _ => {}
                    }
                }
                string.push(' ');
                current = pos + p
            }
        }

        string.push_str(&self[(current..self.len())]);

        string
    }
}
