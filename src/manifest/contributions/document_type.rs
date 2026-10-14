use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentType {
    #[serde(rename = "type")]
    pub kind: String,
    pub title: String,
    pub suffix: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub panel: Option<String>,
}
