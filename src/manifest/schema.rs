use std::sync::OnceLock;

use jsonschema::{Draft, Validator};
use serde_json::Value;

const SCHEMA: &str = include_str!("../../schema/extension.schema.json");
const MOST_PROBLEMS: usize = 8;

static VALIDATOR: OnceLock<Result<Validator, String>> = OnceLock::new();

pub(super) fn problems(manifest: &Value) -> Result<Vec<String>, String> {
    let validator = VALIDATOR
        .get_or_init(build)
        .as_ref()
        .map_err(Clone::clone)?;

    let problems = validator
        .iter_errors(manifest)
        .take(MOST_PROBLEMS)
        .map(|error| {
            let at = error.instance_path().to_string();
            let at = if at.is_empty() { "/".to_owned() } else { at };

            format!("{at}: {error}")
        })
        .collect();

    Ok(problems)
}

fn build() -> Result<Validator, String> {
    let schema: Value = serde_json::from_str(SCHEMA).map_err(|error| error.to_string())?;

    jsonschema::options()
        .with_draft(Draft::Draft202012)
        .build(&schema)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::problems;

    #[test]
    fn the_vendored_schema_accepts_a_manifest() {
        let manifest = json!({ "id": "t", "name": "T", "version": "1.0.0" });

        assert_eq!(problems(&manifest).unwrap(), Vec::<String>::new());
    }

    #[test]
    fn the_vendored_schema_refuses_unknown_keys_and_missing_identity() {
        let unknown = json!({ "id": "t", "name": "T", "version": "1.0.0", "api": "0.2" });
        let missing = json!({ "id": "t", "name": "T" });

        assert!(!problems(&unknown).unwrap().is_empty());
        assert!(!problems(&missing).unwrap().is_empty());
    }
}
