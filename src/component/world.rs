use crate::manifest::Manifest;

const TYPES: &str = "synt:extension/types@";
const EXPORTS: [&str; 2] = ["describe", "run"];
const STORAGE: [&str; 2] = ["storage-get", "storage-set"];
const CALL: &str = "call";
const LOG: &str = "log";
const EMPTY_WASI: [&str; 4] = [
    "wasi:io/",
    "wasi:cli/",
    "wasi:clocks/monotonic-clock@",
    "wasi:random/",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct World {
    imports: Vec<String>,
    exports: Vec<String>,
}

impl World {
    pub fn parse(wit: &str) -> Self {
        Self {
            imports: names(wit, "import "),
            exports: names(wit, "export "),
        }
    }

    pub fn problems(&self, manifest: &Manifest) -> Vec<String> {
        let mut problems = Vec::new();

        problems.extend(self.missing_exports());
        problems.extend(self.api_problem(&manifest.api));
        problems.extend(self.import_problems(manifest));

        problems
            .into_iter()
            .map(|problem| format!("{}: the component {problem}", manifest.id))
            .collect()
    }

    fn missing_exports(&self) -> Vec<String> {
        EXPORTS
            .iter()
            .filter(|export| !self.exports.iter().any(|name| name == *export))
            .map(|export| format!("does not export {export}"))
            .collect()
    }

    fn api_problem(&self, api: &str) -> Option<String> {
        let expected = format!("{TYPES}{api}.");
        let matches = self
            .imports
            .iter()
            .any(|import| import.starts_with(&expected));

        (!matches).then(|| format!("is not built against api {api}"))
    }

    fn import_problems(&self, manifest: &Manifest) -> Vec<String> {
        self.imports
            .iter()
            .filter_map(|import| import_problem(import, manifest))
            .collect()
    }
}

fn import_problem(import: &str, manifest: &Manifest) -> Option<String> {
    let capabilities = &manifest.capabilities;

    if import == CALL && capabilities.call.is_empty() {
        return Some("imports call, but capabilities.call is empty".to_owned());
    }

    if STORAGE.contains(&import) && !capabilities.storage {
        return Some(format!(
            "imports {import}, but capabilities.storage is false"
        ));
    }

    let provided = import == CALL
        || import == LOG
        || STORAGE.contains(&import)
        || import.starts_with(TYPES)
        || EMPTY_WASI.iter().any(|prefix| import.starts_with(prefix));

    (!provided).then(|| format!("imports {import}, which the host does not provide"))
}

fn names(wit: &str, keyword: &str) -> Vec<String> {
    wit.lines()
        .filter_map(|line| line.trim().strip_prefix(keyword))
        .map(|rest| name(rest).to_owned())
        .collect()
}

fn name(declaration: &str) -> &str {
    let declaration = declaration.trim_end_matches(';');

    declaration
        .split_once(": ")
        .map_or(declaration, |(name, _)| name)
        .trim()
}

#[cfg(test)]
mod tests {
    use super::*;

    const UPTIME: &str = "package root:component;

world root {
  import synt:extension/types@0.1.0;
  import wasi:io/poll@0.2.9;
  import wasi:clocks/monotonic-clock@0.2.9;
  import wasi:cli/environment@0.2.9;
  import wasi:cli/stdout@0.2.9;

  use synt:extension/types@0.1.0.{action, error};

  import call: func(action: string, target: option<string>, params: string) -> result<string, error>;

  export describe: func() -> list<action>;
  export run: func(action: string, target: option<string>, params: string) -> result<string, error>;
}
";

    fn manifest() -> Manifest {
        toml::from_str(
            "id = \"uptime\"\nname = \"Uptime\"\nversion = \"0.1.0\"\napi = \"0.1\"\n\
             description = \"d\"\nrepository = \"https://example.com\"\n\n\
             [capabilities]\ncall = [\"ssh.exec\"]\n",
        )
        .expect("manifest")
    }

    #[test]
    fn reads_imports_and_exports() {
        let world = World::parse(UPTIME);

        assert!(world.imports.contains(&"call".to_owned()));
        assert!(
            world
                .imports
                .contains(&"synt:extension/types@0.1.0".to_owned())
        );
        assert_eq!(world.exports, ["describe", "run"]);
    }

    #[test]
    fn accepts_the_uptime_component() {
        assert!(World::parse(UPTIME).problems(&manifest()).is_empty());
    }

    #[test]
    fn rejects_storage_without_the_capability() {
        let wit = UPTIME.replace(
            "import call",
            "import storage-get: func(key: string);\n  import call",
        );

        assert_eq!(
            World::parse(&wit).problems(&manifest()),
            ["uptime: the component imports storage-get, but capabilities.storage is false"]
        );
    }

    #[test]
    fn rejects_files_network_and_wall_clock() {
        let wit = UPTIME.replace(
            "import wasi:io/poll@0.2.9;",
            "import wasi:filesystem/types@0.2.9;\n  import wasi:sockets/tcp@0.2.9;\n  import wasi:clocks/wall-clock@0.2.9;",
        );

        assert_eq!(World::parse(&wit).problems(&manifest()).len(), 3);
    }

    #[test]
    fn rejects_a_different_api() {
        let wit = UPTIME.replace("types@0.1.0;", "types@0.2.0;");

        assert_eq!(
            World::parse(&wit).problems(&manifest()),
            ["uptime: the component is not built against api 0.1"]
        );
    }

    #[test]
    fn rejects_missing_exports() {
        let wit = UPTIME.replace("export run", "export walk");

        assert_eq!(
            World::parse(&wit).problems(&manifest()),
            ["uptime: the component does not export run"]
        );
    }
}
