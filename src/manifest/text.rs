use std::collections::BTreeSet;
use std::str::FromStr;

use jsonc_parser::ast::{Array, Object, Value as Node};
use jsonc_parser::{CollectOptions, ParseOptions, parse_to_ast};
use serde_json::{Map, Number, Value};

const OPTIONS: ParseOptions = ParseOptions {
    allow_comments: true,
    allow_trailing_commas: true,
    allow_loose_object_property_names: false,
    allow_missing_commas: false,
    allow_single_quoted_strings: false,
    allow_hexadecimal_numbers: false,
    allow_unary_plus_numbers: false,
    allow_bare_decimal_point_numbers: false,
    allow_non_finite_numbers: false,
    allow_extended_string_escapes: false,
};

pub(super) fn read(text: &str) -> Result<Value, String> {
    let parsed = parse_to_ast(text, &CollectOptions::default(), &OPTIONS).map_err(|error| {
        format!(
            "line {}, column {}: {}",
            error.line_display(),
            error.column_display(),
            error.kind()
        )
    })?;

    let node = parsed.value.ok_or("the file is empty")?;

    value(node)
}

fn value(node: Node<'_>) -> Result<Value, String> {
    match node {
        Node::NullKeyword(_) => Ok(Value::Null),
        Node::BooleanLit(boolean) => Ok(Value::Bool(boolean.value)),
        Node::StringLit(string) => Ok(Value::String(string.value.into_owned())),
        Node::NumberLit(number) => Number::from_str(number.value)
            .map(Value::Number)
            .map_err(|error| error.to_string()),
        Node::Array(array) => self::array(array),
        Node::Object(object) => self::object(object),
    }
}

fn array(array: Array<'_>) -> Result<Value, String> {
    let elements = array
        .elements
        .into_iter()
        .map(value)
        .collect::<Result<_, _>>()?;

    Ok(Value::Array(elements))
}

fn object(object: Object<'_>) -> Result<Value, String> {
    let mut seen = BTreeSet::new();
    let mut members = Map::new();

    for property in object.properties {
        let key = property.name.into_string();

        if !seen.insert(key.clone()) {
            return Err(format!("{key} appears twice"));
        }

        members.insert(key, value(property.value)?);
    }

    Ok(Value::Object(members))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::read;

    #[test]
    fn reads_comments_and_trailing_commas() {
        let value = read("// manifest\n{ \"a\": [1, 2,], /* b */ \"b\": 1.50, }").unwrap();

        assert_eq!(value, json!({ "a": [1, 2], "b": 1.5 }));
    }

    #[test]
    fn refuses_duplicate_keys_and_loose_syntax() {
        for text in [
            "{ \"a\": 1, \"a\": 2 }",
            "{ a: 1 }",
            "{ 'a': 1 }",
            "",
            "{ \"a\": 0x10 }",
        ] {
            assert!(read(text).is_err(), "{text}");
        }
    }
}
