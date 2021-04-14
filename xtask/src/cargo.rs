use std::path::PathBuf;

use isahc::{HttpClient, ReadResponseExt};
use semver::{Version, VersionReq};
use serde::{de::IgnoredAny, Deserialize};
use serde_json::from_str as from_json_str;
use toml_edit::{value, Document};
use xshell::{cmd, pushd, read_file, write_file};

use crate::{util::ask_yes_no, Result};

const CRATESIO_API: &str = "https://crates.io/api/v1/crates";

/// The metadata of a cargo workspace.
#[derive(Clone, Debug, Deserialize)]
pub struct Metadata {
    pub workspace_root: PathBuf,
    pub packages: Vec<Package>,
}

impl Metadata {
    /// Load a new `Metadata` from the command line.
    pub fn load() -> Result<Metadata> {
        let metadata_json = cmd!("cargo metadata --no-deps --format-version 1").read()?;
        Ok(from_json_str(&metadata_json)?)
    }
}

/// A cargo package.
#[derive(Clone, Debug, Deserialize)]
pub struct Package {
    /// The package name
    pub name: String,

    /// The package version.
    pub version: Version,

    /// The package's manifest path.
    pub manifest_path: PathBuf,

    /// A map of the package dependencies.
    #[serde(default)]
    pub dependencies: Vec<Dependency>,
}

impl Package {
    /// Update the version of this crate.
    pub fn update_version(&mut self, version: &Version) -> Result<()> {
        println!("Updating {} to version {}…", self.name, version);

        let mut document = read_file(&self.manifest_path)?.parse::<Document>()?;

        document["package"]["version"] = value(version.to_string());

        write_file(&self.manifest_path, document.to_string())?;

        self.version = version.clone();

        Ok(())
    }

    /// Update the version of this crate in dependant crates' manifest, with the given version
    /// prefix.
    pub fn update_dependants(&self, packages: &[Package]) -> Result<()> {
        for package in
            packages.iter().filter(|p| p.dependencies.iter().any(|d| d.name == self.name))
        {
            println!("Updating dependency in {} crate…", package.name);

            let mut document = read_file(&package.manifest_path)?.parse::<Document>()?;

            for dependency in package.dependencies.iter().filter(|d| d.name == self.name) {
                let version = if dependency.req.is_exact() {
                    format!("={}", self.version,)
                } else {
                    self.version.to_string()
                };

                match dependency.kind {
                    Some(DependencyKind::Dev) => {
                        document["dev-dependencies"][&self.name]["version"] =
                            value(version.as_str());
                    }
                    None => {
                        document["dependencies"][&self.name]["version"] = value(version.as_str());
                    }
                    _ => {}
                }
            }

            if package
                .dependencies
                .iter()
                .any(|p| p.name == self.name && p.kind == Some(DependencyKind::Dev))
            {}

            write_file(&package.manifest_path, document.to_string())?;
        }

        Ok(())
    }

    /// Update the changelog for the release of the given version. Returns the changes for the
    /// version.
    pub fn update_changelog(&self) -> Result<String> {
        let mut changelog_path = self.manifest_path.clone();
        changelog_path.set_file_name("CHANGELOG.md");

        let changelog = read_file(&changelog_path)?;

        let title_start = match changelog
            .find(&format!("# {}\n", self.version))
            .or_else(|| changelog.find(&format!("# {} (unreleased)\n", self.version)))
            .or_else(|| changelog.find("# [unreleased]\n"))
        {
            Some(p) => p,
            None => {
                return Err("Could not find version title in changelog".into());
            }
        };

        let changes_start = match changelog[title_start..].find('\n') {
            Some(p) => title_start + p + 1,
            None => {
                return Err("Could not find end of version title in changelog".into());
            }
        };

        let changes_end = match changelog[changes_start..].find("\n# ") {
            Some(p) => changes_start + p,
            None => changelog.len(),
        };

        let changes = match changelog[changes_start..changes_end].trim() {
            s if s.is_empty() => "No changes for this version",
            s => s,
        };

        let changelog = format!(
            "{}# [unreleased]\n\n# {}\n\n{}\n{}",
            &changelog[..title_start],
            self.version,
            changes,
            &changelog[changes_end..]
        );

        write_file(&changelog_path, changelog)?;

        println!(
            "Changelog updated: title_start: {}, changes_start: {}, changes_end: {}\nchanges: {}",
            title_start, changes_start, changes_end, changes
        );

        Ok(changes.to_owned())
    }

    /// Check if the current version of the crate is published on crates.io.
    pub fn is_published(&self, client: &HttpClient) -> Result<bool> {
        let response: CratesIoCrate =
            client.get(format!("{}/{}/{}", CRATESIO_API, self.name, self.version))?.json()?;

        Ok(response.version.is_some())
    }

    /// Publish this package on crates.io.
    pub fn publish(&self, client: &HttpClient) -> Result<bool> {
        println!("Publishing {} {} on crates.io…", self.name, self.version);
        let _dir = pushd(&self.manifest_path.parent().unwrap())?;

        if self.is_published(client)? {
            if ask_yes_no("This version is already published. Skip this step and continue?")? {
                Ok(false)
            } else {
                Err("Release interrupted by user.".into())
            }
        } else {
            cmd!("cargo publish").run()?;
            Ok(true)
        }
    }
}

/// A cargo package dependency.
#[derive(Clone, Debug, Deserialize)]
pub struct Dependency {
    /// The package name.
    pub name: String,

    /// The version requirement for this dependency.
    pub req: VersionReq,

    /// The kind of the dependency.
    pub kind: Option<DependencyKind>,
}

/// The kind of a cargo package dependency.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DependencyKind {
    /// A dev dependency.
    Dev,

    /// A build dependency.
    Build,
}

/// A crate from the `GET /crates/{crate}` endpoint of crates.io.
#[derive(Deserialize)]
struct CratesIoCrate {
    version: Option<IgnoredAny>,
}
