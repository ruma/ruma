use std::{
    path::{Path, PathBuf},
    thread::sleep,
    time::Duration,
};

use isahc::{
    auth::{Authentication, Credentials},
    config::Configurable,
    http::StatusCode,
    HttpClient, ReadResponseExt, Request,
};
use itertools::Itertools;
use semver::Version;
use serde::{de::IgnoredAny, Deserialize};
use serde_json::json;
use toml::from_str as from_toml_str;
use xshell::{pushd, read_file};

use crate::{cmd, util::ask_yes_no, GithubConfig, Result};

const CRATESIO_API: &str = "https://crates.io/api/v1/crates";
const GITHUB_API_RUMA: &str = "https://api.github.com/repos/ruma/ruma";

/// Task to create a new release of the given crate.
#[derive(Debug)]
pub struct ReleaseTask {
    /// The crate to release.
    local_crate: LocalCrate,

    /// The root of the workspace.
    project_root: PathBuf,

    /// The http client to use for requests.
    http_client: HttpClient,

    /// The github configuration required to publish a release.
    config: GithubConfig,
}

impl ReleaseTask {
    /// Create a new `ReleaseTask` with the given `name` and `project_root`.
    pub(crate) fn new(name: String, project_root: PathBuf) -> Result<Self> {
        let local_crate = LocalCrate::new(name, &project_root)?;
        let config = crate::config()?.github;
        let http_client = HttpClient::new()?;

        Ok(Self { local_crate, project_root, http_client, config })
    }

    /// Run the task to effectively create a release.
    pub(crate) fn run(self) -> Result<()> {
        let title = &self.title();
        let prerelease = self.local_crate.version.is_prerelease();
        println!(
            "Starting {} for {}…",
            match prerelease {
                true => "pre-release",
                false => "release",
            },
            title
        );

        if self.is_released()? {
            return Err("This crate version is already released".into());
        }

        let remote = Self::git_remote()?;

        println!("Checking status of git repository…");
        if !cmd!("git status -s -uno").read()?.is_empty()
            && !ask_yes_no("This git repository contains untracked files. Continue?")?
        {
            return Ok(());
        }

        if let Some(macros) = self.macros() {
            print!("Found macros crate. ");
            let _dir = pushd(&macros.path)?;
            macros.publish(&self.http_client)?;

            println!("Waiting 10 seconds for the release to make it into the crates.io index…");
            sleep(Duration::from_secs(10));

            println!("Resuming release of {}…", self.title());
        }

        let _dir = pushd(&self.local_crate.path)?;

        self.local_crate.publish(&self.http_client)?;

        if prerelease {
            println!("Pre-release created successfully!");
            return Ok(());
        }

        let changes = &self.local_crate.changes()?;

        let tag = &self.tag_name();

        println!("Creating git tag…");
        if cmd!("git tag -l {tag}").read()?.is_empty() {
            cmd!("git tag -s {tag} -m {title} -m {changes}").read()?;
        } else if !ask_yes_no("This tag already exists. Skip this step and continue?")? {
            return Ok(());
        }

        println!("Pushing tag to remote repository…");
        if cmd!("git ls-remote --tags {remote} {tag}").read()?.is_empty() {
            cmd!("git push {remote} {tag}").run()?;
        } else if !ask_yes_no("This tag has already been pushed. Skip this step and continue?")? {
            return Ok(());
        }

        println!("Creating release on GitHub…");
        let request_body = &json!({
            "tag_name": tag,
            "name": title,
            "body": changes.trim_softbreaks(),
        })
        .to_string();

        self.release(request_body)?;

        println!("Release created successfully!");

        Ok(())
    }

    /// Get the associated `-macros` crate of the current crate, if any.
    fn macros(&self) -> Option<LocalCrate> {
        LocalCrate::new(format!("{}-macros", self.local_crate.name), &self.project_root).ok()
    }

    /// Get the title of this release.
    fn title(&self) -> String {
        format!("{} {}", self.local_crate.name, self.local_crate.version)
    }

    /// Load the GitHub config from the config file.
    fn git_remote() -> Result<String> {
        let branch = cmd!("git rev-parse --abbrev-ref HEAD").read()?;
        let remote = cmd!("git config branch.{branch}.remote").read()?;

        if remote.is_empty() {
            return Err("Could not get current git remote".into());
        }

        Ok(remote)
    }

    /// Get the tag name for this release.
    fn tag_name(&self) -> String {
        format!("{}-{}", self.local_crate.name, self.local_crate.version)
    }

    /// Check if the tag for the current version of the crate has been pushed on GitHub.
    fn is_released(&self) -> Result<bool> {
        let response = self.http_client.get(format!(
            "{}/releases/tags/{}",
            GITHUB_API_RUMA,
            self.tag_name()
        ))?;

        Ok(response.status() == StatusCode::OK)
    }

    /// Create the release on GitHub with the given `config` and `credentials`.
    fn release(&self, body: &str) -> Result<()> {
        let request = Request::post(format!("{}/releases", GITHUB_API_RUMA))
            .authentication(Authentication::basic())
            .credentials(Credentials::new(&self.config.user, &self.config.token))
            .header("Accept", "application/vnd.github.v3+json")
            .body(body)?;

        let mut response = self.http_client.send(request)?;

        if response.status() == StatusCode::CREATED {
            Ok(())
        } else {
            Err(format!("{}: {}", response.status(), response.text()?).into())
        }
    }
}

/// A local Rust crate.
#[derive(Debug)]
struct LocalCrate {
    /// The name of the crate.
    name: String,

    /// The version of the crate.
    version: Version,

    /// The local path of the crate.
    path: PathBuf,
}

impl LocalCrate {
    /// Creates a new `Crate` with the given name and project root.
    pub fn new(name: String, project_root: &Path) -> Result<Self> {
        let path = project_root.join(&name);

        let version = Self::version(&path)?;

        Ok(Self { name, version, path })
    }

    /// The current version of the crate at `path` from its manifest.
    fn version(path: &Path) -> Result<Version> {
        let manifest_toml = read_file(path.join("Cargo.toml"))?;
        let manifest: CargoManifest = from_toml_str(&manifest_toml)?;

        Ok(manifest.package.version)
    }

    /// The changes of the given version from the changelog.
    fn changes(&self) -> Result<String> {
        let changelog = read_file(self.path.join("CHANGELOG.md"))?;
        let lines_nb = changelog.lines().count();
        let mut lines = changelog.lines();

        let start = match lines.position(|l| l.starts_with(&format!("# {}", self.version))) {
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

    /// Check if the current version of the crate is published on crates.io.
    fn is_published(&self, client: &HttpClient) -> Result<bool> {
        let response: CratesIoCrate =
            client.get(format!("{}/{}/{}", CRATESIO_API, self.name, self.version))?.json()?;

        Ok(response.version.is_some())
    }

    /// Publish this package on crates.io.
    fn publish(&self, client: &HttpClient) -> Result<()> {
        println!("Publishing {} {} on crates.io…", self.name, self.version);
        if self.is_published(client)? {
            if ask_yes_no("This version is already published. Skip this step and continue?")? {
                Ok(())
            } else {
                Err("Release interrupted by user.".into())
            }
        } else {
            Ok(cmd!("cargo publish").run()?)
        }
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
    version: Version,
}

/// A crate from the `GET /crates/{crate}` endpoint of crates.io.
#[derive(Deserialize)]
struct CratesIoCrate {
    version: Option<IgnoredAny>,
}

/// A tag from the `GET /repos/{owner}/{repo}/tags` endpoint of GitHub REST API.
#[derive(Debug, Deserialize)]
struct GithubTag {
    /// The name of the tag.
    name: String,
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
