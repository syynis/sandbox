use bevy::ecs::system::Command;
use bevy::prelude::Entity;
use bevy::prelude::Vec2;
use bevy_ecs_tilemap::tiles::TileTextureIndex;
use bevy_xpbd_2d::math::*;
use bevy_xpbd_2d::prelude::*;

use crate::phys::terrain::Platform;
use crate::phys::terrain::Pole;
use crate::phys::terrain::PoleType;
use crate::phys::terrain::Terrain;

use super::placement::TileProperties;

pub enum TileKind {
    Square,
    Slope,
    Pole(PoleType),
    Platform,
}

impl TileKind {
    pub fn is_pole(&self) -> Option<PoleType> {
        use TileKind::*;
        match self {
            Pole(kind) => Some(*kind),
            _ => None,
        }
    }

    pub fn is_platform(&self) -> bool {
        matches!(self, TileKind::Platform)
    }

    pub fn is_solid(&self) -> bool {
        matches!(self, TileKind::Square)
    }

    pub fn is_slope(&self) -> bool {
        matches!(self, TileKind::Slope)
    }

    pub fn offset(&self) -> Vector {
        use TileKind::*;
        match self {
            Platform => Vector::Y * 5.,
            _ => Vector::ZERO,
        }
    }

    pub fn name(&self) -> String {
        use TileKind::*;
        match self {
            Square => "Square".to_owned(),
            Slope => "Slope".to_owned(),
            Pole(_) => "Pole".to_owned(),
            Platform => "Platform".to_owned(),
        }
    }
}

impl Into<TileTextureIndex> for TileKind {
    fn into(self) -> TileTextureIndex {
        use TileKind::*;
        let id = match self {
            Square => 0,
            Slope => 1,
            Pole(kind) => match kind {
                PoleType::Vertical => 2,
                PoleType::Horizontal => 3,
                PoleType::Combined => 4,
            },
            Platform => 5,
        };
        TileTextureIndex(id)
    }
}

impl From<TileTextureIndex> for TileKind {
    fn from(value: TileTextureIndex) -> Self {
        use TileKind::*;
        match value.0 {
            0 => Square,
            1 => Slope,
            2 => Pole(PoleType::Vertical),
            3 => Pole(PoleType::Horizontal),
            4 => Pole(PoleType::Combined),
            5 => Platform,
            _ => unreachable!(),
        }
    }
}

impl From<TileProperties> for Collider {
    fn from(value: TileProperties) -> Self {
        let make_right_triangle = |corner, size, dir: Vector| -> Collider {
            Collider::triangle(
                corner + Vector::X * size * dir.x,
                corner + Vector::Y * size * dir.y,
                corner,
            )
        };
        let dir = match (value.flip.x, value.flip.y) {
            (false, false) => Vector::new(1., 1.),
            (true, false) => Vector::new(-1., 1.),
            (false, true) => Vector::new(1., -1.),
            (true, true) => Vector::new(-1., -1.),
        };

        let cross = Collider::compound(vec![
            (
                Position::default(),
                Rotation::default(),
                Collider::cuboid(4., 16.),
            ),
            (
                Position::default(),
                Rotation::default(),
                Collider::cuboid(16., 4.),
            ),
        ]);

        match value.id.0 {
            0 => Collider::cuboid(16., 16.),
            1 => make_right_triangle(Vector::new(-8., -8.) * dir, 16., dir),
            2 => Collider::cuboid(4., 16.),
            3 => Collider::cuboid(16., 4.),
            4 => cross,
            5 => Collider::cuboid(16., 4.),
            _ => unreachable!(),
        }
    }
}

pub struct InsertTileColliderCommand {
    pub tile_entity: Entity,
    pub pos: Vec2,
    pub properties: TileProperties,
    pub kind: TileKind,
}

impl Command for InsertTileColliderCommand {
    fn apply(self, world: &mut bevy::prelude::World) {
        let pos = self.pos + self.kind.offset();
        let tile_entity = self.tile_entity;
        world.entity_mut(tile_entity).insert((
            RigidBody::Static,
            Collider::from(self.properties),
            Position(pos),
        ));

        if let Some(pole) = TileKind::from(self.properties.id).is_pole() {
            world.entity_mut(tile_entity).insert((Sensor, Pole(pole)));
        } else {
            world.entity_mut(tile_entity).insert(Terrain);
        };

        if matches!(TileKind::from(self.properties.id), TileKind::Platform) {
            world.entity_mut(tile_entity).insert(Platform::default());
        }
    }
}
