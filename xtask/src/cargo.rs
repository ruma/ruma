use std::path::PathBuf;

use isahc::{HttpClient, ReadResponseExt};
use semver::Version;
use serde::{de::IgnoredAny, Deserialize};
use toml_edit::{value, Document};
use xshell::{cmd, pushd, read_file, write_file};

use crate::{util::ask_yes_no, Metadata, Result};

const CRATESIO_API: &str = "https://crates.io/api/v1/crates";

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
    pub fn update_version(&mut self, version: &Version, dry_run: bool) -> Result<()> {
        println!("Updating {} to version {}…", self.name, version);

        if !dry_run {
            let mut document = read_file(&self.manifest_path)?.parse::<Document>()?;

            document["package"]["version"] = value(version.to_string());

            write_file(&self.manifest_path, document.to_string())?;
        }

        self.version = version.clone();

        Ok(())
    }

    /// Update the version of this crate in dependant crates' manifests, with the given version
    /// prefix.
    pub(crate) fn update_dependants(&self, metadata: &Metadata, dry_run: bool) -> Result<()> {
        for package in metadata.packages.iter().filter(|p| {
            p.manifest_path.starts_with(&metadata.workspace_root)
                && p.dependencies.iter().any(|d| d.name == self.name)
        }) {
            println!("Updating dependency in {} crate…", package.name);

            if !dry_run {
                let mut document = read_file(&package.manifest_path)?.parse::<Document>()?;

                let version = if !self.version.pre.is_empty()
                    || self.name.strip_suffix("-macros") == Some(&package.name)
                {
                    format!("={}", self.version)
                } else {
                    self.version.to_string()
                };

                for dependency in package.dependencies.iter().filter(|d| d.name == self.name) {
                    let kind = match dependency.kind {
                        Some(DependencyKind::Dev) => "dev-dependencies",
                        Some(DependencyKind::Build) => "build-dependencies",
                        None => "dependencies",
                    };

                    document[kind][&self.name]["version"] = value(version.as_str());
                }

                write_file(&package.manifest_path, document.to_string())?;
            }
        }

        Ok(())
    }

    /// Get the changes for the version.
    ///
    /// If `update` is `true`, update the changelog for the release of the given version.
    pub fn changes(&self, update: bool) -> Result<String> {
        let mut changelog_path = self.manifest_path.clone();
        changelog_path.set_file_name("CHANGELOG.md");

        let changelog = read_file(&changelog_path)?;
        let version = Version {
            pre: semver::Prerelease::EMPTY,
            build: semver::BuildMetadata::EMPTY,
            ..self.version.clone()
        };

        let update = if changelog.starts_with(&format!("# {}\n", version)) {
            false
        } else if changelog.starts_with(&format!("# {} (unreleased)\n", version))
            || changelog.starts_with("# [unreleased]\n")
        {
            update
        } else {
            return Err("Could not find version title in changelog".into());
        };

        let changes_start = match changelog.find('\n') {
            Some(p) => p + 1,
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

        if update {
            let changelog = format!(
                "# [unreleased]\n\n# {}\n\n{}\n{}",
                self.version,
                changes,
                &changelog[changes_end..]
            );

            write_file(&changelog_path, changelog)?;
        }

        Ok(changes.to_owned())
    }

    /// Check if the current version of the crate is published on crates.io.
    pub fn is_published(&self, client: &HttpClient) -> Result<bool> {
        let response: CratesIoCrate =
            client.get(format!("{}/{}/{}", CRATESIO_API, self.name, self.version))?.json()?;

        Ok(response.version.is_some())
    }

    /// Publish this package on crates.io.
    pub fn publish(&self, client: &HttpClient, dry_run: bool) -> Result<bool> {
        println!("Publishing {} {} on crates.io…", self.name, self.version);
        let _dir = pushd(&self.manifest_path.parent().unwrap())?;

        if self.is_published(client)? {
            if ask_yes_no("This version is already published. Skip this step and continue?")? {
                Ok(false)
            } else {
                Err("Release interrupted by user.".into())
            }
        } else {
            if !dry_run {
                cmd!("cargo publish").run()?;
            }
            Ok(true)
        }
    }
}

/// A cargo package dependency.
#[derive(Clone, Debug, Deserialize)]
pub struct Dependency {
    /// The package name.
    pub name: String,

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
