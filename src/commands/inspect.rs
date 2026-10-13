use std::path::Path;
use std::process::Command;

use crate::component::World;
use crate::error::RegistryError;
use crate::process;
use crate::registry::Extension;

pub fn inspect(extension: &Extension, wasm: &Path) -> Result<Vec<String>, RegistryError> {
    process::run(Command::new("wasm-tools").arg("validate").arg(wasm))?;

    let wit = process::run(
        Command::new("wasm-tools")
            .args(["component", "wit"])
            .arg(wasm),
    )?;

    Ok(World::parse(&wit).problems(extension.manifest()))
}
