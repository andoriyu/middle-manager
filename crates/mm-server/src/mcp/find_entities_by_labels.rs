use mm_core::operations::memory::{FindEntitiesByLabelsCommand, find_entities_by_labels};
use mm_memory::LabelMatchMode;
use mm_utils::IntoJsonSchema;
use rust_mcp_sdk::macros::mcp_tool;
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "find_entities_by_labels",
    description = "Find entities matching labels"
)]
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct FindEntitiesByLabelsTool {
    pub labels: Vec<String>,
    pub match_mode: LabelMatchMode,
    pub required_label: Option<String>,
}

impl FindEntitiesByLabelsTool {
    generate_call_tool!(
        self,
        FindEntitiesByLabelsCommand {
            labels => self.labels.clone(),
            match_mode => self.match_mode,
            required_label => self.required_label.clone()
        },
        find_entities_by_labels
    );
}
