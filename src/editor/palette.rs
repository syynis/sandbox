use bevy::{asset::LoadState, prelude::*};

use super::tiles::TilePixel;

#[derive(Default, Reflect)]
pub struct PaletteMeta {
    pub skycolor: Color,
}

#[derive(Default, Reflect)]
pub struct PaletteRows {
    pub colors: [[Color; 30]; 3],
}

#[derive(Default, Reflect)]
pub struct Palette {
    pub meta: PaletteMeta,
    pub sun: PaletteRows,
    pub shade: PaletteRows,
}

impl Palette {
    pub fn get_color(&self, shade: bool, dir: TilePixel, sub_layer: usize, layer: usize) -> Color {
        let dir = dir as usize;
        if dir == 3 {
            return Color::rgba_u8(0, 0, 0, 0);
        }
        if shade {
            self.shade.colors[dir][10 * layer + sub_layer]
        } else {
            self.sun.colors[dir][10 * layer + sub_layer]
        }
    }

    pub fn get_sun_color(&self, dir: TilePixel, sub_layer: usize, layer: usize) -> Color {
        self.get_color(false, dir, sub_layer, layer)
    }

    pub fn get_shade_color(&self, dir: TilePixel, sub_layer: usize, layer: usize) -> Color {
        self.get_color(true, dir, sub_layer, layer)
    }
}

#[derive(Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct Palettes {
    active_palette: usize,
    palettes: Vec<Palette>,
}

impl Palettes {
    pub fn get_active(&self) -> &Palette {
        &self.palettes[self.active_palette]
    }

    pub fn get(&self, idx: usize) -> &Palette {
        &self.palettes[idx]
    }

    pub fn cycle(&mut self) {
        self.active_palette = (self.active_palette + 1) % self.palettes.len();
    }
}

#[derive(Resource)]
pub struct PaletteHandles(pub Vec<Handle<Image>>);

pub fn load_palette_images(mut cmds: Commands, asset_server: Res<AssetServer>) {
    let palettes = asset_server.load_folder("palettes").unwrap();
    let palettes: Vec<Handle<Image>> = palettes
        .iter()
        .map(|handle| handle.clone().typed::<Image>())
        .collect();
    cmds.insert_resource(PaletteHandles(palettes));
}

pub fn parse_palette_images(
    mut cmds: Commands,
    asset_server: Res<AssetServer>,
    palette_handles: Res<PaletteHandles>,
    images: Res<Assets<Image>>,
    mut once: Local<bool>,
) {
    let palettes_loaded = match asset_server
        .get_group_load_state(palette_handles.0.iter().map(|handle| handle.id()))
    {
        LoadState::Loaded => true,
        _ => false,
    };

    if *once || !palettes_loaded {
        return;
    }

    if !palette_handles
        .0
        .iter()
        .all(|handle| images.get(&handle).is_some())
    {
        return;
    }
    let palette_images: Vec<&Image> = palette_handles
        .0
        .iter()
        .map(|handle| images.get(&handle).unwrap())
        .collect();

    *once = true;

    let mut palettes = Vec::new();
    for palette_image in palette_images {
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
        palettes.push(Palette { meta, sun, shade });
    }
    cmds.insert_resource(Palettes {
        active_palette: 0,
        palettes,
    });
}
