use bevy::utils::hashbrown::HashMap;
use serde::{Deserialize, Serialize};

pub mod paint;
pub mod slope;

pub type ToolId = usize;

// TODO think about config format for toolkit and if its useful
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ToolData {
    pub id: ToolId,
    pub name: String,
    #[serde(skip)]
    pub egui_texture_id: Option<egui::TextureId>,
}

#[derive(Debug, Default, Clone)]
pub struct ToolSet {
    pub tools: HashMap<ToolId, ToolData>,
    pub tool_order: Vec<ToolId>,
    max_id: ToolId,
}

impl ToolSet {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            tool_order: Vec::new(),
            max_id: 0,
        }
    }

    pub fn add(&mut self, tool_name: &str) {
        let tool = ToolData {
            id: self.max_id,
            name: tool_name.into(),
            egui_texture_id: None,
        };
        self.tool_order.push(tool.id);
        self.tools.insert(tool.id, tool);
        self.max_id += 1;
    }
}

pub trait Tool: std::fmt::Debug + Sync + Send {
    fn apply(&mut self, world: &mut bevy::prelude::World);
}
