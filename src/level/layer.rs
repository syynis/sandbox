use bevy::prelude::*;

pub const ALL_LAYERS: [Layer; 3] = [Layer::World, Layer::Near, Layer::Far];

#[repr(u8)]
#[derive(Default, Component, Clone, Copy, Reflect, PartialEq)]
pub enum Layer {
    #[default]
    World,
    Near,
    Far,
}

impl Layer {
    pub fn next(&self) -> Self {
        use Layer::*;
        match self {
            World => Near,
            Near => Far,
            Far => Far,
        }
    }
    pub fn wrapping_next(&self) -> Self {
        use Layer::*;
        match self {
            World => Near,
            Near => Far,
            Far => World,
        }
    }

    pub fn z_index(&self) -> f32 {
        use Layer::*;
        match self {
            World => 0.0f32,
            Near => 1.0f32,
            Far => 2.0f32,
        }
    }

    pub fn name(&self) -> &str {
        use Layer::*;
        match self {
            World => "World",
            Near => "Near",
            Far => "Far",
        }
    }
}
