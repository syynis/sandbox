use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use serde::{Deserialize, Serialize};

pub mod area;
pub mod erase;
pub mod paint;
pub mod platform;
pub mod pole;
pub mod slope;

pub type ToolId = usize;

// TODO think about config format for toolkit and if its useful
#[derive(Debug, Default, Clone, Serialize, Deserialize, Reflect)]
pub struct ToolData {
    pub id: ToolId,
    pub name: String,
    #[serde(skip)]
    #[reflect(ignore)]
    pub egui_texture_id: Option<egui::TextureId>,
}

#[derive(Debug, Default, Clone, Reflect)]
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

pub trait Tool: Sync + Send {
    fn new(world: &mut World) -> Self;
    fn apply(&mut self, world: &mut World);
    fn update(&mut self, world: &mut World);
}

#[derive(Resource)]
struct ToolState<T: 'static + Sync + Send>(HashMap<ToolId, T>);

pub fn run_tool<T: Tool + 'static>(world: &mut World, id: ToolId) {
    if !world.contains_resource::<ToolState<T>>() {
        world.insert_resource(ToolState::<T>(HashMap::new()));
    }

    world.resource_scope(|world, mut states: Mut<ToolState<T>>| {
        let state = states.0.entry(id).or_insert(T::new(world));
        state.apply(world);
    })
}

pub fn update_tool<T: Tool + 'static>(world: &mut World, id: ToolId) {
    if !world.contains_resource::<ToolState<T>>() {
        world.insert_resource(ToolState::<T>(HashMap::new()));
    }

    world.resource_scope(|world, mut states: Mut<ToolState<T>>| {
        let state = states.0.entry(id).or_insert(T::new(world));
        state.update(world);
    })
}
