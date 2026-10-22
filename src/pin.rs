mod id;
mod repository;
mod rev;

use serde_json::{Map, Value};

pub use id::ExtensionId;
pub use repository::Repository;
pub use rev::Rev;

const FIELDS: [&str; 3] = ["id", "repository", "rev"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pin {
    id: ExtensionId,
    repository: Repository,
    rev: Rev,
}

impl Pin {
    pub fn from_value(value: &Value) -> Result<Self, String> {
        let fields = value.as_object().ok_or("each pin is an object")?;

        if let Some(unknown) = fields.keys().find(|key| !FIELDS.contains(&key.as_str())) {
            return Err(format!("unknown field {unknown}"));
        }

        let id =
            ExtensionId::parse(Self::text(fields, "id")?).map_err(|error| error.to_string())?;
        let repository = Repository::parse(Self::text(fields, "repository")?)
            .map_err(|error| error.to_string())?;
        let rev = Rev::parse(Self::text(fields, "rev")?).map_err(|error| error.to_string())?;

        Ok(Self {
            id,
            repository,
            rev,
        })
    }

    pub fn id(&self) -> &ExtensionId {
        &self.id
    }

    pub fn repository(&self) -> &Repository {
        &self.repository
    }

    pub fn rev(&self) -> &Rev {
        &self.rev
    }

    fn text<'a>(fields: &'a Map<String, Value>, name: &str) -> Result<&'a str, String> {
        fields
            .get(name)
            .and_then(Value::as_str)
            .ok_or_else(|| format!("{name} is missing or not a string"))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::Pin;

    #[test]
    fn reads_a_pin() {
        let pin = Pin::from_value(&json!({
            "id": "docker",
            "repository": "https://github.com/someone/synt-docker",
            "rev": "0123456789abcdef0123456789abcdef01234567"
        }))
        .unwrap();

        assert_eq!(pin.id().as_str(), "docker");
        assert_eq!(
            pin.rev().as_str(),
            "0123456789abcdef0123456789abcdef01234567"
        );
    }

    #[test]
    fn refuses_unknown_and_missing_fields() {
        let rev = "0123456789abcdef0123456789abcdef01234567";

        for value in [
            json!({ "id": "docker", "repository": "https://x.y/z" }),
            json!({ "id": "docker", "repository": "https://x.y/z", "rev": rev, "branch": "main" }),
            json!({ "id": "docker", "repository": "https://x.y/z", "rev": "main" }),
            json!("docker"),
        ] {
            assert!(Pin::from_value(&value).is_err(), "{value}");
        }
    }
}
