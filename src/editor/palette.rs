use bevy::{asset::LoadState, prelude::*};

#[derive(Default, Reflect)]
pub struct PaletteMeta {
    pub skycolor: Color,
}

#[derive(Default, Reflect)]
pub struct PaletteRows {
    colors: [[Color; 30]; 3],
}

#[derive(Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct Palette {
    pub meta: PaletteMeta,
    pub sun: PaletteRows,
    pub shade: PaletteRows,
}

#[derive(Resource)]
pub struct PaletteHandle(pub Handle<Image>);

impl Palette {
    pub fn get_color(&self, shade: bool, dir: usize, idx: usize, layer: usize) -> Color {
        if dir == 3 {
            return Color::rgba_u8(0, 0, 0, 0);
        }
        if shade {
            self.shade.colors[dir][10 * layer + idx]
        } else {
            self.sun.colors[dir][10 * layer + idx]
        }
    }

    pub fn get_sun_color(&self, dir: usize, idx: usize, layer: usize) -> Color {
        self.get_color(false, dir, idx, layer)
    }

    pub fn get_shade_color(&self, dir: usize, idx: usize, layer: usize) -> Color {
        self.get_color(true, dir, idx, layer)
    }
}

pub fn load_palette_image(mut cmds: Commands, asset_server: Res<AssetServer>) {
    let palette_asset: Handle<Image> = asset_server.load("palette.png");
    cmds.insert_resource(PaletteHandle(palette_asset));
}

pub fn parse_palette_image(
    mut cmds: Commands,
    asset_server: Res<AssetServer>,
    palette_handle: Res<PaletteHandle>,
    images: Res<Assets<Image>>,
    mut once: Local<bool>,
) {
    let palette_loaded = match asset_server.get_load_state(palette_handle.0.id()) {
        LoadState::Loaded => true,
        LoadState::Failed => {
            bevy::log::error!("Failed to load palette image");
            false
        }
        _ => false,
    };

    if *once || !palette_loaded {
        return;
    }

    let Some(palette_image) = images.get(&palette_handle.0) else {
        return;
    };

    *once = true;

    let mut meta = PaletteMeta::default();
    let mut sun = PaletteRows::default();
    let mut shade = PaletteRows::default();
    for (row, pixel_row) in palette_image.data.chunks_exact(32 * 4).enumerate() {
        for (col, pixel) in pixel_row.chunks_exact(4).enumerate() {
            if col == 30 {
                break;
            }
            let color = Color::rgba_u8(pixel[0], pixel[1], pixel[2], pixel[3]);
            match row {
                0 => {
                    meta.skycolor = color;
                    break;
                }
                1 => {
                    break;
                }
                2..=4 => {
                    sun.colors[row - 2][col] = color;
                }
                5..=7 => {
                    shade.colors[row - 5][col] = color;
                }
                _ => break,
            };
        }
    }
    cmds.insert_resource(Palette { meta, sun, shade });
}
