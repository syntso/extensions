use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

use crate::component::World;
use crate::error::RegistryError;
use crate::process;
use crate::registry::Extension;

const TARGET: &str = "wasm32-wasip2";
const INHERITED: [&str; 3] = ["RUSTUP_TOOLCHAIN", "CARGO_TARGET_DIR", "CARGO_BUILD_TARGET"];

pub fn build(extensions: &[Extension], out: &Path) -> Result<(), RegistryError> {
    fs::create_dir_all(out).map_err(|source| RegistryError::Write {
        path: out.to_owned(),
        source,
    })?;

    let mut problems = Vec::new();

    for extension in extensions {
        problems.extend(build_one(extension, out)?);
    }

    if !problems.is_empty() {
        return Err(RegistryError::Invalid(problems));
    }

    Ok(())
}

fn build_one(extension: &Extension, out: &Path) -> Result<Vec<String>, RegistryError> {
    let manifest = extension.manifest();
    let wasm = compile(extension)?;

    process::run(Command::new("wasm-tools").arg("validate").arg(&wasm))?;
    let wit = process::run(
        Command::new("wasm-tools")
            .args(["component", "wit"])
            .arg(&wasm),
    )?;
    let problems = World::parse(&wit).problems(manifest);

    let copy = out.join(format!("{}.wasm", manifest.id));
    fs::copy(&wasm, &copy).map_err(|source| RegistryError::Write {
        path: copy.clone(),
        source,
    })?;

    println!(
        "built {} {} into {}",
        manifest.id,
        manifest.version,
        copy.display()
    );
    Ok(problems)
}

fn compile(extension: &Extension) -> Result<PathBuf, RegistryError> {
    let mut cargo = Command::new("cargo");

    cargo
        .args(["build", "--release", "--locked", "--target", TARGET])
        .arg("--message-format=json-render-diagnostics")
        .current_dir(extension.folder());

    for variable in INHERITED {
        cargo.env_remove(variable);
    }

    let messages = process::run(&mut cargo)?;

    component(&messages)
        .map(|wasm| extension.folder().join(wasm))
        .ok_or_else(|| RegistryError::NoComponent(extension.manifest().id.clone()))
}

fn component(messages: &str) -> Option<PathBuf> {
    messages
        .lines()
        .rev()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|message| message["reason"] == "compiler-artifact")
        .filter(builds_cdylib)
        .find_map(|message| wasm_file(&message))
}

fn builds_cdylib(message: &Value) -> bool {
    message["target"]["kind"]
        .as_array()
        .is_some_and(|kinds| kinds.iter().any(|kind| kind == "cdylib"))
}

fn wasm_file(message: &Value) -> Option<PathBuf> {
    message["filenames"]
        .as_array()?
        .iter()
        .filter_map(Value::as_str)
        .map(PathBuf::from)
        .find(|file| {
            file.extension()
                .is_some_and(|extension| extension == "wasm")
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_component_among_artifacts() {
        let messages = [
            r#"{"reason":"compiler-artifact","target":{"kind":["lib"]},"filenames":["/t/libserde.rlib"]}"#,
            r#"{"reason":"compiler-artifact","target":{"kind":["cdylib"]},"filenames":["/t/uptime.wasm"]}"#,
            r#"{"reason":"build-finished","success":true}"#,
        ]
        .join("\n");

        assert_eq!(component(&messages), Some(PathBuf::from("/t/uptime.wasm")));
    }

    #[test]
    fn finds_nothing_without_a_cdylib() {
        let messages =
            r#"{"reason":"compiler-artifact","target":{"kind":["lib"]},"filenames":["/t/a.rlib"]}"#;

        assert_eq!(component(messages), None);
    }
}
