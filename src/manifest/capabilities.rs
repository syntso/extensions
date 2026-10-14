use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Capabilities {
    #[serde(default)]
    pub call: Vec<String>,

    #[serde(default)]
    pub storage: bool,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub http: Vec<String>,

    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub secrets: BTreeMap<String, Vec<String>>,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub documents: bool,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<String>,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub timers: bool,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files: Option<String>,
}

impl Capabilities {
    pub fn uses_api_0_2(&self) -> bool {
        !self.http.is_empty()
            || !self.secrets.is_empty()
            || self.documents
            || !self.events.is_empty()
            || self.timers
            || self.files.is_some()
    }
}
