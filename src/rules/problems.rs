use super::{APIS, CALLS};
use crate::manifest::Manifest;
use crate::registry::Entry;

const REPOSITORY_PREFIX: &str = "https://";

pub fn problems(entry: &Entry, manifest: &Manifest) -> Vec<String> {
    let mut problems = Vec::new();

    problems.extend(identity(entry, manifest));
    problems.extend(fields(manifest));
    problems.extend(capabilities(manifest));

    problems
        .into_iter()
        .map(|problem| format!("{}: {problem}", entry.id))
        .collect()
}

fn identity(entry: &Entry, manifest: &Manifest) -> Vec<String> {
    let mut problems = Vec::new();

    if !is_id(&entry.id) {
        problems.push(
            "an id is lowercase letters, digits and dashes, starting with a letter".to_owned(),
        );
    }

    if manifest.id != entry.id {
        problems.push(format!("extension.toml says the id is {}", manifest.id));
    }

    if manifest.version != entry.version {
        problems.push(format!(
            "extensions.toml lists version {}, extension.toml says {}",
            entry.version, manifest.version
        ));
    }

    problems
}

fn fields(manifest: &Manifest) -> Vec<String> {
    let mut problems = Vec::new();

    if !is_version(&manifest.version) {
        problems.push(format!(
            "version {} is not major.minor.patch",
            manifest.version
        ));
    }

    if !APIS.contains(&manifest.api.as_str()) {
        problems.push(format!(
            "api {} is not supported, only {}",
            manifest.api,
            APIS.join(", ")
        ));
    }

    if manifest.name.trim().is_empty() || manifest.description.trim().is_empty() {
        problems.push("name and description must not be empty".to_owned());
    }

    if !manifest.repository.starts_with(REPOSITORY_PREFIX) {
        problems.push(format!(
            "repository {} is not an https address",
            manifest.repository
        ));
    }

    problems
}

fn capabilities(manifest: &Manifest) -> Vec<String> {
    manifest
        .capabilities
        .call
        .iter()
        .filter(|call| !CALLS.contains(&call.as_str()))
        .map(|call| format!("capability call {call} is not a core action extensions may call"))
        .collect()
}

fn is_id(id: &str) -> bool {
    let starts_with_letter = id
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_lowercase());
    let allowed = id.chars().all(|character| {
        character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
    });

    starts_with_letter && allowed
}

fn is_version(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();

    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.chars().all(|digit| digit.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::Capabilities;

    fn entry() -> Entry {
        Entry {
            id: "uptime".to_owned(),
            path: "extensions/uptime".into(),
            version: "0.1.0".to_owned(),
        }
    }

    fn manifest() -> Manifest {
        Manifest {
            id: "uptime".to_owned(),
            name: "Uptime".to_owned(),
            version: "0.1.0".to_owned(),
            api: "0.1".to_owned(),
            description: "Shows uptime.".to_owned(),
            repository: "https://github.com/syntso/extension-uptime".to_owned(),
            capabilities: Capabilities {
                call: vec!["ssh.exec".to_owned()],
                storage: false,
            },
        }
    }

    #[test]
    fn accepts_a_valid_manifest() {
        assert!(problems(&entry(), &manifest()).is_empty());
    }

    #[test]
    fn rejects_a_mismatched_id() {
        let manifest = Manifest {
            id: "load".to_owned(),
            ..manifest()
        };

        assert_eq!(
            problems(&entry(), &manifest),
            ["uptime: extension.toml says the id is load"]
        );
    }

    #[test]
    fn rejects_a_mismatched_version() {
        let manifest = Manifest {
            version: "0.2.0".to_owned(),
            ..manifest()
        };

        assert_eq!(problems(&entry(), &manifest).len(), 1);
    }

    #[test]
    fn rejects_an_unsupported_api() {
        let manifest = Manifest {
            api: "0.2".to_owned(),
            ..manifest()
        };

        assert_eq!(
            problems(&entry(), &manifest),
            ["uptime: api 0.2 is not supported, only 0.1"]
        );
    }

    #[test]
    fn rejects_unknown_and_person_only_calls() {
        let mut manifest = manifest();
        manifest.capabilities.call = vec![
            "ssh.exec".to_owned(),
            "grant.revoke".to_owned(),
            "x.y".to_owned(),
        ];

        assert_eq!(problems(&entry(), &manifest).len(), 2);
    }

    #[test]
    fn checks_id_and_version_shapes() {
        assert!(is_id("disk-usage2"));
        assert!(!is_id("Disk"));
        assert!(!is_id("2disk"));
        assert!(!is_id(""));
        assert!(is_version("10.0.3"));
        assert!(!is_version("1.0"));
        assert!(!is_version("1.0.x"));
    }
}
