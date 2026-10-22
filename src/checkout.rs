use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::RegistryError;
use crate::pin::Pin;

const GIT: &str = "git";

#[derive(Debug, Clone)]
pub struct Checkout {
    path: PathBuf,
}

impl Checkout {
    pub fn fetch(pin: &Pin, work: &Path) -> Result<Self, RegistryError> {
        let path = work.join(pin.id().as_str());

        if path.exists() {
            fs::remove_dir_all(&path).map_err(|source| RegistryError::Write {
                path: path.clone(),
                source,
            })?;
        }

        fs::create_dir_all(&path).map_err(|source| RegistryError::Write {
            path: path.clone(),
            source,
        })?;

        let checkout = Self { path };
        let rev = pin.rev().as_str();

        checkout.git(&["init", "--quiet"])?;
        checkout.git(&[
            "fetch",
            "--quiet",
            "--depth",
            "1",
            pin.repository().as_str(),
            rev,
        ])?;
        checkout.git(&["checkout", "--quiet", "--detach", rev])?;

        let found = checkout.head()?;

        if found != rev {
            return Err(RegistryError::WrongCommit {
                id: pin.id().to_string(),
                found,
                pinned: rev.to_owned(),
            });
        }

        Ok(checkout)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn file(&self, name: &str) -> PathBuf {
        self.path.join(name)
    }

    fn head(&self) -> Result<String, RegistryError> {
        let output = self
            .command(&["rev-parse", "HEAD"])
            .stderr(Stdio::inherit())
            .output()
            .map_err(|source| RegistryError::Spawn {
                program: GIT,
                source,
            })?;

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    }

    fn git(&self, arguments: &[&str]) -> Result<(), RegistryError> {
        let status = self
            .command(arguments)
            .status()
            .map_err(|source| RegistryError::Spawn {
                program: GIT,
                source,
            })?;

        if !status.success() {
            return Err(RegistryError::Failed {
                command: format!("git {}", arguments.join(" ")),
                path: self.path.clone(),
            });
        }

        Ok(())
    }

    fn command(&self, arguments: &[&str]) -> Command {
        let mut command = Command::new(GIT);

        command
            .args([
                "-c",
                "advice.detachedHead=false",
                "-c",
                "init.defaultBranch=main",
            ])
            .args(arguments)
            .current_dir(&self.path)
            .env("GIT_TERMINAL_PROMPT", "0")
            .stdin(Stdio::null());

        command
    }
}
