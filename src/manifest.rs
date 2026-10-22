mod canonical;
mod schema;
mod text;

use std::fs;
use std::path::Path;

use serde_json::{Map, Value};

use crate::RegistryError;
use crate::pin::ExtensionId;
use crate::sha256::Sha256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    id: ExtensionId,
    fields: Map<String, Value>,
}

impl Manifest {
    pub fn read(path: &Path) -> Result<Self, RegistryError> {
        let text = fs::read_to_string(path).map_err(|source| RegistryError::Read {
            path: path.to_owned(),
            source,
        })?;

        Self::parse(&path.display().to_string(), &text)
    }

    pub fn parse(path: &str, text: &str) -> Result<Self, RegistryError> {
        let value = text::read(text).map_err(|message| RegistryError::Json {
            path: path.to_owned(),
            message,
        })?;

        Self::from_value(path, &value)
    }

    pub fn from_value(path: &str, value: &Value) -> Result<Self, RegistryError> {
        let fields = value
            .as_object()
            .ok_or_else(|| RegistryError::NotObject(path.to_owned()))?;

        let problems = schema::problems(value).map_err(|message| RegistryError::Json {
            path: "schema/extension.schema.json".to_owned(),
            message,
        })?;

        if !problems.is_empty() {
            return Err(RegistryError::Schema {
                path: path.to_owned(),
                problems: problems.join("; "),
            });
        }

        let id = ExtensionId::parse(fields.get("id").and_then(Value::as_str).unwrap_or_default())?;

        let Value::Object(fields) = canonical::canonical(path, fields)? else {
            return Err(RegistryError::NotObject(path.to_owned()));
        };

        Ok(Self { id, fields })
    }

    pub fn id(&self) -> &ExtensionId {
        &self.id
    }

    pub fn version(&self) -> &str {
        self.text("version").unwrap_or_default()
    }

    pub fn text(&self, field: &str) -> Option<&str> {
        self.fields.get(field).and_then(Value::as_str)
    }

    pub fn fields(&self) -> &Map<String, Value> {
        &self.fields
    }

    pub fn grants(&self, capability: &str) -> bool {
        self.section("capabilities")
            .is_some_and(|capabilities| capabilities.contains_key(capability))
    }

    pub fn contributes(&self, list: &str) -> bool {
        self.section("contributes")
            .is_some_and(|contributes| contributes.contains_key(list))
    }

    pub fn draws(&self) -> bool {
        ["sidebar", "status", "panels"]
            .iter()
            .any(|list| self.contributes(list))
    }

    pub fn to_json(&self) -> String {
        let value = Value::Object(self.fields.clone());
        let text = serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string());

        format!("{text}\n")
    }

    pub fn sha256(&self) -> Sha256 {
        Sha256::of(self.to_json().as_bytes())
    }

    fn section(&self, name: &str) -> Option<&Map<String, Value>> {
        self.fields.get(name).and_then(Value::as_object)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::Manifest;

    const FIXTURES: [(&str, &str, &str); 5] = [
        (
            "docker",
            include_str!("../tests/fixtures/docker.json"),
            include_str!("../tests/fixtures/docker.canonical.json"),
        ),
        (
            "board",
            include_str!("../tests/fixtures/board.json"),
            include_str!("../tests/fixtures/board.canonical.json"),
        ),
        (
            "everything",
            include_str!("../tests/fixtures/everything.json"),
            include_str!("../tests/fixtures/everything.canonical.json"),
        ),
        (
            "sparse",
            include_str!("../tests/fixtures/sparse.json"),
            include_str!("../tests/fixtures/sparse.canonical.json"),
        ),
        (
            "bare",
            include_str!("../tests/fixtures/bare.json"),
            include_str!("../tests/fixtures/bare.canonical.json"),
        ),
    ];

    #[test]
    fn the_canonical_form_is_byte_for_byte_the_cores() {
        for (name, source, core) in FIXTURES {
            let manifest = Manifest::parse(name, source).unwrap();

            assert_eq!(manifest.to_json(), core, "{name}");
        }
    }

    #[test]
    fn the_canonical_form_is_stable() {
        for (name, _, core) in FIXTURES {
            let again = Manifest::parse(name, core).unwrap();

            assert_eq!(again.to_json(), core, "{name}");
        }
    }

    #[test]
    fn capabilities_and_contributions_are_read_from_the_canonical_form() {
        let manifest = Manifest::parse("sparse", FIXTURES[3].1).unwrap();

        assert!(!manifest.grants("storage"));
        assert!(manifest.draws());
        assert!(manifest.contributes("panels"));
        assert!(!manifest.contributes("status"));
        assert_eq!(manifest.version(), "2.1.0-beta.1+build.7");
    }

    #[test]
    fn refuses_what_the_schema_or_semver_refuse() {
        let edits: [fn(&mut Value); 5] = [
            |value| value["api"] = json!("0.2"),
            |value| value["capabilities"]["network"] = json!(true),
            |value| value["version"] = json!("1.0"),
            |value| value["dependencies"] = json!({ "other": "one" }),
            |value| value["id"] = json!("Docker"),
        ];

        for edit in edits {
            let mut value = json!({ "id": "t", "name": "T", "version": "1.0.0" });
            edit(&mut value);

            assert!(Manifest::from_value("t", &value).is_err(), "{value}");
        }
    }
}
