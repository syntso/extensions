use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::Value;

use crate::RegistryError;
use crate::checkout::Checkout;

const CARGO: &str = "cargo";
const TARGET: &str = "wasm32-wasip2";
const ARGUMENTS: [&str; 7] = [
    "build",
    "--release",
    "--locked",
    "--target",
    TARGET,
    "--message-format",
    "json-render-diagnostics",
];

pub struct Compilation;

impl Compilation {
    pub fn run(checkout: &Checkout) -> Result<Vec<u8>, RegistryError> {
        let manifest_path = Self::canonical(&checkout.file("Cargo.toml"))?;

        let mut child = Command::new(CARGO)
            .args(ARGUMENTS)
            .current_dir(checkout.path())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|source| RegistryError::Spawn {
                program: CARGO,
                source,
            })?;

        let mut components = Vec::new();

        if let Some(stdout) = child.stdout.take() {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                components.extend(Self::components(&line, &manifest_path));
            }
        }

        let status = child.wait().map_err(|source| RegistryError::Spawn {
            program: CARGO,
            source,
        })?;

        if !status.success() {
            return Err(RegistryError::Failed {
                command: format!("cargo {}", ARGUMENTS.join(" ")),
                path: checkout.path().to_owned(),
            });
        }

        let [component] = components.as_slice() else {
            return Err(RegistryError::NoComponent(checkout.path().to_owned()));
        };

        fs::read(component).map_err(|source| RegistryError::Read {
            path: component.clone(),
            source,
        })
    }

    fn components(line: &str, manifest_path: &Path) -> Vec<PathBuf> {
        let Ok(message) = serde_json::from_str::<Value>(line) else {
            return Vec::new();
        };

        let artifact = message["reason"] == "compiler-artifact";
        let own = message["manifest_path"]
            .as_str()
            .and_then(|path| fs::canonicalize(path).ok())
            .is_some_and(|path| path == manifest_path);

        if !artifact || !own {
            return Vec::new();
        }

        message["filenames"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .filter(|file| {
                Path::new(file)
                    .extension()
                    .is_some_and(|extension| extension == "wasm")
            })
            .map(PathBuf::from)
            .collect()
    }

    fn canonical(path: &Path) -> Result<PathBuf, RegistryError> {
        fs::canonicalize(path).map_err(|source| RegistryError::Read {
            path: path.to_owned(),
            source,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::Compilation;

    #[test]
    fn picks_the_component_of_the_root_package() {
        let root = std::env::current_dir().unwrap().join("Cargo.toml");
        let root = std::fs::canonicalize(root).unwrap();
        let line = serde_json::json!({
            "reason": "compiler-artifact",
            "manifest_path": root,
            "filenames": ["/t/everything.wasm", "/t/libeverything.rlib"]
        })
        .to_string();
        let dependency = line.replace("Cargo.toml", "missing/Cargo.toml");

        assert_eq!(
            Compilation::components(&line, &root),
            vec![PathBuf::from("/t/everything.wasm")]
        );
        assert!(Compilation::components(&dependency, &root).is_empty());
        assert!(Compilation::components("Compiling", Path::new("/")).is_empty());
    }
}
