mod import;

use wasmparser::{ComponentImport, ComponentTypeRef, Encoding, Parser, Payload};

use crate::RegistryError;
use crate::manifest::Manifest;
use crate::sha256::Sha256;

use import::Import;

const EXPORTS: [&str; 9] = [
    "run",
    "start",
    "render",
    "children",
    "panel-message",
    "asset",
    "on-event",
    "on-timer",
    "merge",
];

#[derive(Debug, Clone)]
pub struct Component {
    bytes: Vec<u8>,
    sha256: Sha256,
    imports: Vec<String>,
    exports: Vec<String>,
}

impl Component {
    pub fn parse(bytes: Vec<u8>) -> Result<Self, RegistryError> {
        let mut imports = Vec::new();
        let mut exports = Vec::new();
        let mut depth = 0_usize;

        for payload in Parser::new(0).parse_all(&bytes) {
            let payload = payload.map_err(|error| RegistryError::Component(error.to_string()))?;

            match payload {
                Payload::Version { encoding, .. } => {
                    if depth == 0 && encoding != Encoding::Component {
                        return Err(RegistryError::Component("a core module".to_owned()));
                    }

                    depth += 1;
                }
                Payload::End(_) => depth = depth.saturating_sub(1),
                Payload::ComponentImportSection(section) if depth == 1 => {
                    for import in section {
                        let import =
                            import.map_err(|error| RegistryError::Component(error.to_string()))?;

                        if let Some(name) = Self::instance(&import)? {
                            imports.push(name);
                        }
                    }
                }
                Payload::ComponentExportSection(section) if depth == 1 => {
                    for export in section {
                        let export =
                            export.map_err(|error| RegistryError::Component(error.to_string()))?;

                        exports.push(export.name.full_name().into_owned());
                    }
                }
                _ => {}
            }
        }

        let sha256 = Sha256::of(&bytes);

        Ok(Self {
            bytes,
            sha256,
            imports,
            exports,
        })
    }

    pub fn check(&self, manifest: &Manifest) -> Result<(), RegistryError> {
        let id = manifest.id().to_string();

        for name in &self.imports {
            Import::parse(name)
                .check(manifest)
                .map_err(|reason| RegistryError::Import {
                    id: id.clone(),
                    import: name.clone(),
                    reason,
                })?;
        }

        match EXPORTS
            .iter()
            .find(|export| !self.exports.iter().any(|found| found == *export))
        {
            Some(export) => Err(RegistryError::Export { id, export }),
            None => Ok(()),
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn sha256(&self) -> &Sha256 {
        &self.sha256
    }

    pub fn imports(&self) -> &[String] {
        &self.imports
    }

    fn instance(import: &ComponentImport<'_>) -> Result<Option<String>, RegistryError> {
        let name = import.name.full_name();

        if let Some(interface) = import.name.implements {
            return Err(RegistryError::Component(format!(
                "the import {name} implements {interface}, which the core does not link"
            )));
        }

        match import.ty {
            ComponentTypeRef::Type(_) => Ok(None),
            ComponentTypeRef::Instance(_) => Ok(Some(name.into_owned())),
            _ => Err(RegistryError::Component(format!(
                "the import {name} is not an interface, which the core does not link"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Component;
    use crate::manifest::Manifest;

    const EXPORTS: &str = r#"
        (core module $m (func (export "f")))
        (core instance $i (instantiate $m))
        (func $f (canon lift (core func $i "f")))
        (export "run" (func $f))
        (export "start" (func $f))
        (export "render" (func $f))
        (export "children" (func $f))
        (export "panel-message" (func $f))
        (export "asset" (func $f))
        (export "on-event" (func $f))
        (export "on-timer" (func $f))
        (export "merge" (func $f))
    "#;

    fn component(imports: &[&str], exports: &str) -> Component {
        let imports = imports
            .iter()
            .map(|name| format!("(import \"{name}\" (instance))"))
            .collect::<Vec<_>>()
            .join(" ");
        let text = format!("(component {imports} {exports})");

        Component::parse(wat::parse_str(text).unwrap()).unwrap()
    }

    fn manifest(capabilities: &str) -> Manifest {
        Manifest::parse(
            "t",
            &format!(r#"{{ "id": "t", "name": "T", "version": "1.0.0", "capabilities": {capabilities} }}"#),
        )
        .unwrap()
    }

    #[test]
    fn reads_the_imports_of_the_component_itself() {
        let found = component(
            &["synt:extension/host@1.0.0", "wasi:io/streams@0.2.9"],
            EXPORTS,
        );

        assert_eq!(
            found.imports(),
            ["synt:extension/host@1.0.0", "wasi:io/streams@0.2.9"]
        );
        assert!(found.check(&manifest("{}")).is_ok());
    }

    #[test]
    fn an_import_needs_its_capability() {
        let found = component(&["synt:extension/storage@1.0.0"], EXPORTS);

        let refusal = found.check(&manifest("{}")).unwrap_err().to_string();

        assert!(refusal.contains("capabilities.storage"), "{refusal}");
        assert!(found.check(&manifest(r#"{ "storage": true }"#)).is_ok());
    }

    #[test]
    fn the_world_is_exported_in_full() {
        let partial = EXPORTS.replace("(export \"merge\" (func $f))", "");
        let found = component(&[], &partial);

        let refusal = found.check(&manifest("{}")).unwrap_err().to_string();

        assert!(refusal.contains("merge"), "{refusal}");
    }

    #[test]
    fn type_imports_are_left_to_the_linker_and_function_imports_are_refused() {
        let types = "(import \"error\" (type (sub resource)))";
        let found =
            Component::parse(wat::parse_str(format!("(component {types} {EXPORTS})")).unwrap())
                .unwrap();
        let function = wat::parse_str("(component (import \"f\" (func)))").unwrap();

        assert!(found.imports().is_empty());
        assert!(Component::parse(function).is_err());
    }

    #[test]
    fn a_core_module_is_not_a_component() {
        let module = wat::parse_str("(module)").unwrap();

        assert!(Component::parse(module).is_err());
    }
}
