use semver::{Version, VersionReq};
use serde_json::{Map, Value};

use crate::RegistryError;

const CONTRIBUTION_LISTS: [&str; 7] = [
    "resources",
    "documents",
    "commands",
    "menus",
    "sidebar",
    "panels",
    "status",
];

pub(super) fn canonical(path: &str, manifest: &Map<String, Value>) -> Result<Value, RegistryError> {
    let mut fields = manifest.clone();

    fields.remove("$schema");
    drop_if(&mut fields, "description", |value| value == "");
    drop_if(&mut fields, "repository", Value::is_null);

    let version = self::version(path, fields.get("version"))?;
    fields.insert("version".to_owned(), version);

    let dependencies = self::dependencies(path, fields.remove("dependencies"))?;
    if !dependencies.is_empty() {
        fields.insert("dependencies".to_owned(), Value::Object(dependencies));
    }

    let capabilities = self::capabilities(fields.remove("capabilities"));
    fields.insert("capabilities".to_owned(), Value::Object(capabilities));

    let actions = self::actions(fields.remove("actions"));
    if !actions.is_empty() {
        fields.insert("actions".to_owned(), Value::Array(actions));
    }

    let contributes = self::contributions(fields.remove("contributes"));
    if !contributes.is_empty() {
        fields.insert("contributes".to_owned(), Value::Object(contributes));
    }

    Ok(Value::Object(fields))
}

fn version(path: &str, value: Option<&Value>) -> Result<Value, RegistryError> {
    let text = value.and_then(Value::as_str).unwrap_or_default();

    let version = Version::parse(text).map_err(|error| RegistryError::Semver {
        path: path.to_owned(),
        field: "version",
        value: text.to_owned(),
        message: error.to_string(),
    })?;

    Ok(Value::String(version.to_string()))
}

fn dependencies(path: &str, value: Option<Value>) -> Result<Map<String, Value>, RegistryError> {
    let mut dependencies = Map::new();

    for (id, requirement) in object(value) {
        let text = requirement.as_str().unwrap_or_default();

        let parsed = VersionReq::parse(text).map_err(|error| RegistryError::Semver {
            path: path.to_owned(),
            field: "dependencies",
            value: text.to_owned(),
            message: error.to_string(),
        })?;

        dependencies.insert(id, Value::String(parsed.to_string()));
    }

    Ok(dependencies)
}

fn capabilities(value: Option<Value>) -> Map<String, Value> {
    let mut capabilities = object(value);

    capabilities.retain(|_, value| !is_default(value));

    capabilities
}

fn actions(value: Option<Value>) -> Vec<Value> {
    list(value)
        .into_iter()
        .map(|action| {
            let mut fields = object(Some(action));

            fields.retain(|name, value| {
                let flag = matches!(name.as_str(), "agents" | "dependents");
                let defaulted = value.is_null()
                    || (name == "targets" && is_empty_list(value))
                    || (flag && value == false);

                !defaulted
            });

            Value::Object(fields)
        })
        .collect()
}

fn contributions(value: Option<Value>) -> Map<String, Value> {
    let mut contributions = object(value);

    for name in CONTRIBUTION_LISTS {
        let items: Vec<Value> = list(contributions.remove(name))
            .into_iter()
            .map(without_nulls)
            .collect();

        if !items.is_empty() {
            contributions.insert(name.to_owned(), Value::Array(items));
        }
    }

    drop_if(&mut contributions, "settings", Value::is_null);

    contributions
}

fn without_nulls(item: Value) -> Value {
    let mut fields = object(Some(item));

    fields.retain(|_, value| !value.is_null());

    Value::Object(fields)
}

fn is_default(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::Bool(set) => !set,
        Value::Array(items) => items.is_empty(),
        Value::Object(fields) => fields.is_empty(),
        Value::Number(_) | Value::String(_) => false,
    }
}

fn is_empty_list(value: &Value) -> bool {
    value.as_array().is_some_and(Vec::is_empty)
}

fn drop_if(fields: &mut Map<String, Value>, name: &str, defaulted: impl Fn(&Value) -> bool) {
    if fields.get(name).is_some_and(defaulted) {
        fields.remove(name);
    }
}

fn object(value: Option<Value>) -> Map<String, Value> {
    match value {
        Some(Value::Object(fields)) => fields,
        _ => Map::new(),
    }
}

fn list(value: Option<Value>) -> Vec<Value> {
    match value {
        Some(Value::Array(items)) => items,
        _ => Vec::new(),
    }
}
