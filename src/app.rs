use crate::terrain_rules::AtlasTile;
use crate::{ase_assets, ts_ui};
use adapterlibgfx::api::{Adapter, AdapterConfig};
use adapterlibgfx::command::{ScissorRect, TextureEffect};
use adapterlibgfx::vertex::{Rgba8, TexVertex};
use adapterlibgfx::window::{
    FrameProducer, InputButtonState, InputEvent, InputKey, InputMouseButton, WgpuSixWindowApp,
};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::Path;
use std::time::Instant;

#[path = "wldgenerator.rs"]
mod wldgenerator;
#[path = "worldviewer.rs"]
mod worldviewer;

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 800;
#[cfg(test)]
const EXPLORER_WIDTH: u32 = 640;
#[cfg(test)]
const EXPLORER_HEIGHT: u32 = 520;
const UNIT_VIEWER_WIDTH: u32 = 640;
const UNIT_VIEWER_HEIGHT: u32 = 520;
const ICON_VIEWER_WIDTH: u32 = 360;
const ICON_VIEWER_HEIGHT: u32 = 320;
const EVENT_EDITOR_WIDTH: u32 = 760;
const EVENT_EDITOR_HEIGHT: u32 = 520;
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
const WOOD_TABLE_TEXTURE: u32 = 15;
const CLOUD_TEXTURE_BASE: u32 = 100;
const WATER_STATE_TEXTURE_BASE: u32 = 140;
const WATER_STATE_TEXTURE_STRIDE: u32 = 64;
const WATER_FOAM_TEXTURE_BASE: u32 = WATER_STATE_TEXTURE_BASE + WATER_STATE_TEXTURE_STRIDE * 4;
const WATER_DUCK_TEXTURE_BASE: u32 = WATER_FOAM_TEXTURE_BASE + WATER_STATE_TEXTURE_STRIDE;
const PLANT_PROP_TEXTURE_BASE: u32 = 600;
const PLANT_PROP_TEXTURE_STRIDE: u32 = 64;
const GOLD_PROP_TEXTURE_BASE: u32 = 1400;
const GOLD_PROP_TEXTURE_STRIDE: u32 = 64;
const ROCK_PROP_TEXTURE_BASE: u32 = 1900;
#[cfg(test)]
const ASE_EXPLORER_TEXTURE_BASE: u32 = 1000;
const UNIT_VIEWER_TEXTURE_BASE: u32 = 5000;
const ICON_VIEWER_TEXTURE_BASE: u32 = 7000;
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
const WOOD_TABLE_BYTES: &[u8] =
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Wood Table/WoodTable.png");
const ROCK1_BYTES: &[u8] = include_bytes!("../ts_freepack/Rocks/Rock1.png");
const ROCK2_BYTES: &[u8] = include_bytes!("../ts_freepack/Rocks/Rock2.png");
const ROCK3_BYTES: &[u8] = include_bytes!("../ts_freepack/Rocks/Rock3.png");
const ROCK4_BYTES: &[u8] = include_bytes!("../ts_freepack/Rocks/Rock4.png");
const TOOL1_BYTES: &[u8] = include_bytes!("../ts_freepack/Terrain/Tools/Tool_01.png");
const TOOL2_BYTES: &[u8] = include_bytes!("../ts_freepack/Terrain/Tools/Tool_02.png");
const TOOL3_BYTES: &[u8] = include_bytes!("../ts_freepack/Terrain/Tools/Tool_03.png");
const TOOL4_BYTES: &[u8] = include_bytes!("../ts_freepack/Terrain/Tools/Tool_04.png");
const MEAT_RESOURCE_BYTES: &[u8] = include_bytes!("../ts_freepack/Terrain/Tools/Meat Resource.png");
const WOOD_RESOURCE_BYTES: &[u8] = include_bytes!("../ts_freepack/Terrain/Tools/Wood Resource.png");
const UI_ICON_BYTES: [&[u8]; 12] = [
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Icons/Icon_01.png"),
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Icons/Icon_02.png"),
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Icons/Icon_03.png"),
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Icons/Icon_04.png"),
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Icons/Icon_05.png"),
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Icons/Icon_06.png"),
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Icons/Icon_07.png"),
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Icons/Icon_08.png"),
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Icons/Icon_09.png"),
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Icons/Icon_10.png"),
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Icons/Icon_11.png"),
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Icons/Icon_12.png"),
];
const CLOUDS_ASEPRITE_PATH: &str = "ts_freepack/Clouds.aseprite";
const TREES_ASEPRITE_PATH: &str = "ts_freepack/Trees.aseprite";
const BUSHES_ASEPRITE_PATH: &str = "ts_freepack/Bushes.aseprite";
const ARCHER_ASEPRITE_PATH: &str = "ts_freepack/Archer.aseprite";
const RUBBER_DUCK_ASEPRITE_PATH: &str = "ts_freepack/Rubber Duck.aseprite";
const WATER_FOAM_ASEPRITE_PATH: &str = "ts_freepack/Water Foam.aseprite";
const WORLD_SAVE_PATH: &str = "world.json";
const WATER_ROCK_ASEPRITE_PATHS: [&str; 4] = [
    "ts_freepack/Water Rocks_01.aseprite",
    "ts_freepack/Water Rocks_02.aseprite",
    "ts_freepack/Water Rocks_03.aseprite",
    "ts_freepack/Water Rocks_04.aseprite",
];
const WATER_BG: u32 = 0x47ABA9;
const SELECT_CORNER_SOURCES: [ImageRegion; 4] = [
    ImageRegion::new(3, 3, 21, 25),
    ImageRegion::new(104, 3, 21, 25),
    ImageRegion::new(3, 100, 21, 25),
    ImageRegion::new(104, 100, 21, 25),
];
const WOOD_TABLE_TOP_LEFT: ImageRegion = ImageRegion::new(45, 43, 83, 85);
const WOOD_TABLE_TOP_EDGE: ImageRegion = ImageRegion::new(192, 49, 64, 24);
const WOOD_TABLE_TOP_RIGHT: ImageRegion = ImageRegion::new(320, 43, 83, 85);
const WOOD_TABLE_LEFT_EDGE: ImageRegion = ImageRegion::new(49, 192, 18, 64);
const WOOD_TABLE_FILL: ImageRegion = ImageRegion::new(192, 196, 64, 56);
const WOOD_TABLE_RIGHT_EDGE: ImageRegion = ImageRegion::new(383, 192, 16, 64);
const WOOD_TABLE_BOTTOM_LEFT: ImageRegion = ImageRegion::new(44, 384, 84, 39);
const WOOD_TABLE_BOTTOM_EDGE: ImageRegion = ImageRegion::new(192, 384, 64, 39);
const WOOD_TABLE_BOTTOM_RIGHT: ImageRegion = ImageRegion::new(320, 384, 84, 39);
const WOOD_TABLE_TOP_LEFT_OUTSET_X: f32 = 4.0;
const WOOD_TABLE_TOP_RIGHT_OUTSET_X: f32 = 4.0;
const WOOD_TABLE_BOTTOM_LEFT_OUTSET_X: f32 = 5.0;
const WOOD_TABLE_BOTTOM_RIGHT_OUTSET_X: f32 = 5.0;
const WOOD_TABLE_TOP_CORNER_OUTSET_Y: f32 = 6.0;
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
const PILLAR_TILES: [AtlasTile; 6] = [
    AtlasTile { col: 5, row: 4 },
    AtlasTile { col: 6, row: 4 },
    AtlasTile { col: 7, row: 4 },
    AtlasTile { col: 5, row: 5 },
    AtlasTile { col: 6, row: 5 },
    AtlasTile { col: 7, row: 5 },
];

const VIEW_X: f32 = 0.0;
const VIEW_Y: f32 = 0.0;
const PANEL_H: f32 = 320.0;

const TILE_SIZE: f32 = 64.0;
const BUILDING_GRID_DIVISIONS: usize = 2;
const WORLD_COLS: usize = 48;
const WORLD_ROWS: usize = 29;
const BUILDING_SCALE: f32 = TILE_SIZE / TERRAIN_TILE_PX as f32;
const BUILDING_COUNT: usize = 8;
const PLANT_PROP_COUNT: usize = 12;
const GOLD_PROP_COUNT: usize = 7;
const GOLD_PALETTE_COUNT: usize = 1;
const ROCK_PROP_COUNT: usize = 4;
const ROCK_PALETTE_COUNT: usize = 1;
const MIN_CLOUDS: usize = 4;
const MAX_CLOUDS: usize = 6;

const PALETTE_X: f32 = 0.0;
const PALETTE_TILE: f32 = 48.0;
const PALETTE_GAP: f32 = 0.0;

const EDGE_SCROLL_ZONE: f32 = 72.0;
const EDGE_SCROLL_SPEED: f32 = 560.0;
const DEFAULT_SEED: u64 = 0x5EED_2026;
#[cfg(test)]
const ASE_EXPLORER_TINTS: [Rgba8; 5] = [
    Rgba8::WHITE,
    Rgba8::new(255, 214, 214, 255),
    Rgba8::new(216, 255, 218, 255),
    Rgba8::new(214, 235, 255, 255),
    Rgba8::new(255, 243, 188, 255),
];
#[cfg(test)]
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
#[cfg(test)]
const GOLD_EXPLORER_PATHS: [&str; 2] = [
    "ts_freepack/Gold Stones.aseprite",
    "ts_freepack/Gold Resource.aseprite",
];
const PAWN_ASEPRITE_PATH: &str = "ts_freepack/Pawn.aseprite";
const PARTICLE_FX_ASEPRITE_PATH: &str = "ts_freepack/Particle FX.aseprite";
const UNIT_WALK_SPECS: [UnitWalkSpec; 14] = [
    UnitWalkSpec::animated("Archer", "ts_freepack/Archer.aseprite", "Run"),
    UnitWalkSpec::animated("Archer Idle", "ts_freepack/Archer.aseprite", "Idle"),
    UnitWalkSpec::animated("Archer Shoot", "ts_freepack/Archer.aseprite", "Shoot"),
    UnitWalkSpec::animated_offset("Lancer", "ts_freepack/Lancer.aseprite", "Run", 0.0, -16.0),
    UnitWalkSpec::animated("Monk", "ts_freepack/Monk.aseprite", "Run"),
    UnitWalkSpec::animated("Monk Idle", "ts_freepack/Monk.aseprite", "Idle"),
    UnitWalkSpec::animated("Monk Heal", "ts_freepack/Monk.aseprite", "Heal"),
    UnitWalkSpec::animated("Monk Heal FX", "ts_freepack/Monk.aseprite", "Heal Effect"),
    UnitWalkSpec::animated_offset("Sheep", "ts_freepack/Sheep.aseprite", "Move", 0.0, 8.0),
    UnitWalkSpec::animated("Warrior", "ts_freepack/Warrior.aseprite", "Run"),
    UnitWalkSpec::animated("Warrior Idle", "ts_freepack/Warrior.aseprite", "Idle"),
    UnitWalkSpec::animated(
        "Warrior Attack 1",
        "ts_freepack/Warrior.aseprite",
        "Attack 1",
    ),
    UnitWalkSpec::animated(
        "Warrior Attack 2",
        "ts_freepack/Warrior.aseprite",
        "Attack 2",
    ),
    UnitWalkSpec::animated("Warrior Guard", "ts_freepack/Warrior.aseprite", "Guard"),
];

pub(crate) fn run() {
    WgpuSixWindowApp::new(
        "tactics world editor",
        AdapterConfig {
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
        },
        Game::new(),
        "tactics world viewer",
        AdapterConfig {
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
        },
        worldviewer::WorldViewer::new(),
        "tactics unit walk viewer",
        AdapterConfig {
            width: UNIT_VIEWER_WIDTH,
            height: UNIT_VIEWER_HEIGHT,
        },
        UnitWalkViewer::new(),
        "tactics loadscreen",
        AdapterConfig {
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
        },
        LoadScreen::new(),
        "tactics icon viewer",
        AdapterConfig {
            width: ICON_VIEWER_WIDTH,
            height: ICON_VIEWER_HEIGHT,
        },
        IconViewer::new(),
        "tactics event editor",
        AdapterConfig {
            width: EVENT_EDITOR_WIDTH,
            height: EVENT_EDITOR_HEIGHT,
        },
        EventEditor::new(),
    )
    .run()
    .expect("window loop failed");
}

#[derive(Clone, Copy, Debug, Default)]
struct Point {
    x: f32,
    y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct TableRect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
enum BackgroundTile {
    Grass,
    Water,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WorldEdge {
    Top,
    Bottom,
    Left,
    Right,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
enum WaterState {
    #[default]
    Nothing,
    Stone1,
    Stone2,
    Stone3,
    Stone4,
    Animation,
    Duck,
}

#[allow(dead_code)]
impl WaterState {
    const ALL: [Self; 6] = [
        Self::Nothing,
        Self::Stone1,
        Self::Stone2,
        Self::Stone3,
        Self::Stone4,
        Self::Duck,
    ];

    fn next(self) -> Self {
        match self {
            Self::Nothing => Self::Stone1,
            Self::Stone1 => Self::Stone2,
            Self::Stone2 => Self::Stone3,
            Self::Stone3 => Self::Stone4,
            Self::Stone4 | Self::Animation => Self::Duck,
            Self::Duck => Self::Nothing,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Brush {
    Background(BackgroundTile),
    Foreground(AtlasTile),
    Prop(PropKind),
    GoldResource,
    RockResource,
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
            Self::Prop(PropKind::Pillar(tile)) => Some(tile),
            Self::Background(BackgroundTile::Water)
            | Self::Prop(PropKind::Plant(_))
            | Self::Prop(PropKind::Gold(_))
            | Self::Prop(PropKind::Rock(_))
            | Self::GoldResource
            | Self::RockResource
            | Self::Building(_)
            | Self::Ramp(_)
            | Self::FogRect
            | Self::ClearForeground => None,
        }
    }

    fn footprint(self) -> (usize, usize) {
        match self {
            Self::Building(kind) => {
                let spec = building_spec(kind);
                (spec.footprint_cols, spec.footprint_rows)
            }
            Self::Ramp(_) => (1, 2),
            Self::Background(_)
            | Self::Foreground(_)
            | Self::Prop(_)
            | Self::GoldResource
            | Self::RockResource
            | Self::FogRect
            | Self::ClearForeground => (1, 1),
        }
    }

    fn footprint_offset(self) -> (usize, usize) {
        match self {
            Self::Building(kind) => {
                let spec = building_spec(kind);
                (spec.footprint_offset_cols, spec.footprint_offset_rows)
            }
            Self::Background(_)
            | Self::Foreground(_)
            | Self::Prop(_)
            | Self::GoldResource
            | Self::RockResource
            | Self::Ramp(_)
            | Self::FogRect
            | Self::ClearForeground => (0, 0),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Ramp {
    top: AtlasTile,
    bottom: AtlasTile,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
enum PropKind {
    Pillar(AtlasTile),
    Plant(PlantKind),
    Gold(GoldKind),
    Rock(RockKind),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
enum GoldKind {
    Stone1,
    Stone2,
    Stone3,
    Stone4,
    Stone5,
    Stone6,
    Nugget,
}

impl GoldKind {
    const ALL: [Self; GOLD_PROP_COUNT] = [
        Self::Stone1,
        Self::Stone2,
        Self::Stone3,
        Self::Stone4,
        Self::Stone5,
        Self::Stone6,
        Self::Nugget,
    ];

    fn index(self) -> usize {
        match self {
            Self::Stone1 => 0,
            Self::Stone2 => 1,
            Self::Stone3 => 2,
            Self::Stone4 => 3,
            Self::Stone5 => 4,
            Self::Stone6 => 5,
            Self::Nugget => 6,
        }
    }

    fn tag(self) -> &'static str {
        match self {
            Self::Stone1 => "Gold 1",
            Self::Stone2 => "Gold 2",
            Self::Stone3 => "Gold 3",
            Self::Stone4 => "Gold 4",
            Self::Stone5 => "Gold 5",
            Self::Stone6 => "Gold 6",
            Self::Nugget => "Gold Nugget",
        }
    }

    fn source_tag(self) -> Option<&'static str> {
        match self {
            Self::Stone1 => Some("1"),
            Self::Stone2 => Some("2"),
            Self::Stone3 => Some("3"),
            Self::Stone4 => Some("4"),
            Self::Stone5 => Some("5"),
            Self::Stone6 => Some("6"),
            Self::Nugget => None,
        }
    }

    fn next(self) -> Option<Self> {
        match self {
            Self::Stone1 => Some(Self::Stone2),
            Self::Stone2 => Some(Self::Stone3),
            Self::Stone3 => Some(Self::Stone4),
            Self::Stone4 => Some(Self::Stone5),
            Self::Stone5 => Some(Self::Stone6),
            Self::Stone6 => Some(Self::Nugget),
            Self::Nugget => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
enum RockKind {
    Rock1,
    Rock2,
    Rock3,
    Rock4,
}

impl RockKind {
    const ALL: [Self; ROCK_PROP_COUNT] = [Self::Rock1, Self::Rock2, Self::Rock3, Self::Rock4];

    fn index(self) -> usize {
        match self {
            Self::Rock1 => 0,
            Self::Rock2 => 1,
            Self::Rock3 => 2,
            Self::Rock4 => 3,
        }
    }

    fn tag(self) -> &'static str {
        match self {
            Self::Rock1 => "Rock 1",
            Self::Rock2 => "Rock 2",
            Self::Rock3 => "Rock 3",
            Self::Rock4 => "Rock 4",
        }
    }

    fn bytes(self) -> &'static [u8] {
        match self {
            Self::Rock1 => ROCK1_BYTES,
            Self::Rock2 => ROCK2_BYTES,
            Self::Rock3 => ROCK3_BYTES,
            Self::Rock4 => ROCK4_BYTES,
        }
    }

    fn next(self) -> Option<Self> {
        match self {
            Self::Rock1 => Some(Self::Rock2),
            Self::Rock2 => Some(Self::Rock3),
            Self::Rock3 => Some(Self::Rock4),
            Self::Rock4 => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
enum PlantKind {
    Tree1,
    Stump1,
    Tree2,
    Stump2,
    Tree3,
    Stump3,
    Tree4,
    Stump4,
    Bush1,
    Bush2,
    Bush3,
    Bush4,
}

impl PlantKind {
    const ALL: [Self; PLANT_PROP_COUNT] = [
        Self::Tree1,
        Self::Stump1,
        Self::Tree2,
        Self::Stump2,
        Self::Tree3,
        Self::Stump3,
        Self::Tree4,
        Self::Stump4,
        Self::Bush1,
        Self::Bush2,
        Self::Bush3,
        Self::Bush4,
    ];

    fn index(self) -> usize {
        match self {
            Self::Tree1 => 0,
            Self::Stump1 => 1,
            Self::Tree2 => 2,
            Self::Stump2 => 3,
            Self::Tree3 => 4,
            Self::Stump3 => 5,
            Self::Tree4 => 6,
            Self::Stump4 => 7,
            Self::Bush1 => 8,
            Self::Bush2 => 9,
            Self::Bush3 => 10,
            Self::Bush4 => 11,
        }
    }

    fn tag(self) -> &'static str {
        match self {
            Self::Tree1 => "Tree 1",
            Self::Stump1 => "Stump 1",
            Self::Tree2 => "Tree 2",
            Self::Stump2 => "Stump 2",
            Self::Tree3 => "Tree 3",
            Self::Stump3 => "Stump 3",
            Self::Tree4 => "Tree 4",
            Self::Stump4 => "Stump 4",
            Self::Bush1 => "Bush 1",
            Self::Bush2 => "Bush 2",
            Self::Bush3 => "Bush 3",
            Self::Bush4 => "Bush 4",
        }
    }

    fn is_bush(self) -> bool {
        matches!(self, Self::Bush1 | Self::Bush2 | Self::Bush3 | Self::Bush4)
    }

    fn is_big_bush(self) -> bool {
        matches!(self, Self::Bush1 | Self::Bush3)
    }

    fn uses_half_height_footprint(self) -> bool {
        matches!(self, Self::Bush2 | Self::Bush4)
    }

    fn render_offset_y(self) -> f32 {
        if matches!(
            self,
            Self::Stump1 | Self::Stump2 | Self::Stump3 | Self::Stump4
        ) {
            -TILE_SIZE / 2.0
        } else {
            0.0
        }
    }

    fn render_scale(self) -> f32 {
        if matches!(self, Self::Tree2) {
            0.84
        } else {
            1.0
        }
    }

    fn visual_instance_count(self, col: usize, row: usize) -> usize {
        self.visual_instance_count_for_roll(plant_visual_roll(self, col, row))
    }

    fn visual_instance_count_for_roll(self, roll: u8) -> usize {
        if self.is_big_bush() {
            if roll < 25 { 2 } else { 1 }
        } else if matches!(self, Self::Bush2 | Self::Bush4) {
            if roll < 25 {
                3
            } else if roll < 70 {
                2
            } else {
                1
            }
        } else {
            1
        }
    }

    fn visual_instance_offset(self, count: usize, instance: usize) -> Point {
        if !self.is_bush() || count <= 1 {
            return Point::default();
        }

        match (count, instance) {
            (2, 0) => Point { x: -9.0, y: 1.0 },
            (2, 1) => Point { x: 9.0, y: -3.0 },
            (3, 0) => Point { x: 0.0, y: -8.0 },
            (3, 1) => Point { x: -13.0, y: 2.0 },
            (3, 2) => Point { x: 13.0, y: 3.0 },
            _ => Point::default(),
        }
    }

    fn animates_in_environment(self) -> bool {
        matches!(
            self,
            Self::Tree1
                | Self::Tree2
                | Self::Tree3
                | Self::Tree4
                | Self::Bush1
                | Self::Bush2
                | Self::Bush3
                | Self::Bush4
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
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

    fn developer_name(self) -> &'static str {
        match self {
            Self::Archery => "ARCHERY",
            Self::Barracks => "BARRACKS",
            Self::Castle => "CASTLE",
            Self::House1 => "HOUSE 1",
            Self::House2 => "HOUSE 2",
            Self::House3 => "HOUSE 3",
            Self::Monastery => "MONASTERY",
            Self::Tower => "TOWER",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BuildingSpec {
    kind: BuildingKind,
    texture_id: u32,
    bytes: &'static [u8],
    footprint_offset_cols: usize,
    footprint_offset_rows: usize,
    footprint_cols: usize,
    footprint_rows: usize,
}

const BUILDING_SPECS: [BuildingSpec; BUILDING_COUNT] = [
    BuildingSpec {
        kind: BuildingKind::Archery,
        texture_id: BUILDING_ARCHERY_TEXTURE,
        bytes: RED_ARCHERY_BYTES,
        footprint_offset_cols: 0,
        footprint_offset_rows: 1,
        footprint_cols: 3,
        footprint_rows: 3,
    },
    BuildingSpec {
        kind: BuildingKind::Barracks,
        texture_id: BUILDING_BARRACKS_TEXTURE,
        bytes: RED_BARRACKS_BYTES,
        footprint_offset_cols: 0,
        footprint_offset_rows: 1,
        footprint_cols: 3,
        footprint_rows: 3,
    },
    BuildingSpec {
        kind: BuildingKind::Castle,
        texture_id: BUILDING_CASTLE_TEXTURE,
        bytes: RED_CASTLE_BYTES,
        footprint_offset_cols: 0,
        footprint_offset_rows: 1,
        footprint_cols: 5,
        footprint_rows: 3,
    },
    BuildingSpec {
        kind: BuildingKind::House1,
        texture_id: BUILDING_HOUSE1_TEXTURE,
        bytes: RED_HOUSE1_BYTES,
        footprint_offset_cols: 0,
        footprint_offset_rows: 1,
        footprint_cols: 2,
        footprint_rows: 2,
    },
    BuildingSpec {
        kind: BuildingKind::House2,
        texture_id: BUILDING_HOUSE2_TEXTURE,
        bytes: RED_HOUSE2_BYTES,
        footprint_offset_cols: 0,
        footprint_offset_rows: 1,
        footprint_cols: 2,
        footprint_rows: 2,
    },
    BuildingSpec {
        kind: BuildingKind::House3,
        texture_id: BUILDING_HOUSE3_TEXTURE,
        bytes: RED_HOUSE3_BYTES,
        footprint_offset_cols: 0,
        footprint_offset_rows: 1,
        footprint_cols: 2,
        footprint_rows: 2,
    },
    BuildingSpec {
        kind: BuildingKind::Monastery,
        texture_id: BUILDING_MONASTERY_TEXTURE,
        bytes: RED_MONASTERY_BYTES,
        footprint_offset_cols: 0,
        footprint_offset_rows: 2,
        footprint_cols: 3,
        footprint_rows: 3,
    },
    BuildingSpec {
        kind: BuildingKind::Tower,
        texture_id: BUILDING_TOWER_TEXTURE,
        bytes: RED_TOWER_BYTES,
        footprint_offset_cols: 0,
        footprint_offset_rows: 2,
        footprint_cols: 2,
        footprint_rows: 2,
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
    water_visuals: WaterVisualAssets,
    plant_props: [SpriteAnimation; PLANT_PROP_COUNT],
    gold_props: [SpriteAnimation; GOLD_PROP_COUNT],
    rock_props: [ImageAsset; ROCK_PROP_COUNT],
    clouds: Vec<ImageAsset>,
    cloud_instances: Vec<CloudInstance>,
    foreground_tiles: Vec<AtlasTile>,
    world: TileWorld,
    terrain_cache: TerrainDrawCache,
    selected: Option<Brush>,
    fog_drag_start: Option<(usize, usize)>,
    mouse: Point,
    camera: Point,
    left_down: bool,
    right_down: bool,
    ctrl_down: bool,
    uploaded: bool,
    window_width: u32,
    window_height: u32,
    environment_rng: SeededRng,
    water_animations: Vec<ActiveWaterAnimation>,
    plant_animations: Vec<ActivePlantAnimation>,
    gold_animations: Vec<ActiveGoldAnimation>,
    water_animation_timer: f32,
    plant_animation_timer: f32,
    gold_animation_timer: f32,
    started_at: Instant,
    last_frame: Instant,
}

#[cfg(test)]
struct AsepriteExplorer {
    files: Vec<ExplorerFile>,
    selected: usize,
    uploaded: bool,
    window_width: u32,
    window_height: u32,
    started_at: Instant,
}

struct UnitWalkViewer {
    units: Vec<UnitWalkClip>,
    uploaded: bool,
    window_width: u32,
    window_height: u32,
    started_at: Instant,
}

struct LoadScreen {
    wood_table: ImageAsset,
    uploaded: bool,
    window_width: u32,
    window_height: u32,
}

struct IconViewer {
    icons: Vec<IconTile>,
    uploaded: bool,
    window_width: u32,
    window_height: u32,
}

struct IconTile {
    label: String,
    image: ImageAsset,
}

struct EventEditor {
    rules: Vec<ScenarioRule>,
    next_rule_id: u32,
    draft_trigger: ScenarioTrigger,
    draft_condition: ScenarioCondition,
    draft_action: ScenarioAction,
    mouse: Point,
    window_width: u32,
    window_height: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
struct ScenarioRule {
    id: u32,
    enabled: bool,
    trigger: ScenarioTrigger,
    condition: ScenarioCondition,
    action: ScenarioAction,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
enum ScenarioTrigger {
    EveryThirtySeconds,
    OnceAfterOneMinute,
    UnitDeath,
    ResourceCollected,
    EnterRegion,
    DamageTaken,
}

impl ScenarioTrigger {
    const ALL: [Self; 6] = [
        Self::EveryThirtySeconds,
        Self::OnceAfterOneMinute,
        Self::UnitDeath,
        Self::ResourceCollected,
        Self::EnterRegion,
        Self::DamageTaken,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::EveryThirtySeconds => "TIMER 30S",
            Self::OnceAfterOneMinute => "ONCE 60S",
            Self::UnitDeath => "UNIT DEATH",
            Self::ResourceCollected => "RESOURCE",
            Self::EnterRegion => "ENTER AREA",
            Self::DamageTaken => "DMG TAKEN",
        }
    }

    fn next(self) -> Self {
        cycle_enum(Self::ALL, self)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
enum ScenarioCondition {
    Always,
    PlayerHasWood,
    PlayerHasMeat,
    UnitCountAtLeast,
    FlagIsSet,
    RegionHasEnemy,
}

impl ScenarioCondition {
    const ALL: [Self; 6] = [
        Self::Always,
        Self::PlayerHasWood,
        Self::PlayerHasMeat,
        Self::UnitCountAtLeast,
        Self::FlagIsSet,
        Self::RegionHasEnemy,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Always => "ALWAYS",
            Self::PlayerHasWood => "WOOD >= 100",
            Self::PlayerHasMeat => "MEAT >= 100",
            Self::UnitCountAtLeast => "UNITS >= 3",
            Self::FlagIsSet => "FLAG SET",
            Self::RegionHasEnemy => "ENEMY IN AREA",
        }
    }

    fn next(self) -> Self {
        cycle_enum(Self::ALL, self)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
enum ScenarioAction {
    GiveWood,
    GiveMeat,
    SpawnPawn,
    SpawnEnemy,
    DamageArea,
    PlayAnimation,
    SetFlag,
}

impl ScenarioAction {
    const ALL: [Self; 7] = [
        Self::GiveWood,
        Self::GiveMeat,
        Self::SpawnPawn,
        Self::SpawnEnemy,
        Self::DamageArea,
        Self::PlayAnimation,
        Self::SetFlag,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::GiveWood => "GIVE WOOD",
            Self::GiveMeat => "GIVE MEAT",
            Self::SpawnPawn => "SPAWN PAWN",
            Self::SpawnEnemy => "SPAWN ENEMY",
            Self::DamageArea => "DAMAGE AREA",
            Self::PlayAnimation => "PLAY ANIM",
            Self::SetFlag => "SET FLAG",
        }
    }

    fn next(self) -> Self {
        cycle_enum(Self::ALL, self)
    }
}

#[cfg(test)]
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

#[cfg(test)]
struct ExplorerClip {
    name: String,
    frame_indices: Vec<usize>,
    total_duration_ms: u32,
}

struct UnitWalkClip {
    name: String,
    source_tag: String,
    offset_x: f32,
    offset_y: f32,
    frames: Vec<ExplorerFrame>,
    total_duration_ms: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct UnitWalkSpec {
    label: &'static str,
    path: &'static str,
    preferred_tag: &'static str,
    offset_x: f32,
    offset_y: f32,
}

impl UnitWalkSpec {
    const fn animated(
        label: &'static str,
        path: &'static str,
        preferred_tag: &'static str,
    ) -> Self {
        Self {
            label,
            path,
            preferred_tag,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }

    const fn animated_offset(
        label: &'static str,
        path: &'static str,
        preferred_tag: &'static str,
        offset_x: f32,
        offset_y: f32,
    ) -> Self {
        Self {
            label,
            path,
            preferred_tag,
            offset_x,
            offset_y,
        }
    }
}

struct WaterVisualAssets {
    stones: [SpriteAnimation; 4],
    animation: SpriteAnimation,
    duck: SpriteAnimation,
}

struct SpriteAnimation {
    frames: Vec<ImageAsset>,
    durations_ms: Vec<u32>,
    total_duration_ms: u32,
}

impl SpriteAnimation {
    fn first_frame(&self) -> Option<&ImageAsset> {
        self.frames.first()
    }

    fn frame_once(&self, elapsed_ms: u32) -> Option<&ImageAsset> {
        if self.frames.is_empty() {
            return None;
        }
        if self.frames.len() == 1 || elapsed_ms >= self.total_duration_ms {
            return self.frames.first();
        }

        let mut cursor = elapsed_ms;
        for (index, duration) in self.durations_ms.iter().enumerate() {
            if cursor < *duration {
                return self.frames.get(index);
            }
            cursor -= *duration;
        }

        self.frames.first()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ActiveWaterAnimation {
    col: usize,
    row: usize,
    state: WaterState,
    started_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ActivePlantAnimation {
    col: usize,
    row: usize,
    kind: PlantKind,
    instance: usize,
    started_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ActiveGoldAnimation {
    x2: usize,
    y2: usize,
    kind: GoldKind,
    started_ms: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct CloudInstance {
    asset_index: usize,
    x: f32,
    y: f32,
    scale: f32,
    scale_wobble: f32,
    alpha_min: f32,
    alpha_max: f32,
    drift_x: f32,
    drift_y: f32,
    phase: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TerrainDrawCell {
    tile: AtlasTile,
    col: usize,
    row: usize,
}

#[derive(Debug)]
struct TerrainDrawCache {
    backgrounds: Vec<TerrainDrawCell>,
    foregrounds: Vec<TerrainDrawCell>,
    dirty: bool,
}

impl TerrainDrawCache {
    fn new() -> Self {
        Self {
            backgrounds: Vec::new(),
            foregrounds: Vec::new(),
            dirty: true,
        }
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn rebuild_if_dirty(&mut self, world: &TileWorld) {
        if !self.dirty {
            return;
        }

        self.backgrounds.clear();
        self.foregrounds.clear();
        for row in 0..world.rows {
            for col in 0..world.cols {
                if world.render_background(col, row) == BackgroundTile::Grass {
                    self.backgrounds.push(TerrainDrawCell {
                        tile: GRASS_BG_TILE,
                        col,
                        row,
                    });
                }
                if let Some(tile) = world.foreground(col, row) {
                    self.foregrounds.push(TerrainDrawCell { tile, col, row });
                }
            }
        }

        self.dirty = false;
    }
}

fn initial_editor_world() -> TileWorld {
    #[cfg(test)]
    {
        TileWorld::new(WORLD_COLS, WORLD_ROWS)
    }

    #[cfg(not(test))]
    {
        match TileWorld::load_from_path(WORLD_SAVE_PATH) {
            Ok(world) => {
                eprintln!("loaded world from {WORLD_SAVE_PATH}");
                world
            }
            Err(error) => {
                eprintln!("using empty grass world: failed to load {WORLD_SAVE_PATH}: {error}");
                TileWorld::new(WORLD_COLS, WORLD_ROWS)
            }
        }
    }
}

impl Game {
    fn new() -> Self {
        let terrain = TextureAtlas::from_png_bytes(TERRAIN_TEXTURE, TERRAIN_BYTES, TERRAIN_TILE_PX);
        let water_visuals = load_water_visual_assets();
        let plant_props = load_plant_prop_assets();
        let gold_props = load_gold_prop_assets();
        let rock_props = load_rock_prop_assets();
        let clouds = load_cloud_assets();
        let world = initial_editor_world();
        let cloud_instances = generate_clouds(DEFAULT_SEED, &clouds, world.cols, world.rows);
        let foreground_tiles = terrain
            .non_empty_tiles()
            .into_iter()
            .filter(|&tile| tile != GRASS_BG_TILE && !is_ramp_part(tile) && !is_pillar_tile(tile))
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
            water_visuals,
            plant_props,
            gold_props,
            rock_props,
            clouds,
            cloud_instances,
            foreground_tiles,
            world,
            terrain_cache: TerrainDrawCache::new(),
            selected: None,
            fog_drag_start: None,
            mouse: Point::default(),
            camera: Point::default(),
            left_down: false,
            right_down: false,
            ctrl_down: false,
            uploaded: false,
            window_width: WINDOW_WIDTH,
            window_height: WINDOW_HEIGHT,
            environment_rng: SeededRng::new(DEFAULT_SEED ^ 0xA11E_5747_EC05_2026),
            water_animations: Vec::new(),
            plant_animations: Vec::new(),
            gold_animations: Vec::new(),
            water_animation_timer: 0.2,
            plant_animation_timer: 0.4,
            gold_animation_timer: 0.6,
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
        self.panel_y()
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
        for image in self
            .water_visuals
            .stones
            .iter()
            .flat_map(|animation| animation.frames.iter())
            .chain(self.water_visuals.animation.frames.iter())
            .chain(self.water_visuals.duck.frames.iter())
        {
            let rc = adapter.upload_texture_rgba_image(
                image.texture_id,
                image.width,
                image.height,
                &image.rgba,
            );
            assert_eq!(
                rc, 0,
                "failed to upload water state texture {}",
                image.texture_id
            );
        }
        for image in self
            .plant_props
            .iter()
            .flat_map(|animation| animation.frames.iter())
            .chain(
                self.gold_props
                    .iter()
                    .flat_map(|animation| animation.frames.iter()),
            )
            .chain(self.rock_props.iter())
        {
            let rc = adapter.upload_texture_rgba_image(
                image.texture_id,
                image.width,
                image.height,
                &image.rgba,
            );
            assert_eq!(
                rc, 0,
                "failed to upload world prop texture {}",
                image.texture_id
            );
        }
        for image in &self.clouds {
            let rc = adapter.upload_texture_rgba_image(
                image.texture_id,
                image.width,
                image.height,
                &image.rgba,
            );
            assert_eq!(rc, 0, "failed to upload cloud texture {}", image.texture_id);
        }
        self.uploaded = true;
    }

    fn update_camera(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f32().min(0.05);
        self.last_frame = now;
        self.update_environment(dt);
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

    fn update_environment(&mut self, dt: f32) {
        let elapsed_ms = self.started_at.elapsed().as_millis() as u32;
        self.update_plant_environment(dt, elapsed_ms);
        self.update_gold_environment(dt, elapsed_ms);
        self.update_water_environment(dt, elapsed_ms);
    }

    fn update_plant_environment(&mut self, dt: f32, elapsed_ms: u32) {
        let plant_durations =
            PlantKind::ALL.map(|kind| (kind, self.plant_props[kind.index()].total_duration_ms));
        self.plant_animations.retain(|animation| {
            let duration = plant_durations
                .iter()
                .find_map(|&(kind, duration)| (kind == animation.kind).then_some(duration))
                .unwrap_or(0);
            elapsed_ms.saturating_sub(animation.started_ms) < duration
        });

        self.plant_animation_timer -= dt;
        if self.plant_animation_timer > 0.0 {
            return;
        }
        self.plant_animation_timer = self.environment_rng.range_f32(0.45, 1.0);

        let mut candidates = Vec::new();
        for prop in &self.world.props {
            let PropKind::Plant(kind) = prop.kind else {
                continue;
            };
            if !kind.animates_in_environment() {
                continue;
            }
            let instance_count = kind.visual_instance_count(prop.x2, prop.y2);
            for instance in 0..instance_count {
                if self.plant_animations.iter().any(|animation| {
                    animation.col == prop.x2
                        && animation.row == prop.y2
                        && animation.kind == kind
                        && animation.instance == instance
                }) {
                    continue;
                }
                candidates.push((prop.x2, prop.y2, kind, instance));
            }
        }

        let trigger_count = self.environment_rng.range_usize(1, 3).min(candidates.len());
        for _ in 0..trigger_count {
            let index = self.environment_rng.range_usize(0, candidates.len());
            let (col, row, kind, instance) = candidates.swap_remove(index);
            self.plant_animations.push(ActivePlantAnimation {
                col,
                row,
                kind,
                instance,
                started_ms: elapsed_ms,
            });
        }
    }

    fn update_gold_environment(&mut self, dt: f32, elapsed_ms: u32) {
        let gold_durations =
            GoldKind::ALL.map(|kind| (kind, self.gold_props[kind.index()].total_duration_ms));
        self.gold_animations.retain(|animation| {
            let duration = gold_durations
                .iter()
                .find_map(|&(kind, duration)| (kind == animation.kind).then_some(duration))
                .unwrap_or(0);
            elapsed_ms.saturating_sub(animation.started_ms) < duration
        });

        self.gold_animation_timer -= dt;
        if self.gold_animation_timer > 0.0 {
            return;
        }
        self.gold_animation_timer = self.environment_rng.range_f32(0.45, 1.1);

        let mut candidates = Vec::new();
        for prop in &self.world.props {
            let PropKind::Gold(kind) = prop.kind else {
                continue;
            };
            if self.gold_animations.iter().any(|animation| {
                animation.x2 == prop.x2 && animation.y2 == prop.y2 && animation.kind == kind
            }) {
                continue;
            }
            candidates.push((prop.x2, prop.y2, kind));
        }

        let trigger_count = self.environment_rng.range_usize(1, 3).min(candidates.len());
        for _ in 0..trigger_count {
            let index = self.environment_rng.range_usize(0, candidates.len());
            let (x2, y2, kind) = candidates.swap_remove(index);
            self.gold_animations.push(ActiveGoldAnimation {
                x2,
                y2,
                kind,
                started_ms: elapsed_ms,
            });
        }
    }

    fn update_water_environment(&mut self, dt: f32, elapsed_ms: u32) {
        let water_durations = [
            (
                WaterState::Stone1,
                self.water_visuals.stones[0].total_duration_ms,
            ),
            (
                WaterState::Stone2,
                self.water_visuals.stones[1].total_duration_ms,
            ),
            (
                WaterState::Stone3,
                self.water_visuals.stones[2].total_duration_ms,
            ),
            (
                WaterState::Stone4,
                self.water_visuals.stones[3].total_duration_ms,
            ),
            (
                WaterState::Animation,
                self.water_visuals.animation.total_duration_ms,
            ),
            (WaterState::Duck, self.water_visuals.duck.total_duration_ms),
        ];
        self.water_animations.retain(|animation| {
            let duration = water_durations
                .iter()
                .find_map(|&(state, duration)| (state == animation.state).then_some(duration))
                .unwrap_or(0);
            elapsed_ms.saturating_sub(animation.started_ms) < duration
        });

        self.water_animation_timer -= dt;
        if self.water_animation_timer > 0.0 {
            return;
        }
        self.water_animation_timer = self.environment_rng.range_f32(0.35, 0.85);

        let mut candidates = Vec::new();
        for row in 0..self.world.rows {
            for col in 0..self.world.cols {
                let state = match self.world.water_state(col, row) {
                    Some(
                        state @ (WaterState::Stone1
                        | WaterState::Stone2
                        | WaterState::Stone3
                        | WaterState::Stone4),
                    ) => Some(state),
                    _ if self
                        .world
                        .foreground(col, row)
                        .is_some_and(shoreline_tile_accepts_wave) =>
                    {
                        Some(WaterState::Animation)
                    }
                    _ => None,
                };
                let Some(state) = state else {
                    continue;
                };
                if self
                    .water_animations
                    .iter()
                    .any(|animation| animation.col == col && animation.row == row)
                {
                    continue;
                }
                candidates.push((col, row, state));
            }
        }

        let trigger_count = self.environment_rng.range_usize(1, 3).min(candidates.len());
        for _ in 0..trigger_count {
            let index = self.environment_rng.range_usize(0, candidates.len());
            let (col, row, state) = candidates.swap_remove(index);
            self.water_animations.push(ActiveWaterAnimation {
                col,
                row,
                state,
                started_ms: elapsed_ms,
            });
        }
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
        match brush {
            Brush::Building(kind) => {
                let Some((x2, y2)) = self.world_half_cell_at(self.mouse.x, self.mouse.y) else {
                    return;
                };
                let (anchor_x2, anchor_y2) = self.building_anchor_half_cell(kind, x2, y2);
                self.world.paint_building(kind, anchor_x2, anchor_y2);
            }
            Brush::Prop(PropKind::Plant(kind)) => {
                let Some((x2, y2)) = self.world_half_cell_at(self.mouse.x, self.mouse.y) else {
                    return;
                };
                self.world.paint_prop_half(PropKind::Plant(kind), x2, y2);
            }
            _ => {
                let Some((col, row)) = self.world_cell_at(self.mouse.x, self.mouse.y) else {
                    return;
                };
                self.world.paint(col, row, brush);
                self.terrain_cache.mark_dirty();
            }
        }
    }

    fn erase_at_mouse(&mut self) {
        let Some((col, row)) = self.world_cell_at(self.mouse.x, self.mouse.y) else {
            return;
        };
        self.world.clear_foreground(col, row);
        self.terrain_cache.mark_dirty();
    }

    fn edit_world_edge(&mut self, edge: WorldEdge) {
        if self.ctrl_down {
            if !self.world.remove_edge(edge) {
                return;
            }
            match edge {
                WorldEdge::Top => self.camera.y -= TILE_SIZE,
                WorldEdge::Left => self.camera.x -= TILE_SIZE,
                WorldEdge::Bottom | WorldEdge::Right => {}
            }
        } else {
            self.world.add_edge(edge);
            match edge {
                WorldEdge::Top => self.camera.y += TILE_SIZE,
                WorldEdge::Left => self.camera.x += TILE_SIZE,
                WorldEdge::Bottom | WorldEdge::Right => {}
            }
        }

        self.water_animations.clear();
        self.plant_animations.clear();
        self.gold_animations.clear();
        self.fog_drag_start = None;
        self.terrain_cache.mark_dirty();
        self.clamp_camera();
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

    fn world_half_cell_at(&self, x: f32, y: f32) -> Option<(usize, usize)> {
        if !inside_rect(x, y, VIEW_X, VIEW_Y, self.view_w(), self.view_h()) {
            return None;
        }

        let world_x = x - VIEW_X + self.camera.x;
        let world_y = y - VIEW_Y + self.camera.y;
        let half_tile = TILE_SIZE / BUILDING_GRID_DIVISIONS as f32;
        let x2 = (world_x / half_tile).floor() as usize;
        let y2 = (world_y / half_tile).floor() as usize;
        if x2 < self.world.cols * BUILDING_GRID_DIVISIONS
            && y2 < self.world.rows * BUILDING_GRID_DIVISIONS
        {
            Some((x2, y2))
        } else {
            None
        }
    }

    fn building_anchor_half_cell(
        &self,
        kind: BuildingKind,
        x2: usize,
        y2: usize,
    ) -> (isize, isize) {
        let spec = building_spec(kind);
        let center_offset_x2 = spec.footprint_offset_cols * BUILDING_GRID_DIVISIONS
            + spec.footprint_cols * BUILDING_GRID_DIVISIONS / 2;
        let center_offset_y2 = spec.footprint_offset_rows * BUILDING_GRID_DIVISIONS
            + spec.footprint_rows * BUILDING_GRID_DIVISIONS / 2;
        (
            x2 as isize - center_offset_x2 as isize,
            y2 as isize - center_offset_y2 as isize,
        )
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

        Some(self.palette_brush(slot))
    }

    fn draw(&mut self, adapter: &mut Adapter) {
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

    fn draw_world(&mut self, adapter: &mut Adapter) {
        let _ = adapter.set_texture_effect(TextureEffect::World);
        self.terrain_cache.rebuild_if_dirty(&self.world);
        let mut backgrounds = SpriteBatch::new(self.window_width, self.window_height);
        let mut foregrounds = SpriteBatch::new(self.window_width, self.window_height);
        let start_col = (self.camera.x / TILE_SIZE).floor().max(0.0) as usize;
        let start_row = (self.camera.y / TILE_SIZE).floor().max(0.0) as usize;
        let end_col = ((self.camera.x + self.view_w()) / TILE_SIZE).ceil() as usize + 1;
        let end_row = ((self.camera.y + self.view_h()) / TILE_SIZE).ceil() as usize + 1;

        for cell in self
            .terrain_cache
            .backgrounds
            .iter()
            .filter(|cell| terrain_cell_visible(cell, start_col, start_row, end_col, end_row))
        {
            let x = VIEW_X + cell.col as f32 * TILE_SIZE - self.camera.x;
            let y = VIEW_Y + cell.row as f32 * TILE_SIZE - self.camera.y;
            backgrounds.sprite(
                &self.terrain,
                cell.tile,
                x,
                y,
                TILE_SIZE,
                TILE_SIZE,
                Rgba8::WHITE,
            );
        }

        for cell in self
            .terrain_cache
            .foregrounds
            .iter()
            .filter(|cell| terrain_cell_visible(cell, start_col, start_row, end_col, end_row))
        {
            let x = VIEW_X + cell.col as f32 * TILE_SIZE - self.camera.x;
            let y = VIEW_Y + cell.row as f32 * TILE_SIZE - self.camera.y;
            foregrounds.sprite(
                &self.terrain,
                cell.tile,
                x,
                y,
                TILE_SIZE,
                TILE_SIZE,
                Rgba8::WHITE,
            );
        }

        if let (Some(brush), Some((col, row))) = (
            self.selected,
            self.world_cell_at(self.mouse.x, self.mouse.y),
        ) {
            let x = VIEW_X + col as f32 * TILE_SIZE - self.camera.x;
            let y = VIEW_Y + row as f32 * TILE_SIZE - self.camera.y;
            match brush {
                Brush::Ramp(ramp) if row + 1 < self.world.rows => {
                    let tint = if self.world.can_place_ramp(col, row) {
                        Rgba8::new(255, 255, 255, 130)
                    } else {
                        Rgba8::new(255, 96, 96, 155)
                    };
                    foregrounds.sprite(&self.terrain, ramp.top, x, y, TILE_SIZE, TILE_SIZE, tint);
                    foregrounds.sprite(
                        &self.terrain,
                        ramp.bottom,
                        x,
                        y + TILE_SIZE,
                        TILE_SIZE,
                        TILE_SIZE,
                        tint,
                    );
                }
                _ => {
                    if let Some(tile) = brush.preview_tile() {
                        foregrounds.sprite(
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

        let _ = adapter.draw_tex_triangles_no_present(self.terrain.texture_id, &backgrounds.bytes);
        self.draw_water_states(adapter);
        let _ = adapter.draw_tex_triangles_no_present(self.terrain.texture_id, &foregrounds.bytes);
        self.draw_props(adapter);
        self.draw_prop_preview(adapter);
        self.draw_buildings(adapter);
        self.draw_building_preview(adapter);
        let _ = adapter.set_texture_effect(TextureEffect::Plain);
        self.draw_fog(adapter);
        self.draw_clouds(adapter);
    }

    fn draw_water_states(&self, adapter: &mut Adapter) {
        let elapsed_ms = self.started_at.elapsed().as_millis() as u32;
        let start_col = (self.camera.x / TILE_SIZE).floor().max(0.0) as usize;
        let start_row = (self.camera.y / TILE_SIZE).floor().max(0.0) as usize;
        let end_col = ((self.camera.x + self.view_w()) / TILE_SIZE).ceil() as usize + 1;
        let end_row = ((self.camera.y + self.view_h()) / TILE_SIZE).ceil() as usize + 1;
        let mut batches = BTreeMap::new();

        for row in start_row..end_row.min(self.world.rows) {
            for col in start_col..end_col.min(self.world.cols) {
                if self
                    .world
                    .foreground(col, row)
                    .is_some_and(shoreline_tile_accepts_wave)
                {
                    if let Some(image) =
                        self.active_water_visual_frame(WaterState::Animation, col, row, elapsed_ms)
                    {
                        self.push_water_visual_image(&mut batches, image, col, row);
                    }
                }

                let Some(state) = self.world.water_state(col, row) else {
                    continue;
                };
                let image = match state {
                    WaterState::Nothing => continue,
                    WaterState::Stone1
                    | WaterState::Stone2
                    | WaterState::Stone3
                    | WaterState::Stone4
                    | WaterState::Duck => self.water_visual_frame(state, col, row, elapsed_ms),
                    WaterState::Animation => continue,
                };
                let Some(image) = image else {
                    continue;
                };
                self.push_water_visual_image(&mut batches, image, col, row);
            }
        }
        self.draw_image_batches(adapter, batches);
    }

    fn push_water_visual_image(
        &self,
        batches: &mut BTreeMap<u32, SpriteBatch>,
        image: &ImageAsset,
        col: usize,
        row: usize,
    ) {
        let w = image.width as f32 * BUILDING_SCALE;
        let h = image.height as f32 * BUILDING_SCALE;
        let x = VIEW_X + col as f32 * TILE_SIZE - self.camera.x + (TILE_SIZE - w) * 0.5;
        let y = VIEW_Y + row as f32 * TILE_SIZE - self.camera.y + (TILE_SIZE - h) * 0.5;
        self.push_image_batch(
            batches,
            image.texture_id,
            x.floor(),
            y.floor(),
            w.max(1.0),
            h.max(1.0),
            Rgba8::WHITE,
        );
    }

    fn active_water_visual_frame(
        &self,
        state: WaterState,
        col: usize,
        row: usize,
        elapsed_ms: u32,
    ) -> Option<&ImageAsset> {
        let animation = self.water_visual_animation(state)?;
        self.water_animations
            .iter()
            .find(|active| active.col == col && active.row == row && active.state == state)
            .and_then(|active| animation.frame_once(elapsed_ms.saturating_sub(active.started_ms)))
    }

    fn water_visual_frame(
        &self,
        state: WaterState,
        col: usize,
        row: usize,
        elapsed_ms: u32,
    ) -> Option<&ImageAsset> {
        let animation = self.water_visual_animation(state)?;
        let active = self
            .water_animations
            .iter()
            .find(|active| active.col == col && active.row == row && active.state == state);
        active
            .and_then(|active| animation.frame_once(elapsed_ms.saturating_sub(active.started_ms)))
            .or_else(|| animation.first_frame())
    }

    fn water_visual_animation(&self, state: WaterState) -> Option<&SpriteAnimation> {
        match state {
            WaterState::Nothing => None,
            WaterState::Stone1 => Some(&self.water_visuals.stones[0]),
            WaterState::Stone2 => Some(&self.water_visuals.stones[1]),
            WaterState::Stone3 => Some(&self.water_visuals.stones[2]),
            WaterState::Stone4 => Some(&self.water_visuals.stones[3]),
            WaterState::Animation => Some(&self.water_visuals.animation),
            WaterState::Duck => Some(&self.water_visuals.duck),
        }
    }

    fn draw_props(&self, adapter: &mut Adapter) {
        let mut terrain_batch = SpriteBatch::new(self.window_width, self.window_height);
        let mut image_batches = BTreeMap::new();
        let mut current_y2 = None;

        for prop in &self.world.props {
            if current_y2.is_some_and(|y2| y2 != prop.y2) {
                self.draw_prop_batches(adapter, &mut terrain_batch, &mut image_batches);
            }
            current_y2 = Some(prop.y2);

            match prop.kind {
                PropKind::Pillar(tile) => {
                    terrain_batch.sprite(
                        &self.terrain,
                        tile,
                        VIEW_X + half_grid_to_px(prop.x2) - self.camera.x,
                        VIEW_Y + half_grid_to_px(prop.y2) - self.camera.y,
                        TILE_SIZE,
                        TILE_SIZE,
                        Rgba8::WHITE,
                    );
                }
                PropKind::Plant(kind) => {
                    let instance_count = kind.visual_instance_count(prop.x2, prop.y2);
                    for instance in 0..instance_count {
                        let Some(image) = self.plant_frame(kind, prop.x2, prop.y2, instance) else {
                            continue;
                        };
                        let offset = kind.visual_instance_offset(instance_count, instance);
                        self.push_bottom_aligned_image_half(
                            &mut image_batches,
                            image,
                            prop.x2,
                            prop.y2,
                            offset.x,
                            kind.render_offset_y() + offset.y,
                            kind.render_scale(),
                            Rgba8::WHITE,
                        );
                    }
                }
                PropKind::Gold(kind) => {
                    let Some(image) = self.gold_frame(kind, prop.x2, prop.y2) else {
                        continue;
                    };
                    self.push_bottom_aligned_image_half(
                        &mut image_batches,
                        image,
                        prop.x2,
                        prop.y2,
                        0.0,
                        0.0,
                        1.0,
                        Rgba8::WHITE,
                    );
                }
                PropKind::Rock(kind) => {
                    let image = &self.rock_props[kind.index()];
                    self.push_bottom_aligned_image_half(
                        &mut image_batches,
                        image,
                        prop.x2,
                        prop.y2,
                        0.0,
                        0.0,
                        1.0,
                        Rgba8::WHITE,
                    );
                }
            }
        }

        self.draw_prop_batches(adapter, &mut terrain_batch, &mut image_batches);
    }

    fn draw_prop_batches(
        &self,
        adapter: &mut Adapter,
        terrain_batch: &mut SpriteBatch,
        image_batches: &mut BTreeMap<u32, SpriteBatch>,
    ) {
        if !terrain_batch.bytes.is_empty() {
            let _ = adapter
                .draw_tex_triangles_no_present(self.terrain.texture_id, &terrain_batch.bytes);
            terrain_batch.bytes.clear();
        }
        self.draw_image_batches(adapter, std::mem::take(image_batches));
    }

    fn draw_prop_preview(&self, adapter: &mut Adapter) {
        let Some(brush) = self.selected else {
            return;
        };
        let Some((x2, y2)) = self.world_half_cell_at(self.mouse.x, self.mouse.y) else {
            return;
        };

        match brush {
            Brush::Prop(PropKind::Plant(kind)) => {
                let tint = if self
                    .world
                    .can_place_prop_half(PropKind::Plant(kind), x2, y2)
                {
                    Rgba8::new(255, 255, 255, 145)
                } else {
                    Rgba8::new(255, 96, 96, 155)
                };
                let Some(image) = self.plant_props[kind.index()].first_frame() else {
                    return;
                };
                let instance_count = kind.visual_instance_count(x2, y2);
                for instance in 0..instance_count {
                    let offset = kind.visual_instance_offset(instance_count, instance);
                    self.draw_bottom_aligned_image_half(
                        adapter,
                        image,
                        x2,
                        y2,
                        offset.x,
                        kind.render_offset_y() + offset.y,
                        kind.render_scale(),
                        tint,
                    );
                }
            }
            Brush::GoldResource => {
                let Some((col, row)) = self.world_cell_at(self.mouse.x, self.mouse.y) else {
                    return;
                };
                let tint = if self.world.can_cycle_gold(col, row) {
                    Rgba8::new(255, 255, 255, 145)
                } else {
                    Rgba8::new(255, 96, 96, 155)
                };
                let Some(image) = self.gold_props[GoldKind::Nugget.index()].first_frame() else {
                    return;
                };
                self.draw_bottom_aligned_image_half(
                    adapter,
                    image,
                    col * BUILDING_GRID_DIVISIONS,
                    row * BUILDING_GRID_DIVISIONS,
                    0.0,
                    0.0,
                    1.0,
                    tint,
                );
            }
            Brush::RockResource => {
                let Some((col, row)) = self.world_cell_at(self.mouse.x, self.mouse.y) else {
                    return;
                };
                let tint = if self.world.can_cycle_rock(col, row) {
                    Rgba8::new(255, 255, 255, 145)
                } else {
                    Rgba8::new(255, 96, 96, 155)
                };
                self.draw_bottom_aligned_image_half(
                    adapter,
                    &self.rock_props[RockKind::Rock1.index()],
                    col * BUILDING_GRID_DIVISIONS,
                    row * BUILDING_GRID_DIVISIONS,
                    0.0,
                    0.0,
                    1.0,
                    tint,
                );
            }
            _ => {}
        }
    }

    fn plant_frame(
        &self,
        kind: PlantKind,
        col: usize,
        row: usize,
        instance: usize,
    ) -> Option<&ImageAsset> {
        let animation = &self.plant_props[kind.index()];
        if !kind.animates_in_environment() {
            return animation.first_frame();
        }
        let active = self.plant_animations.iter().find(|active| {
            active.col == col
                && active.row == row
                && active.kind == kind
                && active.instance == instance
        });
        active
            .and_then(|active| {
                animation.frame_once(
                    self.started_at
                        .elapsed()
                        .as_millis()
                        .try_into()
                        .unwrap_or(u32::MAX)
                        .saturating_sub(active.started_ms),
                )
            })
            .or_else(|| animation.first_frame())
    }

    fn gold_frame(&self, kind: GoldKind, x2: usize, y2: usize) -> Option<&ImageAsset> {
        let animation = &self.gold_props[kind.index()];
        let active = self
            .gold_animations
            .iter()
            .find(|active| active.x2 == x2 && active.y2 == y2 && active.kind == kind);
        active
            .and_then(|active| {
                animation.frame_once(
                    self.started_at
                        .elapsed()
                        .as_millis()
                        .try_into()
                        .unwrap_or(u32::MAX)
                        .saturating_sub(active.started_ms),
                )
            })
            .or_else(|| animation.first_frame())
    }

    fn draw_bottom_aligned_image_half(
        &self,
        adapter: &mut Adapter,
        image: &ImageAsset,
        x2: usize,
        y2: usize,
        offset_x: f32,
        offset_y: f32,
        scale: f32,
        tint: Rgba8,
    ) {
        let mut batches = BTreeMap::new();
        self.push_bottom_aligned_image_half(
            &mut batches,
            image,
            x2,
            y2,
            offset_x,
            offset_y,
            scale,
            tint,
        );
        self.draw_image_batches(adapter, batches);
    }

    fn push_bottom_aligned_image_half(
        &self,
        batches: &mut BTreeMap<u32, SpriteBatch>,
        image: &ImageAsset,
        x2: usize,
        y2: usize,
        offset_x: f32,
        offset_y: f32,
        scale: f32,
        tint: Rgba8,
    ) {
        let w = image.width as f32 * BUILDING_SCALE * scale;
        let h = image.height as f32 * BUILDING_SCALE * scale;
        let x = VIEW_X + half_grid_to_px(x2) - self.camera.x + (TILE_SIZE - w) * 0.5 + offset_x;
        let y =
            VIEW_Y + half_grid_to_px(y2 + BUILDING_GRID_DIVISIONS) - self.camera.y - h + offset_y;
        self.push_image_batch(
            batches,
            image.texture_id,
            x.floor(),
            y.floor(),
            w.max(1.0),
            h.max(1.0),
            tint,
        );
    }

    fn push_image_batch(
        &self,
        batches: &mut BTreeMap<u32, SpriteBatch>,
        texture_id: u32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        tint: Rgba8,
    ) {
        batches
            .entry(texture_id)
            .or_insert_with(|| SpriteBatch::new(self.window_width, self.window_height))
            .image(x, y, w, h, tint);
    }

    fn draw_image_batches(&self, adapter: &mut Adapter, batches: BTreeMap<u32, SpriteBatch>) {
        for (texture_id, batch) in batches {
            if !batch.bytes.is_empty() {
                let _ = adapter.draw_tex_triangles_no_present(texture_id, &batch.bytes);
            }
        }
    }

    fn draw_clouds(&self, adapter: &mut Adapter) {
        if self.clouds.is_empty() {
            return;
        }

        let elapsed = self.started_at.elapsed().as_secs_f32();
        let mut batches = BTreeMap::new();
        for cloud in &self.cloud_instances {
            let image = &self.clouds[cloud.asset_index % self.clouds.len()];
            let fade = ((elapsed * 0.12 + cloud.phase).sin() + 1.0) * 0.5;
            let alpha =
                (cloud.alpha_min + (cloud.alpha_max - cloud.alpha_min) * fade).clamp(0.0, 1.0);
            let scale =
                cloud.scale * (1.0 + cloud.scale_wobble * (elapsed * 0.18 + cloud.phase).sin());
            let world_w = self.world.width_px();
            let world_h = self.world.height_px();
            let wrap_w = world_w + image.width as f32 * scale;
            let wrap_h = world_h + image.height as f32 * scale;
            let world_x = (cloud.x + cloud.drift_x * elapsed).rem_euclid(wrap_w)
                - image.width as f32 * scale * 0.5;
            let world_y = (cloud.y + cloud.drift_y * elapsed).rem_euclid(wrap_h)
                - image.height as f32 * scale * 0.5;
            let x = VIEW_X + world_x - self.camera.x;
            let y = VIEW_Y + world_y - self.camera.y;
            let w = image.width as f32 * scale;
            let h = image.height as f32 * scale;

            if x + w < VIEW_X
                || y + h < VIEW_Y
                || x > VIEW_X + self.view_w()
                || y > VIEW_Y + self.view_h()
            {
                continue;
            }

            self.push_image_batch(
                &mut batches,
                image.texture_id,
                x.floor(),
                y.floor(),
                w.floor().max(1.0),
                h.floor().max(1.0),
                Rgba8::new(255, 255, 255, (alpha * 255.0).round() as u8),
            );
        }
        self.draw_image_batches(adapter, batches);
    }

    fn draw_buildings(&self, adapter: &mut Adapter) {
        let mut batches = BTreeMap::new();
        for building in &self.world.buildings {
            let image = &self.buildings[building.kind.index()];
            let x = VIEW_X + signed_half_grid_to_px(building.x2) - self.camera.x;
            let y = VIEW_Y + signed_half_grid_to_px(building.y2) - self.camera.y;
            let w = image.width as f32 * BUILDING_SCALE;
            let h = image.height as f32 * BUILDING_SCALE;
            if x + w < VIEW_X
                || y + h < VIEW_Y
                || x > VIEW_X + self.view_w()
                || y > VIEW_Y + self.view_h()
            {
                continue;
            }

            self.push_image_batch(&mut batches, image.texture_id, x, y, w, h, Rgba8::WHITE);
        }
        self.draw_image_batches(adapter, batches);
    }

    fn draw_building_preview(&self, adapter: &mut Adapter) {
        let Some(Brush::Building(kind)) = self.selected else {
            return;
        };
        let Some((x2, y2)) = self.world_half_cell_at(self.mouse.x, self.mouse.y) else {
            return;
        };
        let (x2, y2) = self.building_anchor_half_cell(kind, x2, y2);

        let image = &self.buildings[kind.index()];
        let can_place = self.world.can_place_building_kind(kind, x2, y2);
        let tint = if can_place {
            Rgba8::new(255, 255, 255, 145)
        } else {
            Rgba8::new(255, 96, 96, 155)
        };
        let mut sprite = SpriteBatch::new(self.window_width, self.window_height);
        sprite.image(
            VIEW_X + signed_half_grid_to_px(x2) - self.camera.x,
            VIEW_Y + signed_half_grid_to_px(y2) - self.camera.y,
            image.width as f32 * BUILDING_SCALE,
            image.height as f32 * BUILDING_SCALE,
            tint,
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

                let Some(brush) = self.selected else {
                    return;
                };
                let (footprint_rect2, can_place) = match brush {
                    Brush::Building(kind) => {
                        let Some((x2, y2)) = self.world_half_cell_at(self.mouse.x, self.mouse.y)
                        else {
                            let _ = adapter.draw_rgb_triangles_no_present(&overlay.bytes);
                            return;
                        };
                        let anchor = self.building_anchor_half_cell(kind, x2, y2);
                        (
                            brush_preview_footprint_rect2(
                                brush,
                                anchor.0,
                                anchor.1,
                                row,
                                self.world.rows,
                            ),
                            Some(self.world.can_place_building_kind(kind, anchor.0, anchor.1)),
                        )
                    }
                    Brush::Prop(kind @ PropKind::Plant(_)) => {
                        let Some((x2, y2)) = self.world_half_cell_at(self.mouse.x, self.mouse.y)
                        else {
                            let _ = adapter.draw_rgb_triangles_no_present(&overlay.bytes);
                            return;
                        };
                        (
                            brush_preview_footprint_rect2(
                                brush,
                                x2 as isize,
                                y2 as isize,
                                row,
                                self.world.rows,
                            ),
                            Some(self.world.can_place_prop_half(kind, x2, y2)),
                        )
                    }
                    _ => (
                        brush_preview_footprint_rect2(
                            brush,
                            (col * BUILDING_GRID_DIVISIONS) as isize,
                            (row * BUILDING_GRID_DIVISIONS) as isize,
                            row,
                            self.world.rows,
                        ),
                        match brush {
                            Brush::Ramp(_) => Some(self.world.can_place_ramp(col, row)),
                            Brush::Prop(kind) => Some(self.world.can_place_prop(kind, col, row)),
                            Brush::GoldResource => Some(self.world.can_cycle_gold(col, row)),
                            Brush::RockResource => Some(self.world.can_cycle_rock(col, row)),
                            _ => None,
                        },
                    ),
                };
                let (footprint_x2, footprint_y2, footprint_w2, footprint_h2) = footprint_rect2;
                let x = VIEW_X + signed_half_grid_to_px(footprint_x2) - self.camera.x;
                let y = VIEW_Y + signed_half_grid_to_px(footprint_y2) - self.camera.y;
                let width = half_grid_to_px(footprint_w2);
                let height = half_grid_to_px(footprint_h2);
                outline_rect(
                    &mut overlay,
                    x,
                    y,
                    width,
                    height,
                    2.0,
                    if can_place == Some(false) {
                        Rgba8::new(255, 86, 86, 230)
                    } else {
                        Rgba8::new(255, 225, 118, 210)
                    },
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
        for (slot, &tile) in PILLAR_TILES.iter().enumerate() {
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

        for (slot, kind) in PlantKind::ALL.into_iter().enumerate() {
            let (x, y) = self.palette_slot_rect(slot + 6 + BUILDING_COUNT + PILLAR_TILES.len());
            let Some(image) = self.plant_props[kind.index()].first_frame() else {
                continue;
            };
            let scale = (PALETTE_TILE / image.width as f32)
                .min(PALETTE_TILE / image.height as f32)
                .min(1.0);
            let w = image.width as f32 * scale;
            let h = image.height as f32 * scale;
            let mut plant = SpriteBatch::new(self.window_width, self.window_height);
            plant.image(
                x + (PALETTE_TILE - w) * 0.5,
                y + PALETTE_TILE - h,
                w,
                h,
                Rgba8::WHITE,
            );
            let _ = adapter.draw_tex_triangles_no_present(image.texture_id, &plant.bytes);
        }

        if let Some(image) = self.gold_props[GoldKind::Nugget.index()].first_frame() {
            let (x, y) =
                self.palette_slot_rect(6 + BUILDING_COUNT + PILLAR_TILES.len() + PLANT_PROP_COUNT);
            let scale = (PALETTE_TILE / image.width as f32)
                .min(PALETTE_TILE / image.height as f32)
                .min(1.0);
            let w = image.width as f32 * scale;
            let h = image.height as f32 * scale;
            let mut gold = SpriteBatch::new(self.window_width, self.window_height);
            gold.image(
                x + (PALETTE_TILE - w) * 0.5,
                y + PALETTE_TILE - h,
                w,
                h,
                Rgba8::WHITE,
            );
            let _ = adapter.draw_tex_triangles_no_present(image.texture_id, &gold.bytes);
        }

        {
            let image = &self.rock_props[RockKind::Rock1.index()];
            let (x, y) = self.palette_slot_rect(
                6 + BUILDING_COUNT + PILLAR_TILES.len() + PLANT_PROP_COUNT + GOLD_PALETTE_COUNT,
            );
            let scale = (PALETTE_TILE / image.width as f32)
                .min(PALETTE_TILE / image.height as f32)
                .min(1.0);
            let w = image.width as f32 * scale;
            let h = image.height as f32 * scale;
            let mut rock = SpriteBatch::new(self.window_width, self.window_height);
            rock.image(
                x + (PALETTE_TILE - w) * 0.5,
                y + PALETTE_TILE - h,
                w,
                h,
                Rgba8::WHITE,
            );
            let _ = adapter.draw_tex_triangles_no_present(image.texture_id, &rock.bytes);
        }

        let mut sprites = SpriteBatch::new(self.window_width, self.window_height);
        for (slot, &tile) in self.foreground_tiles.iter().enumerate() {
            let (x, y) = self.palette_slot_rect(
                slot + 6
                    + BUILDING_COUNT
                    + PILLAR_TILES.len()
                    + PLANT_PROP_COUNT
                    + GOLD_PALETTE_COUNT
                    + ROCK_PALETTE_COUNT,
            );
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
            outline_rect(&mut overlay, x, y, PALETTE_TILE, PALETTE_TILE, 2.0, color);
        }

        let _ = adapter.draw_rgb_triangles_no_present(&overlay.bytes);
    }

    fn palette_len(&self) -> usize {
        self.foreground_tiles.len()
            + 6
            + BUILDING_COUNT
            + PILLAR_TILES.len()
            + PLANT_PROP_COUNT
            + GOLD_PALETTE_COUNT
            + ROCK_PALETTE_COUNT
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
            _ if slot < 6 + BUILDING_COUNT + PILLAR_TILES.len() => {
                Brush::Prop(PropKind::Pillar(PILLAR_TILES[slot - 6 - BUILDING_COUNT]))
            }
            _ if slot < 6 + BUILDING_COUNT + PILLAR_TILES.len() + PLANT_PROP_COUNT => Brush::Prop(
                PropKind::Plant(PlantKind::ALL[slot - 6 - BUILDING_COUNT - PILLAR_TILES.len()]),
            ),
            _ if slot
                < 6 + BUILDING_COUNT
                    + PILLAR_TILES.len()
                    + PLANT_PROP_COUNT
                    + GOLD_PALETTE_COUNT =>
            {
                Brush::GoldResource
            }
            _ if slot
                < 6 + BUILDING_COUNT
                    + PILLAR_TILES.len()
                    + PLANT_PROP_COUNT
                    + GOLD_PALETTE_COUNT
                    + ROCK_PALETTE_COUNT =>
            {
                Brush::RockResource
            }
            _ => Brush::Foreground(
                self.foreground_tiles[slot
                    - 6
                    - BUILDING_COUNT
                    - PILLAR_TILES.len()
                    - PLANT_PROP_COUNT
                    - GOLD_PALETTE_COUNT
                    - ROCK_PALETTE_COUNT],
            ),
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
        let caption = self
            .selected
            .map(brush_developer_name)
            .unwrap_or_else(|| "BUILDINGS".to_string());
        let scale = 2.0;
        let text_width = ui_text_width(&caption, scale);
        ui.text(
            &caption,
            (self.window_w() - 30.0 - text_width).max(12.0),
            panel_y + panel_h - 36.0,
            scale,
            Rgba8::new(245, 255, 252, 255),
        );

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

    fn save_world(&self) {
        match self.world.save_to_path(WORLD_SAVE_PATH) {
            Ok(()) => eprintln!("saved world to {WORLD_SAVE_PATH}"),
            Err(error) => eprintln!("failed to save {WORLD_SAVE_PATH}: {error}"),
        }
    }

    fn load_world(&mut self) {
        match TileWorld::load_from_path(WORLD_SAVE_PATH) {
            Ok(world) => {
                self.world = world;
                self.fog_drag_start = None;
                self.left_down = false;
                self.right_down = false;
                self.water_animations.clear();
                self.plant_animations.clear();
                self.gold_animations.clear();
                self.terrain_cache.mark_dirty();
                self.clamp_camera();
                eprintln!("loaded world from {WORLD_SAVE_PATH}");
            }
            Err(error) => eprintln!("failed to load {WORLD_SAVE_PATH}: {error}"),
        }
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
            InputEvent::DigitPressed(1) => self.save_world(),
            InputEvent::DigitPressed(2) => self.load_world(),
            InputEvent::ModifiersChanged { ctrl } => {
                self.ctrl_down = ctrl;
            }
            InputEvent::KeyPressed(InputKey::U) => self.edit_world_edge(WorldEdge::Top),
            InputEvent::KeyPressed(InputKey::J) => self.edit_world_edge(WorldEdge::Bottom),
            InputEvent::KeyPressed(InputKey::H) => self.edit_world_edge(WorldEdge::Left),
            InputEvent::KeyPressed(InputKey::K) => self.edit_world_edge(WorldEdge::Right),
            _ => {}
        }
    }

    fn build_frame(&mut self, adapter: &mut Adapter) {
        self.upload_assets(adapter);
        self.update_camera();
        self.draw(adapter);
    }
}

#[cfg(test)]
impl AsepriteExplorer {
    fn new() -> Self {
        let mut next_texture_id = ASE_EXPLORER_TEXTURE_BASE;
        let files = ASEPRITE_STASH
            .iter()
            .filter_map(|&path| {
                if path == "ts_freepack/Gold Resource.aseprite" {
                    return load_explorer_file_group(&GOLD_EXPLORER_PATHS, &mut next_texture_id);
                }
                if path == "ts_freepack/Gold Stones.aseprite" {
                    return None;
                }

                load_explorer_file(path, &mut next_texture_id)
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

impl UnitWalkViewer {
    fn new() -> Self {
        let mut next_texture_id = UNIT_VIEWER_TEXTURE_BASE;
        let mut units = UNIT_WALK_SPECS
            .iter()
            .filter_map(|spec| load_unit_walk_clip(*spec, &mut next_texture_id))
            .collect::<Vec<_>>();
        units.extend(load_pawn_unit_clips(&mut next_texture_id));
        units.extend(load_particle_fx_unit_clips(&mut next_texture_id));

        Self {
            units,
            uploaded: false,
            window_width: UNIT_VIEWER_WIDTH,
            window_height: UNIT_VIEWER_HEIGHT,
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
                "failed to upload unit viewer ui texture {}",
                image.texture_id
            );
        }

        for unit in &self.units {
            for frame in &unit.frames {
                let rc = adapter.upload_texture_rgba_image(
                    frame.texture_id,
                    frame.width,
                    frame.height,
                    &frame.rgba,
                );
                assert_eq!(
                    rc, 0,
                    "failed to upload unit viewer texture {}",
                    frame.texture_id
                );
            }
        }

        self.uploaded = true;
    }
}

impl LoadScreen {
    fn new() -> Self {
        Self {
            wood_table: ImageAsset::from_png_bytes(WOOD_TABLE_TEXTURE, WOOD_TABLE_BYTES),
            uploaded: false,
            window_width: WINDOW_WIDTH,
            window_height: WINDOW_HEIGHT,
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

        let rc = adapter.upload_texture_rgba_image(
            self.wood_table.texture_id,
            self.wood_table.width,
            self.wood_table.height,
            &self.wood_table.rgba,
        );
        assert_eq!(rc, 0, "failed to upload loadscreen table texture");
        self.uploaded = true;
    }
}

impl IconViewer {
    fn new() -> Self {
        Self {
            icons: load_icon_viewer_icons(),
            uploaded: false,
            window_width: ICON_VIEWER_WIDTH,
            window_height: ICON_VIEWER_HEIGHT,
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
                "failed to upload icon viewer ui texture {}",
                image.texture_id
            );
        }

        for icon in &self.icons {
            let rc = adapter.upload_texture_rgba_image(
                icon.image.texture_id,
                icon.image.width,
                icon.image.height,
                &icon.image.rgba,
            );
            assert_eq!(
                rc, 0,
                "failed to upload icon viewer texture {}",
                icon.image.texture_id
            );
        }

        self.uploaded = true;
    }
}

impl EventEditor {
    fn new() -> Self {
        Self {
            rules: vec![ScenarioRule {
                id: 1,
                enabled: true,
                trigger: ScenarioTrigger::EveryThirtySeconds,
                condition: ScenarioCondition::Always,
                action: ScenarioAction::SpawnPawn,
            }],
            next_rule_id: 2,
            draft_trigger: ScenarioTrigger::EveryThirtySeconds,
            draft_condition: ScenarioCondition::Always,
            draft_action: ScenarioAction::SpawnPawn,
            mouse: Point::default(),
            window_width: EVENT_EDITOR_WIDTH,
            window_height: EVENT_EDITOR_HEIGHT,
        }
    }

    fn resize_view(&mut self, width: u32, height: u32) {
        self.window_width = width.max(1);
        self.window_height = height.max(1);
    }

    fn handle_left_click(&mut self) {
        if inside_rect(self.mouse.x, self.mouse.y, 24.0, 78.0, 218.0, 54.0) {
            self.draft_trigger = self.draft_trigger.next();
            return;
        }
        if inside_rect(self.mouse.x, self.mouse.y, 258.0, 78.0, 218.0, 54.0) {
            self.draft_condition = self.draft_condition.next();
            return;
        }
        if inside_rect(self.mouse.x, self.mouse.y, 492.0, 78.0, 218.0, 54.0) {
            self.draft_action = self.draft_action.next();
            return;
        }
        if inside_rect(self.mouse.x, self.mouse.y, 24.0, 142.0, 130.0, 34.0) {
            self.add_rule();
            return;
        }

        let table_y = 214.0;
        let row_h = 42.0;
        for index in 0..self.rules.len() {
            let y = table_y + index as f32 * row_h;
            if y > self.window_height as f32 - row_h {
                break;
            }
            if inside_rect(self.mouse.x, self.mouse.y, 28.0, y + 7.0, 58.0, 28.0) {
                self.rules[index].enabled = !self.rules[index].enabled;
                return;
            }
            if inside_rect(
                self.mouse.x,
                self.mouse.y,
                self.window_width as f32 - 86.0,
                y + 7.0,
                58.0,
                28.0,
            ) {
                self.rules.remove(index);
                return;
            }
        }
    }

    fn add_rule(&mut self) {
        self.rules.push(ScenarioRule {
            id: self.next_rule_id,
            enabled: true,
            trigger: self.draft_trigger,
            condition: self.draft_condition,
            action: self.draft_action,
        });
        self.next_rule_id += 1;
    }
}

fn load_unit_walk_clip(spec: UnitWalkSpec, next_texture_id: &mut u32) -> Option<UnitWalkClip> {
    let set = ase_assets::load_tinted_aseprite_set(
        spec.path,
        [255, 255, 255, 255],
        ase_assets::TintMode::Multiply,
    )
    .ok()?;
    let tag = set
        .tags
        .iter()
        .find(|tag| tag.name.trim() == spec.preferred_tag)
        .or_else(|| set.tags.iter().find(|tag| tag.name.trim() == "Idle"))?;
    unit_clip_from_frames(
        spec.label.to_string(),
        tag.name.trim().to_string(),
        set.frames
            .get(tag.from_frame as usize..=tag.to_frame as usize)?,
        spec.offset_x,
        spec.offset_y,
        next_texture_id,
    )
}

fn load_pawn_unit_clips(next_texture_id: &mut u32) -> Vec<UnitWalkClip> {
    let Ok(set) = ase_assets::load_tinted_aseprite_set(
        PAWN_ASEPRITE_PATH,
        [255, 255, 255, 255],
        ase_assets::TintMode::Multiply,
    ) else {
        return Vec::new();
    };

    set.tags
        .iter()
        .filter_map(|tag| {
            let tag_name = tag.name.trim().to_string();
            let frames = set
                .frames
                .get(tag.from_frame as usize..=tag.to_frame as usize)?;
            unit_clip_from_frames(
                format!("Pawn {tag_name}"),
                tag_name,
                frames,
                0.0,
                0.0,
                next_texture_id,
            )
        })
        .collect()
}

fn load_particle_fx_unit_clips(next_texture_id: &mut u32) -> Vec<UnitWalkClip> {
    let Ok(set) = ase_assets::load_tinted_aseprite_set(
        PARTICLE_FX_ASEPRITE_PATH,
        [255, 255, 255, 255],
        ase_assets::TintMode::Multiply,
    ) else {
        return Vec::new();
    };

    set.tags
        .iter()
        .filter_map(|tag| {
            let tag_name = tag.name.trim().to_string();
            let frames = set
                .frames
                .get(tag.from_frame as usize..=tag.to_frame as usize)?;
            unit_clip_from_frames(
                format!("Particle {tag_name}"),
                tag_name,
                frames,
                0.0,
                0.0,
                next_texture_id,
            )
        })
        .collect()
}

fn unit_clip_from_frames(
    name: String,
    source_tag: String,
    source_frames: &[ase_assets::RgbaAsset],
    offset_x: f32,
    offset_y: f32,
    next_texture_id: &mut u32,
) -> Option<UnitWalkClip> {
    let frames = source_frames
        .iter()
        .filter(|frame| frame.rgba.chunks_exact(4).any(|pixel| pixel[3] != 0))
        .map(|frame| {
            let texture_id = *next_texture_id;
            *next_texture_id += 1;
            let image = ImageAsset::from_rgba_cropped(
                texture_id,
                frame.width,
                frame.height,
                frame.rgba.clone(),
            );
            ExplorerFrame {
                texture_id: image.texture_id,
                width: image.width,
                height: image.height,
                rgba: image.rgba,
                duration_ms: frame.duration_ms.unwrap_or(120).max(1),
            }
        })
        .collect::<Vec<_>>();
    if frames.is_empty() {
        return None;
    }

    let total_duration_ms = frames
        .iter()
        .map(|frame| frame.duration_ms)
        .sum::<u32>()
        .max(1);
    Some(UnitWalkClip {
        name,
        source_tag,
        offset_x,
        offset_y,
        frames,
        total_duration_ms,
    })
}

fn unit_walk_frame(unit: &UnitWalkClip, elapsed_ms: u32) -> &ExplorerFrame {
    if unit.frames.len() == 1 {
        return &unit.frames[0];
    }

    let mut cursor = elapsed_ms % unit.total_duration_ms;
    for frame in &unit.frames {
        if cursor < frame.duration_ms {
            return frame;
        }
        cursor -= frame.duration_ms;
    }

    &unit.frames[0]
}

fn unit_viewer_flip_x(elapsed_ms: u32) -> bool {
    (elapsed_ms / 5_000) % 2 == 1
}

fn ui_text_width(text: &str, scale: f32) -> f32 {
    text.chars().count() as f32 * 6.0 * scale
}

fn event_editor_selector(
    solid: &mut SolidBatch,
    text: &mut ts_ui::UiBatch,
    mouse: Point,
    label: &str,
    value: &str,
    x: f32,
    y: f32,
    w: f32,
) {
    let hover = inside_rect(mouse.x, mouse.y, x, y, w, 54.0);
    let fill = if hover {
        Rgba8::new(97, 134, 132, 255)
    } else {
        Rgba8::new(77, 111, 111, 255)
    };
    solid.rect(x, y, w, 54.0, fill);
    outline_rect(solid, x, y, w, 54.0, 2.0, Rgba8::new(124, 164, 160, 255));
    text.text(
        label,
        x + 12.0,
        y + 8.0,
        1.0,
        Rgba8::new(183, 205, 197, 255),
    );
    text.text(
        value,
        x + 12.0,
        y + 29.0,
        1.0,
        Rgba8::new(232, 245, 239, 255),
    );
    text.text(
        ">",
        x + w - 22.0,
        y + 29.0,
        1.0,
        Rgba8::new(232, 245, 239, 255),
    );
}

fn event_editor_button(
    solid: &mut SolidBatch,
    text: &mut ts_ui::UiBatch,
    mouse: Point,
    label: &str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) {
    let hover = inside_rect(mouse.x, mouse.y, x, y, w, h);
    let fill = if hover {
        Rgba8::new(130, 164, 143, 255)
    } else {
        Rgba8::new(92, 128, 119, 255)
    };
    solid.rect(x, y, w, h, fill);
    outline_rect(solid, x, y, w, h, 2.0, Rgba8::new(153, 190, 175, 255));
    let label_w = ui_text_width(label, 1.0);
    text.text(
        label,
        x + (w - label_w) * 0.5,
        y + (h - 8.0) * 0.5,
        1.0,
        Rgba8::new(238, 247, 241, 255),
    );
}

fn cycle_enum<T: Copy + PartialEq, const N: usize>(items: [T; N], current: T) -> T {
    let index = items
        .iter()
        .position(|item| *item == current)
        .map(|index| (index + 1) % N)
        .unwrap_or(0);
    items[index]
}

fn brush_developer_name(brush: Brush) -> String {
    match brush {
        Brush::Background(BackgroundTile::Water) => "BG WATER".to_string(),
        Brush::Background(BackgroundTile::Grass) => "BG GRASS".to_string(),
        Brush::Foreground(tile) => atlas_tile_developer_name(tile),
        Brush::Prop(PropKind::Pillar(tile)) => PILLAR_TILES
            .iter()
            .position(|&pillar| pillar == tile)
            .map(|index| format!("PILLAR {}", index + 1))
            .unwrap_or_else(|| atlas_tile_developer_name(tile)),
        Brush::Prop(PropKind::Plant(kind)) => kind.tag().to_ascii_uppercase(),
        Brush::Prop(PropKind::Gold(kind)) => kind.tag().to_ascii_uppercase(),
        Brush::Prop(PropKind::Rock(kind)) => kind.tag().to_ascii_uppercase(),
        Brush::GoldResource => "GOLD RESOURCE".to_string(),
        Brush::RockResource => "ROCK RESOURCE".to_string(),
        Brush::Building(kind) => format!("BUILDING {}", kind.developer_name()),
        Brush::Ramp(ramp) if ramp == RAMP_A => "RAMP A".to_string(),
        Brush::Ramp(ramp) if ramp == RAMP_B => "RAMP B".to_string(),
        Brush::Ramp(_) => "RAMP CUSTOM".to_string(),
        Brush::FogRect => "FOG RECT".to_string(),
        Brush::ClearForeground => "CLEAR FOREGROUND".to_string(),
    }
}

fn atlas_tile_developer_name(tile: AtlasTile) -> String {
    let name = match tile {
        GRASS_BG_TILE => "BG GRASS TILE",
        SHORE_TOP_LEFT => "SHORE TOP LEFT",
        SHORE_TOP => "SHORE TOP",
        SHORE_TOP_RIGHT => "SHORE TOP RIGHT",
        SHORE_LEFT => "SHORE LEFT",
        SHORE_RIGHT => "SHORE RIGHT",
        SHORE_BOTTOM_LEFT => "SHORE BOTTOM LEFT",
        SHORE_BOTTOM => "SHORE BOTTOM",
        SHORE_BOTTOM_RIGHT => "SHORE BOTTOM RIGHT",
        SHORE_NARROW_TOP => "SHORE NARROW TOP",
        SHORE_NARROW_MIDDLE => "SHORE NARROW MIDDLE",
        SHORE_NARROW_BOTTOM => "SHORE NARROW BOTTOM",
        SHORE_NARROW_LEFT => "SHORE NARROW LEFT",
        SHORE_NARROW_CENTER => "SHORE NARROW CENTER",
        SHORE_NARROW_RIGHT => "SHORE NARROW RIGHT",
        SHORE_SINGLE_IN_WATER => "SHORE SINGLE IN WATER",
        SHORE_SINGLE_IN_GRASS => "SHORE SINGLE IN GRASS",
        _ if tile == RAMP_A.top => "RAMP A TOP",
        _ if tile == RAMP_A.bottom => "RAMP A BOTTOM",
        _ if tile == RAMP_B.top => "RAMP B TOP",
        _ if tile == RAMP_B.bottom => "RAMP B BOTTOM",
        _ => return format!("TILE C{} R{}", tile.col, tile.row),
    };
    name.to_string()
}

fn unit_viewer_label(unit: &UnitWalkClip) -> String {
    let name = unit.name.to_uppercase();
    let tag = unit.source_tag.to_uppercase();
    if tag != "RUN" && !name.contains(tag.trim_start_matches("RUN ").trim()) {
        format!("{name} {tag}")
    } else {
        name
    }
}

#[cfg(test)]
fn load_explorer_file(path: &str, next_texture_id: &mut u32) -> Option<ExplorerFile> {
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
            let texture_id = *next_texture_id;
            *next_texture_id += 1;
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
    let clips = explorer_clips_from_tags(path, &set.tags, &frames);

    Some(ExplorerFile {
        frames,
        clips,
        tint_index: 0,
    })
}

#[cfg(test)]
fn load_explorer_file_group(paths: &[&str], next_texture_id: &mut u32) -> Option<ExplorerFile> {
    let mut frames = Vec::new();
    let mut clips = Vec::new();

    for path in paths {
        let file = load_explorer_file(path, next_texture_id)?;
        let frame_offset = frames.len();
        frames.extend(file.frames);
        clips.extend(file.clips.into_iter().map(|mut clip| {
            for frame_index in &mut clip.frame_indices {
                *frame_index += frame_offset;
            }
            clip
        }));
    }

    Some(ExplorerFile {
        frames,
        clips,
        tint_index: 0,
    })
}

#[cfg(test)]
fn explorer_clips_from_tags(
    path: &str,
    tags: &[ase_assets::AsepriteTag],
    frames: &[ExplorerFrame],
) -> Vec<ExplorerClip> {
    let clips = tags
        .iter()
        .filter(|tag| !is_explorer_helper_tag(tag.name.as_str()))
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

    if should_group_untagged_as_single_animation(path, frames) {
        return vec![ExplorerClip {
            name: "Animation".to_string(),
            frame_indices: (0..frames.len()).collect(),
            total_duration_ms: frames
                .iter()
                .map(|frame| frame.duration_ms)
                .sum::<u32>()
                .max(1),
        }];
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

#[cfg(test)]
fn is_explorer_helper_tag(name: &str) -> bool {
    matches!(name, "Still" | "Highlight")
}

#[cfg(test)]
fn should_group_untagged_as_single_animation(path: &str, frames: &[ExplorerFrame]) -> bool {
    frames.len() > 1
        && (path.contains("Rubber Duck")
            || path.contains("Water Rocks")
            || path.contains("Gold Resource"))
}

#[cfg(test)]
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

fn load_cloud_assets() -> Vec<ImageAsset> {
    ase_assets::load_tinted_aseprite_set(
        CLOUDS_ASEPRITE_PATH,
        [255, 255, 255, 255],
        ase_assets::TintMode::Multiply,
    )
    .expect("cloud aseprite should decode")
    .frames
    .into_iter()
    .enumerate()
    .map(|(index, frame)| {
        ImageAsset::from_rgba_cropped(
            CLOUD_TEXTURE_BASE + index as u32,
            frame.width,
            frame.height,
            frame.rgba,
        )
    })
    .filter(|image| image.rgba.chunks_exact(4).any(|pixel| pixel[3] != 0))
    .collect()
}

fn load_water_visual_assets() -> WaterVisualAssets {
    let stones = std::array::from_fn(|index| {
        load_aseprite_animation(
            WATER_ROCK_ASEPRITE_PATHS[index],
            WATER_STATE_TEXTURE_BASE + (index as u32 * WATER_STATE_TEXTURE_STRIDE),
            "water rock",
        )
    });

    WaterVisualAssets {
        stones,
        animation: load_aseprite_animation(
            WATER_FOAM_ASEPRITE_PATH,
            WATER_FOAM_TEXTURE_BASE,
            "water foam",
        ),
        duck: load_aseprite_animation(RUBBER_DUCK_ASEPRITE_PATH, WATER_DUCK_TEXTURE_BASE, "duck"),
    }
}

fn load_aseprite_animation(path: &str, texture_base: u32, label: &str) -> SpriteAnimation {
    let set = ase_assets::load_tinted_aseprite_set(
        path,
        [255, 255, 255, 255],
        ase_assets::TintMode::Multiply,
    )
    .unwrap_or_else(|_| panic!("{label} aseprite should decode"));
    let frames = set
        .frames
        .into_iter()
        .enumerate()
        .map(|(index, frame)| {
            (
                ImageAsset::from_rgba_cropped(
                    texture_base + index as u32,
                    frame.width,
                    frame.height,
                    frame.rgba,
                ),
                frame.duration_ms.unwrap_or(120).max(1),
            )
        })
        .filter(|(image, _)| image.rgba.chunks_exact(4).any(|pixel| pixel[3] != 0))
        .collect::<Vec<_>>();
    let durations_ms = frames
        .iter()
        .map(|(_, duration)| *duration)
        .collect::<Vec<_>>();
    let total_duration_ms = durations_ms.iter().sum::<u32>().max(1);
    let frames = frames.into_iter().map(|(image, _)| image).collect();

    SpriteAnimation {
        frames,
        durations_ms,
        total_duration_ms,
    }
}

fn load_plant_prop_assets() -> [SpriteAnimation; PLANT_PROP_COUNT] {
    let trees = ase_assets::load_tinted_aseprite_set(
        TREES_ASEPRITE_PATH,
        [255, 255, 255, 255],
        ase_assets::TintMode::Multiply,
    )
    .expect("trees aseprite should decode");
    let bushes = ase_assets::load_tinted_aseprite_set(
        BUSHES_ASEPRITE_PATH,
        [255, 255, 255, 255],
        ase_assets::TintMode::Multiply,
    )
    .expect("bushes aseprite should decode");

    std::array::from_fn(|index| {
        let kind = PlantKind::ALL[index];
        let set = if kind.is_bush() { &bushes } else { &trees };
        let frames = set
            .frames_for_tag(kind.tag())
            .unwrap_or_else(|| panic!("plant aseprite tag {} should exist", kind.tag()))
            .iter()
            .enumerate()
            .map(|(frame_index, frame)| {
                (
                    ImageAsset::from_rgba_cropped(
                        PLANT_PROP_TEXTURE_BASE
                            + (index as u32 * PLANT_PROP_TEXTURE_STRIDE)
                            + frame_index as u32,
                        frame.width,
                        frame.height,
                        frame.rgba.clone(),
                    ),
                    frame.duration_ms.unwrap_or(120).max(1),
                )
            })
            .filter(|(image, _)| image.rgba.chunks_exact(4).any(|pixel| pixel[3] != 0))
            .collect::<Vec<_>>();
        assert!(
            !frames.is_empty(),
            "plant aseprite tag {} should have visible frames",
            kind.tag()
        );
        let durations_ms = frames
            .iter()
            .map(|(_, duration)| *duration)
            .collect::<Vec<_>>();
        let total_duration_ms = durations_ms.iter().sum::<u32>().max(1);
        let frames = frames.into_iter().map(|(image, _)| image).collect();

        SpriteAnimation {
            frames,
            durations_ms,
            total_duration_ms,
        }
    })
}

fn load_gold_prop_assets() -> [SpriteAnimation; GOLD_PROP_COUNT] {
    let gold_stones = ase_assets::load_tinted_aseprite_set(
        "ts_freepack/Gold Stones.aseprite",
        [255, 255, 255, 255],
        ase_assets::TintMode::Multiply,
    )
    .expect("gold stones aseprite should decode");
    let gold_resource = ase_assets::load_tinted_aseprite_set(
        "ts_freepack/Gold Resource.aseprite",
        [255, 255, 255, 255],
        ase_assets::TintMode::Multiply,
    )
    .expect("gold resource aseprite should decode");

    std::array::from_fn(|index| {
        let kind = GoldKind::ALL[index];
        let source_frames = if let Some(tag) = kind.source_tag() {
            gold_stones
                .frames_for_tag(tag)
                .unwrap_or_else(|| panic!("gold stones aseprite tag {tag} should exist"))
        } else {
            gold_resource.frames.as_slice()
        };
        let frames = source_frames
            .iter()
            .enumerate()
            .map(|(frame_index, frame)| {
                (
                    ImageAsset::from_rgba_cropped(
                        GOLD_PROP_TEXTURE_BASE
                            + (index as u32 * GOLD_PROP_TEXTURE_STRIDE)
                            + frame_index as u32,
                        frame.width,
                        frame.height,
                        frame.rgba.clone(),
                    ),
                    frame.duration_ms.unwrap_or(120).max(1),
                )
            })
            .filter(|(image, _)| image.rgba.chunks_exact(4).any(|pixel| pixel[3] != 0))
            .collect::<Vec<_>>();
        assert!(
            !frames.is_empty(),
            "gold prop {} should have at least one visible frame",
            kind.tag()
        );
        let durations_ms = frames
            .iter()
            .map(|(_, duration)| *duration)
            .collect::<Vec<_>>();
        let total_duration_ms = durations_ms.iter().sum::<u32>().max(1);
        SpriteAnimation {
            frames: frames.into_iter().map(|(image, _)| image).collect(),
            durations_ms,
            total_duration_ms,
        }
    })
}

fn load_rock_prop_assets() -> [ImageAsset; ROCK_PROP_COUNT] {
    std::array::from_fn(|index| {
        let kind = RockKind::ALL[index];
        ImageAsset::from_png_bytes_cropped(ROCK_PROP_TEXTURE_BASE + index as u32, kind.bytes())
    })
}

fn load_icon_viewer_icons() -> Vec<IconTile> {
    let mut icons = UI_ICON_BYTES
        .iter()
        .enumerate()
        .map(|(index, &bytes)| IconTile {
            label: format!("Icon {:02}", index + 1),
            image: ImageAsset::from_png_bytes_cropped(
                ICON_VIEWER_TEXTURE_BASE + index as u32,
                bytes,
            ),
        })
        .collect::<Vec<_>>();

    let tool_texture_base = ICON_VIEWER_TEXTURE_BASE + UI_ICON_BYTES.len() as u32;
    icons.extend([
        IconTile {
            label: "Tool 01".to_string(),
            image: ImageAsset::from_png_bytes_cropped(tool_texture_base, TOOL1_BYTES),
        },
        IconTile {
            label: "Tool 02".to_string(),
            image: ImageAsset::from_png_bytes_cropped(tool_texture_base + 1, TOOL2_BYTES),
        },
        IconTile {
            label: "Tool 03".to_string(),
            image: ImageAsset::from_png_bytes_cropped(tool_texture_base + 2, TOOL3_BYTES),
        },
        IconTile {
            label: "Tool 04".to_string(),
            image: ImageAsset::from_png_bytes_cropped(tool_texture_base + 3, TOOL4_BYTES),
        },
        IconTile {
            label: "Meat".to_string(),
            image: ImageAsset::from_png_bytes_cropped(tool_texture_base + 4, MEAT_RESOURCE_BYTES),
        },
        IconTile {
            label: "Wood".to_string(),
            image: ImageAsset::from_png_bytes_cropped(tool_texture_base + 5, WOOD_RESOURCE_BYTES),
        },
    ]);

    if let Some(image) = load_archer_arrow_icon(tool_texture_base + 6) {
        icons.push(IconTile {
            label: "Arrow".to_string(),
            image,
        });
    }

    icons
}

fn load_archer_arrow_icon(texture_id: u32) -> Option<ImageAsset> {
    let set = ase_assets::load_tinted_aseprite_set(
        ARCHER_ASEPRITE_PATH,
        [255, 255, 255, 255],
        ase_assets::TintMode::Multiply,
    )
    .ok()?;

    for frame in set.frames_for_tag("Arrow")? {
        let asset = ImageAsset::from_rgba_cropped(
            texture_id,
            frame.width,
            frame.height,
            frame.rgba.clone(),
        );
        if asset.rgba.chunks_exact(4).any(|pixel| pixel[3] != 0) {
            return Some(asset);
        }
    }

    None
}

fn generate_clouds(
    seed: u64,
    assets: &[ImageAsset],
    world_cols: usize,
    world_rows: usize,
) -> Vec<CloudInstance> {
    if assets.is_empty() {
        return Vec::new();
    }

    let mut rng = SeededRng::new(seed ^ 0xC10D_5EED_A51E_2026);
    let count = rng.range_usize(MIN_CLOUDS, MAX_CLOUDS + 1);
    let world_w = world_cols as f32 * TILE_SIZE;
    let world_h = world_rows as f32 * TILE_SIZE;

    (0..count)
        .map(|_| CloudInstance {
            asset_index: rng.range_usize(0, assets.len()),
            x: rng.range_f32(0.0, world_w.max(1.0)),
            y: rng.range_f32(0.0, world_h.max(1.0)),
            scale: rng.range_f32(0.85, 1.45),
            scale_wobble: rng.range_f32(0.015, 0.055),
            alpha_min: rng.range_f32(0.25, 0.45),
            alpha_max: rng.range_f32(0.55, 0.75),
            drift_x: rng.range_f32(1.8, 6.4),
            drift_y: rng.range_f32(-0.8, 0.8),
            phase: rng.range_f32(0.0, std::f32::consts::TAU),
        })
        .collect()
}

#[cfg(test)]
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

impl FrameProducer for UnitWalkViewer {
    fn resize(&mut self, width: u32, height: u32) {
        self.resize_view(width, height);
    }

    fn build_frame(&mut self, adapter: &mut Adapter) {
        self.upload_assets(adapter);

        let _ = adapter.begin_frame(0x243C40);
        let _ = adapter.set_sampler_raw(0, 0, 0, 0);
        let _ = adapter.set_blend_raw(1, 0x0302, 0x0303);

        let viewer_w = self.window_width as f32;
        let viewer_h = self.window_height as f32;
        let mut title = ts_ui::UiBatch::new(self.window_width, self.window_height);
        title.big_ribbon(
            0,
            22.0,
            18.0,
            viewer_w - 44.0,
            48.0,
            Rgba8::new(255, 255, 255, 235),
        );
        title.text("UNIT WALK", 44.0, 35.0, 2.0, Rgba8::new(32, 56, 60, 255));
        let _ =
            adapter.draw_tex_triangles_no_present(ts_ui::BIG_RIBBONS_TEXTURE, &title.texture_bytes);
        let _ = adapter.draw_rgb_triangles_no_present(&title.solid_bytes);

        let elapsed_ms = self.started_at.elapsed().as_millis() as u32;
        let flip_x = unit_viewer_flip_x(elapsed_ms);
        let grid_x = 22.0;
        let grid_y = 84.0;
        let grid_w = (viewer_w - 44.0).max(1.0);
        let grid_h = (viewer_h - 156.0).max(1.0);
        let count = self.units.len().max(1);
        let cols = ((count as f32 * grid_w / grid_h).sqrt().ceil() as usize)
            .max(1)
            .min(count);
        let rows = count.div_ceil(cols).max(1);
        let cell_w = grid_w / cols as f32;
        let cell_h = grid_h / rows as f32;
        let label_scale = 1.0;
        let label_h = 22.0;

        for (index, unit) in self.units.iter().enumerate() {
            let frame = unit_walk_frame(unit, elapsed_ms);
            let col = index % cols;
            let row = index / cols;
            let image_x = grid_x + col as f32 * cell_w;
            let image_y = grid_y + row as f32 * cell_h;
            let image_h = (cell_h - label_h).max(1.0);
            let scale = 1.0;
            let w = frame.width as f32 * scale;
            let h = frame.height as f32 * scale;
            let x = image_x + (cell_w - w) * 0.5 + unit.offset_x;
            let y = image_y + (image_h - h) * 0.5 + unit.offset_y;

            let mut image = SpriteBatch::new(self.window_width, self.window_height);
            let uv = if flip_x {
                [1.0, 0.0, 0.0, 1.0]
            } else {
                [0.0, 0.0, 1.0, 1.0]
            };
            image.image_uv(
                x.floor(),
                y.floor(),
                w.floor().max(1.0),
                h.floor().max(1.0),
                uv,
                Rgba8::WHITE,
            );
            let _ = adapter.draw_tex_triangles_no_present(frame.texture_id, &image.bytes);

            let mut label = ts_ui::UiBatch::new(self.window_width, self.window_height);
            let label_text = unit_viewer_label(unit);
            let label_w = ui_text_width(&label_text, label_scale);
            label.text(
                &label_text,
                image_x + (cell_w - label_w) * 0.5,
                image_y + image_h + 2.0,
                label_scale,
                Rgba8::new(220, 238, 232, 255),
            );
            let _ = adapter.draw_rgb_triangles_no_present(&label.solid_bytes);
        }

        let mut caption = ts_ui::UiBatch::new(self.window_width, self.window_height);
        caption.banner_panel(
            22.0,
            viewer_h - 58.0,
            viewer_w - 44.0,
            34.0,
            12.0,
            Rgba8::new(255, 255, 255, 220),
        );
        caption.text(
            &format!("ANIMS {} CLIPS", self.units.len()),
            44.0,
            viewer_h - 46.0,
            2.0,
            Rgba8::new(32, 56, 60, 255),
        );
        let _ =
            adapter.draw_tex_triangles_no_present(ts_ui::BANNER_TEXTURE, &caption.texture_bytes);
        let _ = adapter.draw_rgb_triangles_no_present(&caption.solid_bytes);

        let _ = adapter.end_frame();
    }
}

impl FrameProducer for LoadScreen {
    fn cursor_visible(&self) -> bool {
        false
    }

    fn window_decorations(&self) -> bool {
        false
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.resize_view(width, height);
    }

    fn build_frame(&mut self, adapter: &mut Adapter) {
        self.upload_assets(adapter);

        let _ = adapter.begin_frame(WATER_BG);
        let _ = adapter.set_sampler_raw(0, 0, 0, 0);
        let _ = adapter.set_blend_raw(1, 0x0302, 0x0303);

        let mut tables = SpriteBatch::new(self.window_width, self.window_height);
        for table in loadscreen_table_layout(self.window_width as f32, self.window_height as f32) {
            draw_wood_table(
                &mut tables,
                &self.wood_table,
                table.x,
                table.y,
                table.w,
                table.h,
            );
        }
        let _ = adapter.draw_tex_triangles_no_present(self.wood_table.texture_id, &tables.bytes);
        let _ = adapter.end_frame();
    }
}

impl FrameProducer for IconViewer {
    fn resize(&mut self, width: u32, height: u32) {
        self.resize_view(width, height);
    }

    fn build_frame(&mut self, adapter: &mut Adapter) {
        self.upload_assets(adapter);

        let _ = adapter.begin_frame(0x243C40);
        let _ = adapter.set_sampler_raw(0, 0, 0, 0);
        let _ = adapter.set_blend_raw(1, 0x0302, 0x0303);

        let viewer_w = self.window_width as f32;
        let viewer_h = self.window_height as f32;
        let grid_x = 24.0;
        let grid_y = 18.0;
        let cell_w = 50.0;
        let cell_h = 58.0;
        let icon_box = 32.0;
        let cols = ((viewer_w - grid_x * 2.0) / cell_w).floor().max(1.0) as usize;
        for (index, icon) in self.icons.iter().enumerate() {
            let col = index % cols;
            let row = index / cols;
            let cell_x = grid_x + col as f32 * cell_w;
            let cell_y = grid_y + row as f32 * cell_h;
            if cell_y > viewer_h - 34.0 {
                break;
            }
            let scale = (icon_box / icon.image.width as f32)
                .min(icon_box / icon.image.height as f32)
                .min(1.0);
            let w = icon.image.width as f32 * scale;
            let h = icon.image.height as f32 * scale;
            let mut sprite = SpriteBatch::new(self.window_width, self.window_height);
            sprite.image(
                (cell_x + (icon_box - w) * 0.5).floor(),
                (cell_y + 2.0 + (icon_box - h) * 0.5).floor(),
                w.max(1.0),
                h.max(1.0),
                Rgba8::WHITE,
            );
            let _ = adapter.draw_tex_triangles_no_present(icon.image.texture_id, &sprite.bytes);

            let mut label = ts_ui::UiBatch::new(self.window_width, self.window_height);
            let label_w = ui_text_width(&icon.label.to_uppercase(), 1.0);
            label.text(
                &icon.label.to_uppercase(),
                cell_x + (icon_box - label_w) * 0.5,
                cell_y + 40.0,
                1.0,
                Rgba8::new(220, 238, 232, 255),
            );
            let _ = adapter.draw_rgb_triangles_no_present(&label.solid_bytes);
        }

        let _ = adapter.end_frame();
    }
}

impl FrameProducer for EventEditor {
    fn resize(&mut self, width: u32, height: u32) {
        self.resize_view(width, height);
    }

    fn handle_input(&mut self, event: InputEvent) {
        match event {
            InputEvent::CursorMoved { x, y } => {
                self.mouse = Point { x, y };
            }
            InputEvent::MouseButton {
                button: InputMouseButton::Left,
                state: InputButtonState::Pressed,
            } => self.handle_left_click(),
            _ => {}
        }
    }

    fn build_frame(&mut self, adapter: &mut Adapter) {
        let _ = adapter.begin_frame(0x243C40);

        let w = self.window_width as f32;
        let h = self.window_height as f32;
        let mut solid = SolidBatch::new(self.window_width, self.window_height);
        let mut text = ts_ui::UiBatch::new(self.window_width, self.window_height);

        solid.rect(14.0, 14.0, w - 28.0, 50.0, Rgba8::new(86, 120, 119, 255));
        text.text(
            "EVENT EDITOR",
            28.0,
            29.0,
            2.0,
            Rgba8::new(232, 245, 239, 255),
        );

        event_editor_selector(
            &mut solid,
            &mut text,
            self.mouse,
            "TRIGGER",
            self.draft_trigger.label(),
            24.0,
            78.0,
            218.0,
        );
        event_editor_selector(
            &mut solid,
            &mut text,
            self.mouse,
            "CONDITION",
            self.draft_condition.label(),
            258.0,
            78.0,
            218.0,
        );
        event_editor_selector(
            &mut solid,
            &mut text,
            self.mouse,
            "ACTION",
            self.draft_action.label(),
            492.0,
            78.0,
            218.0,
        );

        event_editor_button(
            &mut solid, &mut text, self.mouse, "ADD RULE", 24.0, 142.0, 130.0, 34.0,
        );

        solid.rect(24.0, 196.0, w - 48.0, 2.0, Rgba8::new(113, 151, 148, 255));
        text.text("EN", 42.0, 202.0, 1.0, Rgba8::new(183, 205, 197, 255));
        text.text("TRIGGER", 104.0, 202.0, 1.0, Rgba8::new(183, 205, 197, 255));
        text.text(
            "CONDITION",
            286.0,
            202.0,
            1.0,
            Rgba8::new(183, 205, 197, 255),
        );
        text.text("ACTION", 482.0, 202.0, 1.0, Rgba8::new(183, 205, 197, 255));

        let table_y = 214.0;
        let row_h = 42.0;
        for (index, rule) in self.rules.iter().enumerate() {
            let y = table_y + index as f32 * row_h;
            if y > h - row_h {
                break;
            }
            let row_color = if index % 2 == 0 {
                Rgba8::new(72, 103, 103, 190)
            } else {
                Rgba8::new(62, 92, 93, 190)
            };
            solid.rect(24.0, y, w - 48.0, row_h - 4.0, row_color);
            event_editor_button(
                &mut solid,
                &mut text,
                self.mouse,
                if rule.enabled { "ON" } else { "OFF" },
                28.0,
                y + 7.0,
                58.0,
                28.0,
            );
            text.text(
                rule.trigger.label(),
                104.0,
                y + 13.0,
                1.0,
                Rgba8::new(232, 245, 239, 255),
            );
            text.text(
                rule.condition.label(),
                286.0,
                y + 13.0,
                1.0,
                Rgba8::new(232, 245, 239, 255),
            );
            text.text(
                rule.action.label(),
                482.0,
                y + 13.0,
                1.0,
                Rgba8::new(232, 245, 239, 255),
            );
            event_editor_button(
                &mut solid,
                &mut text,
                self.mouse,
                "DEL",
                w - 86.0,
                y + 7.0,
                58.0,
                28.0,
            );
        }

        let _ = adapter.draw_rgb_triangles_no_present(&solid.bytes);
        let _ = adapter.draw_rgb_triangles_no_present(&text.solid_bytes);
        let _ = adapter.end_frame();
    }
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
struct TileWorld {
    cols: usize,
    rows: usize,
    backgrounds: Vec<BackgroundTile>,
    water_states: Vec<WaterState>,
    foregrounds: Vec<Option<AtlasTile>>,
    buildings: Vec<PlacedBuilding>,
    props: Vec<PlacedProp>,
    fog: Vec<bool>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
struct PlacedBuilding {
    kind: BuildingKind,
    x2: isize,
    y2: isize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
struct PlacedProp {
    kind: PropKind,
    x2: usize,
    y2: usize,
}

impl TileWorld {
    fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols,
            rows,
            backgrounds: vec![BackgroundTile::Grass; cols * rows],
            water_states: vec![WaterState::Nothing; cols * rows],
            foregrounds: vec![None; cols * rows],
            buildings: Vec::new(),
            props: Vec::new(),
            fog: vec![false; cols * rows],
        }
    }

    fn add_edge(&mut self, edge: WorldEdge) {
        match edge {
            WorldEdge::Top => {
                self.backgrounds
                    .splice(0..0, vec![BackgroundTile::Grass; self.cols]);
                self.water_states
                    .splice(0..0, vec![WaterState::Nothing; self.cols]);
                self.foregrounds.splice(0..0, vec![None; self.cols]);
                self.fog.splice(0..0, vec![false; self.cols]);
                self.rows += 1;
                self.shift_objects(0, BUILDING_GRID_DIVISIONS as isize);
            }
            WorldEdge::Bottom => {
                self.backgrounds
                    .extend(std::iter::repeat_n(BackgroundTile::Grass, self.cols));
                self.water_states
                    .extend(std::iter::repeat_n(WaterState::Nothing, self.cols));
                self.foregrounds
                    .extend(std::iter::repeat_n(None, self.cols));
                self.fog.extend(std::iter::repeat_n(false, self.cols));
                self.rows += 1;
            }
            WorldEdge::Left => {
                self.insert_column(0);
                self.shift_objects(BUILDING_GRID_DIVISIONS as isize, 0);
            }
            WorldEdge::Right => self.insert_column(self.cols),
        }
    }

    fn remove_edge(&mut self, edge: WorldEdge) -> bool {
        match edge {
            WorldEdge::Top if self.rows > 1 => {
                let removed = (
                    0,
                    0,
                    self.cols * BUILDING_GRID_DIVISIONS,
                    BUILDING_GRID_DIVISIONS,
                );
                self.remove_objects_overlapping(removed);
                self.drop_props_before_shift(|prop| prop.y2 >= BUILDING_GRID_DIVISIONS);
                self.backgrounds.drain(0..self.cols);
                self.water_states.drain(0..self.cols);
                self.foregrounds.drain(0..self.cols);
                self.fog.drain(0..self.cols);
                self.rows -= 1;
                self.shift_objects(0, -(BUILDING_GRID_DIVISIONS as isize));
                true
            }
            WorldEdge::Bottom if self.rows > 1 => {
                let removed = (
                    0,
                    ((self.rows - 1) * BUILDING_GRID_DIVISIONS) as isize,
                    self.cols * BUILDING_GRID_DIVISIONS,
                    BUILDING_GRID_DIVISIONS,
                );
                self.remove_objects_overlapping(removed);
                let start = (self.rows - 1) * self.cols;
                self.backgrounds.drain(start..start + self.cols);
                self.water_states.drain(start..start + self.cols);
                self.foregrounds.drain(start..start + self.cols);
                self.fog.drain(start..start + self.cols);
                self.rows -= 1;
                true
            }
            WorldEdge::Left if self.cols > 1 => {
                let removed = (
                    0,
                    0,
                    BUILDING_GRID_DIVISIONS,
                    self.rows * BUILDING_GRID_DIVISIONS,
                );
                self.remove_objects_overlapping(removed);
                self.drop_props_before_shift(|prop| prop.x2 >= BUILDING_GRID_DIVISIONS);
                self.remove_column(0);
                self.shift_objects(-(BUILDING_GRID_DIVISIONS as isize), 0);
                true
            }
            WorldEdge::Right if self.cols > 1 => {
                let removed = (
                    ((self.cols - 1) * BUILDING_GRID_DIVISIONS) as isize,
                    0,
                    BUILDING_GRID_DIVISIONS,
                    self.rows * BUILDING_GRID_DIVISIONS,
                );
                self.remove_objects_overlapping(removed);
                self.remove_column(self.cols - 1);
                true
            }
            _ => false,
        }
    }

    fn insert_column(&mut self, at: usize) {
        let old_cols = self.cols;
        let rows = self.rows;
        self.backgrounds =
            insert_layer_column(&self.backgrounds, old_cols, rows, at, BackgroundTile::Grass);
        self.water_states =
            insert_layer_column(&self.water_states, old_cols, rows, at, WaterState::Nothing);
        self.foregrounds = insert_layer_column(&self.foregrounds, old_cols, rows, at, None);
        self.fog = insert_layer_column(&self.fog, old_cols, rows, at, false);
        self.cols += 1;
    }

    fn remove_column(&mut self, at: usize) {
        let old_cols = self.cols;
        let rows = self.rows;
        self.backgrounds = remove_layer_column(&self.backgrounds, old_cols, rows, at);
        self.water_states = remove_layer_column(&self.water_states, old_cols, rows, at);
        self.foregrounds = remove_layer_column(&self.foregrounds, old_cols, rows, at);
        self.fog = remove_layer_column(&self.fog, old_cols, rows, at);
        self.cols -= 1;
    }

    fn shift_objects(&mut self, dx2: isize, dy2: isize) {
        for building in &mut self.buildings {
            building.x2 += dx2;
            building.y2 += dy2;
        }

        for prop in &mut self.props {
            prop.x2 = (prop.x2 as isize + dx2) as usize;
            prop.y2 = (prop.y2 as isize + dy2) as usize;
        }
        self.sort_props();
    }

    fn remove_objects_overlapping(&mut self, rect2: (isize, isize, usize, usize)) {
        self.buildings
            .retain(|building| !rects_overlap(rect2, building_footprint_rect2(*building)));
        self.props
            .retain(|prop| !rects_overlap(rect2, prop_footprint_rect2(*prop)));
    }

    fn drop_props_before_shift(&mut self, keep: impl Fn(&PlacedProp) -> bool) {
        self.props.retain(keep);
    }

    fn sort_props(&mut self) {
        self.props.sort_by_key(|prop| (prop.y2, prop.x2));
    }

    fn save_to_path(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|error| format!("world json encode failed: {error}"))?;
        fs::write(path.as_ref(), json).map_err(|error| format!("write failed: {error}"))
    }

    fn load_from_path(path: impl AsRef<Path>) -> Result<Self, String> {
        let json =
            fs::read_to_string(path.as_ref()).map_err(|error| format!("read failed: {error}"))?;
        let mut world = serde_json::from_str::<Self>(&json)
            .map_err(|error| format!("world json decode failed: {error}"))?;
        world.validate()?;
        world.normalize_loaded_state();
        Ok(world)
    }

    fn validate(&self) -> Result<(), String> {
        if self.cols == 0 || self.rows == 0 {
            return Err("world dimensions must be non-zero".to_string());
        }
        let cells = self
            .cols
            .checked_mul(self.rows)
            .ok_or_else(|| "world dimensions overflow".to_string())?;
        for (name, len) in [
            ("backgrounds", self.backgrounds.len()),
            ("water_states", self.water_states.len()),
            ("foregrounds", self.foregrounds.len()),
            ("fog", self.fog.len()),
        ] {
            if len != cells {
                return Err(format!("{name} length {len} does not match {cells} cells"));
            }
        }
        Ok(())
    }

    fn normalize_loaded_state(&mut self) {
        for state in &mut self.water_states {
            if *state == WaterState::Animation {
                *state = WaterState::Nothing;
            }
        }
        self.sort_props();
    }

    fn background(&self, col: usize, row: usize) -> BackgroundTile {
        self.backgrounds[self.index(col, row)]
    }

    fn set_background(&mut self, col: usize, row: usize, background: BackgroundTile) {
        let index = self.index(col, row);
        self.backgrounds[index] = background;
        if background != BackgroundTile::Water || self.foregrounds[index].is_some() {
            self.water_states[index] = WaterState::Nothing;
        }
    }

    #[allow(dead_code)]
    fn water_state(&self, col: usize, row: usize) -> Option<WaterState> {
        self.cell_accepts_water_state(col, row)
            .then_some(self.water_states[self.index(col, row)])
    }

    #[allow(dead_code)]
    fn set_water_state(&mut self, col: usize, row: usize, state: WaterState) {
        if self.cell_accepts_water_state(col, row) {
            let index = self.index(col, row);
            self.water_states[index] = state;
        }
    }

    #[allow(dead_code)]
    fn cell_accepts_water_state(&self, col: usize, row: usize) -> bool {
        self.background(col, row) == BackgroundTile::Water && self.foreground(col, row).is_none()
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
            Brush::Background(background) => {
                if background == BackgroundTile::Water && self.cell_accepts_water_state(col, row) {
                    self.water_states[index] = self.water_states[index].next();
                } else {
                    self.set_background(col, row, background);
                }
            }
            Brush::Foreground(tile) => {
                if is_shoreline_tile(tile) {
                    self.backgrounds[index] = BackgroundTile::Water;
                }
                self.water_states[index] = WaterState::Nothing;
                self.foregrounds[index] = Some(tile);
            }
            Brush::Ramp(ramp) => {
                if self.can_place_ramp(col, row) {
                    self.foregrounds[index] = Some(ramp.top);
                    let bottom_index = self.index(col, row + 1);
                    self.foregrounds[bottom_index] = Some(ramp.bottom);
                }
            }
            Brush::Prop(kind) => self.paint_prop(kind, col, row),
            Brush::GoldResource => self.cycle_gold(col, row),
            Brush::RockResource => self.cycle_rock(col, row),
            Brush::Building(kind) => {
                self.paint_building(
                    kind,
                    (col * BUILDING_GRID_DIVISIONS) as isize,
                    (row * BUILDING_GRID_DIVISIONS) as isize,
                );
            }
            Brush::FogRect => {}
            Brush::ClearForeground => self.clear_foreground(col, row),
        }
    }

    fn clear_foreground(&mut self, col: usize, row: usize) {
        let index = self.index(col, row);
        self.foregrounds[index] = None;
        let tile_rect = (
            (col * BUILDING_GRID_DIVISIONS) as isize,
            (row * BUILDING_GRID_DIVISIONS) as isize,
            BUILDING_GRID_DIVISIONS,
            BUILDING_GRID_DIVISIONS,
        );
        self.buildings
            .retain(|building| !rects_overlap(tile_rect, building_footprint_rect2(*building)));
        self.props
            .retain(|prop| !rects_overlap(tile_rect, prop_footprint_rect2(*prop)));
    }

    fn paint_building(&mut self, kind: BuildingKind, x2: isize, y2: isize) {
        if self.can_place_building_kind(kind, x2, y2) {
            self.buildings.push(PlacedBuilding { kind, x2, y2 });
        }
    }

    fn can_place_building_kind(&self, kind: BuildingKind, x2: isize, y2: isize) -> bool {
        self.can_place_building(building_spec(kind), x2, y2)
    }

    fn paint_prop(&mut self, kind: PropKind, col: usize, row: usize) {
        self.paint_prop_half(
            kind,
            col * BUILDING_GRID_DIVISIONS,
            row * BUILDING_GRID_DIVISIONS,
        );
    }

    fn paint_prop_half(&mut self, kind: PropKind, x2: usize, y2: usize) {
        if self.can_place_prop_half(kind, x2, y2) {
            self.props.push(PlacedProp { kind, x2, y2 });
            self.sort_props();
        }
    }

    fn cycle_gold(&mut self, col: usize, row: usize) {
        let x2 = col * BUILDING_GRID_DIVISIONS;
        let y2 = row * BUILDING_GRID_DIVISIONS;
        if let Some(index) = self.props.iter().position(|prop| {
            prop.x2 == x2 && prop.y2 == y2 && matches!(prop.kind, PropKind::Gold(_))
        }) {
            let PropKind::Gold(kind) = self.props[index].kind else {
                return;
            };
            if let Some(next) = kind.next() {
                self.props[index].kind = PropKind::Gold(next);
            } else {
                self.props.remove(index);
            }
            return;
        }

        self.paint_prop_half(PropKind::Gold(GoldKind::Stone1), x2, y2);
    }

    fn can_cycle_gold(&self, col: usize, row: usize) -> bool {
        let x2 = col * BUILDING_GRID_DIVISIONS;
        let y2 = row * BUILDING_GRID_DIVISIONS;
        self.props
            .iter()
            .any(|prop| prop.x2 == x2 && prop.y2 == y2 && matches!(prop.kind, PropKind::Gold(_)))
            || self.can_place_prop_half(PropKind::Gold(GoldKind::Stone1), x2, y2)
    }

    fn cycle_rock(&mut self, col: usize, row: usize) {
        let x2 = col * BUILDING_GRID_DIVISIONS;
        let y2 = row * BUILDING_GRID_DIVISIONS;
        if let Some(index) = self.props.iter().position(|prop| {
            prop.x2 == x2 && prop.y2 == y2 && matches!(prop.kind, PropKind::Rock(_))
        }) {
            let PropKind::Rock(kind) = self.props[index].kind else {
                return;
            };
            if let Some(next) = kind.next() {
                self.props[index].kind = PropKind::Rock(next);
            } else {
                self.props.remove(index);
            }
            return;
        }

        self.paint_prop_half(PropKind::Rock(RockKind::Rock1), x2, y2);
    }

    fn can_cycle_rock(&self, col: usize, row: usize) -> bool {
        let x2 = col * BUILDING_GRID_DIVISIONS;
        let y2 = row * BUILDING_GRID_DIVISIONS;
        self.props
            .iter()
            .any(|prop| prop.x2 == x2 && prop.y2 == y2 && matches!(prop.kind, PropKind::Rock(_)))
            || self.can_place_prop_half(PropKind::Rock(RockKind::Rock1), x2, y2)
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

    fn can_place_ramp(&self, col: usize, row: usize) -> bool {
        row + 1 < self.rows
            && self.background(col, row) == BackgroundTile::Grass
            && self.background(col, row + 1) == BackgroundTile::Grass
            && self.foreground(col, row).is_none()
            && self.foreground(col, row + 1).is_none()
            && !self.props.iter().any(|prop| {
                rects_overlap(prop_footprint_rect2(*prop), ramp_footprint_rect2(col, row))
            })
    }

    fn can_place_building(&self, spec: BuildingSpec, x2: isize, y2: isize) -> bool {
        let footprint = building_spec_footprint_rect2(spec, x2, y2);
        if footprint.0 < 0
            || footprint.1 < 0
            || footprint.0 + footprint.2 as isize > (self.cols * BUILDING_GRID_DIVISIONS) as isize
            || footprint.1 + footprint.3 as isize > (self.rows * BUILDING_GRID_DIVISIONS) as isize
        {
            return false;
        }

        if self
            .buildings
            .iter()
            .any(|building| rects_overlap(footprint, building_footprint_rect2(*building)))
        {
            return false;
        }

        if self
            .props
            .iter()
            .any(|prop| rects_overlap(footprint, prop_footprint_rect2(*prop)))
        {
            return false;
        }

        let foundation = half_rect_to_tile_rect(footprint);
        if foundation.0 + foundation.2 > self.cols || foundation.1 + foundation.3 > self.rows {
            return false;
        }

        for foundation_row in foundation.1..foundation.1 + foundation.3 {
            for foundation_col in foundation.0..foundation.0 + foundation.2 {
                if self.background(foundation_col, foundation_row) != BackgroundTile::Grass
                    || !self
                        .foreground(foundation_col, foundation_row)
                        .is_none_or(is_shoreline_tile)
                {
                    return false;
                }
            }
        }

        true
    }

    fn can_place_prop(&self, kind: PropKind, col: usize, row: usize) -> bool {
        self.can_place_prop_half(
            kind,
            col * BUILDING_GRID_DIVISIONS,
            row * BUILDING_GRID_DIVISIONS,
        )
    }

    fn can_place_prop_half(&self, kind: PropKind, x2: usize, y2: usize) -> bool {
        let footprint = prop_kind_footprint_rect2(kind, x2, y2);
        if footprint.0 < 0
            || footprint.1 < 0
            || footprint.0 + footprint.2 as isize > (self.cols * BUILDING_GRID_DIVISIONS) as isize
            || footprint.1 + footprint.3 as isize > (self.rows * BUILDING_GRID_DIVISIONS) as isize
        {
            return false;
        }

        if matches!(
            kind,
            PropKind::Plant(_) | PropKind::Gold(_) | PropKind::Rock(_)
        ) {
            let foundation = half_rect_to_tile_rect(footprint);
            for row in foundation.1..foundation.1 + foundation.3 {
                for col in foundation.0..foundation.0 + foundation.2 {
                    if self.background(col, row) != BackgroundTile::Grass
                        || !self.foreground(col, row).is_none_or(is_shoreline_tile)
                    {
                        return false;
                    }
                }
            }
        }

        if matches!(
            kind,
            PropKind::Pillar(_) | PropKind::Gold(_) | PropKind::Rock(_)
        ) && (x2 % BUILDING_GRID_DIVISIONS != 0 || y2 % BUILDING_GRID_DIVISIONS != 0)
        {
            return false;
        }

        if self
            .props
            .iter()
            .any(|prop| rects_overlap(footprint, prop_footprint_rect2(*prop)))
        {
            return false;
        }

        !self
            .buildings
            .iter()
            .any(|building| rects_overlap(footprint, building_footprint_rect2(*building)))
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
                if self.backgrounds[index] != BackgroundTile::Water
                    || self.foregrounds[index].is_some()
                {
                    self.water_states[index] = WaterState::Nothing;
                }
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
    fn from_rgba_cropped(texture_id: u32, width: u32, height: u32, rgba: Vec<u8>) -> Self {
        assert_eq!(
            rgba.len(),
            (width * height * 4) as usize,
            "rgba asset dimensions should match pixel data"
        );

        let mut min_x = width;
        let mut min_y = height;
        let mut max_x = 0;
        let mut max_y = 0;
        let mut found = false;

        for y in 0..height {
            for x in 0..width {
                let alpha = rgba[((y * width + x) * 4 + 3) as usize];
                if alpha != 0 {
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                    max_x = max_x.max(x);
                    max_y = max_y.max(y);
                    found = true;
                }
            }
        }

        if !found {
            return Self {
                texture_id,
                width,
                height,
                rgba,
            };
        }

        let crop_width = max_x - min_x + 1;
        let crop_height = max_y - min_y + 1;
        let mut cropped = Vec::with_capacity((crop_width * crop_height * 4) as usize);
        for y in min_y..=max_y {
            let start = ((y * width + min_x) * 4) as usize;
            let end = ((y * width + max_x + 1) * 4) as usize;
            cropped.extend_from_slice(&rgba[start..end]);
        }

        Self {
            texture_id,
            width: crop_width,
            height: crop_height,
            rgba: cropped,
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

#[cfg(test)]
fn prop_draw_order(props: &[PlacedProp]) -> Vec<PlacedProp> {
    let mut ordered = props.to_vec();
    ordered.sort_by_key(|prop| (prop.y2, prop.x2));
    ordered
}

fn terrain_cell_visible(
    cell: &TerrainDrawCell,
    start_col: usize,
    start_row: usize,
    end_col: usize,
    end_row: usize,
) -> bool {
    cell.col >= start_col && cell.col < end_col && cell.row >= start_row && cell.row < end_row
}

fn loadscreen_table_layout(width: f32, height: f32) -> [TableRect; 3] {
    let margin = (width.min(height) * 0.06).clamp(32.0, 72.0);
    let gap = (width * 0.04).clamp(28.0, 52.0);
    let available_w = (width - margin * 2.0).max(1.0);
    let available_h = (height - margin * 2.0).max(1.0);
    let small_w = ((available_w - gap) * 0.34)
        .clamp(300.0, 420.0)
        .min((available_w - gap) * 0.5);
    let large_w = (available_w - gap - small_w).max(256.0);
    let small_h = ((available_h - gap) * 0.5).max(256.0);

    [
        TableRect {
            x: margin,
            y: margin,
            w: large_w,
            h: available_h,
        },
        TableRect {
            x: margin + large_w + gap,
            y: margin,
            w: small_w,
            h: small_h,
        },
        TableRect {
            x: margin + large_w + gap,
            y: margin + small_h + gap,
            w: small_w,
            h: small_h,
        },
    ]
}

fn draw_wood_table(batch: &mut SpriteBatch, image: &ImageAsset, x: f32, y: f32, w: f32, h: f32) {
    let left_w = WOOD_TABLE_LEFT_EDGE.width as f32;
    let right_w = WOOD_TABLE_RIGHT_EDGE.width as f32;
    let top_h = WOOD_TABLE_TOP_EDGE.height as f32;
    let bottom_h = WOOD_TABLE_BOTTOM_EDGE.height as f32;
    let top_left_w = WOOD_TABLE_TOP_LEFT.width as f32;
    let top_right_w = WOOD_TABLE_TOP_RIGHT.width as f32;
    let bottom_left_w = WOOD_TABLE_BOTTOM_LEFT.width as f32;
    let bottom_right_w = WOOD_TABLE_BOTTOM_RIGHT.width as f32;
    let top_left_h = WOOD_TABLE_TOP_LEFT.height as f32;
    let bottom_left_h = WOOD_TABLE_BOTTOM_LEFT.height as f32;
    let top_left_join_w = top_left_w - WOOD_TABLE_TOP_LEFT_OUTSET_X;
    let top_right_join_w = top_right_w - WOOD_TABLE_TOP_RIGHT_OUTSET_X;
    let bottom_left_join_w = bottom_left_w - WOOD_TABLE_BOTTOM_LEFT_OUTSET_X;
    let bottom_right_join_w = bottom_right_w - WOOD_TABLE_BOTTOM_RIGHT_OUTSET_X;
    let top_join_h = top_left_h - WOOD_TABLE_TOP_CORNER_OUTSET_Y;

    draw_tiled_region(
        batch,
        image,
        WOOD_TABLE_FILL,
        x + left_w,
        y + top_h,
        (w - left_w - right_w).max(0.0),
        (h - top_h - bottom_h).max(0.0),
    );
    draw_tiled_region(
        batch,
        image,
        WOOD_TABLE_TOP_EDGE,
        x + top_left_join_w,
        y,
        (w - top_left_join_w - top_right_join_w).max(0.0),
        top_h,
    );
    draw_tiled_region(
        batch,
        image,
        WOOD_TABLE_BOTTOM_EDGE,
        x + bottom_left_join_w,
        y + h - bottom_h,
        (w - bottom_left_join_w - bottom_right_join_w).max(0.0),
        bottom_h,
    );
    draw_tiled_region(
        batch,
        image,
        WOOD_TABLE_LEFT_EDGE,
        x,
        y + top_join_h,
        left_w,
        (h - top_join_h - bottom_left_h).max(0.0),
    );
    draw_tiled_region(
        batch,
        image,
        WOOD_TABLE_RIGHT_EDGE,
        x + w - right_w,
        y + top_join_h,
        right_w,
        (h - top_join_h - bottom_left_h).max(0.0),
    );

    batch.image_region(
        image,
        WOOD_TABLE_TOP_LEFT,
        x - WOOD_TABLE_TOP_LEFT_OUTSET_X,
        y - WOOD_TABLE_TOP_CORNER_OUTSET_Y,
        WOOD_TABLE_TOP_LEFT.width as f32,
        WOOD_TABLE_TOP_LEFT.height as f32,
        Rgba8::WHITE,
    );
    batch.image_region(
        image,
        WOOD_TABLE_TOP_RIGHT,
        x + w - WOOD_TABLE_TOP_RIGHT.width as f32 + WOOD_TABLE_TOP_RIGHT_OUTSET_X,
        y - WOOD_TABLE_TOP_CORNER_OUTSET_Y,
        WOOD_TABLE_TOP_RIGHT.width as f32,
        WOOD_TABLE_TOP_RIGHT.height as f32,
        Rgba8::WHITE,
    );
    batch.image_region(
        image,
        WOOD_TABLE_BOTTOM_LEFT,
        x - WOOD_TABLE_BOTTOM_LEFT_OUTSET_X,
        y + h - WOOD_TABLE_BOTTOM_LEFT.height as f32,
        WOOD_TABLE_BOTTOM_LEFT.width as f32,
        WOOD_TABLE_BOTTOM_LEFT.height as f32,
        Rgba8::WHITE,
    );
    batch.image_region(
        image,
        WOOD_TABLE_BOTTOM_RIGHT,
        x + w - WOOD_TABLE_BOTTOM_RIGHT.width as f32 + WOOD_TABLE_BOTTOM_RIGHT_OUTSET_X,
        y + h - WOOD_TABLE_BOTTOM_RIGHT.height as f32,
        WOOD_TABLE_BOTTOM_RIGHT.width as f32,
        WOOD_TABLE_BOTTOM_RIGHT.height as f32,
        Rgba8::WHITE,
    );
}

fn draw_tiled_region(
    batch: &mut SpriteBatch,
    image: &ImageAsset,
    region: ImageRegion,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) {
    let tile_w = region.width as f32;
    let tile_h = region.height as f32;
    let mut dy = 0.0;
    while dy < h {
        let draw_h = (h - dy).min(tile_h);
        let mut dx = 0.0;
        while dx < w {
            let draw_w = (w - dx).min(tile_w);
            let source_w = ((region.width as f32) * (draw_w / tile_w))
                .ceil()
                .clamp(1.0, region.width as f32) as u32;
            let source_h = ((region.height as f32) * (draw_h / tile_h))
                .ceil()
                .clamp(1.0, region.height as f32) as u32;
            batch.image_region(
                image,
                ImageRegion::new(region.x, region.y, source_w, source_h),
                x + dx,
                y + dy,
                draw_w,
                draw_h,
                Rgba8::WHITE,
            );
            dx += draw_w;
        }
        dy += draw_h;
    }
}

fn scroll_strength(distance_from_edge: f32) -> f32 {
    ((EDGE_SCROLL_ZONE - distance_from_edge.max(0.0)) / EDGE_SCROLL_ZONE).clamp(0.0, 1.0)
}

fn is_ramp_part(tile: AtlasTile) -> bool {
    tile == RAMP_A.top || tile == RAMP_A.bottom || tile == RAMP_B.top || tile == RAMP_B.bottom
}

fn is_pillar_tile(tile: AtlasTile) -> bool {
    PILLAR_TILES.contains(&tile)
}

fn building_spec(kind: BuildingKind) -> BuildingSpec {
    BUILDING_SPECS[kind.index()]
}

fn building_spec_footprint_rect2(
    spec: BuildingSpec,
    x2: isize,
    y2: isize,
) -> (isize, isize, usize, usize) {
    (
        x2 + (spec.footprint_offset_cols * BUILDING_GRID_DIVISIONS) as isize,
        y2 + (spec.footprint_offset_rows * BUILDING_GRID_DIVISIONS) as isize,
        spec.footprint_cols * BUILDING_GRID_DIVISIONS,
        spec.footprint_rows * BUILDING_GRID_DIVISIONS,
    )
}

#[cfg(test)]
fn building_footprint_rect(building: PlacedBuilding) -> (usize, usize, usize, usize) {
    half_rect_to_tile_rect(building_footprint_rect2(building))
}

fn building_footprint_rect2(building: PlacedBuilding) -> (isize, isize, usize, usize) {
    building_spec_footprint_rect2(building_spec(building.kind), building.x2, building.y2)
}

fn brush_preview_footprint_rect2(
    brush: Brush,
    anchor_x2: isize,
    anchor_y2: isize,
    row: usize,
    world_rows: usize,
) -> (isize, isize, usize, usize) {
    match brush {
        Brush::Building(kind) => {
            building_spec_footprint_rect2(building_spec(kind), anchor_x2, anchor_y2)
        }
        Brush::Prop(kind) if anchor_x2 >= 0 && anchor_y2 >= 0 => {
            prop_kind_footprint_rect2(kind, anchor_x2 as usize, anchor_y2 as usize)
        }
        _ => {
            let (cols, rows) = brush.footprint();
            let (offset_cols, offset_rows) = brush.footprint_offset();
            let rows = if matches!(brush, Brush::Ramp(_)) && row + 1 >= world_rows {
                1
            } else {
                rows
            };
            (
                anchor_x2 + (offset_cols * BUILDING_GRID_DIVISIONS) as isize,
                anchor_y2 + (offset_rows * BUILDING_GRID_DIVISIONS) as isize,
                cols * BUILDING_GRID_DIVISIONS,
                rows * BUILDING_GRID_DIVISIONS,
            )
        }
    }
}

fn prop_footprint_rect2(prop: PlacedProp) -> (isize, isize, usize, usize) {
    prop_kind_footprint_rect2(prop.kind, prop.x2, prop.y2)
}

fn prop_kind_footprint_rect2(kind: PropKind, x2: usize, y2: usize) -> (isize, isize, usize, usize) {
    match kind {
        PropKind::Plant(plant) if plant.uses_half_height_footprint() => {
            (x2 as isize, y2 as isize + 1, BUILDING_GRID_DIVISIONS, 1)
        }
        PropKind::Plant(_) | PropKind::Pillar(_) | PropKind::Gold(_) | PropKind::Rock(_) => (
            x2 as isize,
            y2 as isize,
            BUILDING_GRID_DIVISIONS,
            BUILDING_GRID_DIVISIONS,
        ),
    }
}

fn ramp_footprint_rect2(col: usize, row: usize) -> (isize, isize, usize, usize) {
    (
        (col * BUILDING_GRID_DIVISIONS) as isize,
        (row * BUILDING_GRID_DIVISIONS) as isize,
        BUILDING_GRID_DIVISIONS,
        BUILDING_GRID_DIVISIONS * 2,
    )
}

fn half_rect_to_tile_rect(rect: (isize, isize, usize, usize)) -> (usize, usize, usize, usize) {
    let (x2, y2, w2, h2) = rect;
    assert!(x2 >= 0 && y2 >= 0, "tile rect should be inside the world");
    let x2 = x2 as usize;
    let y2 = y2 as usize;
    let start_col = x2 / BUILDING_GRID_DIVISIONS;
    let start_row = y2 / BUILDING_GRID_DIVISIONS;
    let end_col = (x2 + w2).div_ceil(BUILDING_GRID_DIVISIONS);
    let end_row = (y2 + h2).div_ceil(BUILDING_GRID_DIVISIONS);
    (
        start_col,
        start_row,
        end_col - start_col,
        end_row - start_row,
    )
}

fn half_grid_to_px(value: usize) -> f32 {
    value as f32 * TILE_SIZE / BUILDING_GRID_DIVISIONS as f32
}

fn signed_half_grid_to_px(value: isize) -> f32 {
    value as f32 * TILE_SIZE / BUILDING_GRID_DIVISIONS as f32
}

fn rects_overlap(a: (isize, isize, usize, usize), b: (isize, isize, usize, usize)) -> bool {
    let (ax, ay, aw, ah) = a;
    let (bx, by, bw, bh) = b;
    ax < bx + bw as isize && ax + aw as isize > bx && ay < by + bh as isize && ay + ah as isize > by
}

fn insert_layer_column<T: Copy>(
    layer: &[T],
    old_cols: usize,
    rows: usize,
    at: usize,
    fill: T,
) -> Vec<T> {
    let mut next = Vec::with_capacity((old_cols + 1) * rows);
    for row in 0..rows {
        let start = row * old_cols;
        next.extend_from_slice(&layer[start..start + at]);
        next.push(fill);
        next.extend_from_slice(&layer[start + at..start + old_cols]);
    }
    next
}

fn remove_layer_column<T: Copy>(layer: &[T], old_cols: usize, rows: usize, at: usize) -> Vec<T> {
    let mut next = Vec::with_capacity((old_cols - 1) * rows);
    for row in 0..rows {
        let start = row * old_cols;
        next.extend_from_slice(&layer[start..start + at]);
        next.extend_from_slice(&layer[start + at + 1..start + old_cols]);
    }
    next
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

fn shoreline_tile_accepts_wave(tile: AtlasTile) -> bool {
    matches!(
        tile,
        SHORE_NARROW_TOP
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

fn plant_visual_roll(kind: PlantKind, col: usize, row: usize) -> u8 {
    let mut rng = SeededRng::new(
        0xB05C_A11D_EC05_2026
            ^ ((kind.index() as u64).wrapping_mul(0x94D0_49BB_1331_11EB))
            ^ ((col as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
            ^ ((row as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9)),
    );
    (rng.next_u64() % 100) as u8
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
    fn world_save_json_roundtrips_editor_layers() {
        let mut world = TileWorld::new(4, 4);
        world.paint(0, 0, Brush::Background(BackgroundTile::Water));
        world.set_water_state(0, 0, WaterState::Duck);
        world.paint(1, 1, Brush::Foreground(SHORE_TOP_LEFT));
        world.paint_building(BuildingKind::House1, 4, 2);
        world.paint_prop_half(PropKind::Plant(PlantKind::Bush1), 0, 6);
        world.add_fog_rect((2, 2), (3, 3));

        let path = std::env::temp_dir().join(format!(
            "tactics_world_roundtrip_{}.json",
            std::process::id()
        ));

        world.save_to_path(&path).expect("world should save");
        let loaded = TileWorld::load_from_path(&path).expect("world should load");
        let _ = fs::remove_file(path);

        assert_eq!(loaded, world);
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
        let game = Game::new();

        assert_eq!(
            game.palette_brush(1),
            Brush::Background(BackgroundTile::Grass)
        );
        assert!(!game.foreground_tiles.contains(&GRASS_BG_TILE));
        assert!(!game.foreground_tiles.contains(&RAMP_A.top));
        assert!(!game.foreground_tiles.contains(&RAMP_A.bottom));
        assert!(!game.foreground_tiles.contains(&RAMP_B.top));
        assert!(!game.foreground_tiles.contains(&RAMP_B.bottom));
        for tile in PILLAR_TILES {
            assert!(!game.foreground_tiles.contains(&tile));
        }
        assert_eq!(game.palette_brush(3), Brush::Ramp(RAMP_A));
        assert_eq!(game.palette_brush(4), Brush::Ramp(RAMP_B));
        assert_eq!(
            game.palette_brush(6),
            Brush::Building(BuildingKind::Archery)
        );
        assert_eq!(
            game.palette_brush(6 + BUILDING_COUNT),
            Brush::Prop(PropKind::Pillar(PILLAR_TILES[0]))
        );
        assert_eq!(
            game.palette_brush(6 + BUILDING_COUNT + PILLAR_TILES.len()),
            Brush::Prop(PropKind::Plant(PlantKind::Tree1))
        );
        assert_eq!(
            game.palette_brush(6 + BUILDING_COUNT + PILLAR_TILES.len() + PLANT_PROP_COUNT),
            Brush::GoldResource
        );
        assert_eq!(
            game.palette_brush(
                6 + BUILDING_COUNT + PILLAR_TILES.len() + PLANT_PROP_COUNT + GOLD_PALETTE_COUNT
            ),
            Brush::RockResource
        );
    }

    #[test]
    fn palette_caption_uses_stable_developer_names() {
        assert_eq!(
            brush_developer_name(Brush::Background(BackgroundTile::Water)),
            "BG WATER"
        );
        assert_eq!(brush_developer_name(Brush::Ramp(RAMP_A)), "RAMP A");
        assert_eq!(
            brush_developer_name(Brush::Building(BuildingKind::House1)),
            "BUILDING HOUSE 1"
        );
        assert_eq!(
            brush_developer_name(Brush::Prop(PropKind::Pillar(PILLAR_TILES[2]))),
            "PILLAR 3"
        );
        assert_eq!(
            brush_developer_name(Brush::Prop(PropKind::Plant(PlantKind::Bush2))),
            "BUSH 2"
        );
        assert_eq!(brush_developer_name(Brush::GoldResource), "GOLD RESOURCE");
        assert_eq!(brush_developer_name(Brush::RockResource), "ROCK RESOURCE");
        assert_eq!(
            brush_developer_name(Brush::Foreground(SHORE_SINGLE_IN_GRASS)),
            "SHORE SINGLE IN GRASS"
        );
    }

    #[test]
    fn building_specs_use_foundation_footprints_independent_from_art_size() {
        let game = Game::new();

        assert_eq!(game.buildings.len(), BUILDING_COUNT);
        for (image, spec) in game.buildings.iter().zip(BUILDING_SPECS) {
            assert_eq!(image.texture_id, spec.texture_id);
            assert!(image.width >= spec.footprint_cols as u32 * TERRAIN_TILE_PX);
            assert!(image.height >= spec.footprint_rows as u32 * TERRAIN_TILE_PX);
        }
        assert_eq!(building_spec(BuildingKind::House1).footprint_cols, 2);
        assert_eq!(building_spec(BuildingKind::House1).footprint_rows, 2);
        assert_eq!(building_spec(BuildingKind::House1).footprint_offset_rows, 1);
        assert_eq!(building_spec(BuildingKind::House2).footprint_offset_rows, 1);
        assert_eq!(building_spec(BuildingKind::House3).footprint_offset_rows, 1);
        assert_eq!(building_spec(BuildingKind::House2).footprint_rows, 2);
        assert_eq!(building_spec(BuildingKind::House3).footprint_rows, 2);
        assert_eq!(building_spec(BuildingKind::Tower).footprint_rows, 2);
        assert_eq!(building_spec(BuildingKind::Tower).footprint_offset_rows, 2);
        assert_eq!(building_spec(BuildingKind::Archery).footprint_cols, 3);
        assert_eq!(building_spec(BuildingKind::Archery).footprint_rows, 3);
        assert_eq!(
            building_spec(BuildingKind::Archery).footprint_offset_rows,
            1
        );
        assert_eq!(building_spec(BuildingKind::Barracks).footprint_cols, 3);
        assert_eq!(building_spec(BuildingKind::Barracks).footprint_rows, 3);
        assert_eq!(
            building_spec(BuildingKind::Barracks).footprint_offset_rows,
            1
        );
        assert_eq!(building_spec(BuildingKind::Castle).footprint_cols, 5);
        assert_eq!(building_spec(BuildingKind::Castle).footprint_rows, 3);
        assert_eq!(building_spec(BuildingKind::Castle).footprint_offset_rows, 1);
        assert_eq!(building_spec(BuildingKind::Monastery).footprint_cols, 3);
        assert_eq!(building_spec(BuildingKind::Monastery).footprint_rows, 3);
        assert_eq!(
            building_spec(BuildingKind::Monastery).footprint_offset_rows,
            2
        );
    }

    #[test]
    fn building_brush_places_only_on_clean_grass_foundations() {
        let mut world = TileWorld::new(7, 7);

        world.paint(0, 0, Brush::Building(BuildingKind::House1));
        assert_eq!(
            world.buildings,
            vec![PlacedBuilding {
                kind: BuildingKind::House1,
                x2: 0,
                y2: 0
            }]
        );

        world.paint(2, 1, Brush::Background(BackgroundTile::Water));
        world.paint(2, 0, Brush::Building(BuildingKind::House1));
        assert_eq!(world.buildings.len(), 1);

        world.paint(4, 3, Brush::Foreground(RAMP_A.top));
        world.paint(4, 2, Brush::Building(BuildingKind::House1));
        assert_eq!(world.buildings.len(), 1);
    }

    #[test]
    fn building_brush_can_place_on_shoreline_foundations() {
        let mut world = TileWorld::new(4, 4);

        world.paint(0, 0, Brush::Background(BackgroundTile::Water));
        world.collapse_shorelines();
        assert!(world.foreground(0, 1).is_some_and(is_shoreline_tile));
        world.paint(0, 0, Brush::Building(BuildingKind::House1));

        assert_eq!(world.buildings.len(), 1);
        assert_eq!(building_footprint_rect(world.buildings[0]), (0, 1, 2, 2));
    }

    #[test]
    fn pillar_props_place_on_shorelines_and_block_buildings_and_ramps() {
        let mut world = TileWorld::new(4, 4);
        world.paint(0, 0, Brush::Background(BackgroundTile::Water));
        world.collapse_shorelines();
        assert!(world.foreground(0, 1).is_some_and(is_shoreline_tile));

        world.paint(0, 1, Brush::Prop(PropKind::Pillar(PILLAR_TILES[0])));
        assert_eq!(
            world.props,
            vec![PlacedProp {
                kind: PropKind::Pillar(PILLAR_TILES[0]),
                x2: 0,
                y2: 2
            }]
        );

        world.paint(0, 0, Brush::Building(BuildingKind::House1));
        assert!(world.buildings.is_empty());

        let before_top = world.foreground(0, 1);
        let before_bottom = world.foreground(0, 2);
        world.paint(0, 1, Brush::Ramp(RAMP_A));
        assert_eq!(world.foreground(0, 1), before_top);
        assert_eq!(world.foreground(0, 2), before_bottom);

        world.paint(0, 1, Brush::ClearForeground);
        assert!(world.props.is_empty());
    }

    #[test]
    fn plant_props_place_on_grass_shores_with_single_tile_hitboxes() {
        let mut world = TileWorld::new(4, 4);
        world.paint(0, 0, Brush::Background(BackgroundTile::Water));
        world.collapse_shorelines();
        assert!(world.foreground(0, 1).is_some_and(is_shoreline_tile));

        world.paint(0, 1, Brush::Prop(PropKind::Plant(PlantKind::Tree1)));
        assert_eq!(
            world.props,
            vec![PlacedProp {
                kind: PropKind::Plant(PlantKind::Tree1),
                x2: 0,
                y2: 2
            }]
        );

        world.paint(0, 0, Brush::Prop(PropKind::Plant(PlantKind::Stump1)));
        assert_eq!(world.props.len(), 1);

        world.paint(0, 0, Brush::Building(BuildingKind::House1));
        assert!(world.buildings.is_empty());
        world.clear_foreground(0, 1);
        assert!(world.props.is_empty());
    }

    #[test]
    fn plant_props_use_half_grid_placement_and_foundation_checks() {
        let mut world = TileWorld::new(4, 4);

        world.paint_prop_half(PropKind::Plant(PlantKind::Bush1), 1, 1);
        assert_eq!(
            world.props,
            vec![PlacedProp {
                kind: PropKind::Plant(PlantKind::Bush1),
                x2: 1,
                y2: 1
            }]
        );

        world.paint_prop_half(PropKind::Plant(PlantKind::Bush2), 2, 1);
        assert_eq!(world.props.len(), 1);

        world.paint(1, 1, Brush::Background(BackgroundTile::Water));
        world.paint_prop_half(PropKind::Plant(PlantKind::Bush2), 3, 1);
        assert_eq!(world.props.len(), 1);

        world.clear_foreground(0, 0);
        assert!(world.props.is_empty());
    }

    #[test]
    fn gold_resource_brush_cycles_variants_on_empty_grass() {
        let mut world = TileWorld::new(2, 2);

        for expected in GoldKind::ALL {
            world.paint(0, 0, Brush::GoldResource);
            assert_eq!(
                world.props,
                vec![PlacedProp {
                    kind: PropKind::Gold(expected),
                    x2: 0,
                    y2: 0
                }]
            );
        }

        world.paint(0, 0, Brush::GoldResource);
        assert!(world.props.is_empty());
    }

    #[test]
    fn gold_resource_brush_requires_unoccupied_grass() {
        let mut world = TileWorld::new(4, 4);

        world.paint(0, 0, Brush::Background(BackgroundTile::Water));
        world.paint(0, 0, Brush::GoldResource);
        assert!(world.props.is_empty());

        world.paint(1, 0, Brush::Prop(PropKind::Pillar(PILLAR_TILES[0])));
        world.paint(1, 0, Brush::GoldResource);
        assert_eq!(world.props.len(), 1);

        world.paint(2, 0, Brush::Building(BuildingKind::House1));
        world.paint(2, 1, Brush::GoldResource);
        assert_eq!(world.props.len(), 1);

        world.paint(0, 1, Brush::GoldResource);
        assert_eq!(
            world.props.last().map(|prop| prop.kind),
            Some(PropKind::Gold(GoldKind::Stone1))
        );
    }

    #[test]
    fn rock_resource_brush_cycles_static_rock_props_on_empty_grass() {
        let mut world = TileWorld::new(2, 2);

        for expected in RockKind::ALL {
            world.paint(0, 0, Brush::RockResource);
            assert_eq!(
                world.props,
                vec![PlacedProp {
                    kind: PropKind::Rock(expected),
                    x2: 0,
                    y2: 0
                }]
            );
        }

        world.paint(0, 0, Brush::RockResource);
        assert!(world.props.is_empty());
    }

    #[test]
    fn rock_resource_brush_requires_unoccupied_grass() {
        let mut world = TileWorld::new(4, 4);

        world.paint(0, 0, Brush::Background(BackgroundTile::Water));
        world.paint(0, 0, Brush::RockResource);
        assert!(world.props.is_empty());

        world.paint(1, 0, Brush::Prop(PropKind::Pillar(PILLAR_TILES[0])));
        world.paint(1, 0, Brush::RockResource);
        assert_eq!(world.props.len(), 1);

        world.paint(2, 0, Brush::Building(BuildingKind::House1));
        world.paint(2, 1, Brush::RockResource);
        assert_eq!(world.props.len(), 1);

        world.paint(0, 1, Brush::RockResource);
        assert_eq!(
            world.props.last().map(|prop| prop.kind),
            Some(PropKind::Rock(RockKind::Rock1))
        );
    }

    #[test]
    fn small_bush_props_use_bottom_half_tile_footprints() {
        let mut world = TileWorld::new(2, 2);

        world.paint_prop_half(PropKind::Plant(PlantKind::Bush2), 0, 0);
        world.paint_prop_half(PropKind::Plant(PlantKind::Bush4), 0, 1);
        assert_eq!(world.props.len(), 2);
        assert_eq!(prop_footprint_rect2(world.props[0]), (0, 1, 2, 1));
        assert_eq!(prop_footprint_rect2(world.props[1]), (0, 2, 2, 1));

        world.paint_prop_half(PropKind::Plant(PlantKind::Bush1), 0, 0);
        assert_eq!(world.props.len(), 2);

        world.paint_prop_half(PropKind::Plant(PlantKind::Bush2), 0, 3);
        assert_eq!(world.props.len(), 2);
    }

    #[test]
    fn small_bush_preview_uses_bottom_half_tile_footprint() {
        assert_eq!(
            brush_preview_footprint_rect2(
                Brush::Prop(PropKind::Plant(PlantKind::Bush2)),
                4,
                6,
                3,
                WORLD_ROWS,
            ),
            (4, 7, 2, 1)
        );
        assert_eq!(
            brush_preview_footprint_rect2(
                Brush::Prop(PropKind::Plant(PlantKind::Bush1)),
                4,
                6,
                3,
                WORLD_ROWS,
            ),
            (4, 6, 2, 2)
        );
    }

    #[test]
    fn world_edges_can_be_added_and_removed_without_losing_layers() {
        let mut world = TileWorld::new(2, 2);
        world.set_background(1, 1, BackgroundTile::Water);
        let old_index = world.index(1, 1);
        world.foregrounds[old_index] = Some(SHORE_TOP);
        world.fog[old_index] = true;
        world.buildings.push(PlacedBuilding {
            kind: BuildingKind::House1,
            x2: 2,
            y2: 0,
        });
        world.props.push(PlacedProp {
            kind: PropKind::Plant(PlantKind::Bush1),
            x2: 2,
            y2: 2,
        });

        world.add_edge(WorldEdge::Top);
        world.add_edge(WorldEdge::Left);

        assert_eq!((world.cols, world.rows), (3, 3));
        assert_eq!(world.background(2, 2), BackgroundTile::Water);
        assert_eq!(world.foreground(2, 2), Some(SHORE_TOP));
        assert!(world.fog(2, 2));
        assert_eq!((world.buildings[0].x2, world.buildings[0].y2), (4, 2));
        assert_eq!((world.props[0].x2, world.props[0].y2), (4, 4));

        assert!(world.remove_edge(WorldEdge::Top));
        assert!(world.remove_edge(WorldEdge::Left));

        assert_eq!((world.cols, world.rows), (2, 2));
        assert_eq!(world.background(1, 1), BackgroundTile::Water);
        assert_eq!(world.foreground(1, 1), Some(SHORE_TOP));
        assert!(world.fog(1, 1));
        assert_eq!((world.buildings[0].x2, world.buildings[0].y2), (2, 0));
        assert_eq!((world.props[0].x2, world.props[0].y2), (2, 2));
    }

    #[test]
    fn removing_world_edges_drops_overlapping_objects() {
        let mut world = TileWorld::new(2, 2);
        world.props.push(PlacedProp {
            kind: PropKind::Plant(PlantKind::Bush1),
            x2: 0,
            y2: 0,
        });
        world.buildings.push(PlacedBuilding {
            kind: BuildingKind::House1,
            x2: 0,
            y2: -2,
        });

        assert!(world.remove_edge(WorldEdge::Left));
        assert_eq!((world.cols, world.rows), (1, 2));
        assert!(world.props.is_empty());
        assert!(world.buildings.is_empty());

        assert!(!world.remove_edge(WorldEdge::Left));
        assert_eq!((world.cols, world.rows), (1, 2));
    }

    #[test]
    fn prop_draw_order_puts_lower_map_tiles_in_front() {
        let upper = PlacedProp {
            kind: PropKind::Plant(PlantKind::Tree1),
            x2: 2,
            y2: 2,
        };
        let lower = PlacedProp {
            kind: PropKind::Plant(PlantKind::Tree2),
            x2: 2,
            y2: 4,
        };

        assert_eq!(prop_draw_order(&[lower, upper]), vec![upper, lower]);
        assert_eq!(prop_draw_order(&[upper, lower]), vec![upper, lower]);
    }

    #[test]
    fn prop_insertions_keep_draw_order_cached_in_world() {
        let mut world = TileWorld::new(4, 4);
        world.paint_prop_half(PropKind::Plant(PlantKind::Tree1), 2, 4);
        world.paint_prop_half(PropKind::Plant(PlantKind::Tree2), 0, 2);
        world.paint_prop_half(PropKind::Plant(PlantKind::Tree3), 2, 2);

        assert_eq!(
            world
                .props
                .iter()
                .map(|prop| (prop.x2, prop.y2))
                .collect::<Vec<_>>(),
            vec![(0, 2), (2, 2), (2, 4)]
        );
    }

    #[test]
    fn terrain_draw_cache_rebuilds_static_background_and_foreground_entries() {
        let mut world = TileWorld::new(2, 2);
        world.paint(0, 0, Brush::Background(BackgroundTile::Water));
        world.paint(1, 0, Brush::Foreground(SHORE_TOP));

        let mut cache = TerrainDrawCache::new();
        cache.rebuild_if_dirty(&world);

        assert!(!cache.dirty);
        assert_eq!(cache.backgrounds.len(), 2);
        assert_eq!(
            cache.foregrounds,
            vec![TerrainDrawCell {
                tile: SHORE_TOP,
                col: 1,
                row: 0
            }]
        );
    }

    #[test]
    fn loadscreen_uses_one_large_and_two_small_wood_tables() {
        let tables = loadscreen_table_layout(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32);

        assert_eq!(tables.len(), 3);
        assert!(tables[0].w > tables[1].w);
        assert!(tables[0].h > tables[1].h);
        assert_eq!(tables[1].w, tables[2].w);
        assert_eq!(tables[1].h, tables[2].h);
        for table in tables {
            assert!(table.x >= 0.0);
            assert!(table.y >= 0.0);
            assert!(table.x + table.w <= WINDOW_WIDTH as f32);
            assert!(table.y + table.h <= WINDOW_HEIGHT as f32);
        }
    }

    #[test]
    fn loadscreen_window_is_undecorated_and_draws_tiled_tables() {
        let loadscreen = LoadScreen::new();
        assert!(!loadscreen.window_decorations());
        assert!(!loadscreen.cursor_visible());

        let mut batch = SpriteBatch::new(WINDOW_WIDTH, WINDOW_HEIGHT);
        let table = loadscreen_table_layout(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32)[0];
        draw_wood_table(
            &mut batch,
            &loadscreen.wood_table,
            table.x,
            table.y,
            table.w,
            table.h,
        );

        assert!(!batch.bytes.is_empty());
    }

    #[test]
    fn loadscreen_wood_table_corners_use_source_edge_offsets() {
        assert_eq!(
            WOOD_TABLE_TOP_LEFT_OUTSET_X,
            (WOOD_TABLE_LEFT_EDGE.x - WOOD_TABLE_TOP_LEFT.x) as f32
        );
        assert_eq!(
            WOOD_TABLE_BOTTOM_LEFT_OUTSET_X,
            (WOOD_TABLE_LEFT_EDGE.x - WOOD_TABLE_BOTTOM_LEFT.x) as f32
        );
        assert_eq!(
            WOOD_TABLE_TOP_RIGHT.width as f32
                - WOOD_TABLE_TOP_RIGHT_OUTSET_X
                - WOOD_TABLE_RIGHT_EDGE.width as f32,
            (WOOD_TABLE_RIGHT_EDGE.x - WOOD_TABLE_TOP_RIGHT.x) as f32
        );
        assert_eq!(
            WOOD_TABLE_BOTTOM_RIGHT.width as f32
                - WOOD_TABLE_BOTTOM_RIGHT_OUTSET_X
                - WOOD_TABLE_RIGHT_EDGE.width as f32,
            (WOOD_TABLE_RIGHT_EDGE.x - WOOD_TABLE_BOTTOM_RIGHT.x) as f32
        );
        assert_eq!(
            WOOD_TABLE_TOP_CORNER_OUTSET_Y,
            (WOOD_TABLE_TOP_EDGE.y - WOOD_TABLE_TOP_LEFT.y) as f32
        );
    }

    #[test]
    fn building_hitboxes_are_shifted_down_from_sprite_anchor() {
        let mut world = TileWorld::new(5, 5);

        world.paint(0, 0, Brush::Building(BuildingKind::House1));
        assert_eq!(building_footprint_rect(world.buildings[0]), (0, 1, 2, 2));

        world.clear_foreground(0, 0);
        assert_eq!(world.buildings.len(), 1);
        world.clear_foreground(0, 1);
        assert!(world.buildings.is_empty());

        world.paint(2, 0, Brush::Building(BuildingKind::Tower));
        assert_eq!(building_footprint_rect(world.buildings[0]), (2, 2, 2, 2));
    }

    #[test]
    fn building_brush_anchor_centers_hitbox_on_cursor_cell() {
        let game = Game::new();

        assert_eq!(
            game.building_anchor_half_cell(BuildingKind::House1, 6, 6),
            (4, 2)
        );
        assert_eq!(
            building_spec_footprint_rect2(building_spec(BuildingKind::House1), 4, 2),
            (4, 4, 4, 4)
        );
        assert_eq!(
            half_rect_to_tile_rect(building_spec_footprint_rect2(
                building_spec(BuildingKind::House1),
                4,
                2
            )),
            (2, 2, 2, 2)
        );
        assert_eq!(
            game.building_anchor_half_cell(BuildingKind::Archery, 9, 10),
            (6, 5)
        );
        assert_eq!(
            building_spec_footprint_rect2(building_spec(BuildingKind::Archery), 6, 5),
            (6, 7, 6, 6)
        );
        assert_eq!(
            game.building_anchor_half_cell(BuildingKind::Tower, 0, 0),
            (-2, -6)
        );
    }

    #[test]
    fn half_tile_building_placement_rejects_any_water_under_the_footprint() {
        let mut world = TileWorld::new(6, 5);

        world.paint(2, 1, Brush::Background(BackgroundTile::Water));
        assert!(!world.can_place_building_kind(BuildingKind::House1, 5, 0));
        world.paint_building(BuildingKind::House1, 5, 0);
        assert!(world.buildings.is_empty());

        assert!(world.can_place_building_kind(BuildingKind::House1, 6, 0));
        world.paint_building(BuildingKind::House1, 6, 0);
        assert_eq!(world.buildings.len(), 1);
        assert_eq!(world.buildings[0].x2, 6);
        assert_eq!(world.buildings[0].y2, 0);
        assert_eq!(
            half_rect_to_tile_rect(building_spec_footprint_rect2(
                building_spec(BuildingKind::House1),
                6,
                0
            )),
            (3, 1, 2, 2)
        );
    }

    #[test]
    fn editor_building_anchor_can_place_foundations_on_top_world_row() {
        let game = Game::new();
        let mut world = TileWorld::new(6, 5);
        let anchor = game.building_anchor_half_cell(BuildingKind::House1, 2, 2);

        assert_eq!(anchor, (0, -2));
        assert!(world.can_place_building_kind(BuildingKind::House1, anchor.0, anchor.1));
        world.paint_building(BuildingKind::House1, anchor.0, anchor.1);
        assert_eq!(world.buildings.len(), 1);
        assert_eq!(building_footprint_rect(world.buildings[0]), (0, 0, 2, 2));
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
    fn foreground_can_place_on_water_but_ramps_need_grass() {
        let mut world = TileWorld::new(3, 3);
        let foreground = AtlasTile { col: 0, row: 0 };

        world.paint(1, 1, Brush::Background(BackgroundTile::Water));
        world.paint(1, 1, Brush::Foreground(foreground));
        assert_eq!(world.foreground(1, 1), Some(foreground));

        world.paint(0, 0, Brush::Background(BackgroundTile::Water));
        world.paint(0, 0, Brush::Ramp(RAMP_A));
        assert_eq!(world.foreground(0, 0), None);
        assert_eq!(world.foreground(0, 1), None);

        world.paint(2, 1, Brush::Background(BackgroundTile::Water));
        world.paint(2, 0, Brush::Ramp(RAMP_A));
        assert_eq!(world.foreground(2, 0), None);
        assert_eq!(world.foreground(2, 1), None);
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
    fn empty_water_tiles_can_hold_explicit_visual_states() {
        let mut world = TileWorld::new(3, 3);
        world.paint(1, 1, Brush::Background(BackgroundTile::Water));

        assert_eq!(WaterState::ALL.len(), 6);
        assert!(!WaterState::ALL.contains(&WaterState::Animation));
        for state in WaterState::ALL {
            world.set_water_state(1, 1, state);
            assert_eq!(world.water_state(1, 1), Some(state));
        }

        world.paint(1, 1, Brush::Background(BackgroundTile::Grass));
        assert_eq!(world.water_state(1, 1), None);

        world.paint(1, 1, Brush::Background(BackgroundTile::Water));
        world.set_water_state(1, 1, WaterState::Stone2);
        world.paint(1, 1, Brush::Foreground(SHORE_SINGLE_IN_WATER));
        assert_eq!(world.water_state(1, 1), None);
        world.paint(1, 1, Brush::ClearForeground);
        assert_eq!(world.water_state(1, 1), Some(WaterState::Nothing));
    }

    #[test]
    fn water_brush_cycles_empty_water_tile_state_in_editor_path() {
        let mut world = TileWorld::new(2, 2);

        world.paint(0, 0, Brush::Background(BackgroundTile::Water));
        assert_eq!(world.water_state(0, 0), Some(WaterState::Nothing));

        for state in [
            WaterState::Stone1,
            WaterState::Stone2,
            WaterState::Stone3,
            WaterState::Stone4,
            WaterState::Duck,
            WaterState::Nothing,
        ] {
            world.paint(0, 0, Brush::Background(BackgroundTile::Water));
            assert_eq!(world.water_state(0, 0), Some(state));
        }

        world.paint(0, 0, Brush::Foreground(SHORE_SINGLE_IN_WATER));
        world.paint(0, 0, Brush::Background(BackgroundTile::Water));
        assert_eq!(world.water_state(0, 0), None);
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
        let game = Game::new();

        assert_eq!(game.palette_brush(5), Brush::FogRect);
    }

    #[test]
    fn palette_hit_testing_uses_the_same_slots_as_palette_drawing() {
        let game = Game::new();

        for slot in [
            6,
            6 + BUILDING_COUNT - 1,
            6 + BUILDING_COUNT,
            6 + BUILDING_COUNT + PILLAR_TILES.len() - 1,
            6 + BUILDING_COUNT + PILLAR_TILES.len(),
            6 + BUILDING_COUNT + PILLAR_TILES.len() + PLANT_PROP_COUNT - 1,
            6 + BUILDING_COUNT + PILLAR_TILES.len() + PLANT_PROP_COUNT,
            game.palette_len() - 1,
        ] {
            let (x, y) = game.palette_slot_rect(slot);
            assert_eq!(
                game.palette_brush_at(x + PALETTE_TILE * 0.5, y + PALETTE_TILE * 0.5),
                Some(game.palette_brush(slot))
            );
        }
    }

    #[test]
    fn fog_tool_keeps_the_regular_cursor() {
        let mut game = Game::new();
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

        let game = Game::new();
        let cursor_select = ImageAsset::from_png_bytes(CURSOR_SELECT_TEXTURE, CURSOR_SELECT_BYTES);
        assert_eq!(game.cursor_select.width, cursor_select.width);
        assert_eq!(game.cursor_select.height, cursor_select.height);
    }

    #[test]
    fn fog_shadow_png_is_cropped_before_stretching_to_tiles() {
        let full = ImageAsset::from_png_bytes(FOG_TEXTURE, FOG_BYTES);
        let game = Game::new();

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
    fn wave_environment_only_targets_water_heavy_shoreline_tiles() {
        assert!(!shoreline_tile_accepts_wave(SHORE_TOP_LEFT));
        assert!(!shoreline_tile_accepts_wave(SHORE_TOP));
        assert!(!shoreline_tile_accepts_wave(SHORE_RIGHT));
        assert!(shoreline_tile_accepts_wave(SHORE_NARROW_CENTER));
        assert!(shoreline_tile_accepts_wave(SHORE_SINGLE_IN_WATER));
        assert!(shoreline_tile_accepts_wave(SHORE_SINGLE_IN_GRASS));
    }

    #[test]
    fn resize_expands_visible_world_without_scaling_layout_space() {
        let mut game = Game::new();
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
    fn aseprite_explorer_groups_untagged_duck_and_water_rocks_as_single_animations() {
        let explorer = AsepriteExplorer::new();
        let duck = &explorer.files[8];
        let water_rock = &explorer.files[13];

        assert_eq!(duck.frames.len(), 3);
        assert_eq!(duck.clips.len(), 1);
        assert_eq!(duck.clips[0].frame_indices.len(), 3);
        assert_eq!(water_rock.frames.len(), 16);
        assert_eq!(water_rock.clips.len(), 1);
        assert_eq!(water_rock.clips[0].frame_indices.len(), 16);
    }

    #[test]
    fn unit_walk_viewer_loads_one_clip_for_each_unit() {
        let viewer = UnitWalkViewer::new();

        assert_eq!(viewer.units.len(), UNIT_WALK_SPECS.len() + 23 + 8);
        assert!(viewer.units.iter().all(|unit| !unit.frames.is_empty()));
        assert!(viewer.units.iter().all(|unit| unit.total_duration_ms > 0));
        assert!(viewer.units.iter().any(|unit| unit.source_tag == "Run"));
        assert!(viewer.units.iter().any(|unit| unit.source_tag == "Move"));
        assert_eq!(
            viewer
                .units
                .iter()
                .filter(|unit| unit.name.starts_with("Pawn"))
                .count(),
            23
        );
        assert!(
            viewer
                .units
                .iter()
                .any(|unit| unit.name == "Pawn Interact Pickaxe")
        );
        assert!(
            viewer
                .units
                .iter()
                .any(|unit| unit.name == "Pawn Run Pickaxe")
        );
        assert_eq!(
            viewer
                .units
                .iter()
                .filter(|unit| unit.name.starts_with("Archer"))
                .count(),
            3
        );
        assert_eq!(
            viewer
                .units
                .iter()
                .filter(|unit| unit.name.starts_with("Monk"))
                .count(),
            4
        );
        assert_eq!(
            viewer
                .units
                .iter()
                .filter(|unit| unit.name.starts_with("Warrior"))
                .count(),
            5
        );
        assert_eq!(
            viewer
                .units
                .iter()
                .filter(|unit| unit.name.starts_with("Particle"))
                .count(),
            8
        );
        assert!(
            viewer
                .units
                .iter()
                .any(|unit| unit.name == "Particle Explosion 1")
        );
        assert!(
            viewer
                .units
                .iter()
                .any(|unit| unit.name == "Particle Water Splash")
        );
        assert!(viewer.units.iter().any(|unit| unit.source_tag == "Shoot"));
        assert!(
            viewer
                .units
                .iter()
                .any(|unit| unit.source_tag == "Heal Effect")
        );
        assert!(
            viewer
                .units
                .iter()
                .any(|unit| unit.source_tag == "Attack 2")
        );
        assert_eq!(
            viewer
                .units
                .iter()
                .find(|unit| unit.name == "Sheep")
                .map(|unit| (unit.source_tag.as_str(), unit.offset_y)),
            Some(("Move", 8.0))
        );
        assert_eq!(
            viewer
                .units
                .iter()
                .find(|unit| unit.name == "Lancer")
                .map(|unit| unit.offset_y),
            Some(-16.0)
        );
    }

    #[test]
    fn lancer_animation_metadata_matches_aseprite_tags() {
        let set = ase_assets::load_tinted_aseprite_set(
            "ts_freepack/Lancer.aseprite",
            [255, 255, 255, 255],
            ase_assets::TintMode::Multiply,
        )
        .expect("lancer aseprite should decode");

        assert_eq!(
            set.tags.len(),
            crate::game_object::LancerAnimation::ALL.len()
        );
        for animation in crate::game_object::LancerAnimation::ALL {
            let expected = animation.aseprite_ref();
            let actual = &set.tags[expected.tag_index];

            assert_eq!(actual.name.trim(), expected.tag_name);
        }
    }

    #[test]
    fn unit_walk_viewer_flips_direction_every_five_seconds() {
        assert!(!unit_viewer_flip_x(0));
        assert!(!unit_viewer_flip_x(4_999));
        assert!(unit_viewer_flip_x(5_000));
        assert!(unit_viewer_flip_x(9_999));
        assert!(!unit_viewer_flip_x(10_000));
    }

    #[test]
    fn cloud_assets_load_as_static_variants() {
        let clouds = load_cloud_assets();

        assert_eq!(clouds.len(), 8);
        assert!(
            clouds
                .iter()
                .all(|cloud| cloud.width > 0 && cloud.height > 0)
        );
        assert!(
            clouds
                .iter()
                .all(|cloud| cloud.rgba.chunks_exact(4).any(|pixel| pixel[3] != 0))
        );
    }

    #[test]
    fn water_state_visual_assets_load_for_all_non_empty_states() {
        let water = load_water_visual_assets();

        assert_eq!(water.stones.len(), 4);
        assert!(
            water
                .stones
                .iter()
                .all(|stone| !stone.frames.is_empty() && stone.total_duration_ms > 0)
        );
        assert!(!water.animation.frames.is_empty());
        assert_eq!(
            water.animation.frames.len(),
            water.animation.durations_ms.len()
        );
        assert!(water.animation.total_duration_ms > 0);
        assert!(!water.duck.frames.is_empty());
        assert_eq!(water.duck.frames.len(), water.duck.durations_ms.len());
        assert!(water.duck.total_duration_ms > 0);
    }

    #[test]
    fn water_visual_texture_ids_do_not_overlap_between_states() {
        let water = load_water_visual_assets();
        let mut ids = HashSet::new();
        let all_frames = water
            .stones
            .iter()
            .flat_map(|animation| animation.frames.iter())
            .chain(water.animation.frames.iter())
            .chain(water.duck.frames.iter());

        for frame in all_frames {
            assert!(
                ids.insert(frame.texture_id),
                "duplicate water visual texture id {}",
                frame.texture_id
            );
        }
    }

    #[test]
    fn water_state_animations_are_one_shots_with_still_frames() {
        let water = load_water_visual_assets();
        let stone = &water.stones[0];
        let first = stone.first_frame().unwrap().texture_id;

        assert_ne!(
            stone
                .frame_once(stone.durations_ms.first().copied().unwrap_or(1))
                .unwrap()
                .texture_id,
            first
        );
        assert_eq!(
            stone
                .frame_once(stone.total_duration_ms + 100)
                .unwrap()
                .texture_id,
            first
        );
        assert_eq!(
            water
                .animation
                .frame_once(water.animation.total_duration_ms + 100)
                .unwrap()
                .texture_id,
            water.animation.first_frame().unwrap().texture_id
        );
    }

    #[test]
    fn environment_animator_triggers_stone_and_wave_water_states_but_not_ducks() {
        let mut game = Game::new();
        game.world = TileWorld::new(2, 1);
        game.world
            .paint(0, 0, Brush::Background(BackgroundTile::Water));
        game.world.set_water_state(0, 0, WaterState::Duck);
        game.world
            .paint(1, 0, Brush::Foreground(SHORE_SINGLE_IN_GRASS));
        game.water_animation_timer = 0.0;

        game.update_environment(1.0);

        assert_eq!(game.water_animations.len(), 1);
        assert_eq!(game.water_animations[0].state, WaterState::Animation);
        assert_eq!(
            (game.water_animations[0].col, game.water_animations[0].row),
            (1, 0)
        );
    }

    #[test]
    fn environment_animation_timers_are_fast_enough_for_visible_feedback() {
        let mut game = Game::new();
        game.plant_animation_timer = 0.0;
        game.water_animation_timer = 0.0;
        game.gold_animation_timer = 0.0;

        game.update_environment(1.0);

        assert!(game.plant_animation_timer <= 1.0);
        assert!(game.water_animation_timer <= 0.85);
        assert!(game.gold_animation_timer <= 1.1);
    }

    #[test]
    fn environment_animator_triggers_trees_and_bushes_but_not_stumps() {
        let mut game = Game::new();
        game.world = TileWorld::new(3, 3);
        game.world
            .paint(0, 0, Brush::Prop(PropKind::Plant(PlantKind::Tree1)));
        game.world
            .paint(1, 0, Brush::Prop(PropKind::Plant(PlantKind::Stump1)));
        game.world
            .paint(2, 0, Brush::Prop(PropKind::Plant(PlantKind::Bush1)));
        game.plant_animation_timer = 0.0;

        game.update_environment(1.0);

        assert!(!game.plant_animations.is_empty());
        assert!(game.plant_animations.len() <= 2);
        assert!(
            game.plant_animations
                .iter()
                .all(|animation| animation.kind == PlantKind::Tree1
                    || animation.kind == PlantKind::Bush1)
        );
    }

    #[test]
    fn environment_animator_triggers_gold_props_as_one_shot_animations() {
        let mut game = Game::new();
        game.world = TileWorld::new(3, 1);
        game.world.paint(0, 0, Brush::GoldResource);
        game.world.paint(1, 0, Brush::GoldResource);
        game.world.paint(1, 0, Brush::GoldResource);
        game.gold_animation_timer = 0.0;

        game.update_environment(1.0);

        assert!(!game.gold_animations.is_empty());
        assert!(game.gold_animations.len() <= 2);
        assert!(
            game.gold_animations
                .iter()
                .all(|animation| matches!(animation.kind, GoldKind::Stone1 | GoldKind::Stone2))
        );

        let active = game.gold_animations[0];
        let first = game.gold_props[active.kind.index()]
            .first_frame()
            .unwrap()
            .texture_id;
        let duration = game.gold_props[active.kind.index()]
            .durations_ms
            .first()
            .copied()
            .unwrap_or(1);
        assert_ne!(
            game.gold_props[active.kind.index()]
                .frame_once(duration)
                .unwrap()
                .texture_id,
            first
        );
    }

    #[test]
    fn plant_prop_assets_load_for_tree_stump_and_bush_palette() {
        let plants = load_plant_prop_assets();

        assert_eq!(plants.len(), PLANT_PROP_COUNT);
        assert!(
            plants
                .iter()
                .all(|plant| !plant.frames.is_empty() && plant.total_duration_ms > 0)
        );
        assert!(plants[PlantKind::Tree1.index()].frames.len() > 1);
        assert_eq!(plants[PlantKind::Stump1.index()].frames.len(), 1);
        assert!(plants[PlantKind::Bush1.index()].frames.len() > 1);
        assert!(
            plants[PlantKind::Tree1.index()]
                .first_frame()
                .unwrap()
                .height
                > plants[PlantKind::Stump1.index()]
                    .first_frame()
                    .unwrap()
                    .height
        );
    }

    #[test]
    fn plant_prop_texture_ids_do_not_overlap_between_kinds() {
        let plants = load_plant_prop_assets();
        let mut ids = HashSet::new();

        for frame in plants.iter().flat_map(|animation| animation.frames.iter()) {
            assert!(
                ids.insert(frame.texture_id),
                "duplicate plant prop texture id {}",
                frame.texture_id
            );
        }
    }

    #[test]
    fn gold_prop_assets_load_for_palette_and_cycle_states() {
        let gold = load_gold_prop_assets();

        assert_eq!(gold.len(), GOLD_PROP_COUNT);
        assert!(
            gold.iter()
                .all(|animation| !animation.frames.is_empty() && animation.total_duration_ms > 0)
        );
        assert_eq!(gold[GoldKind::Stone1.index()].frames.len(), 7);
        assert_eq!(gold[GoldKind::Nugget.index()].frames.len(), 7);
    }

    #[test]
    fn gold_prop_texture_ids_do_not_overlap_between_kinds() {
        let gold = load_gold_prop_assets();
        let mut ids = HashSet::new();

        for frame in gold.iter().flat_map(|animation| animation.frames.iter()) {
            assert!(
                ids.insert(frame.texture_id),
                "duplicate gold prop texture id {}",
                frame.texture_id
            );
        }
    }

    #[test]
    fn rock_prop_assets_load_as_static_grass_props() {
        let rocks = load_rock_prop_assets();

        assert_eq!(rocks.len(), ROCK_PROP_COUNT);
        assert!(rocks.iter().all(|rock| {
            rock.width <= 64
                && rock.height <= 64
                && rock.rgba.chunks_exact(4).any(|pixel| pixel[3] != 0)
        }));
    }

    #[test]
    fn icon_viewer_loads_ui_tool_resource_and_arrow_icons() {
        let viewer = IconViewer::new();
        let labels = viewer
            .icons
            .iter()
            .map(|icon| icon.label.as_str())
            .collect::<Vec<_>>();

        for index in 1..=12 {
            assert!(labels.contains(&format!("Icon {index:02}").as_str()));
        }
        for expected in ["Tool 01", "Tool 02", "Tool 03", "Tool 04", "Meat", "Wood"] {
            assert!(labels.contains(&expected));
        }
        assert!(labels.contains(&"Arrow"));
    }

    #[test]
    fn event_editor_cycles_draft_selectors_and_adds_rules() {
        let mut editor = EventEditor::new();
        assert_eq!(editor.rules.len(), 1);

        editor.mouse = Point { x: 32.0, y: 86.0 };
        editor.handle_left_click();
        assert_eq!(editor.draft_trigger, ScenarioTrigger::OnceAfterOneMinute);

        editor.mouse = Point { x: 266.0, y: 86.0 };
        editor.handle_left_click();
        assert_eq!(editor.draft_condition, ScenarioCondition::PlayerHasWood);

        editor.mouse = Point { x: 500.0, y: 86.0 };
        editor.handle_left_click();
        assert_eq!(editor.draft_action, ScenarioAction::SpawnEnemy);

        editor.mouse = Point { x: 32.0, y: 150.0 };
        editor.handle_left_click();

        assert_eq!(editor.rules.len(), 2);
        assert_eq!(editor.rules[1].trigger, ScenarioTrigger::OnceAfterOneMinute);
        assert_eq!(editor.rules[1].condition, ScenarioCondition::PlayerHasWood);
        assert_eq!(editor.rules[1].action, ScenarioAction::SpawnEnemy);
    }

    #[test]
    fn event_editor_table_buttons_toggle_and_delete_rules() {
        let mut editor = EventEditor::new();
        editor.add_rule();
        assert_eq!(editor.rules.len(), 2);
        assert!(editor.rules[0].enabled);

        editor.mouse = Point { x: 36.0, y: 224.0 };
        editor.handle_left_click();
        assert!(!editor.rules[0].enabled);

        editor.mouse = Point {
            x: EVENT_EDITOR_WIDTH as f32 - 56.0,
            y: 224.0,
        };
        editor.handle_left_click();

        assert_eq!(editor.rules.len(), 1);
        assert_eq!(editor.rules[0].id, 2);
    }

    #[test]
    fn stump_props_render_half_tile_above_their_hitbox() {
        assert_eq!(PlantKind::Stump1.render_offset_y(), -TILE_SIZE / 2.0);
        assert_eq!(PlantKind::Stump2.render_offset_y(), -TILE_SIZE / 2.0);
        assert_eq!(PlantKind::Stump3.render_offset_y(), -TILE_SIZE / 2.0);
        assert_eq!(PlantKind::Stump4.render_offset_y(), -TILE_SIZE / 2.0);
        assert_eq!(PlantKind::Tree1.render_offset_y(), 0.0);
        assert_eq!(PlantKind::Bush1.render_offset_y(), 0.0);
    }

    #[test]
    fn tall_pine_renders_slightly_smaller_without_changing_other_plants() {
        assert_eq!(PlantKind::Tree2.render_scale(), 0.84);
        assert_eq!(PlantKind::Tree1.render_scale(), 1.0);
        assert_eq!(PlantKind::Tree3.render_scale(), 1.0);
        assert_eq!(PlantKind::Stump2.render_scale(), 1.0);
        assert_eq!(PlantKind::Bush1.render_scale(), 1.0);
    }

    #[test]
    fn bush_props_can_render_as_static_combo_tiles() {
        assert_eq!(PlantKind::Bush1.visual_instance_count_for_roll(24), 2);
        assert_eq!(PlantKind::Bush1.visual_instance_count_for_roll(25), 1);
        assert_eq!(PlantKind::Bush3.visual_instance_count_for_roll(0), 2);
        assert_eq!(PlantKind::Bush3.visual_instance_count_for_roll(99), 1);

        assert_eq!(PlantKind::Bush2.visual_instance_count_for_roll(24), 3);
        assert_eq!(PlantKind::Bush2.visual_instance_count_for_roll(25), 2);
        assert_eq!(PlantKind::Bush2.visual_instance_count_for_roll(69), 2);
        assert_eq!(PlantKind::Bush2.visual_instance_count_for_roll(70), 1);
        assert_eq!(PlantKind::Bush4.visual_instance_count_for_roll(0), 3);
        assert_eq!(PlantKind::Bush4.visual_instance_count_for_roll(99), 1);

        assert_eq!(PlantKind::Tree1.visual_instance_count_for_roll(0), 1);
        assert_eq!(PlantKind::Stump1.visual_instance_count_for_roll(0), 1);

        let offset = PlantKind::Bush2.visual_instance_offset(3, 1);
        assert!(offset.x < 0.0);
        assert!(PlantKind::Bush2.visual_instance_offset(3, 2).x > 0.0);
    }

    #[test]
    fn seeded_cloud_generation_is_deterministic_and_subtle() {
        let clouds = load_cloud_assets();
        let a = generate_clouds(DEFAULT_SEED, &clouds, WORLD_COLS, WORLD_ROWS);
        let b = generate_clouds(DEFAULT_SEED, &clouds, WORLD_COLS, WORLD_ROWS);
        let c = generate_clouds(DEFAULT_SEED + 1, &clouds, WORLD_COLS, WORLD_ROWS);

        assert_eq!(a, b);
        assert_ne!(a, c);
        assert!((MIN_CLOUDS..=MAX_CLOUDS).contains(&a.len()));
        assert!(a.iter().all(|cloud| cloud.alpha_min >= 0.25));
        assert!(a.iter().all(|cloud| cloud.alpha_max <= 0.75));
        assert!(a.iter().all(|cloud| cloud.scale_wobble <= 0.055));
    }

    #[test]
    fn aseprite_explorer_ignores_gold_stone_helper_tags() {
        let explorer = AsepriteExplorer::new();
        let gold_stones = &explorer.files[3];

        assert_eq!(gold_stones.frames.len(), 49);
        assert_eq!(gold_stones.clips.len(), 7);
        assert_eq!(gold_stones.clips[0].name, "1");
        assert!(
            gold_stones
                .clips
                .iter()
                .all(|clip| clip.name != "Still" && clip.name != "Highlight")
        );
    }

    #[test]
    fn doubled_palette_panel_contains_all_slots_at_default_size() {
        let game = Game::new();
        let last_slot = game.palette_len() - 1;
        let (_, y) = game.palette_slot_rect(last_slot);

        assert_eq!(PANEL_H, 320.0);
        assert!(y + PALETTE_TILE <= game.window_h() - 10.0);
    }

    #[test]
    fn world_generation_collapses_sparse_seed_to_full_raw_tiles() {
        let a = wldgenerator::generate_world(100, 100, 123);
        let b = wldgenerator::generate_world(100, 100, 123);
        let c = wldgenerator::generate_world(100, 100, 124);

        assert_eq!(a.cells, b.cells);
        assert_ne!(a.cells, c.cells);
        let water_count = a
            .cells
            .iter()
            .filter(|&&cell| cell == wldgenerator::GeneratedCell::Water)
            .count();
        let grass_count = a
            .cells
            .iter()
            .filter(|&&cell| cell == wldgenerator::GeneratedCell::Grass)
            .count();
        assert!(
            a.cells
                .iter()
                .all(|&cell| cell != wldgenerator::GeneratedCell::Unknown)
        );
        assert!(water_count > 0);
        assert!(grass_count > 0);
        assert_eq!(water_count + grass_count, 100 * 100);
        assert!(
            a.cells
                .iter()
                .filter_map(|&cell| cell.background())
                .all(|background| matches!(
                    background,
                    BackgroundTile::Grass | BackgroundTile::Water
                ))
        );
    }

    #[test]
    fn seeded_generator_world_keeps_dimensions() {
        let world = wldgenerator::generate_world(WORLD_COLS, WORLD_ROWS, DEFAULT_SEED);

        assert_eq!(world.cols, WORLD_COLS);
        assert_eq!(world.rows, WORLD_ROWS);
        assert_eq!(world.cells.len(), WORLD_COLS * WORLD_ROWS);
    }
}
