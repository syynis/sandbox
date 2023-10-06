use bevy_ecs_tilemap::tiles::TileTextureIndex;

pub fn texture_name(idx: TileTextureIndex) -> String {
    match idx.0 {
        0 => "Square".to_string(),
        1 => "Slope".to_string(),
        2 => "PoleV".to_string(),
        3 => "PoleH".to_string(),
        4 => "PoleC".to_string(),
        5 => "Platform".to_string(),
        _ => "Invalid".to_string(),
    }
}
