use bevy::prelude::*;

pub const WORLD: f32 = 0.0f32;
pub const NEAR_BACKGROUND: f32 = 1.0f32;
pub const FAR_BACKGROUND: f32 = 2.0f32;

pub const ALL_LAYERS: [LayerId; 3] = [LayerId::World, LayerId::Near, LayerId::Far];
#[repr(u8)]
#[derive(Default, Clone, Copy, Reflect)]
pub enum LayerId {
    #[default]
    World,
    Near,
    Far,
}

impl LayerId {
    pub fn next(self) -> Self {
        use LayerId::*;
        match self {
            World => Near,
            Near => Far,
            Far => World,
        }
    }
}

pub trait Layer {
    /// Returns the z-index for the layer.
    fn z_index() -> f32;
    fn layer_id() -> LayerId;
}

#[derive(Component)]
pub struct WorldLayer;

impl Layer for WorldLayer {
    fn z_index() -> f32 {
        WORLD
    }
    fn layer_id() -> LayerId {
        LayerId::World
    }
}

#[derive(Component)]
pub struct NearLayer;

impl Layer for NearLayer {
    fn z_index() -> f32 {
        NEAR_BACKGROUND
    }
    fn layer_id() -> LayerId {
        LayerId::Near
    }
}

#[derive(Component)]
pub struct FarLayer;

impl Layer for FarLayer {
    fn z_index() -> f32 {
        FAR_BACKGROUND
    }
    fn layer_id() -> LayerId {
        LayerId::Far
    }
}
