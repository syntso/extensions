use std::path::Path;

use crate::error::RegistryError;
use crate::manifest::Manifest;
use crate::registry::{Extension, Registry};
use crate::rules;

const MANIFEST: &str = "extension.toml";

pub fn check(root: &Path, registry: &Registry) -> Result<Vec<Extension>, RegistryError> {
    let mut extensions = Vec::new();
    let mut problems = rules::duplicates(registry.entries());

    for entry in registry.entries() {
        let folder = root.join(&entry.path);
        let manifest = Manifest::load(&folder.join(MANIFEST))?;

        problems.extend(rules::problems(entry, &manifest));
        extensions.push(Extension::new(folder, manifest));
    }

    if !problems.is_empty() {
        return Err(RegistryError::Invalid(problems));
    }

    for extension in &extensions {
        let manifest = extension.manifest();
        println!("checked {} {}", manifest.id, manifest.version);
    }

    Ok(extensions)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    const LIST: &str =
        "[[extension]]\nid = \"uptime\"\npath = \"extensions/uptime\"\nversion = \"0.1.0\"\n";

    fn write_manifest(root: &Path, api: &str) {
        let folder = root.join("extensions/uptime");
        fs::create_dir_all(&folder).expect("folder");

        let manifest = format!(
            "id = \"uptime\"\nname = \"Uptime\"\nversion = \"0.1.0\"\napi = \"{api}\"\n\
             description = \"Shows uptime.\"\nrepository = \"https://github.com/syntso/extension-uptime\"\n\n\
             [capabilities]\ncall = [\"ssh.exec\"]\n"
        );
        fs::write(folder.join(MANIFEST), manifest).expect("manifest");
        fs::write(root.join("extensions.toml"), LIST).expect("list");
    }

    #[test]
    fn accepts_a_valid_registry() {
        let root = tempfile::tempdir().expect("temp dir");
        write_manifest(root.path(), "0.1");

        let registry = Registry::load(root.path()).expect("registry");
        let extensions = check(root.path(), &registry).expect("valid");

        assert_eq!(extensions.len(), 1);
        assert_eq!(extensions[0].manifest().id, "uptime");
    }

    #[test]
    fn rejects_an_unsupported_api() {
        let root = tempfile::tempdir().expect("temp dir");
        write_manifest(root.path(), "9.0");

        let registry = Registry::load(root.path()).expect("registry");
        let error = check(root.path(), &registry).expect_err("invalid");

        assert!(matches!(error, RegistryError::Invalid(problems) if problems.len() == 1));
    }

    #[test]
    fn fails_when_the_manifest_is_missing() {
        let root = tempfile::tempdir().expect("temp dir");
        fs::write(root.path().join("extensions.toml"), LIST).expect("list");

        let registry = Registry::load(root.path()).expect("registry");

        assert!(matches!(
            check(root.path(), &registry),
            Err(RegistryError::Read { .. })
        ));
    }
}
