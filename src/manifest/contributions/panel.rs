use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Panel {
    pub id: String,
    pub title: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry: Option<String>,
}
