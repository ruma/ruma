use serde_json::json;
use xshell::{cmd, read_file};

use crate::{config, flags, project_root, Result};

impl flags::Release {
    /// Run the release command to effectively create a release.
    pub(crate) fn run(self) -> Result<()> {
        let config = self.get_config()?;

        let version = self.get_version()?;
        println!("Making GitHub release for {} {}…", self.name, version);

        let changes = self.get_changes(&version)?;

        let tag = format!("{}-{}", self.name, version);
        let name = format!("{} {}", self.name, version);

        cmd!("git tag").args(&["-a", &tag, "-m", &name, "-m", &changes]).secret(true).run()?;

        cmd!("git push").args(&[&config.remote, &tag]).run()?;

        cmd!("curl")
            .args(&[
                "-u",
                &format!("{}:{}", config.user, config.token),
                "-X",
                "POST",
                "-H",
                "Accept: application/vnd.github.v3+json",
                &format!("https://api.github.com/repos/{}/releases", config.repo),
                "-d",
                &json!({
                    "tag_name": tag,
                    "name": name,
                    "body": changes,
                })
                .to_string(),
            ])
            .secret(true)
            .run()?;

        Ok(())
    }

    /// Get the current version of the crate from the manifest.
    fn get_version(&self) -> Result<String> {
        let manifest = read_file(project_root().join(&self.name).join("Cargo.toml"))?;
        for line in manifest.lines() {
            if line.starts_with("version = ") {
                return Ok(line[10..].trim_matches('"').into());
            }
        }

        Err("Could not find crate version in manifest")?
    }

    /// Get the changes of the given version from the changelog.
    fn get_changes(&self, version: &str) -> Result<String> {
        let changelog = read_file(project_root().join(&self.name).join("CHANGELOG.md"))?;
        let lines_nb = changelog.lines().count();
        let mut lines = changelog.lines();

        let start = match lines.position(|l| l.starts_with(&format!("# {}", version))) {
            Some(p) => p + 1,
            None => {
                return Err("Could not find version title in changelog")?;
            }
        };

        let end = match lines.position(|l| l.starts_with("# ")) {
            Some(p) => start + p,
            None => lines_nb,
        };

        let changes = changelog.lines().collect::<Vec<&str>>()[start..end].to_owned();

        let trim_start = changes.iter().position(|s| !s.trim().is_empty()).unwrap_or(0);
        let trim_end = changes.iter().rev().position(|s| !s.trim().is_empty()).unwrap_or(0);

        Ok(changes[trim_start..(changes.len() - trim_end)].join("\n"))
    }

    /// Load the GitHub config from the config file.
    fn get_config(&self) -> Result<GithubConfig> {
        let all_config = config()?;
        let mut lines = all_config.lines();
        let mut config = GithubConfig::new();

        lines.position(|l| l == "[github]");
        for line in lines {
            match line {
                l if l.starts_with("remote =") => {
                    config.remote = line[9..].trim().trim_matches('"').into();
                }
                l if l.starts_with("user =") => {
                    config.user = line[7..].trim().trim_matches('"').into();
                }
                l if l.starts_with("token =") => {
                    config.token = line[8..].trim().trim_matches('"').into();
                }
                _ => {}
            }

            if config.is_loaded() {
                break;
            }
        }

        if !config.is_loaded() {
            Err("Some fields are missing for GitHub in the config file")?;
        }

        config.repo = {
            let repo_url = cmd!("git remote get-url --push").arg(&config.remote).read()?;
            repo_url
                .as_str()
                .trim_start_matches("git@github.com:")
                .trim_start_matches("https://github.com/")
                .trim_end_matches(".git")
                .into()
        };

        Ok(config)
    }
}

/// The configuration used to make a release to GitHub.
#[derive(Debug, Default)]
struct GithubConfig {
    /// The local name of the git remote to push to, from the config file.
    remote: String,

    /// The username to use for authentication, from the config file.
    user: String,

    /// The personal access token to use for authentication, from the config file.
    token: String,

    /// The remote repository on GitHub as `owner/repo`.
    repo: String,
}

impl GithubConfig {
    /// Create an empty config.
    fn new() -> Self {
        Default::default()
    }

    /// Check if all the fields required in the config file have a value.
    fn is_loaded(&self) -> bool {
        !self.remote.is_empty() && !self.user.is_empty() && !self.token.is_empty()
    }
}
