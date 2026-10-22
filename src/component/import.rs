use semver::Version;

use crate::manifest::Manifest;

const PACKAGE: &str = "synt:extension/";
const MAJOR: u64 = 1;

const WASI: [(&str, &[&str]); 4] = [
    ("wasi:io", &["error", "poll", "streams"]),
    (
        "wasi:cli",
        &[
            "environment",
            "exit",
            "stdin",
            "stdout",
            "stderr",
            "terminal-input",
            "terminal-output",
            "terminal-stdin",
            "terminal-stdout",
            "terminal-stderr",
        ],
    ),
    ("wasi:clocks", &["monotonic-clock"]),
    ("wasi:random", &["random", "insecure", "insecure-seed"]),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Import<'a> {
    package: &'a str,
    interface: &'a str,
    version: Option<Version>,
}

impl<'a> Import<'a> {
    pub(super) fn parse(name: &'a str) -> Self {
        let (path, version) = match name.split_once('@') {
            Some((path, version)) => (path, Version::parse(version).ok()),
            None => (name, None),
        };

        let (package, interface) = path.split_once('/').unwrap_or((path, ""));

        Self {
            package,
            interface,
            version,
        }
    }

    pub(super) fn check(&self, manifest: &Manifest) -> Result<(), String> {
        if format!("{}/", self.package) == PACKAGE {
            return self.check_synt(manifest);
        }

        self.check_wasi()
    }

    fn check_synt(&self, manifest: &Manifest) -> Result<(), String> {
        if self
            .version
            .as_ref()
            .is_none_or(|version| version.major != MAJOR)
        {
            return Err(format!("is not synt:extension {MAJOR}.x"));
        }

        let (allowed, needs) = match self.interface {
            "types" | "host" => (true, "nothing"),
            "storage" | "cache" => (manifest.grants("storage"), "capabilities.storage"),
            "http" => (manifest.grants("http"), "capabilities.http"),
            "documents" => (manifest.grants("documents"), "capabilities.documents"),
            "files" => (manifest.grants("files"), "capabilities.files"),
            "resources" => (
                manifest.grants("resources") || manifest.contributes("resources"),
                "capabilities.resources or contributes.resources",
            ),
            "timers" => (manifest.grants("timers"), "capabilities.timers"),
            "terminal" => (manifest.grants("terminal"), "capabilities.terminal"),
            "notifications" => (
                manifest.grants("notifications"),
                "capabilities.notifications",
            ),
            "dialogs" => (manifest.grants("dialogs"), "capabilities.dialogs"),
            "progress" => (manifest.grants("progress"), "capabilities.progress"),
            "ui" => (manifest.draws(), "a contributed view, tree or web panel"),
            _ => return Err("is not an interface of synt:extension".to_owned()),
        };

        if !allowed {
            return Err(format!("needs {needs} in the manifest"));
        }

        Ok(())
    }

    fn check_wasi(&self) -> Result<(), String> {
        let linked = WASI.iter().any(|(package, interfaces)| {
            *package == self.package && interfaces.contains(&self.interface)
        });
        let preview2 = self
            .version
            .as_ref()
            .is_some_and(|version| version.major == 0 && version.minor == 2);

        if !linked || !preview2 {
            return Err(
                "the core does not link; only synt:extension 1.x and the wasi 0.2 io, cli, monotonic clock and random interfaces are"
                    .to_owned(),
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Import;
    use crate::manifest::Manifest;

    fn manifest(capabilities: &str, contributes: &str) -> Manifest {
        Manifest::parse(
            "t",
            &format!(
                r#"{{ "id": "t", "name": "T", "version": "1.0.0",
                     "capabilities": {capabilities}, "contributes": {contributes} }}"#
            ),
        )
        .unwrap()
    }

    fn refused(name: &str, manifest: &Manifest) -> bool {
        Import::parse(name).check(manifest).is_err()
    }

    #[test]
    fn types_and_host_are_always_allowed() {
        let bare = manifest("{}", "{}");

        assert!(!refused("synt:extension/types@1.0.0", &bare));
        assert!(!refused("synt:extension/host@1.4.0", &bare));
    }

    #[test]
    fn every_interface_needs_its_capability() {
        let bare = manifest("{}", "{}");

        for interface in [
            "storage",
            "cache",
            "http",
            "documents",
            "files",
            "resources",
            "timers",
            "terminal",
            "notifications",
            "dialogs",
            "progress",
            "ui",
        ] {
            assert!(
                refused(&format!("synt:extension/{interface}@1.0.0"), &bare),
                "{interface}"
            );
        }
    }

    #[test]
    fn granted_interfaces_pass() {
        let granted = manifest(
            r#"{ "storage": true, "http": ["api.example.com"], "files": "read", "timers": true,
                 "terminal": true, "notifications": true, "dialogs": true, "progress": true,
                 "resources": ["host"] }"#,
            r#"{ "sidebar": [{ "id": "s", "title": "S", "icon": "box", "tree": "t" }] }"#,
        );

        for interface in [
            "storage",
            "cache",
            "http",
            "files",
            "resources",
            "timers",
            "terminal",
            "notifications",
            "dialogs",
            "progress",
            "ui",
        ] {
            assert!(
                !refused(&format!("synt:extension/{interface}@1.0.0"), &granted),
                "{interface}"
            );
        }
    }

    #[test]
    fn own_resource_kinds_allow_the_resources_interface() {
        let kinds = manifest(
            "{}",
            r#"{ "resources": [{ "kind": "box", "title": "Box", "icon": "box",
                                 "schema": { "type": "object" } }] }"#,
        );

        assert!(!refused("synt:extension/resources@1.0.0", &kinds));
    }

    #[test]
    fn only_version_one_of_the_package_is_allowed() {
        let bare = manifest("{}", "{}");

        for name in [
            "synt:extension/host@0.2.0",
            "synt:extension/host@2.0.0",
            "synt:extension/host",
            "synt:extension/printer@1.0.0",
        ] {
            assert!(refused(name, &bare), "{name}");
        }
    }

    #[test]
    fn only_the_linked_wasi_interfaces_are_allowed() {
        let bare = manifest("{}", "{}");

        for name in [
            "wasi:io/streams@0.2.9",
            "wasi:cli/terminal-stderr@0.2.0",
            "wasi:clocks/monotonic-clock@0.2.3",
            "wasi:random/insecure-seed@0.2.9",
        ] {
            assert!(!refused(name, &bare), "{name}");
        }

        for name in [
            "wasi:filesystem/types@0.2.9",
            "wasi:sockets/tcp@0.2.9",
            "wasi:clocks/wall-clock@0.2.9",
            "wasi:http/outgoing-handler@0.2.9",
            "wasi:io/streams@0.3.0",
            "wasi:io/streams",
            "env",
        ] {
            assert!(refused(name, &bare), "{name}");
        }
    }
}
