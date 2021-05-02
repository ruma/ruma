use std::{
    io::{stdin, stdout, BufRead, Write},
    thread::sleep,
    time::Duration,
};

use isahc::{
    auth::{Authentication, Credentials},
    config::Configurable,
    http::StatusCode,
    HttpClient, ReadResponseExt, Request,
};
use semver::{Identifier, Version};
use serde::Deserialize;
use serde_json::json;

use crate::{cargo::Package, cmd, util::ask_yes_no, GithubConfig, Metadata, Result};

const GITHUB_API_RUMA: &str = "https://api.github.com/repos/ruma/ruma";

/// Task to create a new release of the given crate.
#[derive(Debug)]
pub struct ReleaseTask {
    /// The metadata of the cargo workspace.
    metadata: Metadata,

    /// The crate to release.
    package: Package,

    /// The new version of the crate.
    version: Version,

    /// The http client to use for requests.
    http_client: HttpClient,

    /// The github configuration required to publish a release.
    config: GithubConfig,
}

impl ReleaseTask {
    /// Create a new `ReleaseTask` with the given `name` and `version`.
    pub(crate) fn new(name: String, version: Version) -> Result<Self> {
        let metadata = Metadata::load()?;

        let package = metadata
            .packages
            .clone()
            .into_iter()
            .find(|p| p.name == name)
            .ok_or(format!("Package {} not found in cargo metadata", name))?;

        let config = crate::Config::load()?.github;

        let http_client = HttpClient::new()?;

        Ok(Self { metadata, package, version, http_client, config })
    }

    /// Run the task to effectively create a release.
    pub(crate) fn run(&mut self) -> Result<()> {
        let title = &self.title();
        let prerelease = self.version.is_prerelease();
        let publish_only = self.package.name == "ruma-identifiers-validation";

        if let Some(name) = self.package.name.strip_suffix("-macros") {
            return Err(format!(
                "Macro crates are always released together with their parent crate.\n\
                 To release both {main_cr} and {macro_cr}, simply run\n\
                 \n\
                 cargo xtask release {main_cr}",
                main_cr = name,
                macro_cr = self.package.name,
            )
            .into());
        }

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
            && !ask_yes_no("This git repository contains uncommitted changes. Continue?")?
        {
            return Ok(());
        }

        if self.package.version != self.version
            && !self.package.version.is_next(&self.version)
            && !ask_yes_no(&format!(
                "Version {} should not follow version {}. Do you really want to continue?",
                self.version, self.package.version,
            ))?
        {
            return Ok(());
        }

        let mut macros = self.macros();

        if self.package.version != self.version {
            if let Some(m) = macros.as_mut() {
                println!("Found macros crate {}.", m.name);

                m.update_version(&self.version)?;
                m.update_dependants(&self.metadata)?;

                println!("Resuming release of {}…", self.title());
            }

            self.package.update_version(&self.version)?;
            self.package.update_dependants(&self.metadata)?;
        }

        let changes = &self.package.changes(!prerelease)?;

        if self.package.version != self.version {
            self.commit()?;
        } else if !ask_yes_no(&format!(
            "Package is already version {}. Skip creating a commit and continue?",
            &self.version
        ))? {
            return Ok(());
        }

        if let Some(m) = macros {
            let published = m.publish(&self.http_client)?;

            if published {
                // Crate was published, instead of publishing skipped (because release already
                // existed).
                println!("Waiting 20 seconds for the release to make it into the crates.io index…");
                sleep(Duration::from_secs(20));
            }
        }

        self.package.publish(&self.http_client)?;

        let branch = cmd!("git rev-parse --abbrev-ref HEAD").read()?;
        if publish_only {
            println!("Pushing to remote repository…");
            cmd!("git push {remote} {branch}").run()?;

            println!("Crate published successfully!");
            return Ok(());
        }

        let tag = &self.tag_name();

        println!("Creating git tag…");
        if cmd!("git tag -l {tag}").read()?.is_empty() {
            cmd!("git tag -s {tag} -m {title} -m {changes}").read()?;
        } else if !ask_yes_no("This tag already exists. Skip this step and continue?")? {
            return Ok(());
        }

        println!("Pushing to remote repository…");
        if cmd!("git ls-remote --tags {remote} {tag}").read()?.is_empty() {
            cmd!("git push {remote} {branch} {tag}").run()?;
        } else if !ask_yes_no("This tag has already been pushed. Skip this step and continue?")? {
            return Ok(());
        }

        if prerelease {
            println!("Pre-release created successfully!");
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
    fn macros(&self) -> Option<Package> {
        self.metadata
            .packages
            .clone()
            .into_iter()
            .find(|p| p.name == format!("{}-macros", self.package.name))
    }

    /// Get the title of this release.
    fn title(&self) -> String {
        format!("{} {}", self.package.name, self.version)
    }

    /// Get the tag name for this release.
    fn tag_name(&self) -> String {
        format!("{}-{}", self.package.name, self.version)
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

    /// Commit and push all the changes in the git repository.
    fn commit(&self) -> Result<()> {
        let stdin = stdin();

        let instructions = "Ready to commit the changes. [continue/abort/diff]: ";
        print!("{}", instructions);
        stdout().flush()?;

        let mut handle = stdin.lock();

        let mut input = String::new();
        loop {
            let eof = handle.read_line(&mut input)? == 0;
            if eof {
                return Err("User aborted commit".into());
            }

            match input.trim().to_ascii_lowercase().as_str() {
                "c" | "con" | "continue" => {
                    break;
                }
                "a" | "abort" => {
                    return Err("User aborted commit".into());
                }
                "d" | "diff" => {
                    cmd!("git diff").run()?;
                }
                _ => {
                    println!("Unknown command.");
                }
            }
            print!("{}", instructions);
            stdout().flush()?;

            input.clear();
        }

        let message = format!("Release {}", self.title());

        println!("Creating commit…");
        cmd!("git commit -a -m {message}").read()?;

        Ok(())
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

/// Extra Version increment methods for crate release.
trait VersionExt {
    /// Adds a pre-release label and number if there is none.
    fn add_pre_release(&mut self);

    /// Increments the pre-release number, if this is a pre-release.
    fn increment_pre_number(&mut self);

    /// Increments the pre-release label from `alpha` to `beta` if this is a pre-release and it is
    /// possible, otherwise does nothing.
    fn increment_pre_label(&mut self);

    /// If the given version can be the next after this one.
    ///
    /// This checks all the version bumps of the format MAJOR.MINOR.PATCH-PRE_LABEL.PRE_NUMBER, with
    /// PRE_LABEL = alpha or beta.
    fn is_next(&self, version: &Version) -> bool;
}

impl VersionExt for Version {
    fn add_pre_release(&mut self) {
        if !self.is_prerelease() {
            self.pre = vec![Identifier::AlphaNumeric("alpha".into()), Identifier::Numeric(1)];
        }
    }

    fn increment_pre_number(&mut self) {
        if self.is_prerelease() {
            if let Identifier::Numeric(n) = self.pre[1] {
                self.pre[1] = Identifier::Numeric(n + 1);
            }
        }
    }

    fn increment_pre_label(&mut self) {
        if self.is_prerelease() {
            match &self.pre[0] {
                Identifier::AlphaNumeric(n) if n == "alpha" => {
                    self.pre =
                        vec![Identifier::AlphaNumeric("beta".into()), Identifier::Numeric(1)];
                }
                _ => {}
            }
        }
    }

    fn is_next(&self, version: &Version) -> bool {
        let mut next = self.clone();

        if self.is_prerelease() {
            next.increment_pre_number();
            if next == *version {
                return true;
            }

            next.increment_pre_label();
            if next == *version {
                return true;
            }

            next.pre = vec![];
            if next == *version {
                return true;
            }
        } else {
            next.increment_patch();
            if next == *version {
                return true;
            }

            next.add_pre_release();
            if next == *version {
                return true;
            }

            next.increment_minor();
            if next == *version {
                return true;
            }

            next.add_pre_release();
            if next == *version {
                return true;
            }

            next.increment_major();
            if next == *version {
                return true;
            }

            next.add_pre_release();
            if next == *version {
                return true;
            }
        }

        false
    }
}
