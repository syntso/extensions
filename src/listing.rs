use serde_json::{Map, Value, json};

use crate::RegistryError;
use crate::component::Component;
use crate::manifest::Manifest;

const FILE: &str = "file";
const SHA256: &str = "sha256";
const SIGNATURE: &str = "signature";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listing {
    manifest: Manifest,
    file: String,
    sha256: String,
    signature: Option<String>,
}

impl Listing {
    pub fn new(manifest: Manifest, component: &Component) -> Self {
        let file = Self::file_name(&manifest);

        Self {
            manifest,
            file,
            sha256: component.sha256().to_string(),
            signature: None,
        }
    }

    pub fn from_value(value: &Value) -> Result<Self, RegistryError> {
        let mut fields = value
            .as_object()
            .cloned()
            .ok_or_else(|| RegistryError::NotObject("a listing".to_owned()))?;

        let label = fields
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("a listing")
            .to_owned();

        let file = Self::take(&mut fields, &label, FILE)?;
        let sha256 = Self::take(&mut fields, &label, SHA256)?;
        let signature = Self::take(&mut fields, &label, SIGNATURE).ok();

        let rest = Value::Object(fields);
        let manifest = Manifest::from_value(&label, &rest)?;

        if rest.as_object() != Some(manifest.fields()) {
            return Err(RegistryError::NotCanonical(label));
        }

        Ok(Self {
            manifest,
            file,
            sha256,
            signature,
        })
    }

    pub fn file_name(manifest: &Manifest) -> String {
        format!(
            "{}-{}.wasm",
            manifest.id(),
            manifest.version().replace('+', "_")
        )
    }

    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    pub fn file(&self) -> &str {
        &self.file
    }

    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    pub fn signed(self, signature: String) -> Self {
        Self {
            signature: Some(signature),
            ..self
        }
    }

    pub fn seal(&self) -> Value {
        let manifest = &self.manifest;

        json!({
            "id": manifest.id().as_str(),
            "name": manifest.text("name"),
            "version": manifest.version(),
            "description": manifest.text("description").unwrap_or_default(),
            "repository": manifest.text("repository"),
            "sha256": self.sha256,
            "manifest_sha256": manifest.sha256().as_str(),
        })
    }

    pub fn to_value(&self) -> Value {
        let mut fields = self.manifest.fields().clone();

        fields.insert(FILE.to_owned(), Value::String(self.file.clone()));
        fields.insert(SHA256.to_owned(), Value::String(self.sha256.clone()));

        if let Some(signature) = &self.signature {
            fields.insert(SIGNATURE.to_owned(), Value::String(signature.clone()));
        }

        Value::Object(fields)
    }

    fn take(
        fields: &mut Map<String, Value>,
        label: &str,
        name: &'static str,
    ) -> Result<String, RegistryError> {
        match fields.remove(name) {
            Some(Value::String(text)) => Ok(text),
            _ => Err(RegistryError::ListingField(label.to_owned(), name)),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::Listing;
    use crate::manifest::Manifest;

    fn listing() -> Listing {
        let manifest = Manifest::parse(
            "t",
            r#"{ "id": "t", "name": "Test", "version": "0.1.0+b1", "capabilities": { "storage": true } }"#,
        )
        .unwrap();

        Listing {
            file: Listing::file_name(&manifest),
            manifest,
            sha256: "ab".repeat(32),
            signature: None,
        }
    }

    #[test]
    fn the_file_name_stays_inside_the_release() {
        assert_eq!(listing().file(), "t-0.1.0_b1.wasm");
    }

    #[test]
    fn the_seal_names_what_the_core_compares() {
        let seal = listing().seal();

        assert_eq!(seal["id"], "t");
        assert_eq!(seal["name"], "Test");
        assert_eq!(seal["version"], "0.1.0+b1");
        assert_eq!(seal["description"], "");
        assert_eq!(seal["repository"], json!(null));
        assert_eq!(seal["sha256"], "ab".repeat(32));
        assert_eq!(seal["manifest_sha256"].as_str().unwrap().len(), 64);
    }

    #[test]
    fn a_listing_reads_back() {
        let signed = listing().signed("signature text".to_owned());

        let again = Listing::from_value(&signed.to_value()).unwrap();

        assert_eq!(again, signed);
    }

    #[test]
    fn a_listing_with_a_manifest_out_of_canonical_form_is_refused() {
        let mut value = listing().to_value();
        value["capabilities"]["timers"] = json!(false);

        assert!(Listing::from_value(&value).is_err());
    }
}
