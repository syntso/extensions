use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SidebarSection {
    pub id: String,
    pub title: String,
    pub view: String,
}
