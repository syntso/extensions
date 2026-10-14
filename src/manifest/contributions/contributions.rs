use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Command, DocumentType, Panel, Resource, SidebarSection, StatusItem};

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Contributions {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sidebar: Vec<SidebarSection>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub document: Vec<DocumentType>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub panel: Vec<Panel>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub command: Vec<Command>,

    #[serde(default, skip_serializing_if = "Resource::is_empty")]
    pub resource: Resource,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub status: Vec<StatusItem>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settings: Option<Value>,
}

impl Contributions {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    pub fn actions(&self) -> impl Iterator<Item = &str> {
        let commands = self.command.iter().map(|command| command.action.as_str());
        let menus = self.resource.menu.iter().map(|item| item.action.as_str());

        commands.chain(menus)
    }
}
