use std::io::{stdin, stdout, BufRead, Write};

use clap::Args;
use reqwest::{blocking::Client, StatusCode};
use semver::Version;
use serde_json::json;

use crate::{cargo::Package, cmd, util::ask_yes_no, GithubConfig, Metadata, Result};

const GITHUB_API_RUMA: &str = "https://api.github.com/repos/ruma/ruma";

#[derive(Args)]
pub struct ReleaseArgs {
    /// The crate to release
    pub package: String,

    /// The new version of the crate
    pub version: Version,

    /// List the steps but don't actually change anything
    #[clap(long)]
    pub dry_run: bool,
}

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
    http_client: Client,

    /// The github configuration required to publish a release.
    config: GithubConfig,

    /// List the steps but don't actually change anything
    pub dry_run: bool,
}

impl ReleaseTask {
    /// Create a new `ReleaseTask` with the given `name` and `version`.
    pub(crate) fn new(name: String, version: Version, dry_run: bool) -> Result<Self> {
        let metadata = Metadata::load()?;

        let package = metadata
            .packages
            .clone()
            .into_iter()
            .find(|p| p.name == name)
            .ok_or(format!("Package {name} not found in cargo metadata"))?;

        let config = crate::Config::load()?.github;

        let http_client = Client::builder().user_agent("ruma xtask").build()?;

        Ok(Self { metadata, package, version, http_client, config, dry_run })
    }

    /// Run the task to effectively create a release.
    pub(crate) fn run(&mut self) -> Result<()> {
        let title = &self.title();
        let prerelease = !self.version.pre.is_empty();
        let publish_only =
            ["ruma-identifiers-validation", "ruma-macros"].contains(&self.package.name.as_str());

        println!(
            "Starting {} for {title}…",
            match prerelease {
                true => "pre-release",
                false => "release",
            },
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

        let create_commit = if self.package.version != self.version {
            self.package.update_version(&self.version, self.dry_run)?;
            self.package.update_dependants(&self.metadata, self.dry_run)?;
            true
        } else if !ask_yes_no(&format!(
            "Package is already version {}. Skip creating a commit and continue?",
            &self.version
        ))? {
            return Ok(());
        } else {
            false
        };

        let changes = &self.package.changes(!prerelease && !self.dry_run)?;

        if create_commit {
            self.commit()?;
        }

        self.package.publish(&self.http_client, self.dry_run)?;

        let branch = cmd!("git rev-parse --abbrev-ref HEAD").read()?;
        if publish_only {
            println!("Pushing to remote repository…");
            if !self.dry_run {
                cmd!("git push {remote} {branch}").run()?;
            }

            println!("Crate published successfully!");
            return Ok(());
        }

        let tag = &self.tag_name();

        println!("Creating git tag '{tag}'…");
        if cmd!("git tag -l {tag}").read()?.is_empty() {
            if !self.dry_run {
                cmd!("git tag -s {tag} -m {title} -m {changes}").read()?;
            }
        } else if !ask_yes_no("This tag already exists. Skip this step and continue?")? {
            return Ok(());
        }

        println!("Pushing to remote repository…");
        if cmd!("git ls-remote --tags {remote} {tag}").read()?.is_empty() {
            if !self.dry_run {
                cmd!("git push {remote} {branch} {tag}").run()?;
            }
        } else if !ask_yes_no("This tag has already been pushed. Skip this step and continue?")? {
            return Ok(());
        }

        if prerelease {
            println!("Pre-release created successfully!");
            return Ok(());
        }

        println!("Creating release on GitHub…");
        let request_body = json!({
            "tag_name": tag,
            "name": title,
            "body": changes.trim_softbreaks(),
        })
        .to_string();

        if !self.dry_run {
            self.release(request_body)?;
        }

        println!("Release created successfully!");

        if self.package.name == "ruma-macros" {
            println!(
                "Reminder: Make sure to release new versions of both ruma-common and ruma-events \
                 so users can actually start using this release"
            );
        }

        Ok(())
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

        if !self.dry_run {
            let instructions = "Ready to commit the changes. [continue/abort/diff]: ";
            print!("{instructions}");
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
                print!("{instructions}");
                stdout().flush()?;

                input.clear();
            }
        }

        let message = format!("Release {}", self.title());

        println!("Creating commit with message '{message}'…");

        if !self.dry_run {
            cmd!("git commit -a -m {message}").read()?;
        }

        Ok(())
    }

    /// Check if the tag for the current version of the crate has been pushed on GitHub.
    fn is_released(&self) -> Result<bool> {
        let response = self
            .http_client
            .get(format!("{GITHUB_API_RUMA}/releases/tags/{}", self.tag_name()))
            .send()?;

        Ok(response.status() == StatusCode::OK)
    }

    /// Create the release on GitHub with the given `config` and `credentials`.
    fn release(&self, body: String) -> Result<()> {
        let response = self
            .http_client
            .post(format!("{GITHUB_API_RUMA}/releases"))
            .basic_auth(&self.config.user, Some(&self.config.token))
            .header("Accept", "application/vnd.github.v3+json")
            .body(body)
            .send()?;

        if response.status() == StatusCode::CREATED {
            Ok(())
        } else {
            Err(format!("{}: {}", response.status(), response.text()?).into())
        }
    }
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
        if self.pre.is_empty() {
            self.pre = semver::Prerelease::new("alpha.1").unwrap();
        }
    }

    fn increment_pre_number(&mut self) {
        if let Some((prefix, num)) = self.pre.as_str().rsplit_once('.') {
            if let Ok(num) = num.parse::<u8>() {
                self.pre = semver::Prerelease::new(&format!("{prefix}.{}", num + 1)).unwrap();
            }
        }
    }

    fn increment_pre_label(&mut self) {
        if self.pre.as_str().starts_with("alpha.") {
            self.pre = semver::Prerelease::new("beta.1").unwrap();
        }
    }

    fn is_next(&self, version: &Version) -> bool {
        let mut next = self.clone();

        if !self.pre.is_empty() {
            next.increment_pre_number();
            if next == *version {
                return true;
            }

            next.increment_pre_label();
            if next == *version {
                return true;
            }

            next.pre = semver::Prerelease::EMPTY;
        } else {
            next.patch += 1;
            if next == *version {
                return true;
            }

            next.add_pre_release();
            if next == *version {
                return true;
            }

            next.pre = semver::Prerelease::EMPTY;
            next.patch = 0;
            next.minor += 1;
            if next == *version {
                return true;
            }

            next.add_pre_release();
            if next == *version {
                return true;
            }

            next.pre = semver::Prerelease::EMPTY;
            next.minor = 0;
            next.major += 1;
            if next == *version {
                return true;
            }

            next.add_pre_release();
        }

        next == *version
    }
}
