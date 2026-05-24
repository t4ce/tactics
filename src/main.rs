mod ase_assets;
mod game_object;
mod terrain_rules;
mod ts_ui;

use adapterlibgfx::api::{Adapter, AdapterConfig};
use adapterlibgfx::command::ScissorRect;
use adapterlibgfx::vertex::{Rgba8, TexVertex};
use adapterlibgfx::window::{
    FrameProducer, InputButtonState, InputEvent, InputMouseButton, WgpuTwoWindowApp,
};
use game_object::{GameObjectKind, RockKind};
use std::collections::HashSet;
use std::time::Instant;
use terrain_rules::AtlasTile;

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 800;
const EXPLORER_WIDTH: u32 = 640;
const EXPLORER_HEIGHT: u32 = 520;
const TERRAIN_TEXTURE: u32 = 1;
const CURSOR_DEFAULT_TEXTURE: u32 = 2;
const CURSOR_HOVER_TEXTURE: u32 = 3;
const CURSOR_DELETE_TEXTURE: u32 = 4;
const CURSOR_SELECT_TEXTURE: u32 = 5;
const FOG_TEXTURE: u32 = 6;
const BUILDING_ARCHERY_TEXTURE: u32 = 7;
const BUILDING_BARRACKS_TEXTURE: u32 = 8;
const BUILDING_CASTLE_TEXTURE: u32 = 9;
const BUILDING_HOUSE1_TEXTURE: u32 = 10;
const BUILDING_HOUSE2_TEXTURE: u32 = 11;
const BUILDING_HOUSE3_TEXTURE: u32 = 12;
const BUILDING_MONASTERY_TEXTURE: u32 = 13;
const BUILDING_TOWER_TEXTURE: u32 = 14;
const RESOURCE_ROCK1_TEXTURE: u32 = 15;
const RESOURCE_ROCK2_TEXTURE: u32 = 16;
const RESOURCE_ROCK3_TEXTURE: u32 = 17;
const RESOURCE_ROCK4_TEXTURE: u32 = 18;
const RESOURCE_MEAT_TEXTURE: u32 = 19;
const RESOURCE_WOOD_TEXTURE: u32 = 20;
const RESOURCE_BUSH_TEXTURE_BASE: u32 = 24;
const RESOURCE_TREE_TEXTURE_BASE: u32 = 300;
const ASE_EXPLORER_TEXTURE_BASE: u32 = 1000;
const TERRAIN_TILE_PX: u32 = 64;
const TERRAIN_BYTES: &[u8] = include_bytes!("../ts_freepack/Terrain/Tileset/Tilemap_color2.png");
const CURSOR_DEFAULT_BYTES: &[u8] =
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Cursors/Cursor_01.png");
const CURSOR_HOVER_BYTES: &[u8] =
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Cursors/Cursor_02.png");
const CURSOR_DELETE_BYTES: &[u8] =
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Cursors/Cursor_03.png");
const CURSOR_SELECT_BYTES: &[u8] =
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Cursors/Cursor_04.png");
const FOG_BYTES: &[u8] = include_bytes!("../ts_freepack/Terrain/Tileset/Shadow.png");
const RED_ARCHERY_BYTES: &[u8] =
    include_bytes!("../ts_freepack/Buildings/Red Buildings/Archery.png");
const RED_BARRACKS_BYTES: &[u8] =
    include_bytes!("../ts_freepack/Buildings/Red Buildings/Barracks.png");
const RED_CASTLE_BYTES: &[u8] = include_bytes!("../ts_freepack/Buildings/Red Buildings/Castle.png");
const RED_HOUSE1_BYTES: &[u8] = include_bytes!("../ts_freepack/Buildings/Red Buildings/House1.png");
const RED_HOUSE2_BYTES: &[u8] = include_bytes!("../ts_freepack/Buildings/Red Buildings/House2.png");
const RED_HOUSE3_BYTES: &[u8] = include_bytes!("../ts_freepack/Buildings/Red Buildings/House3.png");
const RED_MONASTERY_BYTES: &[u8] =
    include_bytes!("../ts_freepack/Buildings/Red Buildings/Monastery.png");
const RED_TOWER_BYTES: &[u8] = include_bytes!("../ts_freepack/Buildings/Red Buildings/Tower.png");
const ROCK1_BYTES: &[u8] = include_bytes!("../ts_freepack/Rocks/Rock1.png");
const ROCK2_BYTES: &[u8] = include_bytes!("../ts_freepack/Rocks/Rock2.png");
const ROCK3_BYTES: &[u8] = include_bytes!("../ts_freepack/Rocks/Rock3.png");
const ROCK4_BYTES: &[u8] = include_bytes!("../ts_freepack/Rocks/Rock4.png");
const MEAT_BYTES: &[u8] = include_bytes!("../ts_freepack/Terrain/Tools/Meat Resource.png");
const WOOD_BYTES: &[u8] = include_bytes!("../ts_freepack/Terrain/Tools/Wood Resource.png");
const BUSHES_ASEPRITE_PATH: &str = "ts_freepack/Bushes.aseprite";
const TREES_ASEPRITE_PATH: &str = "ts_freepack/Trees.aseprite";
const WATER_BG: u32 = 0x47ABA9;
const SELECT_CORNER_SOURCES: [ImageRegion; 4] = [
    ImageRegion::new(3, 3, 21, 25),
    ImageRegion::new(104, 3, 21, 25),
    ImageRegion::new(3, 100, 21, 25),
    ImageRegion::new(104, 100, 21, 25),
];
const GRASS_BG_TILE: AtlasTile = AtlasTile { col: 1, row: 1 };
const SHORE_TOP_LEFT: AtlasTile = AtlasTile { col: 0, row: 0 };
const SHORE_TOP: AtlasTile = AtlasTile { col: 1, row: 0 };
const SHORE_TOP_RIGHT: AtlasTile = AtlasTile { col: 2, row: 0 };
const SHORE_LEFT: AtlasTile = AtlasTile { col: 0, row: 1 };
const SHORE_RIGHT: AtlasTile = AtlasTile { col: 2, row: 1 };
const SHORE_BOTTOM_LEFT: AtlasTile = AtlasTile { col: 0, row: 2 };
const SHORE_BOTTOM: AtlasTile = AtlasTile { col: 1, row: 2 };
const SHORE_BOTTOM_RIGHT: AtlasTile = AtlasTile { col: 2, row: 2 };
const SHORE_NARROW_TOP: AtlasTile = AtlasTile { col: 3, row: 0 };
const SHORE_NARROW_MIDDLE: AtlasTile = AtlasTile { col: 3, row: 1 };
const SHORE_NARROW_BOTTOM: AtlasTile = AtlasTile { col: 3, row: 2 };
const SHORE_NARROW_LEFT: AtlasTile = AtlasTile { col: 0, row: 3 };
const SHORE_NARROW_CENTER: AtlasTile = AtlasTile { col: 1, row: 3 };
const SHORE_NARROW_RIGHT: AtlasTile = AtlasTile { col: 2, row: 3 };
const SHORE_SINGLE_IN_WATER: AtlasTile = AtlasTile { col: 3, row: 3 };
const SHORE_SINGLE_IN_GRASS: AtlasTile = AtlasTile { col: 8, row: 3 };
const RAMP_A: Ramp = Ramp {
    top: AtlasTile { col: 0, row: 4 },
    bottom: AtlasTile { col: 0, row: 5 },
};
const RAMP_B: Ramp = Ramp {
    top: AtlasTile { col: 3, row: 4 },
    bottom: AtlasTile { col: 3, row: 5 },
};

const VIEW_X: f32 = 0.0;
const VIEW_Y: f32 = 0.0;
const PANEL_H: f32 = 320.0;

const TILE_SIZE: f32 = 64.0;
const WORLD_COLS: usize = 38;
const WORLD_ROWS: usize = 19;
const GENERATED_RAMP_COUNT: usize = 8;
const BUILDING_SCALE: f32 = TILE_SIZE / TERRAIN_TILE_PX as f32;
const BUILDING_COUNT: usize = 8;
const GROUND_RESOURCE_COUNT: usize = 6;
const RESOURCE_COUNT: usize = 8;
const GENERATED_RESOURCE_COUNT: usize = 32;
const GENERATED_BUSH_COUNT: usize = 18;
const GENERATED_TREE_PATCH_COUNT: usize = 9;
const VEGETATION_VARIANT_COUNT: usize = 4;

const PALETTE_X: f32 = 46.0;
const PALETTE_TILE: f32 = 48.0;
const PALETTE_GAP: f32 = 3.0;

const EDGE_SCROLL_ZONE: f32 = 72.0;
const EDGE_SCROLL_SPEED: f32 = 560.0;
const DEFAULT_SEED: u64 = 0x5EED_2026;
const ASE_EXPLORER_TINTS: [Rgba8; 5] = [
    Rgba8::WHITE,
    Rgba8::new(255, 214, 214, 255),
    Rgba8::new(216, 255, 218, 255),
    Rgba8::new(214, 235, 255, 255),
    Rgba8::new(255, 243, 188, 255),
];
const ASEPRITE_STASH: [&str; 18] = [
    "ts_freepack/Archer.aseprite",
    "ts_freepack/Bushes.aseprite",
    "ts_freepack/Clouds.aseprite",
    "ts_freepack/Gold Resource.aseprite",
    "ts_freepack/Gold Stones.aseprite",
    "ts_freepack/Lancer.aseprite",
    "ts_freepack/Monk.aseprite",
    "ts_freepack/Particle FX.aseprite",
    "ts_freepack/Pawn.aseprite",
    "ts_freepack/Rubber Duck.aseprite",
    "ts_freepack/Sheep.aseprite",
    "ts_freepack/Trees.aseprite",
    "ts_freepack/Warrior.aseprite",
    "ts_freepack/Water Foam.aseprite",
    "ts_freepack/Water Rocks_01.aseprite",
    "ts_freepack/Water Rocks_02.aseprite",
    "ts_freepack/Water Rocks_03.aseprite",
    "ts_freepack/Water Rocks_04.aseprite",
];

fn main() {
    let seed = seed_from_args(std::env::args().skip(1));
    WgpuTwoWindowApp::new(
        format!("tactics world editor - seed {seed}"),
        AdapterConfig {
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
        },
        Game::new(seed),
        "tactics aseprite explorer",
        AdapterConfig {
            width: EXPLORER_WIDTH,
            height: EXPLORER_HEIGHT,
        },
        AsepriteExplorer::new(),
    )
    .run()
    .expect("window loop failed");
}

#[derive(Clone, Copy, Debug, Default)]
struct Point {
    x: f32,
    y: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ImageRegion {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl ImageRegion {
    const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    fn uv(self, image: &ImageAsset) -> [f32; 4] {
        [
            self.x as f32 / image.width as f32,
            self.y as f32 / image.height as f32,
            (self.x + self.width) as f32 / image.width as f32,
            (self.y + self.height) as f32 / image.height as f32,
        ]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BackgroundTile {
    Grass,
    Water,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Brush {
    Background(BackgroundTile),
    Foreground(AtlasTile),
    Building(BuildingKind),
    Ramp(Ramp),
    FogRect,
    ClearForeground,
}

impl Brush {
    fn preview_tile(self) -> Option<AtlasTile> {
        match self {
            Self::Background(BackgroundTile::Grass) => Some(GRASS_BG_TILE),
            Self::Foreground(tile) => Some(tile),
            Self::Background(BackgroundTile::Water)
            | Self::Building(_)
            | Self::Ramp(_)
            | Self::FogRect
            | Self::ClearForeground => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Ramp {
    top: AtlasTile,
    bottom: AtlasTile,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BuildingKind {
    Archery,
    Barracks,
    Castle,
    House1,
    House2,
    House3,
    Monastery,
    Tower,
}

impl BuildingKind {
    fn index(self) -> usize {
        match self {
            Self::Archery => 0,
            Self::Barracks => 1,
            Self::Castle => 2,
            Self::House1 => 3,
            Self::House2 => 4,
            Self::House3 => 5,
            Self::Monastery => 6,
            Self::Tower => 7,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BuildingSpec {
    kind: BuildingKind,
    texture_id: u32,
    bytes: &'static [u8],
    footprint_cols: usize,
    footprint_rows: usize,
}

const BUILDING_SPECS: [BuildingSpec; BUILDING_COUNT] = [
    BuildingSpec {
        kind: BuildingKind::Archery,
        texture_id: BUILDING_ARCHERY_TEXTURE,
        bytes: RED_ARCHERY_BYTES,
        footprint_cols: 3,
        footprint_rows: 4,
    },
    BuildingSpec {
        kind: BuildingKind::Barracks,
        texture_id: BUILDING_BARRACKS_TEXTURE,
        bytes: RED_BARRACKS_BYTES,
        footprint_cols: 3,
        footprint_rows: 4,
    },
    BuildingSpec {
        kind: BuildingKind::Castle,
        texture_id: BUILDING_CASTLE_TEXTURE,
        bytes: RED_CASTLE_BYTES,
        footprint_cols: 5,
        footprint_rows: 4,
    },
    BuildingSpec {
        kind: BuildingKind::House1,
        texture_id: BUILDING_HOUSE1_TEXTURE,
        bytes: RED_HOUSE1_BYTES,
        footprint_cols: 2,
        footprint_rows: 3,
    },
    BuildingSpec {
        kind: BuildingKind::House2,
        texture_id: BUILDING_HOUSE2_TEXTURE,
        bytes: RED_HOUSE2_BYTES,
        footprint_cols: 2,
        footprint_rows: 3,
    },
    BuildingSpec {
        kind: BuildingKind::House3,
        texture_id: BUILDING_HOUSE3_TEXTURE,
        bytes: RED_HOUSE3_BYTES,
        footprint_cols: 2,
        footprint_rows: 3,
    },
    BuildingSpec {
        kind: BuildingKind::Monastery,
        texture_id: BUILDING_MONASTERY_TEXTURE,
        bytes: RED_MONASTERY_BYTES,
        footprint_cols: 3,
        footprint_rows: 5,
    },
    BuildingSpec {
        kind: BuildingKind::Tower,
        texture_id: BUILDING_TOWER_TEXTURE,
        bytes: RED_TOWER_BYTES,
        footprint_cols: 2,
        footprint_rows: 4,
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ResourceKind {
    Rock1,
    Rock2,
    Rock3,
    Rock4,
    Meat,
    Wood,
    Bush,
    Tree,
}

impl ResourceKind {
    fn index(self) -> usize {
        match self {
            Self::Rock1 => 0,
            Self::Rock2 => 1,
            Self::Rock3 => 2,
            Self::Rock4 => 3,
            Self::Meat => 4,
            Self::Wood => 5,
            Self::Bush => 6,
            Self::Tree => 7,
        }
    }

    #[allow(dead_code)]
    fn game_object_kind(self) -> GameObjectKind {
        match self {
            Self::Rock1 => GameObjectKind::Rock(RockKind::One),
            Self::Rock2 => GameObjectKind::Rock(RockKind::Two),
            Self::Rock3 => GameObjectKind::Rock(RockKind::Three),
            Self::Rock4 => GameObjectKind::Rock(RockKind::Four),
            Self::Meat => GameObjectKind::MeatResource,
            Self::Wood => GameObjectKind::WoodResource,
            Self::Bush => GameObjectKind::Bush,
            Self::Tree => GameObjectKind::PlantTree,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ResourceSpec {
    kind: ResourceKind,
    texture_id_base: u32,
    source: ResourceSource,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ResourceSource {
    Png(&'static [u8]),
    AsepriteGrid {
        path: &'static str,
        grid_px: u32,
        cell_cols: u32,
        cell_rows: u32,
        variant_count: usize,
    },
}

const RESOURCE_SPECS: [ResourceSpec; RESOURCE_COUNT] = [
    ResourceSpec {
        kind: ResourceKind::Rock1,
        texture_id_base: RESOURCE_ROCK1_TEXTURE,
        source: ResourceSource::Png(ROCK1_BYTES),
    },
    ResourceSpec {
        kind: ResourceKind::Rock2,
        texture_id_base: RESOURCE_ROCK2_TEXTURE,
        source: ResourceSource::Png(ROCK2_BYTES),
    },
    ResourceSpec {
        kind: ResourceKind::Rock3,
        texture_id_base: RESOURCE_ROCK3_TEXTURE,
        source: ResourceSource::Png(ROCK3_BYTES),
    },
    ResourceSpec {
        kind: ResourceKind::Rock4,
        texture_id_base: RESOURCE_ROCK4_TEXTURE,
        source: ResourceSource::Png(ROCK4_BYTES),
    },
    ResourceSpec {
        kind: ResourceKind::Meat,
        texture_id_base: RESOURCE_MEAT_TEXTURE,
        source: ResourceSource::Png(MEAT_BYTES),
    },
    ResourceSpec {
        kind: ResourceKind::Wood,
        texture_id_base: RESOURCE_WOOD_TEXTURE,
        source: ResourceSource::Png(WOOD_BYTES),
    },
    ResourceSpec {
        kind: ResourceKind::Bush,
        texture_id_base: RESOURCE_BUSH_TEXTURE_BASE,
        source: ResourceSource::AsepriteGrid {
            path: BUSHES_ASEPRITE_PATH,
            grid_px: TERRAIN_TILE_PX,
            cell_cols: 1,
            cell_rows: 1,
            variant_count: VEGETATION_VARIANT_COUNT,
        },
    },
    ResourceSpec {
        kind: ResourceKind::Tree,
        texture_id_base: RESOURCE_TREE_TEXTURE_BASE,
        source: ResourceSource::AsepriteGrid {
            path: TREES_ASEPRITE_PATH,
            grid_px: TERRAIN_TILE_PX,
            cell_cols: 2,
            cell_rows: 2,
            variant_count: VEGETATION_VARIANT_COUNT,
        },
    },
];

struct Game {
    terrain: TextureAtlas,
    cursor_default: ImageAsset,
    cursor_hover: ImageAsset,
    cursor_delete: ImageAsset,
    cursor_select: ImageAsset,
    fog: ImageAsset,
    ui_banner: ImageAsset,
    ui_big_ribbons: ImageAsset,
    ui_small_ribbons: ImageAsset,
    buildings: [ImageAsset; BUILDING_COUNT],
    resources: [ResourceAsset; RESOURCE_COUNT],
    foreground_tiles: Vec<AtlasTile>,
    world: TileWorld,
    selected: Option<Brush>,
    fog_drag_start: Option<(usize, usize)>,
    mouse: Point,
    camera: Point,
    left_down: bool,
    right_down: bool,
    uploaded: bool,
    window_width: u32,
    window_height: u32,
    started_at: Instant,
    last_frame: Instant,
}

struct AsepriteExplorer {
    files: Vec<ExplorerFile>,
    selected: usize,
    uploaded: bool,
    window_width: u32,
    window_height: u32,
    started_at: Instant,
}

struct ExplorerFile {
    frames: Vec<ExplorerFrame>,
    clips: Vec<ExplorerClip>,
    tint_index: usize,
}

struct ExplorerFrame {
    texture_id: u32,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    duration_ms: u32,
}

struct ExplorerClip {
    name: String,
    frame_indices: Vec<usize>,
    total_duration_ms: u32,
}

struct ResourceAsset {
    variants: Vec<ResourceAnimation>,
}

struct ResourceAnimation {
    frames: Vec<ImageAsset>,
    durations_ms: Vec<u32>,
    total_duration_ms: u32,
}

impl ResourceAsset {
    fn from_spec(spec: ResourceSpec) -> Self {
        match spec.source {
            ResourceSource::Png(bytes) => Self {
                variants: vec![ResourceAnimation {
                    frames: vec![ImageAsset::from_png_bytes(spec.texture_id_base, bytes)],
                    durations_ms: vec![1000],
                    total_duration_ms: 1000,
                }],
            },
            ResourceSource::AsepriteGrid {
                path,
                grid_px,
                cell_cols,
                cell_rows,
                variant_count,
            } => {
                let set = ase_assets::load_tinted_aseprite_set(
                    path,
                    [255, 255, 255, 255],
                    ase_assets::TintMode::Multiply,
                )
                .expect("resource aseprite should decode");
                let origins = variant_grid_origins(
                    set.frames
                        .first()
                        .expect("resource aseprite should have frames"),
                    grid_px,
                    cell_cols,
                    cell_rows,
                    variant_count,
                );
                let frame_count = set.frames.len();
                let mut variants = Vec::with_capacity(variant_count);

                for (variant, (cell_col, cell_row)) in origins.into_iter().enumerate() {
                    let mut frames = Vec::with_capacity(frame_count);
                    let mut durations_ms = Vec::with_capacity(frame_count);

                    for (frame_index, frame) in set.frames.iter().enumerate() {
                        let width = grid_px * cell_cols;
                        let height = grid_px * cell_rows;
                        let rgba = extract_grid_region(
                            frame, grid_px, cell_cols, cell_rows, cell_col, cell_row,
                        )
                        .unwrap_or_else(|| vec![0; (width * height * 4) as usize]);
                        frames.push(ImageAsset::from_rgba(
                            spec.texture_id_base + (variant * frame_count + frame_index) as u32,
                            width,
                            height,
                            rgba,
                        ));
                        durations_ms.push(frame.duration_ms.unwrap_or(120).max(1));
                    }

                    let total_duration_ms = durations_ms.iter().sum::<u32>().max(1);
                    variants.push(ResourceAnimation {
                        frames,
                        durations_ms,
                        total_duration_ms,
                    });
                }

                Self { variants }
            }
        }
    }

    fn frame(&self, elapsed_ms: u32, variant: usize) -> &ImageAsset {
        let animation = &self.variants[variant % self.variants.len()];
        if animation.frames.len() == 1 {
            return &animation.frames[0];
        }

        let mut cursor = elapsed_ms % animation.total_duration_ms;
        for (index, duration) in animation.durations_ms.iter().enumerate() {
            if cursor < *duration {
                return &animation.frames[index];
            }
            cursor -= *duration;
        }

        &animation.frames[0]
    }
}

impl Game {
    fn new(seed: u64) -> Self {
        let terrain = TextureAtlas::from_png_bytes(TERRAIN_TEXTURE, TERRAIN_BYTES, TERRAIN_TILE_PX);
        let foreground_tiles = terrain
            .non_empty_tiles()
            .into_iter()
            .filter(|&tile| tile != GRASS_BG_TILE && !is_ramp_part(tile))
            .collect();

        let now = Instant::now();

        Self {
            terrain,
            cursor_default: ImageAsset::from_png_bytes_cropped(
                CURSOR_DEFAULT_TEXTURE,
                CURSOR_DEFAULT_BYTES,
            ),
            cursor_hover: ImageAsset::from_png_bytes_cropped(
                CURSOR_HOVER_TEXTURE,
                CURSOR_HOVER_BYTES,
            ),
            cursor_delete: ImageAsset::from_png_bytes_cropped(
                CURSOR_DELETE_TEXTURE,
                CURSOR_DELETE_BYTES,
            ),
            cursor_select: ImageAsset::from_png_bytes(CURSOR_SELECT_TEXTURE, CURSOR_SELECT_BYTES),
            fog: ImageAsset::from_png_bytes_cropped(FOG_TEXTURE, FOG_BYTES),
            ui_banner: ImageAsset::from_png_bytes(ts_ui::BANNER_TEXTURE, ts_ui::BANNER_BYTES),
            ui_big_ribbons: ImageAsset::from_png_bytes(
                ts_ui::BIG_RIBBONS_TEXTURE,
                ts_ui::BIG_RIBBONS_BYTES,
            ),
            ui_small_ribbons: ImageAsset::from_png_bytes(
                ts_ui::SMALL_RIBBONS_TEXTURE,
                ts_ui::SMALL_RIBBONS_BYTES,
            ),
            buildings: std::array::from_fn(|index| {
                let spec = BUILDING_SPECS[index];
                ImageAsset::from_png_bytes(spec.texture_id, spec.bytes)
            }),
            resources: std::array::from_fn(|index| {
                let spec = RESOURCE_SPECS[index];
                ResourceAsset::from_spec(spec)
            }),
            foreground_tiles,
            world: TileWorld::seeded(WORLD_COLS, WORLD_ROWS, seed),
            selected: None,
            fog_drag_start: None,
            mouse: Point::default(),
            camera: Point::default(),
            left_down: false,
            right_down: false,
            uploaded: false,
            window_width: WINDOW_WIDTH,
            window_height: WINDOW_HEIGHT,
            started_at: now,
            last_frame: now,
        }
    }

    fn resize_view(&mut self, width: u32, height: u32) {
        self.window_width = width.max(1);
        self.window_height = height.max(1);
        self.clamp_camera();
    }

    fn window_w(&self) -> f32 {
        self.window_width as f32
    }

    fn window_h(&self) -> f32 {
        self.window_height as f32
    }

    fn view_w(&self) -> f32 {
        self.window_w()
    }

    fn view_h(&self) -> f32 {
        self.panel_y()
    }

    fn panel_y(&self) -> f32 {
        if self.window_h() <= PANEL_H + TILE_SIZE {
            (self.window_h() - TILE_SIZE).max(0.0)
        } else {
            self.window_h() - PANEL_H
        }
    }

    fn palette_y(&self) -> f32 {
        self.panel_y() + 26.0
    }

    fn upload_assets(&mut self, adapter: &mut Adapter) {
        if self.uploaded {
            return;
        }

        let rc = adapter.upload_texture_rgba_image(
            self.terrain.texture_id,
            self.terrain.width,
            self.terrain.height,
            &self.terrain.rgba,
        );
        assert_eq!(rc, 0, "failed to upload terrain texture");
        for image in [
            &self.cursor_default,
            &self.cursor_hover,
            &self.cursor_delete,
            &self.cursor_select,
            &self.fog,
            &self.ui_banner,
            &self.ui_big_ribbons,
            &self.ui_small_ribbons,
        ] {
            let rc = adapter.upload_texture_rgba_image(
                image.texture_id,
                image.width,
                image.height,
                &image.rgba,
            );
            assert_eq!(rc, 0, "failed to upload ui texture {}", image.texture_id);
        }
        for image in &self.buildings {
            let rc = adapter.upload_texture_rgba_image(
                image.texture_id,
                image.width,
                image.height,
                &image.rgba,
            );
            assert_eq!(
                rc, 0,
                "failed to upload building texture {}",
                image.texture_id
            );
        }
        for resource in &self.resources {
            for animation in &resource.variants {
                for image in &animation.frames {
                    let rc = adapter.upload_texture_rgba_image(
                        image.texture_id,
                        image.width,
                        image.height,
                        &image.rgba,
                    );
                    assert_eq!(
                        rc, 0,
                        "failed to upload resource texture {}",
                        image.texture_id
                    );
                }
            }
        }
        self.uploaded = true;
    }

    fn update_camera(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f32().min(0.05);
        self.last_frame = now;
        let view_w = self.view_w();
        let view_h = self.view_h();

        if !inside_rect(self.mouse.x, self.mouse.y, VIEW_X, VIEW_Y, view_w, view_h) {
            return;
        }

        let mut dx = 0.0;
        let mut dy = 0.0;
        if self.mouse.x < VIEW_X + EDGE_SCROLL_ZONE {
            dx = -scroll_strength(self.mouse.x - VIEW_X);
        } else if self.mouse.x > VIEW_X + view_w - EDGE_SCROLL_ZONE {
            dx = scroll_strength(VIEW_X + view_w - self.mouse.x);
        }

        if self.mouse.y < VIEW_Y + EDGE_SCROLL_ZONE {
            dy = -scroll_strength(self.mouse.y - VIEW_Y);
        } else if self.mouse.y > VIEW_Y + view_h - EDGE_SCROLL_ZONE {
            dy = scroll_strength(VIEW_Y + view_h - self.mouse.y);
        }

        self.camera.x += dx * EDGE_SCROLL_SPEED * dt;
        self.camera.y += dy * EDGE_SCROLL_SPEED * dt;
        self.clamp_camera();
    }

    fn clamp_camera(&mut self) {
        let max_x = (self.world.width_px() - self.view_w()).max(0.0);
        let max_y = (self.world.height_px() - self.view_h()).max(0.0);
        self.camera.x = self.camera.x.clamp(0.0, max_x);
        self.camera.y = self.camera.y.clamp(0.0, max_y);
    }

    fn handle_left_press(&mut self) {
        if let Some(brush) = self.palette_brush_at(self.mouse.x, self.mouse.y) {
            self.selected = Some(brush);
            self.fog_drag_start = None;
            return;
        }

        if self.selected == Some(Brush::FogRect) {
            self.fog_drag_start = self.world_cell_at(self.mouse.x, self.mouse.y);
            return;
        }

        self.paint_at_mouse();
    }

    fn paint_at_mouse(&mut self) {
        let Some(brush) = self.selected else {
            return;
        };
        let Some((col, row)) = self.world_cell_at(self.mouse.x, self.mouse.y) else {
            return;
        };
        self.world.paint(col, row, brush);
    }

    fn erase_at_mouse(&mut self) {
        let Some((col, row)) = self.world_cell_at(self.mouse.x, self.mouse.y) else {
            return;
        };
        self.world.clear_foreground(col, row);
    }

    fn world_cell_at(&self, x: f32, y: f32) -> Option<(usize, usize)> {
        if !inside_rect(x, y, VIEW_X, VIEW_Y, self.view_w(), self.view_h()) {
            return None;
        }

        let world_x = x - VIEW_X + self.camera.x;
        let world_y = y - VIEW_Y + self.camera.y;
        let col = (world_x / TILE_SIZE).floor() as usize;
        let row = (world_y / TILE_SIZE).floor() as usize;
        if col < self.world.cols && row < self.world.rows {
            Some((col, row))
        } else {
            None
        }
    }

    fn palette_brush_at(&self, x: f32, y: f32) -> Option<Brush> {
        let palette_y = self.palette_y();
        if y < palette_y {
            return None;
        }

        let rel_x = x - PALETTE_X;
        let rel_y = y - palette_y;
        if rel_x < 0.0 || rel_y < 0.0 {
            return None;
        }

        let step = PALETTE_TILE + PALETTE_GAP;
        let col = (rel_x / step).floor() as usize;
        let row = (rel_y / step).floor() as usize;
        let tile_x = PALETTE_X + col as f32 * step;
        let tile_y = palette_y + row as f32 * step;
        if x > tile_x + PALETTE_TILE || y > tile_y + PALETTE_TILE {
            return None;
        }

        let slot = row * self.palette_slots_per_row() + col;
        if slot >= self.palette_len() {
            return None;
        }

        match slot {
            0 => Some(Brush::Background(BackgroundTile::Water)),
            1 => Some(Brush::Background(BackgroundTile::Grass)),
            2 => Some(Brush::ClearForeground),
            3 => Some(Brush::Ramp(RAMP_A)),
            4 => Some(Brush::Ramp(RAMP_B)),
            5 => Some(Brush::FogRect),
            _ if slot < 6 + BUILDING_COUNT => Some(Brush::Building(BUILDING_SPECS[slot - 6].kind)),
            _ => self
                .foreground_tiles
                .get(slot - 6 - BUILDING_COUNT)
                .copied()
                .map(Brush::Foreground),
        }
    }

    fn draw(&self, adapter: &mut Adapter) {
        let _ = adapter.begin_frame(WATER_BG);
        let _ = adapter.set_sampler_raw(0, 0, 0, 0);
        let _ = adapter.set_blend_raw(1, 0x0302, 0x0303);
        let panel_y = self.panel_y();
        let panel_h = self.window_h() - panel_y;
        let view_w = self.view_w();
        let view_h = self.view_h();

        let mut solid = SolidBatch::new(self.window_width, self.window_height);
        solid.rect(
            0.0,
            panel_y,
            self.window_w(),
            panel_h,
            Rgba8::new(39, 91, 93, 215),
        );
        solid.rect(
            10.0,
            panel_y + 10.0,
            self.window_w() - 20.0,
            panel_h - 20.0,
            Rgba8::new(30, 74, 77, 225),
        );
        let _ = adapter.draw_rgb_triangles_no_present(&solid.bytes);

        let _ = adapter.set_scissor(Some(ScissorRect {
            x: VIEW_X as u32,
            y: VIEW_Y as u32,
            width: view_w as u32,
            height: view_h as u32,
        }));
        self.draw_world(adapter);
        self.draw_world_overlay(adapter);

        let _ = adapter.set_scissor(None);
        self.draw_palette(adapter);
        self.draw_ui(adapter);
        self.draw_cursor(adapter);

        let _ = adapter.end_frame();
    }

    fn draw_world(&self, adapter: &mut Adapter) {
        let mut sprites = SpriteBatch::new(self.window_width, self.window_height);
        let start_col = (self.camera.x / TILE_SIZE).floor().max(0.0) as usize;
        let start_row = (self.camera.y / TILE_SIZE).floor().max(0.0) as usize;
        let end_col = ((self.camera.x + self.view_w()) / TILE_SIZE).ceil() as usize + 1;
        let end_row = ((self.camera.y + self.view_h()) / TILE_SIZE).ceil() as usize + 1;

        for row in start_row..end_row.min(self.world.rows) {
            for col in start_col..end_col.min(self.world.cols) {
                let x = VIEW_X + col as f32 * TILE_SIZE - self.camera.x;
                let y = VIEW_Y + row as f32 * TILE_SIZE - self.camera.y;
                if self.world.render_background(col, row) == BackgroundTile::Grass {
                    sprites.sprite(
                        &self.terrain,
                        GRASS_BG_TILE,
                        x,
                        y,
                        TILE_SIZE,
                        TILE_SIZE,
                        Rgba8::WHITE,
                    );
                }

                if let Some(tile) = self.world.foreground(col, row) {
                    sprites.sprite(
                        &self.terrain,
                        tile,
                        x,
                        y,
                        TILE_SIZE,
                        TILE_SIZE,
                        Rgba8::WHITE,
                    );
                }
            }
        }

        if let (Some(brush), Some((col, row))) = (
            self.selected,
            self.world_cell_at(self.mouse.x, self.mouse.y),
        ) {
            let x = VIEW_X + col as f32 * TILE_SIZE - self.camera.x;
            let y = VIEW_Y + row as f32 * TILE_SIZE - self.camera.y;
            match brush {
                Brush::Ramp(ramp) if row + 1 < self.world.rows => {
                    sprites.sprite(
                        &self.terrain,
                        ramp.top,
                        x,
                        y,
                        TILE_SIZE,
                        TILE_SIZE,
                        Rgba8::new(255, 255, 255, 130),
                    );
                    sprites.sprite(
                        &self.terrain,
                        ramp.bottom,
                        x,
                        y + TILE_SIZE,
                        TILE_SIZE,
                        TILE_SIZE,
                        Rgba8::new(255, 255, 255, 130),
                    );
                }
                _ => {
                    if let Some(tile) = brush.preview_tile() {
                        sprites.sprite(
                            &self.terrain,
                            tile,
                            x,
                            y,
                            TILE_SIZE,
                            TILE_SIZE,
                            Rgba8::new(255, 255, 255, 130),
                        );
                    }
                }
            }
        }

        let _ = adapter.draw_tex_triangles_no_present(self.terrain.texture_id, &sprites.bytes);
        self.draw_buildings(adapter);
        self.draw_resources(adapter);
        self.draw_building_preview(adapter);
        self.draw_fog(adapter);
    }

    fn draw_buildings(&self, adapter: &mut Adapter) {
        for building in &self.world.buildings {
            let image = &self.buildings[building.kind.index()];
            let x = VIEW_X + building.col as f32 * TILE_SIZE - self.camera.x;
            let y = VIEW_Y + building.row as f32 * TILE_SIZE - self.camera.y;
            let w = image.width as f32 * BUILDING_SCALE;
            let h = image.height as f32 * BUILDING_SCALE;
            if x + w < VIEW_X
                || y + h < VIEW_Y
                || x > VIEW_X + self.view_w()
                || y > VIEW_Y + self.view_h()
            {
                continue;
            }

            let mut sprite = SpriteBatch::new(self.window_width, self.window_height);
            sprite.image(x, y, w, h, Rgba8::WHITE);
            let _ = adapter.draw_tex_triangles_no_present(image.texture_id, &sprite.bytes);
        }
    }

    fn draw_resources(&self, adapter: &mut Adapter) {
        let elapsed_ms = self.started_at.elapsed().as_millis() as u32;
        let mut resources: Vec<_> = self.world.resources.iter().collect();
        resources.sort_by_key(|resource| {
            let (_, rows) = resource_footprint(resource.kind);
            (resource.row + rows, resource.col)
        });

        for resource in resources {
            let image = self.resources[resource.kind.index()].frame(elapsed_ms, resource.variant);
            let x = VIEW_X + resource.col as f32 * TILE_SIZE - self.camera.x;
            let y = VIEW_Y + resource.row as f32 * TILE_SIZE - self.camera.y;
            let w = image.width as f32 * BUILDING_SCALE;
            let h = image.height as f32 * BUILDING_SCALE;
            if x + w < VIEW_X
                || y + h < VIEW_Y
                || x > VIEW_X + self.view_w()
                || y > VIEW_Y + self.view_h()
            {
                continue;
            }

            let mut sprite = SpriteBatch::new(self.window_width, self.window_height);
            sprite.image(
                x,
                y,
                w,
                h,
                Rgba8::new(
                    resource.tint[0],
                    resource.tint[1],
                    resource.tint[2],
                    resource.tint[3],
                ),
            );
            let _ = adapter.draw_tex_triangles_no_present(image.texture_id, &sprite.bytes);
        }
    }

    fn draw_building_preview(&self, adapter: &mut Adapter) {
        let Some(Brush::Building(kind)) = self.selected else {
            return;
        };
        let Some((col, row)) = self.world_cell_at(self.mouse.x, self.mouse.y) else {
            return;
        };

        let image = &self.buildings[kind.index()];
        let mut sprite = SpriteBatch::new(self.window_width, self.window_height);
        sprite.image(
            VIEW_X + col as f32 * TILE_SIZE - self.camera.x,
            VIEW_Y + row as f32 * TILE_SIZE - self.camera.y,
            image.width as f32 * BUILDING_SCALE,
            image.height as f32 * BUILDING_SCALE,
            Rgba8::new(255, 255, 255, 145),
        );
        let _ = adapter.draw_tex_triangles_no_present(image.texture_id, &sprite.bytes);
    }

    fn draw_fog(&self, adapter: &mut Adapter) {
        let mut fog = SpriteBatch::new(self.window_width, self.window_height);
        let start_col = (self.camera.x / TILE_SIZE).floor().max(0.0) as usize;
        let start_row = (self.camera.y / TILE_SIZE).floor().max(0.0) as usize;
        let end_col = ((self.camera.x + self.view_w()) / TILE_SIZE).ceil() as usize + 1;
        let end_row = ((self.camera.y + self.view_h()) / TILE_SIZE).ceil() as usize + 1;

        for row in start_row..end_row.min(self.world.rows) {
            for col in start_col..end_col.min(self.world.cols) {
                if self.world.fog(col, row) {
                    fog.image(
                        VIEW_X + col as f32 * TILE_SIZE - self.camera.x,
                        VIEW_Y + row as f32 * TILE_SIZE - self.camera.y,
                        TILE_SIZE,
                        TILE_SIZE,
                        Rgba8::new(255, 255, 255, 190),
                    );
                }
            }
        }

        let _ = adapter.draw_tex_triangles_no_present(self.fog.texture_id, &fog.bytes);
    }

    fn draw_world_overlay(&self, adapter: &mut Adapter) {
        let mut overlay = SolidBatch::new(self.window_width, self.window_height);
        if self.selected.is_some() {
            if let Some((col, row)) = self.world_cell_at(self.mouse.x, self.mouse.y) {
                if self.selected == Some(Brush::FogRect) {
                    if self.left_down && self.fog_drag_start.is_some() {
                        let rect = self.fog_selection_rect((col, row));
                        let x = VIEW_X + rect.0 as f32 * TILE_SIZE - self.camera.x;
                        let y = VIEW_Y + rect.1 as f32 * TILE_SIZE - self.camera.y;
                        let width = ((rect.2 - rect.0 + 1) as f32) * TILE_SIZE;
                        let height = ((rect.3 - rect.1 + 1) as f32) * TILE_SIZE;
                        self.draw_selection_corners(adapter, x, y, width, height);
                    }
                    let _ = adapter.draw_rgb_triangles_no_present(&overlay.bytes);
                    return;
                }

                let rect = (col, row, col, row);
                let height =
                    if matches!(self.selected, Some(Brush::Ramp(_))) && row + 1 < self.world.rows {
                        TILE_SIZE * 2.0
                    } else {
                        ((rect.3 - rect.1 + 1) as f32) * TILE_SIZE
                    };
                let x = VIEW_X + rect.0 as f32 * TILE_SIZE - self.camera.x;
                let y = VIEW_Y + rect.1 as f32 * TILE_SIZE - self.camera.y;
                let width = ((rect.2 - rect.0 + 1) as f32) * TILE_SIZE;
                outline_rect(
                    &mut overlay,
                    x,
                    y,
                    width,
                    height,
                    2.0,
                    Rgba8::new(255, 225, 118, 210),
                );
            }
        }

        let _ = adapter.draw_rgb_triangles_no_present(&overlay.bytes);
    }

    fn draw_selection_corners(&self, adapter: &mut Adapter, x: f32, y: f32, w: f32, h: f32) {
        let mut corners = SpriteBatch::new(self.window_width, self.window_height);
        let [top_left, top_right, bottom_left, bottom_right] = SELECT_CORNER_SOURCES;
        let corner_w = top_left.width as f32;
        let corner_h = top_left.height as f32;

        corners.image_region(
            &self.cursor_select,
            top_left,
            x,
            y,
            corner_w,
            corner_h,
            Rgba8::WHITE,
        );
        corners.image_region(
            &self.cursor_select,
            top_right,
            x + w - corner_w,
            y,
            corner_w,
            corner_h,
            Rgba8::WHITE,
        );
        corners.image_region(
            &self.cursor_select,
            bottom_left,
            x,
            y + h - corner_h,
            corner_w,
            corner_h,
            Rgba8::WHITE,
        );
        corners.image_region(
            &self.cursor_select,
            bottom_right,
            x + w - corner_w,
            y + h - corner_h,
            corner_w,
            corner_h,
            Rgba8::WHITE,
        );

        let _ =
            adapter.draw_tex_triangles_no_present(self.cursor_select.texture_id, &corners.bytes);
    }

    fn draw_palette(&self, adapter: &mut Adapter) {
        let mut solid = SolidBatch::new(self.window_width, self.window_height);
        let (water_x, water_y) = self.palette_slot_rect(0);
        solid.rect(
            water_x,
            water_y,
            PALETTE_TILE,
            PALETTE_TILE,
            Rgba8::new(71, 171, 169, 255),
        );
        let (clear_x, clear_y) = self.palette_slot_rect(2);
        solid.rect(
            clear_x,
            clear_y,
            PALETTE_TILE,
            PALETTE_TILE,
            Rgba8::new(19, 45, 48, 255),
        );
        let _ = adapter.draw_rgb_triangles_no_present(&solid.bytes);

        let mut delete_tool = SpriteBatch::new(self.window_width, self.window_height);
        let (delete_x, delete_y) = self.palette_slot_rect(2);
        delete_tool.image(delete_x, delete_y, PALETTE_TILE, PALETTE_TILE, Rgba8::WHITE);
        let _ = adapter
            .draw_tex_triangles_no_present(self.cursor_delete.texture_id, &delete_tool.bytes);

        let mut sprites = SpriteBatch::new(self.window_width, self.window_height);
        let (grass_x, grass_y) = self.palette_slot_rect(1);
        sprites.sprite(
            &self.terrain,
            GRASS_BG_TILE,
            grass_x,
            grass_y,
            PALETTE_TILE,
            PALETTE_TILE,
            Rgba8::WHITE,
        );
        for (slot, ramp) in [RAMP_A, RAMP_B].into_iter().enumerate() {
            let (x, y) = self.palette_slot_rect(slot + 3);
            sprites.sprite(
                &self.terrain,
                ramp.top,
                x,
                y,
                PALETTE_TILE,
                PALETTE_TILE / 2.0,
                Rgba8::WHITE,
            );
            sprites.sprite(
                &self.terrain,
                ramp.bottom,
                x,
                y + PALETTE_TILE / 2.0,
                PALETTE_TILE,
                PALETTE_TILE / 2.0,
                Rgba8::WHITE,
            );
        }
        let _ = adapter.draw_tex_triangles_no_present(self.terrain.texture_id, &sprites.bytes);

        let mut fog_tool = SpriteBatch::new(self.window_width, self.window_height);
        let (fog_x, fog_y) = self.palette_slot_rect(5);
        fog_tool.image(fog_x, fog_y, PALETTE_TILE, PALETTE_TILE, Rgba8::WHITE);
        let _ =
            adapter.draw_tex_triangles_no_present(self.cursor_select.texture_id, &fog_tool.bytes);

        for (index, image) in self.buildings.iter().enumerate() {
            let (x, y) = self.palette_slot_rect(index + 6);
            let mut building = SpriteBatch::new(self.window_width, self.window_height);
            building.image(x, y, PALETTE_TILE, PALETTE_TILE, Rgba8::WHITE);
            let _ = adapter.draw_tex_triangles_no_present(image.texture_id, &building.bytes);
        }

        let mut sprites = SpriteBatch::new(self.window_width, self.window_height);
        for (slot, &tile) in self.foreground_tiles.iter().enumerate() {
            let (x, y) = self.palette_slot_rect(slot + 6 + BUILDING_COUNT);
            sprites.sprite(
                &self.terrain,
                tile,
                x,
                y,
                PALETTE_TILE,
                PALETTE_TILE,
                Rgba8::WHITE,
            );
        }
        let _ = adapter.draw_tex_triangles_no_present(self.terrain.texture_id, &sprites.bytes);

        let mut overlay = SolidBatch::new(self.window_width, self.window_height);
        for slot in 0..self.palette_len() {
            let (x, y) = self.palette_slot_rect(slot);
            let brush = self.palette_brush(slot);
            let color = if Some(brush) == self.selected {
                Rgba8::new(255, 225, 118, 255)
            } else if self.palette_brush_at(self.mouse.x, self.mouse.y) == Some(brush) {
                Rgba8::new(183, 205, 197, 180)
            } else {
                Rgba8::new(99, 120, 121, 140)
            };
            outline_rect(
                &mut overlay,
                x - 2.0,
                y - 2.0,
                PALETTE_TILE + 4.0,
                PALETTE_TILE + 4.0,
                2.0,
                color,
            );
        }

        let _ = adapter.draw_rgb_triangles_no_present(&overlay.bytes);
    }

    fn palette_len(&self) -> usize {
        self.foreground_tiles.len() + 6 + BUILDING_COUNT
    }

    fn palette_slots_per_row(&self) -> usize {
        ((self.window_w() - PALETTE_X * 2.0 + PALETTE_GAP) / (PALETTE_TILE + PALETTE_GAP))
            .floor()
            .max(1.0) as usize
    }

    fn palette_slot_rect(&self, slot: usize) -> (f32, f32) {
        let slots_per_row = self.palette_slots_per_row();
        let col = slot % slots_per_row;
        let row = slot / slots_per_row;
        let step = PALETTE_TILE + PALETTE_GAP;
        (
            PALETTE_X + col as f32 * step,
            self.palette_y() + row as f32 * step,
        )
    }

    fn palette_brush(&self, slot: usize) -> Brush {
        match slot {
            0 => Brush::Background(BackgroundTile::Water),
            1 => Brush::Background(BackgroundTile::Grass),
            2 => Brush::ClearForeground,
            3 => Brush::Ramp(RAMP_A),
            4 => Brush::Ramp(RAMP_B),
            5 => Brush::FogRect,
            _ if slot < 6 + BUILDING_COUNT => Brush::Building(BUILDING_SPECS[slot - 6].kind),
            _ => Brush::Foreground(self.foreground_tiles[slot - 6 - BUILDING_COUNT]),
        }
    }

    fn fog_selection_rect(&self, current: (usize, usize)) -> (usize, usize, usize, usize) {
        let start = self.fog_drag_start.unwrap_or(current);
        (
            start.0.min(current.0),
            start.1.min(current.1),
            start.0.max(current.0),
            start.1.max(current.1),
        )
    }

    fn draw_cursor(&self, adapter: &mut Adapter) {
        let image = self.cursor_image();
        let mut cursor = SpriteBatch::new(self.window_width, self.window_height);
        let size = if image.texture_id == CURSOR_SELECT_TEXTURE {
            42.0
        } else {
            28.0
        };
        cursor.image(self.mouse.x, self.mouse.y, size, size, Rgba8::WHITE);
        let _ = adapter.draw_tex_triangles_no_present(image.texture_id, &cursor.bytes);
    }

    fn draw_ui(&self, adapter: &mut Adapter) {
        let mut ui = ts_ui::UiBatch::new(self.window_width, self.window_height);
        let panel_y = self.panel_y();
        let panel_h = self.window_h() - panel_y;
        ui.small_ribbon(
            0,
            self.window_w() - 236.0,
            panel_y + panel_h - 48.0,
            206.0,
            34.0,
            Rgba8::new(255, 255, 255, 235),
        );
        ui.text(
            "BUILDINGS",
            self.window_w() - 198.0,
            panel_y + panel_h - 36.0,
            2.0,
            Rgba8::new(34, 57, 60, 255),
        );

        let _ = adapter
            .draw_tex_triangles_no_present(self.ui_small_ribbons.texture_id, &ui.texture_bytes);
        let _ = adapter.draw_rgb_triangles_no_present(&ui.solid_bytes);
    }

    fn cursor_image(&self) -> &ImageAsset {
        if self.selected == Some(Brush::ClearForeground) {
            &self.cursor_delete
        } else if self.palette_brush_at(self.mouse.x, self.mouse.y).is_some() {
            &self.cursor_hover
        } else {
            &self.cursor_default
        }
    }

    fn finish_fog_drag(&mut self) {
        if self.selected != Some(Brush::FogRect) {
            self.fog_drag_start = None;
            return;
        }

        let Some(current) = self.world_cell_at(self.mouse.x, self.mouse.y) else {
            self.fog_drag_start = None;
            return;
        };
        let Some(start) = self.fog_drag_start.take() else {
            return;
        };
        self.world.add_fog_rect(start, current);
    }
}

impl FrameProducer for Game {
    fn cursor_visible(&self) -> bool {
        false
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.resize_view(width, height);
    }

    fn handle_input(&mut self, event: InputEvent) {
        match event {
            InputEvent::CursorMoved { x, y } => {
                self.mouse = Point { x, y };
                if self.left_down && self.selected != Some(Brush::FogRect) {
                    self.paint_at_mouse();
                }
                if self.right_down {
                    self.erase_at_mouse();
                }
            }
            InputEvent::MouseButton {
                button: InputMouseButton::Left,
                state: InputButtonState::Pressed,
            } => {
                self.left_down = true;
                self.handle_left_press();
            }
            InputEvent::MouseButton {
                button: InputMouseButton::Left,
                state: InputButtonState::Released,
            } => {
                self.finish_fog_drag();
                self.left_down = false;
            }
            InputEvent::MouseButton {
                button: InputMouseButton::Right,
                state: InputButtonState::Pressed,
            } => {
                self.right_down = true;
                self.erase_at_mouse();
            }
            InputEvent::MouseButton {
                button: InputMouseButton::Right,
                state: InputButtonState::Released,
            } => {
                self.right_down = false;
            }
            InputEvent::EscapePressed => {
                self.selected = None;
                self.fog_drag_start = None;
                self.left_down = false;
                self.right_down = false;
            }
            _ => {}
        }
    }

    fn build_frame(&mut self, adapter: &mut Adapter) {
        self.upload_assets(adapter);
        self.update_camera();
        self.draw(adapter);
    }
}

impl AsepriteExplorer {
    fn new() -> Self {
        let mut next_texture_id = ASE_EXPLORER_TEXTURE_BASE;
        let files = ASEPRITE_STASH
            .iter()
            .filter_map(|path| {
                let set = ase_assets::load_tinted_aseprite_set(
                    path,
                    [255, 255, 255, 255],
                    ase_assets::TintMode::Multiply,
                )
                .ok()?;
                let frames = set
                    .frames
                    .into_iter()
                    .map(|frame| {
                        let texture_id = next_texture_id;
                        next_texture_id += 1;
                        ExplorerFrame {
                            texture_id,
                            width: frame.width,
                            height: frame.height,
                            rgba: frame.rgba,
                            duration_ms: frame.duration_ms.unwrap_or(120).max(1),
                        }
                    })
                    .collect::<Vec<_>>();
                if frames.is_empty() {
                    return None;
                }
                let clips = explorer_clips_from_tags(&set.tags, &frames);

                Some(ExplorerFile {
                    frames,
                    clips,
                    tint_index: 0,
                })
            })
            .collect();

        Self {
            files,
            selected: 0,
            uploaded: false,
            window_width: EXPLORER_WIDTH,
            window_height: EXPLORER_HEIGHT,
            started_at: Instant::now(),
        }
    }

    fn resize_view(&mut self, width: u32, height: u32) {
        self.window_width = width.max(1);
        self.window_height = height.max(1);
    }

    fn upload_assets(&mut self, adapter: &mut Adapter) {
        if self.uploaded {
            return;
        }

        for image in [
            ImageAsset::from_png_bytes(ts_ui::BANNER_TEXTURE, ts_ui::BANNER_BYTES),
            ImageAsset::from_png_bytes(ts_ui::BIG_RIBBONS_TEXTURE, ts_ui::BIG_RIBBONS_BYTES),
        ] {
            let rc = adapter.upload_texture_rgba_image(
                image.texture_id,
                image.width,
                image.height,
                &image.rgba,
            );
            assert_eq!(
                rc, 0,
                "failed to upload explorer ui texture {}",
                image.texture_id
            );
        }

        for file in &self.files {
            for frame in &file.frames {
                let rc = adapter.upload_texture_rgba_image(
                    frame.texture_id,
                    frame.width,
                    frame.height,
                    &frame.rgba,
                );
                assert_eq!(
                    rc, 0,
                    "failed to upload explorer aseprite texture {}",
                    frame.texture_id
                );
            }
        }

        self.uploaded = true;
    }

    fn select_next(&mut self, steps: isize) {
        if self.files.is_empty() {
            return;
        }

        let len = self.files.len() as isize;
        self.selected = (self.selected as isize + steps).rem_euclid(len) as usize;
    }

    fn cycle_tint(&mut self) {
        if let Some(file) = self.files.get_mut(self.selected) {
            file.tint_index = (file.tint_index + 1) % ASE_EXPLORER_TINTS.len();
        }
    }
}

fn explorer_clips_from_tags(
    tags: &[ase_assets::AsepriteTag],
    frames: &[ExplorerFrame],
) -> Vec<ExplorerClip> {
    let clips = tags
        .iter()
        .filter_map(|tag| {
            let frame_indices = (tag.from_frame..=tag.to_frame)
                .filter_map(|frame| {
                    let index = frame as usize;
                    (index < frames.len()).then_some(index)
                })
                .collect::<Vec<_>>();
            if frame_indices.is_empty() {
                return None;
            }

            let total_duration_ms = frame_indices
                .iter()
                .map(|&index| frames[index].duration_ms)
                .sum::<u32>()
                .max(1);

            Some(ExplorerClip {
                name: tag.name.clone(),
                frame_indices,
                total_duration_ms,
            })
        })
        .collect::<Vec<_>>();

    if !clips.is_empty() {
        return clips;
    }

    frames
        .iter()
        .enumerate()
        .map(|(index, frame)| ExplorerClip {
            name: format!("Frame {}", index + 1),
            frame_indices: vec![index],
            total_duration_ms: frame.duration_ms.max(1),
        })
        .collect()
}

fn explorer_clip_frame<'a>(
    clip: &ExplorerClip,
    frames: &'a [ExplorerFrame],
    elapsed_ms: u32,
) -> &'a ExplorerFrame {
    if clip.frame_indices.len() == 1 {
        return &frames[clip.frame_indices[0]];
    }

    let mut cursor = elapsed_ms % clip.total_duration_ms;
    for &frame_index in &clip.frame_indices {
        let frame = &frames[frame_index];
        if cursor < frame.duration_ms {
            return frame;
        }
        cursor -= frame.duration_ms;
    }

    &frames[clip.frame_indices[0]]
}

impl FrameProducer for AsepriteExplorer {
    fn resize(&mut self, width: u32, height: u32) {
        self.resize_view(width, height);
    }

    fn handle_input(&mut self, event: InputEvent) {
        match event {
            InputEvent::MouseWheel { y, .. } => {
                if y > 0.0 {
                    self.select_next(-1);
                } else if y < 0.0 {
                    self.select_next(1);
                }
            }
            InputEvent::MouseButton {
                button: InputMouseButton::Left,
                state: InputButtonState::Pressed,
            } => {
                self.cycle_tint();
            }
            _ => {}
        }
    }

    fn build_frame(&mut self, adapter: &mut Adapter) {
        self.upload_assets(adapter);

        let _ = adapter.begin_frame(0x243C40);
        let _ = adapter.set_sampler_raw(0, 0, 0, 0);
        let _ = adapter.set_blend_raw(1, 0x0302, 0x0303);

        let explorer_w = self.window_width as f32;
        let explorer_h = self.window_height as f32;
        let mut title = ts_ui::UiBatch::new(self.window_width, self.window_height);
        title.big_ribbon(
            0,
            22.0,
            18.0,
            explorer_w - 44.0,
            48.0,
            Rgba8::new(255, 255, 255, 235),
        );
        let total = self.files.len().max(1);
        title.text(
            &format!("ASE {:02}-{:02}", self.selected + 1, total),
            44.0,
            35.0,
            2.0,
            Rgba8::new(32, 56, 60, 255),
        );
        let _ =
            adapter.draw_tex_triangles_no_present(ts_ui::BIG_RIBBONS_TEXTURE, &title.texture_bytes);
        let _ = adapter.draw_rgb_triangles_no_present(&title.solid_bytes);

        if let Some(file) = self.files.get(self.selected) {
            let elapsed_ms = self.started_at.elapsed().as_millis() as u32;
            let grid_x = 22.0;
            let grid_y = 82.0;
            let grid_w = (explorer_w - 44.0).max(1.0);
            let grid_h = (explorer_h - 154.0).max(1.0);
            let count = file.clips.len().max(1);
            let cols = ((count as f32 * grid_w / grid_h).sqrt().ceil() as usize)
                .max(1)
                .min(count);
            let rows = count.div_ceil(cols).max(1);
            let cell_w = grid_w / cols as f32;
            let cell_h = grid_h / rows as f32;
            let tint = ASE_EXPLORER_TINTS[file.tint_index];

            for (index, clip) in file.clips.iter().enumerate() {
                let frame = explorer_clip_frame(clip, &file.frames, elapsed_ms);
                let col = index % cols;
                let row = index / cols;
                let scale = (cell_w / frame.width as f32)
                    .min(cell_h / frame.height as f32)
                    .min(2.0);
                let w = frame.width as f32 * scale;
                let h = frame.height as f32 * scale;
                let x = grid_x + col as f32 * cell_w + (cell_w - w) * 0.5;
                let y = grid_y + row as f32 * cell_h + (cell_h - h) * 0.5;

                let mut image = SpriteBatch::new(self.window_width, self.window_height);
                image.image(
                    x.floor(),
                    y.floor(),
                    w.floor().max(1.0),
                    h.floor().max(1.0),
                    tint,
                );
                let _ = adapter.draw_tex_triangles_no_present(frame.texture_id, &image.bytes);
            }

            let mut caption = ts_ui::UiBatch::new(self.window_width, self.window_height);
            caption.banner_panel(
                22.0,
                explorer_h - 58.0,
                explorer_w - 44.0,
                34.0,
                12.0,
                Rgba8::new(255, 255, 255, 220),
            );
            caption.text(
                &format!(
                    "ASE {}-{} {}",
                    file.clips.len(),
                    file.frames.len(),
                    file.clips
                        .first()
                        .map(|clip| clip.name.as_str())
                        .unwrap_or("frames")
                ),
                44.0,
                explorer_h - 46.0,
                2.0,
                Rgba8::new(32, 56, 60, 255),
            );
            let _ = adapter
                .draw_tex_triangles_no_present(ts_ui::BANNER_TEXTURE, &caption.texture_bytes);
            let _ = adapter.draw_rgb_triangles_no_present(&caption.solid_bytes);
        }

        let _ = adapter.end_frame();
    }
}

struct TileWorld {
    cols: usize,
    rows: usize,
    backgrounds: Vec<BackgroundTile>,
    foregrounds: Vec<Option<AtlasTile>>,
    buildings: Vec<PlacedBuilding>,
    resources: Vec<PlacedResource>,
    fog: Vec<bool>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PlacedBuilding {
    kind: BuildingKind,
    col: usize,
    row: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PlacedResource {
    kind: ResourceKind,
    col: usize,
    row: usize,
    variant: usize,
    tint: [u8; 4],
}

impl TileWorld {
    fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols,
            rows,
            backgrounds: vec![BackgroundTile::Grass; cols * rows],
            foregrounds: vec![None; cols * rows],
            buildings: Vec::new(),
            resources: Vec::new(),
            fog: vec![false; cols * rows],
        }
    }

    fn seeded(cols: usize, rows: usize, seed: u64) -> Self {
        let mut world = Self::new(cols, rows);
        let mut rng = SeededRng::new(seed);
        let blob_count = 5 + rng.range_usize(0, 5);

        for _ in 0..blob_count {
            let center_col = rng.range_f32(4.0, cols as f32 - 4.0);
            let center_row = rng.range_f32(4.0, rows as f32 - 4.0);
            let radius_x = rng.range_f32(3.5, 10.0);
            let radius_y = rng.range_f32(2.5, 7.0);
            let wobble = rng.range_f32(-0.16, 0.08);

            for row in 0..rows {
                for col in 0..cols {
                    let dx = (col as f32 + 0.5 - center_col) / radius_x;
                    let dy = (row as f32 + 0.5 - center_row) / radius_y;
                    let noise = seeded_cell_noise(seed, col, row) * 0.28;
                    if dx * dx + dy * dy + noise + wobble < 1.0 {
                        world.paint(col, row, Brush::Background(BackgroundTile::Water));
                    }
                }
            }
        }

        world.collapse_shorelines();
        world.scatter_ramps(&mut rng, GENERATED_RAMP_COUNT);
        world.scatter_resources(&mut rng, GENERATED_RESOURCE_COUNT);
        world.scatter_bushes(&mut rng, GENERATED_BUSH_COUNT);
        world.scatter_tree_patches(&mut rng, GENERATED_TREE_PATCH_COUNT);
        world
    }

    fn background(&self, col: usize, row: usize) -> BackgroundTile {
        self.backgrounds[self.index(col, row)]
    }

    fn foreground(&self, col: usize, row: usize) -> Option<AtlasTile> {
        self.foregrounds[self.index(col, row)]
    }

    fn render_background(&self, col: usize, row: usize) -> BackgroundTile {
        if self.foreground(col, row).is_some_and(is_shoreline_tile) {
            BackgroundTile::Water
        } else {
            self.background(col, row)
        }
    }

    fn fog(&self, col: usize, row: usize) -> bool {
        self.fog[self.index(col, row)]
    }

    fn paint(&mut self, col: usize, row: usize, brush: Brush) {
        let index = self.index(col, row);
        match brush {
            Brush::Background(background) => self.backgrounds[index] = background,
            Brush::Foreground(tile) => {
                if is_shoreline_tile(tile) {
                    self.backgrounds[index] = BackgroundTile::Water;
                }
                self.foregrounds[index] = Some(tile);
            }
            Brush::Ramp(ramp) => {
                if row + 1 < self.rows {
                    self.foregrounds[index] = Some(ramp.top);
                    let bottom_index = self.index(col, row + 1);
                    self.foregrounds[bottom_index] = Some(ramp.bottom);
                }
            }
            Brush::Building(kind) => {
                let spec = building_spec(kind);
                if self.can_place_building(spec, col, row) {
                    self.buildings.push(PlacedBuilding { kind, col, row });
                }
            }
            Brush::FogRect => {}
            Brush::ClearForeground => self.clear_foreground(col, row),
        }
    }

    fn clear_foreground(&mut self, col: usize, row: usize) {
        let index = self.index(col, row);
        self.foregrounds[index] = None;
        self.buildings.retain(|building| {
            let spec = building_spec(building.kind);
            col < building.col
                || col >= building.col + spec.footprint_cols
                || row < building.row
                || row >= building.row + spec.footprint_rows
        });
    }

    fn add_fog_rect(&mut self, start: (usize, usize), end: (usize, usize)) {
        let min_col = start.0.min(end.0);
        let max_col = start.0.max(end.0);
        let min_row = start.1.min(end.1);
        let max_row = start.1.max(end.1);

        for row in min_row..=max_row {
            for col in min_col..=max_col {
                let index = self.index(col, row);
                self.fog[index] = true;
            }
        }
    }

    fn scatter_ramps(&mut self, rng: &mut SeededRng, count: usize) {
        let mut placed = 0;
        let max_attempts = count * 80;

        for _ in 0..max_attempts {
            if placed >= count || self.cols == 0 || self.rows < 2 {
                break;
            }

            let col = rng.range_usize(0, self.cols);
            let row = rng.range_usize(0, self.rows - 1);
            if !self.can_place_generated_ramp(col, row) {
                continue;
            }

            let ramp = if rng.next_u64() & 1 == 0 {
                RAMP_A
            } else {
                RAMP_B
            };
            self.paint(col, row, Brush::Ramp(ramp));
            placed += 1;
        }
    }

    fn can_place_generated_ramp(&self, col: usize, row: usize) -> bool {
        row + 1 < self.rows
            && self.background(col, row) == BackgroundTile::Grass
            && self.background(col, row + 1) == BackgroundTile::Grass
            && self.foreground(col, row).is_none()
            && self.foreground(col, row + 1).is_none()
    }

    fn can_place_building(&self, spec: BuildingSpec, col: usize, row: usize) -> bool {
        if col + spec.footprint_cols > self.cols || row + spec.footprint_rows > self.rows {
            return false;
        }

        if self.buildings.iter().any(|building| {
            let other = building_spec(building.kind);
            rects_overlap(
                (col, row, spec.footprint_cols, spec.footprint_rows),
                (
                    building.col,
                    building.row,
                    other.footprint_cols,
                    other.footprint_rows,
                ),
            )
        }) {
            return false;
        }

        if self.resources.iter().any(|resource| {
            let (resource_cols, resource_rows) = resource_footprint(resource.kind);
            rects_overlap(
                (col, row, spec.footprint_cols, spec.footprint_rows),
                (resource.col, resource.row, resource_cols, resource_rows),
            )
        }) {
            return false;
        }

        for foundation_row in row..row + spec.footprint_rows {
            for foundation_col in col..col + spec.footprint_cols {
                if self.background(foundation_col, foundation_row) != BackgroundTile::Grass
                    || self.foreground(foundation_col, foundation_row).is_some()
                {
                    return false;
                }
            }
        }

        true
    }

    fn scatter_resources(&mut self, rng: &mut SeededRng, count: usize) {
        let max_attempts = count * 70;

        for _ in 0..max_attempts {
            if self.resources.len() >= count || self.cols == 0 || self.rows == 0 {
                break;
            }

            let spec = RESOURCE_SPECS[rng.range_usize(0, GROUND_RESOURCE_COUNT)];
            let col = rng.range_usize(0, self.cols);
            let row = rng.range_usize(0, self.rows);
            if !self.can_place_resource(spec.kind, col, row) {
                continue;
            }

            self.resources.push(PlacedResource {
                kind: spec.kind,
                col,
                row,
                variant: 0,
                tint: [255, 255, 255, 255],
            });
        }
    }

    fn scatter_bushes(&mut self, rng: &mut SeededRng, count: usize) {
        let mut placed = 0;
        let max_attempts = count * 90;

        for _ in 0..max_attempts {
            if placed >= count || self.cols == 0 || self.rows == 0 {
                break;
            }

            let col = rng.range_usize(0, self.cols);
            let row = rng.range_usize(0, self.rows);
            if !self.can_place_vegetation(ResourceKind::Bush, col, row) {
                continue;
            }

            self.resources.push(PlacedResource {
                kind: ResourceKind::Bush,
                col,
                row,
                variant: rng.range_usize(0, VEGETATION_VARIANT_COUNT),
                tint: random_leaf_tint(rng),
            });
            placed += 1;
        }
    }

    fn scatter_tree_patches(&mut self, rng: &mut SeededRng, patch_count: usize) {
        for _ in 0..patch_count {
            if self.cols == 0 || self.rows == 0 {
                break;
            }

            let center_col = rng.range_usize(0, self.cols);
            let center_row = rng.range_usize(0, self.rows);
            let radius = rng.range_f32(1.4, 2.8);
            let min_col = center_col.saturating_sub(radius.ceil() as usize);
            let max_col = (center_col + radius.ceil() as usize).min(self.cols - 1);
            let min_row = center_row.saturating_sub(radius.ceil() as usize);
            let max_row = (center_row + radius.ceil() as usize).min(self.rows - 1);

            for row in min_row..=max_row {
                for col in min_col..=max_col {
                    let dx = col as f32 + 0.5 - center_col as f32;
                    let dy = row as f32 + 0.5 - center_row as f32;
                    let distance = (dx * dx + dy * dy).sqrt();
                    let edge_chance = 1.0 - (distance / radius).clamp(0.0, 1.0);
                    if rng.next_f32() > 0.34 + edge_chance * 0.58 {
                        continue;
                    }
                    if !self.can_place_vegetation(ResourceKind::Tree, col, row) {
                        continue;
                    }

                    self.resources.push(PlacedResource {
                        kind: ResourceKind::Tree,
                        col,
                        row,
                        variant: rng.range_usize(0, VEGETATION_VARIANT_COUNT),
                        tint: random_leaf_tint(rng),
                    });
                }
            }
        }
    }

    fn can_place_resource(&self, kind: ResourceKind, col: usize, row: usize) -> bool {
        self.can_place_vegetation(kind, col, row)
            && !self
                .resources
                .iter()
                .any(|resource| resource.col.abs_diff(col) <= 1 && resource.row.abs_diff(row) <= 1)
    }

    fn can_place_vegetation(&self, kind: ResourceKind, col: usize, row: usize) -> bool {
        let (cols, rows) = resource_footprint(kind);
        if col + cols > self.cols || row + rows > self.rows {
            return false;
        }

        for check_row in row..row + rows {
            for check_col in col..col + cols {
                if self.background(check_col, check_row) != BackgroundTile::Grass
                    || self.foreground(check_col, check_row).is_some()
                {
                    return false;
                }
            }
        }

        !self.resources.iter().any(|resource| {
            let (resource_cols, resource_rows) = resource_footprint(resource.kind);
            rects_overlap(
                (col, row, cols, rows),
                (resource.col, resource.row, resource_cols, resource_rows),
            )
        }) && !self.buildings.iter().any(|building| {
            let spec = building_spec(building.kind);
            rects_overlap(
                (col, row, cols, rows),
                (
                    building.col,
                    building.row,
                    spec.footprint_cols,
                    spec.footprint_rows,
                ),
            )
        })
    }

    fn collapse_shorelines(&mut self) {
        for foreground in &mut self.foregrounds {
            *foreground = None;
        }

        for row in 0..self.rows {
            for col in 0..self.cols {
                let index = self.index(col, row);
                self.foregrounds[index] = match self.backgrounds[index] {
                    BackgroundTile::Grass => self.shoreline_tile(col, row),
                    BackgroundTile::Water
                        if self.is_surrounded_by(col, row, BackgroundTile::Grass) =>
                    {
                        Some(SHORE_SINGLE_IN_GRASS)
                    }
                    BackgroundTile::Water => None,
                };
            }
        }
    }

    fn shoreline_tile(&self, col: usize, row: usize) -> Option<AtlasTile> {
        let water_n = self.is_water_or_edge(col, row, 0, -1);
        let water_e = self.is_water_or_edge(col, row, 1, 0);
        let water_s = self.is_water_or_edge(col, row, 0, 1);
        let water_w = self.is_water_or_edge(col, row, -1, 0);

        if !(water_n || water_e || water_s || water_w) {
            return None;
        }

        let tile = match (water_n, water_e, water_s, water_w) {
            (true, true, false, true) => SHORE_NARROW_TOP,
            (false, true, false, true) => SHORE_NARROW_MIDDLE,
            (false, true, true, true) => SHORE_NARROW_BOTTOM,
            (true, false, true, true) => SHORE_NARROW_LEFT,
            (true, false, true, false) => SHORE_NARROW_CENTER,
            (true, true, true, false) => SHORE_NARROW_RIGHT,
            (true, true, true, true) => SHORE_SINGLE_IN_WATER,
            (true, false, false, true) => SHORE_TOP_LEFT,
            (true, true, false, false) => SHORE_TOP_RIGHT,
            (false, false, true, true) => SHORE_BOTTOM_LEFT,
            (false, true, true, false) => SHORE_BOTTOM_RIGHT,
            (true, _, _, _) => SHORE_TOP,
            (_, _, true, _) => SHORE_BOTTOM,
            (_, true, _, _) => SHORE_RIGHT,
            (_, _, _, true) => SHORE_LEFT,
            _ => return None,
        };
        Some(tile)
    }

    fn is_water_or_edge(&self, col: usize, row: usize, dc: isize, dr: isize) -> bool {
        let next_col = col as isize + dc;
        let next_row = row as isize + dr;
        if next_col < 0
            || next_row < 0
            || next_col >= self.cols as isize
            || next_row >= self.rows as isize
        {
            return true;
        }

        self.background(next_col as usize, next_row as usize) == BackgroundTile::Water
    }

    fn is_surrounded_by(&self, col: usize, row: usize, background: BackgroundTile) -> bool {
        for dr in -1..=1 {
            for dc in -1..=1 {
                if dc == 0 && dr == 0 {
                    continue;
                }

                let next_col = col as isize + dc;
                let next_row = row as isize + dr;
                if next_col < 0
                    || next_row < 0
                    || next_col >= self.cols as isize
                    || next_row >= self.rows as isize
                {
                    return false;
                }

                if self.background(next_col as usize, next_row as usize) != background {
                    return false;
                }
            }
        }

        true
    }

    fn width_px(&self) -> f32 {
        self.cols as f32 * TILE_SIZE
    }

    fn height_px(&self) -> f32 {
        self.rows as f32 * TILE_SIZE
    }

    fn index(&self, col: usize, row: usize) -> usize {
        debug_assert!(col < self.cols);
        debug_assert!(row < self.rows);
        row * self.cols + col
    }
}

struct TextureAtlas {
    texture_id: u32,
    width: u32,
    height: u32,
    tile_px: u32,
    cols: u32,
    rows: u32,
    rgba: Vec<u8>,
}

struct ImageAsset {
    texture_id: u32,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

impl ImageAsset {
    fn from_rgba(texture_id: u32, width: u32, height: u32, rgba: Vec<u8>) -> Self {
        assert_eq!(
            rgba.len(),
            (width * height * 4) as usize,
            "rgba asset dimensions should match pixel data"
        );
        Self {
            texture_id,
            width,
            height,
            rgba,
        }
    }

    fn from_png_bytes(texture_id: u32, bytes: &[u8]) -> Self {
        let image = image::load_from_memory(bytes)
            .expect("image asset png should decode")
            .to_rgba8();
        let (width, height) = image.dimensions();

        Self {
            texture_id,
            width,
            height,
            rgba: image.into_raw(),
        }
    }

    fn from_png_bytes_cropped(texture_id: u32, bytes: &[u8]) -> Self {
        let image = image::load_from_memory(bytes)
            .expect("image asset png should decode")
            .to_rgba8();
        let (width, height) = image.dimensions();
        let Some((min_x, min_y, max_x, max_y)) = alpha_bounds(&image) else {
            return Self {
                texture_id,
                width,
                height,
                rgba: image.into_raw(),
            };
        };

        let crop_width = max_x - min_x + 1;
        let crop_height = max_y - min_y + 1;
        let mut rgba = Vec::with_capacity((crop_width * crop_height * 4) as usize);
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                rgba.extend_from_slice(&image.get_pixel(x, y).0);
            }
        }

        Self {
            texture_id,
            width: crop_width,
            height: crop_height,
            rgba,
        }
    }
}

fn variant_grid_origins(
    frame: &ase_assets::RgbaAsset,
    grid_px: u32,
    cell_cols: u32,
    cell_rows: u32,
    variant_count: usize,
) -> Vec<(u32, u32)> {
    let grid_cols = frame.width / grid_px;
    let grid_rows = frame.height / grid_px;
    let origin_cols = (grid_cols.saturating_sub(cell_cols) + 1).max(1);
    let row_step = cell_rows.max(1);
    let mut origins = Vec::with_capacity(variant_count);

    for variant in 0..variant_count {
        let cell_col = (variant as u32 % origin_cols).min(grid_cols.saturating_sub(cell_cols));
        let mut cell_row = (variant as u32 / origin_cols) * row_step;
        if cell_row + cell_rows > grid_rows {
            cell_row = grid_rows.saturating_sub(cell_rows);
        }
        origins.push((cell_col, cell_row));
    }

    origins
}

fn extract_grid_region(
    frame: &ase_assets::RgbaAsset,
    grid_px: u32,
    cell_cols: u32,
    cell_rows: u32,
    cell_col: u32,
    cell_row: u32,
) -> Option<Vec<u8>> {
    let region_width = grid_px * cell_cols;
    let region_height = grid_px * cell_rows;
    let x0 = cell_col * grid_px;
    let y0 = cell_row * grid_px;
    if x0 + region_width > frame.width || y0 + region_height > frame.height {
        return None;
    }

    let frame_width = frame.width as usize;
    let x0 = x0 as usize;
    let y0 = y0 as usize;
    let region_width = region_width as usize;
    let region_height = region_height as usize;
    let mut rgba = Vec::with_capacity(region_width * region_height * 4);
    let mut has_pixels = false;

    for y in y0..y0 + region_height {
        for x in x0..x0 + region_width {
            let offset = (y * frame_width + x) * 4;
            let pixel = &frame.rgba[offset..offset + 4];
            has_pixels |= pixel[3] != 0;
            rgba.extend_from_slice(pixel);
        }
    }

    has_pixels.then_some(rgba)
}

impl TextureAtlas {
    fn from_png_bytes(texture_id: u32, bytes: &[u8], tile_px: u32) -> Self {
        let image = image::load_from_memory(bytes)
            .expect("terrain tileset png should decode")
            .to_rgba8();
        let (width, height) = image.dimensions();
        assert_eq!(width % tile_px, 0, "tileset width must align to tiles");
        assert_eq!(height % tile_px, 0, "tileset height must align to tiles");

        Self {
            texture_id,
            width,
            height,
            tile_px,
            cols: width / tile_px,
            rows: height / tile_px,
            rgba: image.into_raw(),
        }
    }

    fn non_empty_tiles(&self) -> Vec<AtlasTile> {
        let mut tiles = Vec::new();
        let mut seen_pixels = HashSet::new();
        let tile_px = self.tile_px as usize;
        let width = self.width as usize;

        for row in 0..self.rows {
            for col in 0..self.cols {
                let mut pixels = Vec::with_capacity(tile_px * tile_px * 4);
                let x0 = col as usize * tile_px;
                let y0 = row as usize * tile_px;

                for y in y0..y0 + tile_px {
                    for x in x0..x0 + tile_px {
                        let offset = (y * width + x) * 4;
                        pixels.extend_from_slice(&self.rgba[offset..offset + 4]);
                    }
                }

                let has_pixels = pixels.chunks_exact(4).any(|pixel| pixel[3] != 0);
                if has_pixels && seen_pixels.insert(pixels) {
                    tiles.push(AtlasTile { col, row });
                }
            }
        }

        tiles
    }

    fn uv(&self, tile: AtlasTile) -> [f32; 4] {
        let left = (tile.col * self.tile_px) as f32 / self.width as f32;
        let top = (tile.row * self.tile_px) as f32 / self.height as f32;
        let right = ((tile.col + 1) * self.tile_px) as f32 / self.width as f32;
        let bottom = ((tile.row + 1) * self.tile_px) as f32 / self.height as f32;
        [left, top, right, bottom]
    }
}

struct SpriteBatch {
    window_width: u32,
    window_height: u32,
    bytes: Vec<u8>,
}

impl SpriteBatch {
    fn new(window_width: u32, window_height: u32) -> Self {
        Self {
            window_width,
            window_height,
            bytes: Vec::new(),
        }
    }

    fn sprite(
        &mut self,
        atlas: &TextureAtlas,
        tile: AtlasTile,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: Rgba8,
    ) {
        let [u0, v0, u1, v1] = atlas.uv(tile);
        let (x0, y0) = self.to_clip(x, y);
        let (x1, y1) = self.to_clip(x + w, y + h);

        self.vertex(TexVertex {
            x: x0,
            y: y0,
            u: u0,
            v: v0,
            color,
        });
        self.vertex(TexVertex {
            x: x1,
            y: y0,
            u: u1,
            v: v0,
            color,
        });
        self.vertex(TexVertex {
            x: x1,
            y: y1,
            u: u1,
            v: v1,
            color,
        });
        self.vertex(TexVertex {
            x: x0,
            y: y0,
            u: u0,
            v: v0,
            color,
        });
        self.vertex(TexVertex {
            x: x1,
            y: y1,
            u: u1,
            v: v1,
            color,
        });
        self.vertex(TexVertex {
            x: x0,
            y: y1,
            u: u0,
            v: v1,
            color,
        });
    }

    fn image(&mut self, x: f32, y: f32, w: f32, h: f32, color: Rgba8) {
        self.image_uv(x, y, w, h, [0.0, 0.0, 1.0, 1.0], color);
    }

    fn image_region(
        &mut self,
        image: &ImageAsset,
        region: ImageRegion,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: Rgba8,
    ) {
        self.image_uv(x, y, w, h, region.uv(image), color);
    }

    fn image_uv(&mut self, x: f32, y: f32, w: f32, h: f32, uv: [f32; 4], color: Rgba8) {
        let [u0, v0, u1, v1] = uv;
        let (x0, y0) = self.to_clip(x, y);
        let (x1, y1) = self.to_clip(x + w, y + h);

        self.vertex(TexVertex {
            x: x0,
            y: y0,
            u: u0,
            v: v0,
            color,
        });
        self.vertex(TexVertex {
            x: x1,
            y: y0,
            u: u1,
            v: v0,
            color,
        });
        self.vertex(TexVertex {
            x: x1,
            y: y1,
            u: u1,
            v: v1,
            color,
        });
        self.vertex(TexVertex {
            x: x0,
            y: y0,
            u: u0,
            v: v0,
            color,
        });
        self.vertex(TexVertex {
            x: x1,
            y: y1,
            u: u1,
            v: v1,
            color,
        });
        self.vertex(TexVertex {
            x: x0,
            y: y1,
            u: u0,
            v: v1,
            color,
        });
    }

    fn vertex(&mut self, vertex: TexVertex) {
        push_f32(&mut self.bytes, vertex.x);
        push_f32(&mut self.bytes, vertex.y);
        push_f32(&mut self.bytes, vertex.u);
        push_f32(&mut self.bytes, vertex.v);
        self.bytes.extend_from_slice(&[
            vertex.color.r,
            vertex.color.g,
            vertex.color.b,
            vertex.color.a,
        ]);
    }

    fn to_clip(&self, x: f32, y: f32) -> (f32, f32) {
        (
            (x / self.window_width as f32) * 2.0 - 1.0,
            1.0 - (y / self.window_height as f32) * 2.0,
        )
    }
}

struct SolidBatch {
    window_width: u32,
    window_height: u32,
    bytes: Vec<u8>,
}

impl SolidBatch {
    fn new(window_width: u32, window_height: u32) -> Self {
        Self {
            window_width,
            window_height,
            bytes: Vec::new(),
        }
    }

    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Rgba8) {
        let (x0, y0) = self.to_clip(x, y);
        let (x1, y1) = self.to_clip(x + w, y + h);
        self.vertex(x0, y0, color);
        self.vertex(x1, y0, color);
        self.vertex(x1, y1, color);
        self.vertex(x0, y0, color);
        self.vertex(x1, y1, color);
        self.vertex(x0, y1, color);
    }

    fn vertex(&mut self, x: f32, y: f32, color: Rgba8) {
        push_f32(&mut self.bytes, x);
        push_f32(&mut self.bytes, y);
        self.bytes
            .extend_from_slice(&[color.r, color.g, color.b, color.a]);
    }

    fn to_clip(&self, x: f32, y: f32) -> (f32, f32) {
        (
            (x / self.window_width as f32) * 2.0 - 1.0,
            1.0 - (y / self.window_height as f32) * 2.0,
        )
    }
}

fn outline_rect(
    out: &mut SolidBatch,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    thickness: f32,
    color: Rgba8,
) {
    out.rect(x, y, w, thickness, color);
    out.rect(x, y + h - thickness, w, thickness, color);
    out.rect(x, y, thickness, h, color);
    out.rect(x + w - thickness, y, thickness, h, color);
}

fn inside_rect(x: f32, y: f32, rect_x: f32, rect_y: f32, rect_w: f32, rect_h: f32) -> bool {
    x >= rect_x && x < rect_x + rect_w && y >= rect_y && y < rect_y + rect_h
}

fn scroll_strength(distance_from_edge: f32) -> f32 {
    ((EDGE_SCROLL_ZONE - distance_from_edge.max(0.0)) / EDGE_SCROLL_ZONE).clamp(0.0, 1.0)
}

fn is_ramp_part(tile: AtlasTile) -> bool {
    tile == RAMP_A.top || tile == RAMP_A.bottom || tile == RAMP_B.top || tile == RAMP_B.bottom
}

fn building_spec(kind: BuildingKind) -> BuildingSpec {
    BUILDING_SPECS[kind.index()]
}

fn resource_footprint(kind: ResourceKind) -> (usize, usize) {
    match kind {
        ResourceKind::Tree => (2, 2),
        ResourceKind::Rock1
        | ResourceKind::Rock2
        | ResourceKind::Rock3
        | ResourceKind::Rock4
        | ResourceKind::Meat
        | ResourceKind::Wood
        | ResourceKind::Bush => (1, 1),
    }
}

fn rects_overlap(a: (usize, usize, usize, usize), b: (usize, usize, usize, usize)) -> bool {
    let (ax, ay, aw, ah) = a;
    let (bx, by, bw, bh) = b;
    ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by
}

fn is_shoreline_tile(tile: AtlasTile) -> bool {
    matches!(
        tile,
        SHORE_TOP_LEFT
            | SHORE_TOP
            | SHORE_TOP_RIGHT
            | SHORE_LEFT
            | SHORE_RIGHT
            | SHORE_BOTTOM_LEFT
            | SHORE_BOTTOM
            | SHORE_BOTTOM_RIGHT
            | SHORE_NARROW_TOP
            | SHORE_NARROW_MIDDLE
            | SHORE_NARROW_BOTTOM
            | SHORE_NARROW_LEFT
            | SHORE_NARROW_CENTER
            | SHORE_NARROW_RIGHT
            | SHORE_SINGLE_IN_WATER
            | SHORE_SINGLE_IN_GRASS
    )
}

fn alpha_bounds(image: &image::RgbaImage) -> Option<(u32, u32, u32, u32)> {
    let (width, height) = image.dimensions();
    let mut min_x = width;
    let mut min_y = height;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut found = false;

    for y in 0..height {
        for x in 0..width {
            if image.get_pixel(x, y).0[3] != 0 {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
                found = true;
            }
        }
    }

    found.then_some((min_x, min_y, max_x, max_y))
}

fn seed_from_args(args: impl IntoIterator<Item = String>) -> u64 {
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        if let Some(seed) = arg.strip_prefix("--seed=") {
            return parse_seed(seed).unwrap_or(DEFAULT_SEED);
        }

        if arg == "--seed" {
            return args
                .next()
                .and_then(|seed| parse_seed(&seed))
                .unwrap_or(DEFAULT_SEED);
        }
    }

    DEFAULT_SEED
}

fn parse_seed(seed: &str) -> Option<u64> {
    if let Some(hex) = seed.strip_prefix("0x").or_else(|| seed.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).ok()
    } else {
        seed.parse().ok()
    }
}

fn seeded_cell_noise(seed: u64, col: usize, row: usize) -> f32 {
    let mut rng = SeededRng::new(
        seed ^ ((col as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
            ^ ((row as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9)),
    );
    rng.range_f32(-1.0, 1.0)
}

fn random_leaf_tint(rng: &mut SeededRng) -> [u8; 4] {
    [
        rng.range_usize(218, 256) as u8,
        rng.range_usize(228, 256) as u8,
        rng.range_usize(206, 246) as u8,
        255,
    ]
}

struct SeededRng {
    state: u64,
}

impl SeededRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        value ^ (value >> 31)
    }

    fn next_f32(&mut self) -> f32 {
        ((self.next_u64() >> 40) as f32) / ((1u32 << 24) as f32)
    }

    fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.next_f32()
    }

    fn range_usize(&mut self, min: usize, max: usize) -> usize {
        min + (self.next_u64() as usize % (max - min).max(1))
    }
}

fn push_f32(out: &mut Vec<u8>, value: f32) {
    out.extend_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_every_non_empty_tile_from_color2_tileset() {
        let atlas = TextureAtlas::from_png_bytes(TERRAIN_TEXTURE, TERRAIN_BYTES, TERRAIN_TILE_PX);
        let tiles = atlas.non_empty_tiles();

        assert_eq!(atlas.cols, 9);
        assert_eq!(atlas.rows, 6);
        assert_eq!(tiles.len(), 43);
        assert!(tiles.contains(&AtlasTile { col: 3, row: 0 }));
        assert!(tiles.contains(&AtlasTile { col: 3, row: 5 }));
        assert!(tiles.contains(&AtlasTile { col: 1, row: 1 }));
        assert!(!tiles.contains(&AtlasTile { col: 6, row: 1 }));
        assert!(!tiles.contains(&AtlasTile { col: 4, row: 0 }));
        assert!(!tiles.contains(&AtlasTile { col: 4, row: 5 }));
    }

    #[test]
    fn background_brushes_do_not_clear_foreground() {
        let mut world = TileWorld::new(2, 2);
        let foreground = AtlasTile { col: 0, row: 0 };

        world.paint(0, 0, Brush::Foreground(foreground));
        world.paint(1, 0, Brush::Foreground(foreground));
        world.paint(1, 0, Brush::Background(BackgroundTile::Grass));
        assert_eq!(world.background(1, 0), BackgroundTile::Grass);
        assert_eq!(world.foreground(1, 0), Some(foreground));

        world.paint(0, 0, Brush::Background(BackgroundTile::Water));
        assert_eq!(world.background(0, 0), BackgroundTile::Water);
        assert_eq!(world.foreground(0, 0), Some(foreground));

        world.paint(0, 0, Brush::ClearForeground);
        assert_eq!(world.background(0, 0), BackgroundTile::Water);
        assert_eq!(world.foreground(0, 0), None);
    }

    #[test]
    fn clear_tool_removes_buildings_without_touching_background() {
        let mut world = TileWorld::new(4, 4);
        world.paint(0, 0, Brush::Background(BackgroundTile::Water));
        world.paint(1, 0, Brush::Building(BuildingKind::House1));
        assert_eq!(world.buildings.len(), 1);

        world.paint(2, 1, Brush::ClearForeground);

        assert_eq!(world.background(0, 0), BackgroundTile::Water);
        assert!(world.buildings.is_empty());
    }

    #[test]
    fn shoreline_foreground_tiles_render_on_water_background() {
        let mut world = TileWorld::new(2, 2);

        world.paint(0, 0, Brush::Foreground(SHORE_TOP_LEFT));
        assert_eq!(world.background(0, 0), BackgroundTile::Water);
        assert_eq!(world.render_background(0, 0), BackgroundTile::Water);

        world.paint(0, 0, Brush::Background(BackgroundTile::Grass));
        assert_eq!(world.background(0, 0), BackgroundTile::Grass);
        assert_eq!(world.render_background(0, 0), BackgroundTile::Water);

        world.paint(1, 1, Brush::Foreground(RAMP_A.top));
        assert_eq!(world.background(1, 1), BackgroundTile::Grass);
        assert_eq!(world.render_background(1, 1), BackgroundTile::Grass);
    }

    #[test]
    fn full_grass_tile_is_a_background_tool_not_a_foreground_palette_tile() {
        let game = Game::new(DEFAULT_SEED);

        assert_eq!(
            game.palette_brush(1),
            Brush::Background(BackgroundTile::Grass)
        );
        assert!(!game.foreground_tiles.contains(&GRASS_BG_TILE));
        assert!(!game.foreground_tiles.contains(&RAMP_A.top));
        assert!(!game.foreground_tiles.contains(&RAMP_A.bottom));
        assert!(!game.foreground_tiles.contains(&RAMP_B.top));
        assert!(!game.foreground_tiles.contains(&RAMP_B.bottom));
        assert_eq!(game.palette_brush(3), Brush::Ramp(RAMP_A));
        assert_eq!(game.palette_brush(4), Brush::Ramp(RAMP_B));
        assert_eq!(
            game.palette_brush(6),
            Brush::Building(BuildingKind::Archery)
        );
    }

    #[test]
    fn red_building_assets_are_loaded_at_world_scale() {
        let game = Game::new(DEFAULT_SEED);

        assert_eq!(game.buildings.len(), BUILDING_COUNT);
        for (image, spec) in game.buildings.iter().zip(BUILDING_SPECS) {
            assert_eq!(image.texture_id, spec.texture_id);
            assert_eq!(
                image.width as usize,
                spec.footprint_cols * TERRAIN_TILE_PX as usize
            );
            assert_eq!(
                image.height as usize,
                spec.footprint_rows * TERRAIN_TILE_PX as usize
            );
        }
    }

    #[test]
    fn building_brush_places_only_on_clean_grass_foundations() {
        let mut world = TileWorld::new(7, 7);

        world.paint(0, 0, Brush::Building(BuildingKind::House1));
        assert_eq!(
            world.buildings,
            vec![PlacedBuilding {
                kind: BuildingKind::House1,
                col: 0,
                row: 0
            }]
        );

        world.paint(2, 0, Brush::Background(BackgroundTile::Water));
        world.paint(2, 0, Brush::Building(BuildingKind::House1));
        assert_eq!(world.buildings.len(), 1);

        world.resources.push(PlacedResource {
            kind: ResourceKind::Wood,
            col: 4,
            row: 3,
            variant: 0,
            tint: [255, 255, 255, 255],
        });
        world.paint(4, 3, Brush::Building(BuildingKind::House1));
        assert_eq!(world.buildings.len(), 1);
    }

    #[test]
    fn resource_assets_keep_expected_world_size() {
        let game = Game::new(DEFAULT_SEED);

        assert_eq!(game.resources.len(), RESOURCE_COUNT);
        for (resource, spec) in game.resources.iter().zip(RESOURCE_SPECS) {
            let (expected_width, expected_height) = match spec.source {
                ResourceSource::Png(_) => (TERRAIN_TILE_PX, TERRAIN_TILE_PX),
                ResourceSource::AsepriteGrid {
                    grid_px,
                    cell_cols,
                    cell_rows,
                    ..
                } => (grid_px * cell_cols, grid_px * cell_rows),
            };

            assert_eq!(
                resource.variants[0].frames[0].texture_id,
                spec.texture_id_base
            );
            assert!(
                resource
                    .variants
                    .iter()
                    .all(|variant| variant.total_duration_ms > 0)
            );
            for variant in &resource.variants {
                for image in &variant.frames {
                    assert_eq!(image.width, expected_width);
                    assert_eq!(image.height, expected_height);
                }
            }
        }
        assert_eq!(
            game.resources[ResourceKind::Bush.index()].variants.len(),
            VEGETATION_VARIANT_COUNT
        );
        assert_eq!(
            game.resources[ResourceKind::Tree.index()].variants.len(),
            VEGETATION_VARIANT_COUNT
        );
        assert!(
            game.resources[ResourceKind::Bush.index()]
                .variants
                .iter()
                .all(|variant| variant.frames.len() > 1)
        );
        assert!(
            game.resources[ResourceKind::Tree.index()]
                .variants
                .iter()
                .all(|variant| variant.frames.len() > 1)
        );
    }

    #[test]
    fn ramp_brush_places_both_vertical_halves_together() {
        let mut world = TileWorld::new(3, 3);

        world.paint(1, 1, Brush::Ramp(RAMP_A));
        assert_eq!(world.foreground(1, 1), Some(RAMP_A.top));
        assert_eq!(world.foreground(1, 2), Some(RAMP_A.bottom));
    }

    #[test]
    fn ramp_brush_does_not_place_partial_ramp_on_bottom_row() {
        let mut world = TileWorld::new(3, 3);

        world.paint(1, 2, Brush::Ramp(RAMP_A));
        assert_eq!(world.foreground(1, 2), None);
    }

    #[test]
    fn foreground_and_ramps_can_place_on_water_background() {
        let mut world = TileWorld::new(3, 3);
        let foreground = AtlasTile { col: 0, row: 0 };

        world.paint(1, 1, Brush::Background(BackgroundTile::Water));
        world.paint(1, 1, Brush::Foreground(foreground));
        assert_eq!(world.foreground(1, 1), Some(foreground));

        world.paint(0, 0, Brush::Background(BackgroundTile::Water));
        world.paint(0, 0, Brush::Ramp(RAMP_A));
        assert_eq!(world.foreground(0, 0), Some(RAMP_A.top));
        assert_eq!(world.foreground(0, 1), Some(RAMP_A.bottom));

        world.paint(2, 1, Brush::Background(BackgroundTile::Water));
        world.paint(2, 0, Brush::Ramp(RAMP_A));
        assert_eq!(world.foreground(2, 0), Some(RAMP_A.top));
        assert_eq!(world.foreground(2, 1), Some(RAMP_A.bottom));
    }

    #[test]
    fn water_background_does_not_auto_place_neighbor_foreground_tiles() {
        let mut world = TileWorld::new(3, 3);

        world.paint(1, 1, Brush::Background(BackgroundTile::Water));
        assert_eq!(world.foreground(1, 0), None);
        assert_eq!(world.foreground(2, 1), None);
        assert_eq!(world.foreground(1, 2), None);
        assert_eq!(world.foreground(0, 1), None);
    }

    #[test]
    fn fog_rect_marks_every_cell_inclusive() {
        let mut world = TileWorld::new(5, 5);

        world.add_fog_rect((3, 1), (1, 3));

        assert!(world.fog(1, 1));
        assert!(world.fog(2, 2));
        assert!(world.fog(3, 3));
        assert!(!world.fog(0, 2));
        assert!(!world.fog(4, 2));
    }

    #[test]
    fn fog_tool_is_a_palette_brush() {
        let game = Game::new(DEFAULT_SEED);

        assert_eq!(game.palette_brush(5), Brush::FogRect);
    }

    #[test]
    fn fog_tool_keeps_the_regular_cursor() {
        let mut game = Game::new(DEFAULT_SEED);
        game.selected = Some(Brush::FogRect);
        game.mouse = Point {
            x: VIEW_X + TILE_SIZE,
            y: VIEW_Y + TILE_SIZE,
        };

        assert_eq!(game.cursor_image().texture_id, CURSOR_DEFAULT_TEXTURE);
    }

    #[test]
    fn small_cursor_pngs_are_cropped_but_selection_cursor_stays_full() {
        for (texture_id, bytes) in [
            (CURSOR_DEFAULT_TEXTURE, CURSOR_DEFAULT_BYTES),
            (CURSOR_HOVER_TEXTURE, CURSOR_HOVER_BYTES),
            (CURSOR_DELETE_TEXTURE, CURSOR_DELETE_BYTES),
        ] {
            let full = ImageAsset::from_png_bytes(texture_id, bytes);
            let cropped = ImageAsset::from_png_bytes_cropped(texture_id, bytes);

            assert!(cropped.width < full.width);
            assert!(cropped.height < full.height);
        }

        let game = Game::new(DEFAULT_SEED);
        let cursor_select = ImageAsset::from_png_bytes(CURSOR_SELECT_TEXTURE, CURSOR_SELECT_BYTES);
        assert_eq!(game.cursor_select.width, cursor_select.width);
        assert_eq!(game.cursor_select.height, cursor_select.height);
    }

    #[test]
    fn fog_shadow_png_is_cropped_before_stretching_to_tiles() {
        let full = ImageAsset::from_png_bytes(FOG_TEXTURE, FOG_BYTES);
        let game = Game::new(DEFAULT_SEED);

        assert!(game.fog.width < full.width);
        assert!(game.fog.height < full.height);
    }

    #[test]
    fn shoreline_collapse_places_edges_around_water_background() {
        let mut world = TileWorld::new(5, 5);

        world.paint(2, 2, Brush::Background(BackgroundTile::Water));
        world.collapse_shorelines();

        assert_eq!(world.foreground(2, 1), Some(SHORE_BOTTOM));
        assert_eq!(world.foreground(3, 2), Some(SHORE_LEFT));
        assert_eq!(world.foreground(2, 3), Some(SHORE_TOP));
        assert_eq!(world.foreground(1, 2), Some(SHORE_RIGHT));
        assert_eq!(world.foreground(2, 2), Some(SHORE_SINGLE_IN_GRASS));
    }

    #[test]
    fn shoreline_collapse_uses_corners_and_never_full_grass_foreground() {
        let mut world = TileWorld::new(3, 3);

        world.paint(1, 0, Brush::Background(BackgroundTile::Water));
        world.paint(0, 1, Brush::Background(BackgroundTile::Water));
        world.collapse_shorelines();

        assert_eq!(world.foreground(1, 1), Some(SHORE_TOP_LEFT));
        assert!(!world.foregrounds.contains(&Some(GRASS_BG_TILE)));
    }

    #[test]
    fn shoreline_collapse_uses_narrow_vertical_grass_strip_tiles() {
        let mut world = TileWorld::new(3, 5);

        for row in 0..5 {
            world.paint(0, row, Brush::Background(BackgroundTile::Water));
            world.paint(2, row, Brush::Background(BackgroundTile::Water));
        }
        world.paint(1, 0, Brush::Background(BackgroundTile::Water));
        world.paint(1, 4, Brush::Background(BackgroundTile::Water));
        world.collapse_shorelines();

        assert_eq!(world.foreground(1, 1), Some(SHORE_NARROW_TOP));
        assert_eq!(world.foreground(1, 2), Some(SHORE_NARROW_MIDDLE));
        assert_eq!(world.foreground(1, 3), Some(SHORE_NARROW_BOTTOM));

        world.paint(1, 2, Brush::Background(BackgroundTile::Water));
        world.collapse_shorelines();
        assert_eq!(world.foreground(1, 1), Some(SHORE_SINGLE_IN_WATER));
        assert_eq!(world.foreground(1, 3), Some(SHORE_SINGLE_IN_WATER));
    }

    #[test]
    fn shoreline_collapse_uses_narrow_horizontal_grass_strip_tiles() {
        let mut world = TileWorld::new(5, 3);

        for col in 0..5 {
            world.paint(col, 0, Brush::Background(BackgroundTile::Water));
            world.paint(col, 2, Brush::Background(BackgroundTile::Water));
        }
        world.paint(0, 1, Brush::Background(BackgroundTile::Water));
        world.paint(4, 1, Brush::Background(BackgroundTile::Water));
        world.collapse_shorelines();

        assert_eq!(world.foreground(1, 1), Some(SHORE_NARROW_LEFT));
        assert_eq!(world.foreground(2, 1), Some(SHORE_NARROW_CENTER));
        assert_eq!(world.foreground(3, 1), Some(SHORE_NARROW_RIGHT));

        world.paint(2, 1, Brush::Background(BackgroundTile::Water));
        world.collapse_shorelines();
        assert_eq!(world.foreground(1, 1), Some(SHORE_SINGLE_IN_WATER));
        assert_eq!(world.foreground(3, 1), Some(SHORE_SINGLE_IN_WATER));
    }

    #[test]
    fn shoreline_collapse_uses_distinct_single_tiles_for_water_and_grass_contexts() {
        let mut grass_island = TileWorld::new(3, 3);
        for row in 0..3 {
            for col in 0..3 {
                grass_island.paint(col, row, Brush::Background(BackgroundTile::Water));
            }
        }
        grass_island.paint(1, 1, Brush::Background(BackgroundTile::Grass));
        grass_island.collapse_shorelines();

        assert_eq!(grass_island.foreground(1, 1), Some(SHORE_SINGLE_IN_WATER));

        let mut water_hole = TileWorld::new(3, 3);
        water_hole.paint(1, 1, Brush::Background(BackgroundTile::Water));
        water_hole.collapse_shorelines();

        assert_eq!(water_hole.foreground(1, 1), Some(SHORE_SINGLE_IN_GRASS));
    }

    #[test]
    fn parses_seed_from_cli_args() {
        assert_eq!(seed_from_args(Vec::<String>::new()), DEFAULT_SEED);
        assert_eq!(seed_from_args(["--seed=42".to_string()]), 42);
        assert_eq!(
            seed_from_args(["--seed".to_string(), "0x2A".to_string()]),
            42
        );
    }

    #[test]
    fn resize_expands_visible_world_without_scaling_layout_space() {
        let mut game = Game::new(DEFAULT_SEED);
        let windowed_view_h = game.view_h();
        let windowed_slots = game.palette_slots_per_row();

        game.resize_view(1920, 1080);

        assert_eq!(game.window_width, 1920);
        assert_eq!(game.window_height, 1080);
        assert_eq!(game.view_w(), 1920.0);
        assert!(game.view_h() > windowed_view_h);
        assert!(game.palette_slots_per_row() > windowed_slots);
    }

    #[test]
    fn aseprite_explorer_groups_tagged_files_into_animation_clips() {
        let explorer = AsepriteExplorer::new();
        let bushes = &explorer.files[1];

        assert_eq!(bushes.frames.len(), 32);
        assert_eq!(bushes.clips.len(), 4);
        assert_eq!(bushes.clips[0].name, "Bush 1");
        assert_eq!(bushes.clips[0].frame_indices.len(), 8);
    }

    #[test]
    fn doubled_palette_panel_contains_all_slots_at_default_size() {
        let game = Game::new(DEFAULT_SEED);
        let last_slot = game.palette_len() - 1;
        let (_, y) = game.palette_slot_rect(last_slot);

        assert_eq!(PANEL_H, 320.0);
        assert!(y + PALETTE_TILE <= game.window_h() - 10.0);
    }

    #[test]
    fn seeded_world_generation_is_deterministic() {
        let a = TileWorld::seeded(20, 20, 123);
        let b = TileWorld::seeded(20, 20, 123);
        let c = TileWorld::seeded(20, 20, 124);

        assert_eq!(a.backgrounds, b.backgrounds);
        assert_ne!(a.backgrounds, c.backgrounds);
        assert!(
            a.backgrounds
                .iter()
                .any(|&background| background == BackgroundTile::Water)
        );
        assert!(a.foregrounds.iter().any(Option::is_some));
        assert_eq!(
            a.foregrounds
                .iter()
                .filter(|&&tile| tile == Some(RAMP_A.top) || tile == Some(RAMP_B.top))
                .count(),
            GENERATED_RAMP_COUNT
        );
        assert!(a.buildings.is_empty());
        assert!(b.buildings.is_empty());
        assert_eq!(a.resources, b.resources);
        assert!(!a.resources.is_empty());
        for resource in &a.resources {
            assert_eq!(
                a.background(resource.col, resource.row),
                BackgroundTile::Grass
            );
            assert_eq!(a.foreground(resource.col, resource.row), None);
            assert!(!a.buildings.iter().any(|building| {
                let spec = building_spec(building.kind);
                resource.col >= building.col
                    && resource.col < building.col + spec.footprint_cols
                    && resource.row >= building.row
                    && resource.row < building.row + spec.footprint_rows
            }));
        }
        assert!(!a.foregrounds.contains(&Some(GRASS_BG_TILE)));
    }

    #[test]
    fn seeded_editor_world_places_vegetation_patches() {
        let world = TileWorld::seeded(WORLD_COLS, WORLD_ROWS, DEFAULT_SEED);

        assert!(
            world
                .resources
                .iter()
                .any(|resource| resource.kind == ResourceKind::Bush)
        );
        assert!(
            world
                .resources
                .iter()
                .any(|resource| resource.kind == ResourceKind::Tree)
        );
        assert!(
            world
                .resources
                .iter()
                .any(|resource| resource.kind == ResourceKind::Tree)
        );
    }
}
