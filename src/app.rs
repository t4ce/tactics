use crate::terrain_rules::AtlasTile;
use crate::{ase_assets, ts_ui};
use adapterlibgfx::api::Adapter;
use adapterlibgfx::command::{ScissorRect, TextureEffect};
use adapterlibgfx::vertex::{Rgba8, TexVertex};
use adapterlibgfx::window::{
    FrameProducer, InputButtonState, InputEvent, InputKey, InputMouseButton,
};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

#[path = "cli.rs"]
mod cli;
#[path = "loadscreen.rs"]
mod loadscreen;
#[path = "ui_cli.rs"]
mod ui_cli;
#[path = "wldgenerator.rs"]
mod wldgenerator;
#[path = "worldviewer.rs"]
mod worldviewer;

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 800;
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
const UNIT_VIEWER_TEXTURE_BASE: u32 = 5000;
const ICON_VIEWER_TEXTURE_BASE: u32 = 7000;
const IDLE_WORLD_HOUSE_ICON_TEXTURE_BASE: u32 = 8800;
const IDLE_WORLD_TOOL_CURSOR_TEXTURE_BASE: u32 = 8850;
const IDLE_WORLD_TEXTURE_BASE: u32 = 9000;
const IDLE_RETILE_COVER_TEXTURE: u32 = 13_000;
const IDLE_RETILE_COVER_BYTES: &[u8] =
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Wood Table/WoodTable_Slots.png");
const IDLE_RETILE_COVER_TILE_PX: u32 = 64;
const IDLE_RETILE_COVER_LIGHTEN_PERCENT: u8 = 8;
const IDLE_RETILE_PARTICLE_TEXTURE_BASE: u32 = 13_100;
const IDLE_RETILE_FLYOUT_MS: u32 = 500;
const IDLE_RETILE_FLYOUT_DISTANCE_PX: f32 = 192.0;
const IDLE_RETILE_COVER_HOLD_MS: u32 = 250;
const IDLE_RETILE_REVEAL_STAGGER_MS: u32 = 35;
const IDLE_RETILE_SEQUENTIAL_REVEAL_TILES: usize = 50;
const IDLE_RETILE_REVEAL_MS: u32 = 50;
const IDLE_RETILE_BOB_AMPLITUDE_PX: f32 = 2.25;
const IDLE_RETILE_BOB_PERIOD_MS: f32 = 720.0;
const IDLE_RETILE_ROTATION_AMPLITUDE_RAD: f32 = 0.0125;
const IDLE_RETILE_ROTATION_PERIOD_MS: f32 = 960.0;
const IDLE_MONK_RETILE_MIN_DISTANCE_TILES: usize = 2;
const IDLE_MONK_RETILE_MAX_DISTANCE_TILES: usize = 3;
const IDLE_MONK_RETILE_MIN_SIZE: usize = 2;
const IDLE_MONK_RETILE_MAX_SIZE: usize = 15;
const IDLE_RETILE_PATCH_PADDING_TILES: usize = 2;
const IDLE_RIGHT_CLICK_INDICATOR_MS: u128 = 250;
const IDLE_RIGHT_CLICK_INDICATOR_SIZE: f32 = 42.0;
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
const IDLE_WORLD_VIRTUAL_WIDTH: f32 = WINDOW_WIDTH as f32;
const IDLE_WORLD_VIRTUAL_HEIGHT: f32 = WINDOW_HEIGHT as f32;
const IDLE_WORLD_MIN_SELECTION_DRAG_PX: f32 = 5.0;
const IDLE_WORLD_UNIT_SPEED: f32 = 150.0;
const IDLE_WORLD_RUN_COMMAND_DISTANCE_PX: f32 = 5.0;
const IDLE_PATH_CARDINAL_COST: u32 = 10;
const IDLE_PATH_DIAGONAL_COST: u32 = 14;
const IDLE_PATH_WATER_PENALTY: u32 = 80;
const IDLE_PATH_OBJECT_PENALTY: u32 = 120;
const IDLE_PATH_CLIFF_PENALTY: u32 = 160;
const IDLE_RESOURCE_WORK_MS: u32 = 5_000;
const IDLE_TREE_MIN_HP: u8 = 5;
const IDLE_TREE_MAX_HP: u8 = 7;
const IDLE_WORLD_HOUSE_PREVIEW_TEXTURE_BASE: u32 = 8700;
const IDLE_HOTKEY_ENTRIES: [ts_ui::HotkeyEntry<'static>; 10] = [
    ts_ui::HotkeyEntry {
        prefix: None,
        key: "1",
        label: "HOUSE",
    },
    ts_ui::HotkeyEntry {
        prefix: None,
        key: "2",
        label: "BARRACKS",
    },
    ts_ui::HotkeyEntry {
        prefix: None,
        key: "3",
        label: "ARCHERY",
    },
    ts_ui::HotkeyEntry {
        prefix: None,
        key: "4",
        label: "TOWER",
    },
    ts_ui::HotkeyEntry {
        prefix: None,
        key: "5",
        label: "CASTLE",
    },
    ts_ui::HotkeyEntry {
        prefix: Some("STR"),
        key: "A",
        label: "MOVE",
    },
    ts_ui::HotkeyEntry {
        prefix: Some("SPC"),
        key: "B",
        label: "STOP",
    },
    ts_ui::HotkeyEntry {
        prefix: Some("ALT"),
        key: "C",
        label: "ATTACK",
    },
    ts_ui::HotkeyEntry {
        prefix: Some("SFT"),
        key: "D",
        label: "SELECT",
    },
    ts_ui::HotkeyEntry {
        prefix: None,
        key: "E",
        label: "CAMERA",
    },
];
const PAWN_ASEPRITE_PATH: &str = "ts_freepack/Pawn.aseprite";
const PARTICLE_FX_ASEPRITE_PATH: &str = "ts_freepack/Particle FX.aseprite";
const IDLE_WORLD_UNIT_SPECS: [UnitWalkSpec; 5] = [
    UnitWalkSpec::animated("Archer Idle", "ts_freepack/Archer.aseprite", "Idle"),
    UnitWalkSpec::animated_offset(
        "Lancer Idle",
        "ts_freepack/Lancer.aseprite",
        "Idle",
        0.0,
        -16.0,
    ),
    UnitWalkSpec::animated("Monk Idle", "ts_freepack/Monk.aseprite", "Idle"),
    UnitWalkSpec::animated("Warrior Idle", "ts_freepack/Warrior.aseprite", "Idle"),
    UnitWalkSpec::animated_offset("Sheep Idle", "ts_freepack/Sheep.aseprite", "Idle", 0.0, 8.0),
];
const IDLE_WORLD_MOVE_SPECS: [UnitWalkSpec; 5] = [
    UnitWalkSpec::animated("Archer Run", "ts_freepack/Archer.aseprite", "Run"),
    UnitWalkSpec::animated_offset(
        "Lancer Run",
        "ts_freepack/Lancer.aseprite",
        "Run",
        0.0,
        -16.0,
    ),
    UnitWalkSpec::animated("Monk Run", "ts_freepack/Monk.aseprite", "Run"),
    UnitWalkSpec::animated("Warrior Run", "ts_freepack/Warrior.aseprite", "Run"),
    UnitWalkSpec::animated_offset("Sheep Move", "ts_freepack/Sheep.aseprite", "Move", 0.0, 8.0),
];
const IDLE_WORLD_ATTACK_SPECS: [&[UnitWalkSpec]; 5] = [
    &[UnitWalkSpec::animated(
        "Archer Shoot",
        "ts_freepack/Archer.aseprite",
        "Shoot",
    )],
    &[UnitWalkSpec::animated_offset(
        "Lancer Attack",
        "ts_freepack/Lancer.aseprite",
        "Attack",
        0.0,
        -16.0,
    )],
    &[UnitWalkSpec::animated(
        "Monk Heal",
        "ts_freepack/Monk.aseprite",
        "Heal",
    )],
    &[
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
    ],
    &[],
];
const PAWN_IDLE_TAGS: [&str; 8] = [
    "Idle",
    "Idle Wood",
    "Idle Meat",
    "Idle Gold",
    "Idle Hammer",
    "Idle Axe",
    "Idle Knife",
    "Idle Pickaxe",
];
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
    ui_cli::tactics_window();
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
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
    fn invalidates_generated_source(self) -> bool {
        matches!(
            self,
            Self::Background(_) | Self::Foreground(_) | Self::Ramp(_)
        )
    }

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

    fn uses_half_height_footprint(self) -> bool {
        matches!(self, Self::Rock1)
    }

    fn visual_instance_count(self, col: usize, row: usize) -> usize {
        if !matches!(self, Self::Rock1) {
            return 1;
        }

        let roll = rock_visual_roll(self, col, row);
        if roll < 25 {
            3
        } else if roll < 70 {
            2
        } else {
            1
        }
    }

    fn visual_instance_offset(self, count: usize, instance: usize) -> Point {
        if !matches!(self, Self::Rock1) || count <= 1 {
            return Point::default();
        }

        match (count, instance) {
            (2, 0) => Point { x: -8.0, y: 1.0 },
            (2, 1) => Point { x: 8.0, y: -3.0 },
            (3, 0) => Point { x: 0.0, y: -7.0 },
            (3, 1) => Point { x: -11.0, y: 2.0 },
            (3, 2) => Point { x: 11.0, y: 3.0 },
            _ => Point::default(),
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

    fn is_tree(self) -> bool {
        matches!(self, Self::Tree1 | Self::Tree2 | Self::Tree3 | Self::Tree4)
    }

    fn stump(self) -> Option<Self> {
        match self {
            Self::Tree1 => Some(Self::Stump1),
            Self::Tree2 => Some(Self::Stump2),
            Self::Tree3 => Some(Self::Stump3),
            Self::Tree4 => Some(Self::Stump4),
            _ => None,
        }
    }

    fn is_big_bush(self) -> bool {
        matches!(self, Self::Bush1 | Self::Bush3)
    }

    fn uses_half_height_footprint(self) -> bool {
        matches!(self, Self::Bush2 | Self::Bush4)
    }

    fn render_offset_y(self) -> f32 {
        match self {
            Self::Tree1 | Self::Tree2 | Self::Tree3 | Self::Tree4 => -TILE_SIZE / 2.0,
            Self::Stump1 | Self::Stump2 | Self::Stump3 | Self::Stump4 => -TILE_SIZE / 4.0,
            Self::Bush1 | Self::Bush2 | Self::Bush3 | Self::Bush4 => 0.0,
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

    fn is_house(self) -> bool {
        matches!(self, Self::House1 | Self::House2 | Self::House3)
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

struct UnitWalkViewer {
    units: Vec<UnitWalkClip>,
    uploaded: bool,
    window_width: u32,
    window_height: u32,
    started_at: Instant,
}

struct RightClickIndicator {
    position: Point,
    started: Instant,
}

struct IdleWorldViewer {
    terrain: TextureAtlas,
    fog: ImageAsset,
    buildings: [ImageAsset; BUILDING_COUNT],
    water_visuals: WaterVisualAssets,
    plant_props: [SpriteAnimation; PLANT_PROP_COUNT],
    gold_props: [SpriteAnimation; GOLD_PROP_COUNT],
    rock_props: [ImageAsset; ROCK_PROP_COUNT],
    retile_cover: ImageAsset,
    particle_dust: SpriteAnimation,
    world: TileWorld,
    terrain_cache: TerrainDrawCache,
    cursor_default: ImageAsset,
    cursor_hover: ImageAsset,
    cursor_select: ImageAsset,
    cursor_pickaxe: ImageAsset,
    cursor_axe: ImageAsset,
    cursor_dagger: ImageAsset,
    right_click_indicator: ImageAsset,
    regular_paper: ImageAsset,
    special_paper: ImageAsset,
    house_previews: [ImageAsset; 3],
    house_icon_inlays: [ImageAsset; 3],
    clouds: Vec<ImageAsset>,
    cloud_instances: Vec<CloudInstance>,
    units: Vec<IdleWorldUnit>,
    pawn_templates: Vec<PawnClipTemplate>,
    path_worker: IdlePathWorker,
    selected_units: Vec<usize>,
    right_click_indicators: Vec<RightClickIndicator>,
    selection_start: Option<Point>,
    last_move_command: Option<IdleWorldMoveCommand>,
    placement_building: Option<BuildingKind>,
    show_hotkey_menu: bool,
    mouse: Point,
    camera: Point,
    panning: bool,
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
    attack_rng: SeededRng,
    retile_transition: Option<IdleRetileTransition>,
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

#[derive(Clone, Copy, Debug, PartialEq)]
struct IdleUnitDraw {
    index: usize,
    depth_y: f32,
    texture_id: u32,
    rect: TableRect,
    flip_x: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IdleCursorTool {
    Pickaxe,
    Axe,
    Dagger,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IdleResourceKind {
    Wood,
    Meat,
    Ore,
}

impl IdleResourceKind {
    fn tool(self) -> IdleCursorTool {
        match self {
            Self::Wood => IdleCursorTool::Axe,
            Self::Meat => IdleCursorTool::Dagger,
            Self::Ore => IdleCursorTool::Pickaxe,
        }
    }

    fn tool_idle_tag(self) -> &'static str {
        match self {
            Self::Wood => "Idle Axe",
            Self::Meat => "Idle Knife",
            Self::Ore => "Idle Pickaxe",
        }
    }

    fn carrying_idle_tag(self) -> &'static str {
        match self {
            Self::Wood => "Idle Wood",
            Self::Meat => "Idle Meat",
            Self::Ore => "Idle Gold",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct IdleResourceTarget {
    kind: IdleResourceKind,
    position: Point,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct IdleResourceTask {
    kind: IdleResourceKind,
    resource: Point,
    house: Option<Point>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum IdleWorldEvent {
    ResourceCollected(IdleResourceCollected),
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct IdleResourceCollected {
    kind: IdleResourceKind,
    position: Point,
    depleted: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IdleResourcePhase {
    ToolPickup,
    ToResource,
    ToHouse,
}

#[derive(Clone, Debug, PartialEq)]
struct IdleWorldMoveCommand {
    target: Point,
    selected_units: Vec<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct IdlePathCell {
    col: usize,
    row: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct IdlePathQueueEntry {
    estimated_total: u32,
    cost: u32,
    index: usize,
}

impl Ord for IdlePathQueueEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .estimated_total
            .cmp(&self.estimated_total)
            .then_with(|| other.cost.cmp(&self.cost))
            .then_with(|| other.index.cmp(&self.index))
    }
}

impl PartialOrd for IdlePathQueueEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum IdlePathIntent {
    Move {
        running: bool,
        started_ms: u32,
    },
    Resource {
        task: IdleResourceTask,
        phase: IdleResourcePhase,
        running: bool,
        started_ms: u32,
    },
}

#[derive(Clone, Debug)]
struct IdlePathRequest {
    unit_index: usize,
    generation: u64,
    from: Point,
    to: Point,
    intent: IdlePathIntent,
}

#[derive(Clone, Debug)]
struct IdlePathResult {
    unit_index: usize,
    generation: u64,
    to: Point,
    path: Vec<Point>,
    intent: IdlePathIntent,
}

struct IdlePathWorker {
    runtime: tokio::runtime::Runtime,
    tx: tokio::sync::mpsc::UnboundedSender<IdlePathResult>,
    rx: tokio::sync::mpsc::UnboundedReceiver<IdlePathResult>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct IdleRetileRect {
    col: usize,
    row: usize,
    cols: usize,
    rows: usize,
}

struct IdleRetileTransition {
    rect: IdleRetileRect,
    tiles: Vec<IdleRetileTile>,
    flyout_tiles: Vec<IdleRetileFlyoutTile>,
    started: Instant,
    finish_ms: u32,
}

impl IdlePathWorker {
    fn new() -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .thread_name("idle-a-star")
            .build()
            .expect("idle path worker runtime should build");
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        Self { runtime, tx, rx }
    }

    fn request(&self, world: Arc<TileWorld>, request: IdlePathRequest) {
        let tx = self.tx.clone();
        self.runtime.spawn(async move {
            let path = world.idle_move_path(request.from, request.to);
            let _ = tx.send(IdlePathResult {
                unit_index: request.unit_index,
                generation: request.generation,
                to: request.to,
                path,
                intent: request.intent,
            });
        });
    }

    fn drain_results(&mut self) -> Vec<IdlePathResult> {
        let mut results = Vec::new();
        while let Ok(result) = self.rx.try_recv() {
            results.push(result);
        }
        results
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct IdleRetileTile {
    col: usize,
    row: usize,
    reveal_start_ms: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct IdleRetileFlyoutTile {
    col: usize,
    row: usize,
    background: BackgroundTile,
    under_foreground: Option<AtlasTile>,
    foreground: Option<AtlasTile>,
    dir_x: f32,
    dir_y: f32,
}

impl IdleRetileTransition {
    fn new(
        rect: IdleRetileRect,
        seed: u64,
        dust_duration_ms: u32,
        flyout_tiles: Vec<IdleRetileFlyoutTile>,
    ) -> Self {
        let mut tiles = shuffled_idle_retile_tiles(idle_retile_rect_tiles(rect), seed);
        let dust_duration_ms = dust_duration_ms.max(1);
        let mut finish_ms =
            IDLE_RETILE_FLYOUT_MS.max(IDLE_RETILE_COVER_HOLD_MS.saturating_add(dust_duration_ms));

        let tiles = tiles
            .drain(..)
            .enumerate()
            .map(|(order, (col, row))| {
                let reveal_order = order.min(IDLE_RETILE_SEQUENTIAL_REVEAL_TILES);
                let reveal_start_ms = IDLE_RETILE_COVER_HOLD_MS.saturating_add(
                    (reveal_order as u32).saturating_mul(IDLE_RETILE_REVEAL_STAGGER_MS),
                );
                finish_ms = finish_ms.max(reveal_start_ms.saturating_add(dust_duration_ms));
                IdleRetileTile {
                    col,
                    row,
                    reveal_start_ms,
                }
            })
            .collect();

        Self {
            rect,
            tiles,
            flyout_tiles,
            started: Instant::now(),
            finish_ms,
        }
    }

    fn cover_reveal_elapsed_ms(
        &self,
        tile: IdleRetileTile,
        elapsed_ms: u32,
    ) -> Option<Option<u32>> {
        if elapsed_ms < tile.reveal_start_ms {
            return Some(None);
        }
        let elapsed = elapsed_ms - tile.reveal_start_ms;
        (elapsed < IDLE_RETILE_REVEAL_MS).then_some(Some(elapsed))
    }

    fn cover_bob_y(&self, elapsed_ms: u32) -> f32 {
        let radians = elapsed_ms as f32 / IDLE_RETILE_BOB_PERIOD_MS * std::f32::consts::TAU;
        radians.sin() * IDLE_RETILE_BOB_AMPLITUDE_PX
    }

    fn cover_rotation(&self, elapsed_ms: u32) -> f32 {
        let radians = elapsed_ms as f32 / IDLE_RETILE_ROTATION_PERIOD_MS * std::f32::consts::TAU;
        radians.sin() * IDLE_RETILE_ROTATION_AMPLITUDE_RAD
    }

    fn cover_region(&self, tile: IdleRetileTile) -> ImageRegion {
        idle_retile_cover_region_for_rect(
            tile.col.saturating_sub(self.rect.col),
            tile.row.saturating_sub(self.rect.row),
            self.rect.cols,
            self.rect.rows,
        )
    }
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

#[derive(Clone)]
struct ExplorerFrame {
    texture_id: u32,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    duration_ms: u32,
}

#[derive(Clone)]
struct UnitWalkClip {
    name: String,
    source_tag: String,
    offset_x: f32,
    offset_y: f32,
    frames: Vec<ExplorerFrame>,
    total_duration_ms: u32,
}

struct IdleWorldUnit {
    idle: UnitWalkClip,
    movement: UnitWalkClip,
    attacks: Vec<UnitWalkClip>,
    position: Point,
    movement_path: Vec<Point>,
    path_generation: u64,
    facing_left: bool,
    is_pawn: bool,
    is_monk: bool,
    state: IdleWorldUnitState,
}

#[derive(Clone)]
struct PawnClipTemplate {
    idle_tag: String,
    idle: UnitWalkClip,
    movement: UnitWalkClip,
    attacks: Vec<UnitWalkClip>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum IdleWorldUnitState {
    Idle,
    Moving {
        target: Point,
        running: bool,
        started_ms: u32,
    },
    Attacking {
        started_ms: u32,
        attack_index: usize,
    },
    ResourceMoving {
        task: IdleResourceTask,
        phase: IdleResourcePhase,
        target: Point,
        running: bool,
        started_ms: u32,
    },
    ResourceWorking {
        task: IdleResourceTask,
        started_ms: u32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DemoUnitTeam {
    Player,
    Ally,
    Neutral,
    Enemy,
    Wild,
}

impl DemoUnitTeam {
    const ALL: [Self; 5] = [
        Self::Player,
        Self::Ally,
        Self::Neutral,
        Self::Enemy,
        Self::Wild,
    ];

    fn for_unit_index(index: usize) -> Self {
        Self::ALL[index % Self::ALL.len()]
    }

    fn health_bar_color(self) -> Rgba8 {
        match self {
            Self::Player => Rgba8::new(80, 182, 255, 255),
            Self::Ally => Rgba8::new(94, 214, 127, 255),
            Self::Neutral => Rgba8::new(228, 213, 143, 255),
            Self::Enemy => Rgba8::new(238, 84, 73, 255),
            Self::Wild => Rgba8::new(190, 116, 232, 255),
        }
    }
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
    under_foregrounds: Vec<TerrainDrawCell>,
    foregrounds: Vec<TerrainDrawCell>,
    dirty: bool,
}

impl TerrainDrawCache {
    fn new() -> Self {
        Self {
            backgrounds: Vec::new(),
            under_foregrounds: Vec::new(),
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
        self.under_foregrounds.clear();
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
                if let Some(tile) = world.under_foreground(col, row) {
                    self.under_foregrounds
                        .push(TerrainDrawCell { tile, col, row });
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
    match TileWorld::load_for_editor_from_path(WORLD_SAVE_PATH) {
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
                    _ if self.world.cell_accepts_wave_animation(col, row) => {
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
                if brush.invalidates_generated_source()
                    || (brush == Brush::ClearForeground && self.world.has_tile_overlay(col, row))
                {
                    self.world.generated_source = None;
                }
                self.world.paint(col, row, brush);
                self.terrain_cache.mark_dirty();
            }
        }
    }

    fn erase_at_mouse(&mut self) {
        let Some((col, row)) = self.world_cell_at(self.mouse.x, self.mouse.y) else {
            return;
        };
        if self.world.has_tile_overlay(col, row) {
            self.world.generated_source = None;
        }
        self.world.clear_foreground(col, row);
        self.terrain_cache.mark_dirty();
    }

    fn edit_world_edge(&mut self, edge: WorldEdge) {
        self.world.generated_source = None;
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
        let mut water = SolidBatch::new(self.window_width, self.window_height);
        let mut backgrounds = SpriteBatch::new(self.window_width, self.window_height);
        let mut under_foregrounds = SpriteBatch::new(self.window_width, self.window_height);
        let mut foregrounds = SpriteBatch::new(self.window_width, self.window_height);
        let start_col = (self.camera.x / TILE_SIZE).floor().max(0.0) as usize;
        let start_row = (self.camera.y / TILE_SIZE).floor().max(0.0) as usize;
        let end_col = ((self.camera.x + self.view_w()) / TILE_SIZE).ceil() as usize + 1;
        let end_row = ((self.camera.y + self.view_h()) / TILE_SIZE).ceil() as usize + 1;

        for row in start_row..end_row.min(self.world.rows) {
            for col in start_col..end_col.min(self.world.cols) {
                if self.world.render_background(col, row) == BackgroundTile::Water {
                    water.rect(
                        VIEW_X + col as f32 * TILE_SIZE - self.camera.x,
                        VIEW_Y + row as f32 * TILE_SIZE - self.camera.y,
                        TILE_SIZE,
                        TILE_SIZE,
                        Rgba8::new(71, 171, 169, 255),
                    );
                }
            }
        }

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
            .under_foregrounds
            .iter()
            .filter(|cell| terrain_cell_visible(cell, start_col, start_row, end_col, end_row))
        {
            let x = VIEW_X + cell.col as f32 * TILE_SIZE - self.camera.x;
            let y = VIEW_Y + cell.row as f32 * TILE_SIZE - self.camera.y;
            under_foregrounds.sprite(
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

        let _ = adapter.draw_rgb_triangles_no_present(&water.bytes);
        let _ = adapter.draw_tex_triangles_no_present(self.terrain.texture_id, &backgrounds.bytes);
        self.draw_water_states(adapter);
        let _ = adapter
            .draw_tex_triangles_no_present(self.terrain.texture_id, &under_foregrounds.bytes);
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
                if self.world.cell_accepts_wave_animation(col, row) {
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
                    let instance_count = kind.visual_instance_count(prop.x2, prop.y2);
                    for instance in 0..instance_count {
                        let offset = kind.visual_instance_offset(instance_count, instance);
                        self.push_bottom_aligned_image_half(
                            &mut image_batches,
                            image,
                            prop.x2,
                            prop.y2,
                            offset.x,
                            offset.y,
                            1.0,
                            Rgba8::WHITE,
                        );
                    }
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
                let kind = RockKind::Rock1;
                let x2 = col * BUILDING_GRID_DIVISIONS;
                let y2 = row * BUILDING_GRID_DIVISIONS;
                let image = &self.rock_props[kind.index()];
                let instance_count = kind.visual_instance_count(x2, y2);
                for instance in 0..instance_count {
                    let offset = kind.visual_instance_offset(instance_count, instance);
                    self.draw_bottom_aligned_image_half(
                        adapter, image, x2, y2, offset.x, offset.y, 1.0, tint,
                    );
                }
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
        let scale = 1.0;
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
        match TileWorld::load_for_editor_from_path(WORLD_SAVE_PATH) {
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
            ImageAsset::from_png_bytes(ts_ui::SMALL_BAR_BASE_TEXTURE, ts_ui::SMALL_BAR_BASE_BYTES),
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

impl IdleWorldViewer {
    fn new() -> Self {
        let mut next_texture_id = IDLE_WORLD_TEXTURE_BASE;
        let units = load_idle_world_units(&mut next_texture_id);
        let pawn_templates = units
            .iter()
            .filter(|unit| unit.is_pawn)
            .map(|unit| PawnClipTemplate {
                idle_tag: unit.idle.source_tag.clone(),
                idle: unit.idle.clone(),
                movement: unit.movement.clone(),
                attacks: unit.attacks.clone(),
            })
            .collect::<Vec<_>>();
        let world = initial_editor_world();
        let water_visuals = load_water_visual_assets();
        let plant_props = load_plant_prop_assets();
        let gold_props = load_gold_prop_assets();
        let rock_props = load_rock_prop_assets();
        let clouds = load_cloud_assets();
        let cloud_instances =
            generate_clouds(DEFAULT_SEED ^ 0x1D1E_2026, &clouds, world.cols, world.rows);
        let mut retile_cover =
            ImageAsset::from_png_bytes(IDLE_RETILE_COVER_TEXTURE, IDLE_RETILE_COVER_BYTES);
        lighten_rgba_toward_white(&mut retile_cover.rgba, IDLE_RETILE_COVER_LIGHTEN_PERCENT);

        Self {
            terrain: TextureAtlas::from_png_bytes(TERRAIN_TEXTURE, TERRAIN_BYTES, TERRAIN_TILE_PX),
            fog: ImageAsset::from_png_bytes_cropped(FOG_TEXTURE, FOG_BYTES),
            buildings: std::array::from_fn(|index| {
                let spec = BUILDING_SPECS[index];
                ImageAsset::from_png_bytes(spec.texture_id, spec.bytes)
            }),
            water_visuals,
            plant_props,
            gold_props,
            rock_props,
            retile_cover,
            particle_dust: load_idle_retile_particle_dust(),
            world,
            terrain_cache: TerrainDrawCache::new(),
            cursor_default: ImageAsset::from_png_bytes_cropped(
                CURSOR_DEFAULT_TEXTURE,
                CURSOR_DEFAULT_BYTES,
            ),
            cursor_hover: ImageAsset::from_png_bytes_cropped(
                CURSOR_HOVER_TEXTURE,
                CURSOR_HOVER_BYTES,
            ),
            cursor_select: ImageAsset::from_png_bytes(CURSOR_SELECT_TEXTURE, CURSOR_SELECT_BYTES),
            cursor_pickaxe: ImageAsset::from_png_bytes_cropped(
                IDLE_WORLD_TOOL_CURSOR_TEXTURE_BASE,
                TOOL2_BYTES,
            ),
            cursor_axe: ImageAsset::from_png_bytes_cropped(
                IDLE_WORLD_TOOL_CURSOR_TEXTURE_BASE + 1,
                TOOL3_BYTES,
            ),
            cursor_dagger: ImageAsset::from_png_bytes_cropped(
                IDLE_WORLD_TOOL_CURSOR_TEXTURE_BASE + 2,
                TOOL4_BYTES,
            ),
            right_click_indicator: ImageAsset::from_png_bytes(
                ts_ui::SMALL_BLUE_ROUND_BUTTON_TEXTURE,
                ts_ui::SMALL_BLUE_ROUND_BUTTON_BYTES,
            ),
            regular_paper: ImageAsset::from_png_bytes(
                ts_ui::REGULAR_PAPER_TEXTURE,
                ts_ui::REGULAR_PAPER_BYTES,
            ),
            special_paper: ImageAsset::from_png_bytes(
                ts_ui::SPECIAL_PAPER_TEXTURE,
                ts_ui::SPECIAL_PAPER_BYTES,
            ),
            house_previews: [
                ImageAsset::from_png_bytes_cropped(
                    IDLE_WORLD_HOUSE_PREVIEW_TEXTURE_BASE,
                    RED_HOUSE1_BYTES,
                ),
                ImageAsset::from_png_bytes_cropped(
                    IDLE_WORLD_HOUSE_PREVIEW_TEXTURE_BASE + 1,
                    RED_HOUSE2_BYTES,
                ),
                ImageAsset::from_png_bytes_cropped(
                    IDLE_WORLD_HOUSE_PREVIEW_TEXTURE_BASE + 2,
                    RED_HOUSE3_BYTES,
                ),
            ],
            house_icon_inlays: [
                ImageAsset::from_png_bytes_cropped(
                    IDLE_WORLD_HOUSE_ICON_TEXTURE_BASE,
                    UI_ICON_BYTES[1],
                ),
                ImageAsset::from_png_bytes_cropped(
                    IDLE_WORLD_HOUSE_ICON_TEXTURE_BASE + 1,
                    UI_ICON_BYTES[2],
                ),
                ImageAsset::from_png_bytes_cropped(
                    IDLE_WORLD_HOUSE_ICON_TEXTURE_BASE + 2,
                    UI_ICON_BYTES[3],
                ),
            ],
            clouds,
            cloud_instances,
            units,
            pawn_templates,
            path_worker: IdlePathWorker::new(),
            selected_units: Vec::new(),
            right_click_indicators: Vec::new(),
            selection_start: None,
            last_move_command: None,
            placement_building: None,
            show_hotkey_menu: false,
            mouse: Point::default(),
            camera: Point::default(),
            panning: false,
            uploaded: false,
            window_width: WINDOW_WIDTH,
            window_height: WINDOW_HEIGHT,
            environment_rng: SeededRng::new(DEFAULT_SEED ^ 0x1D1E_EA57_2026),
            water_animations: Vec::new(),
            plant_animations: Vec::new(),
            gold_animations: Vec::new(),
            water_animation_timer: 0.2,
            plant_animation_timer: 0.4,
            gold_animation_timer: 0.6,
            started_at: Instant::now(),
            last_frame: Instant::now(),
            attack_rng: SeededRng::new(DEFAULT_SEED ^ 0xA77A_CAFE_2026),
            retile_transition: None,
        }
    }

    fn resize_view(&mut self, width: u32, height: u32) {
        self.window_width = width.max(1);
        self.window_height = height.max(1);
        self.clamp_idle_camera();
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
        assert_eq!(rc, 0, "failed to upload idle world terrain texture");

        let small_bar_base =
            ImageAsset::from_png_bytes(ts_ui::SMALL_BAR_BASE_TEXTURE, ts_ui::SMALL_BAR_BASE_BYTES);
        let banner = ImageAsset::from_png_bytes(ts_ui::BANNER_TEXTURE, ts_ui::BANNER_BYTES);
        let hotkey_button = ImageAsset::from_png_bytes(
            ts_ui::SMALL_BLUE_SQUARE_BUTTON_TEXTURE,
            ts_ui::SMALL_BLUE_SQUARE_BUTTON_BYTES,
        );
        let big_ribbons =
            ImageAsset::from_png_bytes(ts_ui::BIG_RIBBONS_TEXTURE, ts_ui::BIG_RIBBONS_BYTES);
        for image in [
            &self.cursor_default,
            &self.cursor_hover,
            &self.cursor_select,
            &self.cursor_pickaxe,
            &self.cursor_axe,
            &self.cursor_dagger,
            &self.right_click_indicator,
            &self.fog,
            &self.retile_cover,
            &self.regular_paper,
            &self.special_paper,
            &small_bar_base,
            &banner,
            &hotkey_button,
            &big_ribbons,
        ] {
            let rc = adapter.upload_texture_rgba_image(
                image.texture_id,
                image.width,
                image.height,
                &image.rgba,
            );
            assert_eq!(
                rc, 0,
                "failed to upload idle world ui texture {}",
                image.texture_id
            );
        }

        for image in self
            .house_previews
            .iter()
            .chain(self.house_icon_inlays.iter())
        {
            let rc = adapter.upload_texture_rgba_image(
                image.texture_id,
                image.width,
                image.height,
                &image.rgba,
            );
            assert_eq!(
                rc, 0,
                "failed to upload idle world house preview texture {}",
                image.texture_id
            );
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
                "failed to upload idle world building texture {}",
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
                "failed to upload idle world water texture {}",
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
            .chain(self.particle_dust.frames.iter())
        {
            let rc = adapter.upload_texture_rgba_image(
                image.texture_id,
                image.width,
                image.height,
                &image.rgba,
            );
            assert_eq!(
                rc, 0,
                "failed to upload idle world prop texture {}",
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
            assert_eq!(
                rc, 0,
                "failed to upload idle world cloud texture {}",
                image.texture_id
            );
        }

        for unit in &self.units {
            for frame in unit
                .idle
                .frames
                .iter()
                .chain(unit.movement.frames.iter())
                .chain(unit.attacks.iter().flat_map(|clip| clip.frames.iter()))
            {
                let rc = adapter.upload_texture_rgba_image(
                    frame.texture_id,
                    frame.width,
                    frame.height,
                    &frame.rgba,
                );
                assert_eq!(
                    rc, 0,
                    "failed to upload idle world texture {}",
                    frame.texture_id
                );
            }
        }

        self.uploaded = true;
    }

    fn elapsed_ms(&self) -> u32 {
        self.started_at.elapsed().as_millis() as u32
    }

    fn idle_view_w(&self) -> f32 {
        self.window_width as f32
    }

    fn idle_view_h(&self) -> f32 {
        self.window_height as f32
    }

    fn clamp_idle_camera(&mut self) {
        let max_x = (self.world.width_px() - self.idle_view_w()).max(0.0);
        let max_y = (self.world.height_px() - self.idle_view_h()).max(0.0);
        self.camera.x = self.camera.x.clamp(0.0, max_x);
        self.camera.y = self.camera.y.clamp(0.0, max_y);
    }

    fn scroll_idle_camera(&mut self, dx: f32, dy: f32) {
        self.camera.x += dx;
        self.camera.y += dy;
        self.clamp_idle_camera();
    }

    fn update_idle_camera(&mut self, dt: f32) {
        if self.show_hotkey_menu
            || !inside_rect(
                self.mouse.x,
                self.mouse.y,
                0.0,
                0.0,
                self.idle_view_w(),
                self.idle_view_h(),
            )
        {
            return;
        }

        let mut dx = 0.0;
        let mut dy = 0.0;
        if self.mouse.x < EDGE_SCROLL_ZONE {
            dx = -scroll_strength(self.mouse.x);
        } else if self.mouse.x > self.idle_view_w() - EDGE_SCROLL_ZONE {
            dx = scroll_strength(self.idle_view_w() - self.mouse.x);
        }

        if self.mouse.y < EDGE_SCROLL_ZONE {
            dy = -scroll_strength(self.mouse.y);
        } else if self.mouse.y > self.idle_view_h() - EDGE_SCROLL_ZONE {
            dy = scroll_strength(self.idle_view_h() - self.mouse.y);
        }

        self.scroll_idle_camera(dx * EDGE_SCROLL_SPEED * dt, dy * EDGE_SCROLL_SPEED * dt);
    }

    fn idle_world_point_at(&self, point: Point) -> Point {
        Point {
            x: point.x + self.camera.x,
            y: point.y + self.camera.y,
        }
    }

    fn next_unit_path_generation(unit: &mut IdleWorldUnit) -> u64 {
        unit.path_generation = unit.path_generation.wrapping_add(1);
        unit.path_generation
    }

    fn queue_idle_path_requests(&mut self, requests: Vec<IdlePathRequest>) {
        if requests.is_empty() {
            return;
        }

        let world = Arc::new(self.world.clone());
        for request in requests {
            self.path_worker.request(Arc::clone(&world), request);
        }
    }

    fn apply_idle_path_results(&mut self) {
        for result in self.path_worker.drain_results() {
            let Some(unit) = self.units.get_mut(result.unit_index) else {
                continue;
            };
            if unit.path_generation != result.generation {
                continue;
            }

            let mut path = result.path;
            while path
                .first()
                .is_some_and(|&target| point_distance(unit.position, target) <= 1.0)
            {
                path.remove(0);
            }
            let target = if path.is_empty() {
                result.to
            } else {
                path.remove(0)
            };

            match (result.intent, unit.state) {
                (
                    IdlePathIntent::Move {
                        running,
                        started_ms,
                    },
                    IdleWorldUnitState::Moving { .. },
                ) => {
                    unit.movement_path = path;
                    let dx = target.x - unit.position.x;
                    if dx.abs() > 0.5 {
                        unit.facing_left = dx < 0.0;
                    }
                    unit.state = IdleWorldUnitState::Moving {
                        target,
                        running,
                        started_ms,
                    };
                }
                (
                    IdlePathIntent::Resource {
                        task,
                        phase,
                        running,
                        started_ms,
                    },
                    IdleWorldUnitState::ResourceMoving {
                        task: current_task,
                        phase: current_phase,
                        ..
                    },
                ) if task == current_task && phase == current_phase => {
                    unit.movement_path = path;
                    let dx = target.x - unit.position.x;
                    if dx.abs() > 0.5 {
                        unit.facing_left = dx < 0.0;
                    }
                    unit.state = IdleWorldUnitState::ResourceMoving {
                        task,
                        phase,
                        target,
                        running,
                        started_ms,
                    };
                }
                _ => {}
            }
        }
    }

    fn update_units(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f32().min(0.05);
        self.last_frame = now;
        let elapsed_ms = self.elapsed_ms();
        self.update_idle_environment(dt, elapsed_ms);
        self.update_idle_camera(dt);
        self.update_right_click_indicators();
        self.update_idle_retile_transition();
        self.apply_idle_path_results();
        let attack_rng = &mut self.attack_rng;

        let mut pawn_clip_changes = Vec::new();
        let mut monk_retile_requests = Vec::new();
        let mut path_requests = Vec::new();
        let mut idle_world_events = Vec::new();

        for (index, unit) in self.units.iter_mut().enumerate() {
            match unit.state {
                IdleWorldUnitState::Idle => {}
                IdleWorldUnitState::Moving {
                    target,
                    running,
                    started_ms,
                } => {
                    let dx = target.x - unit.position.x;
                    let dy = target.y - unit.position.y;
                    let distance = (dx * dx + dy * dy).sqrt();
                    let speed = if running {
                        IDLE_WORLD_UNIT_SPEED
                    } else {
                        IDLE_WORLD_UNIT_SPEED * 0.5
                    };
                    let step = speed * dt;
                    if distance <= step.max(1.0) {
                        unit.position = target;
                        if !unit.movement_path.is_empty() {
                            let target = unit.movement_path.remove(0);
                            unit.state = IdleWorldUnitState::Moving {
                                target,
                                running,
                                started_ms,
                            };
                            continue;
                        }
                        unit.state = if unit.attacks.is_empty() {
                            IdleWorldUnitState::Idle
                        } else {
                            if unit.is_monk {
                                monk_retile_requests.push((index, unit.position));
                            }
                            IdleWorldUnitState::Attacking {
                                started_ms: elapsed_ms,
                                attack_index: attack_rng.range_usize(0, unit.attacks.len()),
                            }
                        };
                    } else if distance > 0.0 {
                        unit.facing_left = dx < 0.0;
                        unit.position.x += dx / distance * step;
                        unit.position.y += dy / distance * step;
                    }
                }
                IdleWorldUnitState::Attacking {
                    started_ms,
                    attack_index,
                } => {
                    if unit.attacks.get(attack_index).is_none_or(|clip| {
                        elapsed_ms.saturating_sub(started_ms) >= clip.total_duration_ms
                    }) {
                        unit.state = IdleWorldUnitState::Idle;
                    }
                }
                IdleWorldUnitState::ResourceMoving {
                    task,
                    phase,
                    target,
                    running,
                    started_ms,
                } => {
                    let dx = target.x - unit.position.x;
                    let dy = target.y - unit.position.y;
                    let distance = (dx * dx + dy * dy).sqrt();
                    let speed = if running {
                        IDLE_WORLD_UNIT_SPEED
                    } else {
                        IDLE_WORLD_UNIT_SPEED * 0.5
                    };
                    let step = speed * dt;
                    if distance <= step.max(1.0) {
                        unit.position = target;
                        if !unit.movement_path.is_empty() {
                            let target = unit.movement_path.remove(0);
                            unit.state = IdleWorldUnitState::ResourceMoving {
                                task,
                                phase,
                                target,
                                running,
                                started_ms,
                            };
                            continue;
                        }
                        unit.state = match phase {
                            IdleResourcePhase::ToolPickup => {
                                pawn_clip_changes.push((index, task.kind.tool_idle_tag()));
                                if !self.world.idle_resource_available(task) {
                                    Self::next_unit_path_generation(unit);
                                    unit.movement_path.clear();
                                    IdleWorldUnitState::Idle
                                } else {
                                    let generation = Self::next_unit_path_generation(unit);
                                    unit.movement_path.clear();
                                    path_requests.push(IdlePathRequest {
                                        unit_index: index,
                                        generation,
                                        from: unit.position,
                                        to: task.resource,
                                        intent: IdlePathIntent::Resource {
                                            task,
                                            phase: IdleResourcePhase::ToResource,
                                            running: true,
                                            started_ms: elapsed_ms,
                                        },
                                    });
                                    IdleWorldUnitState::ResourceMoving {
                                        task,
                                        phase: IdleResourcePhase::ToResource,
                                        target: task.resource,
                                        running: true,
                                        started_ms: elapsed_ms,
                                    }
                                }
                            }
                            IdleResourcePhase::ToResource => {
                                Self::next_unit_path_generation(unit);
                                unit.movement_path.clear();
                                if self.world.idle_resource_available(task) {
                                    IdleWorldUnitState::ResourceWorking {
                                        task,
                                        started_ms: elapsed_ms,
                                    }
                                } else {
                                    IdleWorldUnitState::Idle
                                }
                            }
                            IdleResourcePhase::ToHouse => {
                                pawn_clip_changes.push((index, task.kind.tool_idle_tag()));
                                if !self.world.idle_resource_available(task) {
                                    Self::next_unit_path_generation(unit);
                                    unit.movement_path.clear();
                                    IdleWorldUnitState::Idle
                                } else {
                                    let generation = Self::next_unit_path_generation(unit);
                                    unit.movement_path.clear();
                                    path_requests.push(IdlePathRequest {
                                        unit_index: index,
                                        generation,
                                        from: unit.position,
                                        to: task.resource,
                                        intent: IdlePathIntent::Resource {
                                            task,
                                            phase: IdleResourcePhase::ToResource,
                                            running: true,
                                            started_ms: elapsed_ms,
                                        },
                                    });
                                    IdleWorldUnitState::ResourceMoving {
                                        task,
                                        phase: IdleResourcePhase::ToResource,
                                        target: task.resource,
                                        running: true,
                                        started_ms: elapsed_ms,
                                    }
                                }
                            }
                        };
                    } else if distance > 0.0 {
                        unit.facing_left = dx < 0.0;
                        unit.position.x += dx / distance * step;
                        unit.position.y += dy / distance * step;
                    }
                }
                IdleWorldUnitState::ResourceWorking { task, started_ms } => {
                    if elapsed_ms.saturating_sub(started_ms) >= IDLE_RESOURCE_WORK_MS {
                        if let Some(event) = self.world.collect_idle_resource(task) {
                            idle_world_events.push(event);
                            pawn_clip_changes.push((index, task.kind.carrying_idle_tag()));
                            unit.state = if let Some(house) = task.house {
                                let generation = Self::next_unit_path_generation(unit);
                                unit.movement_path.clear();
                                path_requests.push(IdlePathRequest {
                                    unit_index: index,
                                    generation,
                                    from: unit.position,
                                    to: house,
                                    intent: IdlePathIntent::Resource {
                                        task,
                                        phase: IdleResourcePhase::ToHouse,
                                        running: false,
                                        started_ms: elapsed_ms,
                                    },
                                });
                                IdleWorldUnitState::ResourceMoving {
                                    task,
                                    phase: IdleResourcePhase::ToHouse,
                                    target: house,
                                    running: false,
                                    started_ms: elapsed_ms,
                                }
                            } else {
                                Self::next_unit_path_generation(unit);
                                unit.movement_path.clear();
                                IdleWorldUnitState::Idle
                            };
                        } else {
                            Self::next_unit_path_generation(unit);
                            unit.movement_path.clear();
                            unit.state = IdleWorldUnitState::Idle;
                        };
                    }
                }
            }
        }

        self.handle_idle_world_events(&idle_world_events);

        for (index, idle_tag) in pawn_clip_changes {
            self.apply_pawn_idle_tag(index, idle_tag);
        }

        self.queue_idle_path_requests(path_requests);

        if let Some((index, position)) = monk_retile_requests.into_iter().next() {
            self.proc_monk_retile_from_attack(index, position, elapsed_ms);
        }
    }

    fn handle_idle_world_events(&mut self, events: &[IdleWorldEvent]) {
        for event in events {
            let IdleWorldEvent::ResourceCollected(collected) = *event;
            if collected.kind == IdleResourceKind::Wood && collected.depleted {
                self.plant_animations.retain(|animation| {
                    let position = idle_resource_prop_target_in_world(
                        self.world.cols,
                        self.world.rows,
                        PlacedProp::new(
                            PropKind::Plant(animation.kind),
                            animation.col,
                            animation.row,
                        ),
                    );
                    point_distance(position, collected.position) > 1.0
                });
            }
        }
    }

    fn update_idle_retile_transition(&mut self) {
        let Some(transition) = &self.retile_transition else {
            return;
        };
        let elapsed_ms = transition.started.elapsed().as_millis() as u32;
        if elapsed_ms >= transition.finish_ms {
            self.retile_transition = None;
        }
    }

    fn proc_monk_retile_from_attack(&mut self, index: usize, position: Point, elapsed_ms: u32) {
        if self.retile_transition.is_some() {
            return;
        }
        if self.units.get(index).is_none_or(|unit| !unit.is_monk) {
            return;
        }

        let rect = self.random_monk_retile_rect(position);
        let seed = DEFAULT_SEED
            ^ 0x1D1E_2E71_1EAF_2026
            ^ self.environment_rng.next_u64()
            ^ ((elapsed_ms as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let Some(rect) = self.begin_idle_retile(rect, seed) else {
            return;
        };

        if let Some(unit) = self.units.get_mut(index) {
            let rect_center_x = (rect.col as f32 + rect.cols as f32 * 0.5) * TILE_SIZE;
            unit.facing_left = rect_center_x < unit.position.x;
        }
    }

    fn random_monk_retile_rect(&mut self, position: Point) -> IdleRetileRect {
        const DIRECTIONS: [(isize, isize); 8] = [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ];

        let max_cols = IDLE_MONK_RETILE_MAX_SIZE.min(self.world.cols);
        let max_rows = IDLE_MONK_RETILE_MAX_SIZE.min(self.world.rows);
        let min_cols = IDLE_MONK_RETILE_MIN_SIZE.min(max_cols).max(1);
        let min_rows = IDLE_MONK_RETILE_MIN_SIZE.min(max_rows).max(1);
        let cols = self.environment_rng.range_usize(min_cols, max_cols + 1);
        let rows = self.environment_rng.range_usize(min_rows, max_rows + 1);
        let distance = self.environment_rng.range_usize(
            IDLE_MONK_RETILE_MIN_DISTANCE_TILES,
            IDLE_MONK_RETILE_MAX_DISTANCE_TILES + 1,
        ) as isize;
        let (dir_x, dir_y) = DIRECTIONS[self.environment_rng.range_usize(0, DIRECTIONS.len())];
        let monk_col = (position.x / TILE_SIZE).floor() as isize;
        let monk_row = (position.y / TILE_SIZE).floor() as isize;

        let col = if dir_x < 0 {
            monk_col - distance - cols as isize + 1
        } else if dir_x > 0 {
            monk_col + distance
        } else {
            monk_col - cols as isize / 2
        };
        let row = if dir_y < 0 {
            monk_row - distance - rows as isize + 1
        } else if dir_y > 0 {
            monk_row + distance
        } else {
            monk_row - rows as isize / 2
        };

        IdleRetileRect {
            col: col.clamp(0, self.world.cols.saturating_sub(cols) as isize) as usize,
            row: row.clamp(0, self.world.rows.saturating_sub(rows) as isize) as usize,
            cols,
            rows,
        }
    }

    fn begin_idle_retile(&mut self, rect: IdleRetileRect, seed: u64) -> Option<IdleRetileRect> {
        if self.retile_transition.is_some() {
            return None;
        }
        let rect = clamp_idle_retile_rect(rect, self.world.cols, self.world.rows)?;
        let flyout_tiles = idle_retile_flyout_tiles(&self.world, rect, seed);
        self.world.replace_with_generated_rect(rect, seed);
        self.terrain_cache.mark_dirty();
        self.retile_transition = Some(IdleRetileTransition::new(
            rect,
            seed,
            self.particle_dust.total_duration_ms,
            flyout_tiles,
        ));
        Some(rect)
    }

    fn update_idle_environment(&mut self, dt: f32, elapsed_ms: u32) {
        self.update_idle_plant_environment(dt, elapsed_ms);
        self.update_idle_gold_environment(dt, elapsed_ms);
        self.update_idle_water_environment(dt, elapsed_ms);
    }

    fn update_idle_plant_environment(&mut self, dt: f32, elapsed_ms: u32) {
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

    fn update_idle_gold_environment(&mut self, dt: f32, elapsed_ms: u32) {
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

    fn update_idle_water_environment(&mut self, dt: f32, elapsed_ms: u32) {
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
                    _ if self.world.cell_accepts_wave_animation(col, row) => {
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

    fn unit_clip_and_elapsed(unit: &IdleWorldUnit, elapsed_ms: u32) -> (&UnitWalkClip, u32) {
        match unit.state {
            IdleWorldUnitState::Idle => (&unit.idle, elapsed_ms),
            IdleWorldUnitState::Moving {
                started_ms,
                running,
                ..
            }
            | IdleWorldUnitState::ResourceMoving {
                started_ms,
                running,
                ..
            } => {
                let movement_elapsed = elapsed_ms.saturating_sub(started_ms);
                let movement_elapsed = if running {
                    movement_elapsed
                } else {
                    movement_elapsed / 2
                };
                (&unit.movement, movement_elapsed)
            }
            IdleWorldUnitState::Attacking {
                started_ms,
                attack_index,
            } => unit
                .attacks
                .get(attack_index)
                .map(|clip| (clip, elapsed_ms.saturating_sub(started_ms)))
                .unwrap_or((&unit.idle, elapsed_ms)),
            IdleWorldUnitState::ResourceWorking { started_ms, .. } => unit
                .attacks
                .first()
                .map(|clip| {
                    let elapsed = elapsed_ms.saturating_sub(started_ms);
                    let looped = if clip.total_duration_ms > 0 {
                        elapsed % clip.total_duration_ms
                    } else {
                        elapsed
                    };
                    (clip, looped)
                })
                .unwrap_or((&unit.idle, elapsed_ms)),
        }
    }

    fn unit_rect(&self, index: usize, elapsed_ms: u32) -> Option<(TableRect, &ExplorerFrame)> {
        let unit = self.units.get(index)?;
        let (clip, clip_elapsed_ms) = Self::unit_clip_and_elapsed(unit, elapsed_ms);
        let frame = unit_walk_frame(clip, clip_elapsed_ms);
        let scale = 1.0;
        let w = frame.width as f32 * scale;
        let h = frame.height as f32 * scale;
        Some((
            TableRect {
                x: unit.position.x - self.camera.x - w * 0.5 + clip.offset_x,
                y: unit.position.y - self.camera.y - h + clip.offset_y,
                w,
                h,
            },
            frame,
        ))
    }

    fn unit_draws(&self, elapsed_ms: u32) -> Vec<IdleUnitDraw> {
        let mut draws = self
            .units
            .iter()
            .enumerate()
            .filter_map(|(index, _)| {
                let (rect, frame) = self.unit_rect(index, elapsed_ms)?;
                Some(IdleUnitDraw {
                    index,
                    depth_y: rect.y + rect.h,
                    texture_id: frame.texture_id,
                    rect,
                    flip_x: self.units[index].facing_left,
                })
            })
            .collect::<Vec<_>>();
        draws.sort_by(|a, b| {
            a.depth_y
                .partial_cmp(&b.depth_y)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        draws
    }

    fn unit_at_point(&self, point: Point, elapsed_ms: u32) -> Option<usize> {
        for draw in self.unit_draws(elapsed_ms).iter().rev() {
            let (rect, frame) = self.unit_rect(draw.index, elapsed_ms)?;
            if !inside_rect(point.x, point.y, rect.x, rect.y, rect.w, rect.h) {
                continue;
            }

            let px = ((point.x - rect.x) / rect.w * frame.width as f32).floor() as usize;
            let py = ((point.y - rect.y) / rect.h * frame.height as f32).floor() as usize;
            if px >= frame.width as usize || py >= frame.height as usize {
                continue;
            }
            let alpha = frame.rgba[(py * frame.width as usize + px) * 4 + 3];
            if alpha != 0 {
                return Some(draw.index);
            }
        }

        None
    }

    fn handle_left_click(&mut self) {
        self.selected_units.clear();
        if let Some(index) = self.unit_at_point(self.mouse, self.elapsed_ms()) {
            self.selected_units.push(index);
        }
    }

    fn start_selection(&mut self) {
        self.selection_start = Some(self.mouse);
    }

    fn finish_selection(&mut self) {
        if self.selection_drag_is_large_enough() {
            self.selected_units = self.units_in_selection_rect(self.elapsed_ms());
        } else {
            self.handle_left_click();
        }
        self.selection_start = None;
    }

    fn cancel_selection(&mut self) {
        self.selection_start = None;
        self.selected_units.clear();
    }

    fn hotkey_menu_rect(&self) -> TableRect {
        let (w, h) = ts_ui::hotkey_menu_page_size(IDLE_HOTKEY_ENTRIES.len());
        TableRect {
            x: ((self.window_width as f32 - w) * 0.5).round(),
            y: ((self.window_height as f32 - h) * 0.5).round(),
            w,
            h,
        }
    }

    fn mouse_inside_hotkey_menu(&self) -> bool {
        let rect = self.hotkey_menu_rect();
        inside_rect(self.mouse.x, self.mouse.y, rect.x, rect.y, rect.w, rect.h)
    }

    fn move_command_is_repeat_run(&self, target: Point) -> bool {
        let Some(command) = &self.last_move_command else {
            return false;
        };
        command.selected_units == self.selected_units
            && point_distance(command.target, target) <= IDLE_WORLD_RUN_COMMAND_DISTANCE_PX
    }

    fn issue_move_order(&mut self, target: Point) {
        if self.selected_units.is_empty() {
            self.cancel_selection();
            return;
        }
        let running = self.move_command_is_repeat_run(target);
        let selected_units = self.selected_units.clone();
        let started_ms = self.elapsed_ms();

        let mut center = Point::default();
        let mut count = 0.0;
        for &index in &selected_units {
            if let Some(unit) = self.units.get(index) {
                center.x += unit.position.x;
                center.y += unit.position.y;
                count += 1.0;
            }
        }
        if count <= 0.0 {
            return;
        }
        center.x /= count;
        center.y /= count;

        let mut path_requests = Vec::new();
        for &index in &selected_units {
            if let Some(unit) = self.units.get_mut(index) {
                let target = Point {
                    x: target.x + unit.position.x - center.x,
                    y: target.y + unit.position.y - center.y,
                };
                let dx = target.x - unit.position.x;
                if dx.abs() > 0.5 {
                    unit.facing_left = dx < 0.0;
                }
                let generation = Self::next_unit_path_generation(unit);
                unit.movement_path.clear();
                path_requests.push(IdlePathRequest {
                    unit_index: index,
                    generation,
                    from: unit.position,
                    to: target,
                    intent: IdlePathIntent::Move {
                        running,
                        started_ms,
                    },
                });
                unit.state = IdleWorldUnitState::Moving {
                    target,
                    running,
                    started_ms,
                };
            }
        }
        self.queue_idle_path_requests(path_requests);

        self.last_move_command = Some(IdleWorldMoveCommand {
            target,
            selected_units,
        });
    }

    fn issue_resource_order(&mut self, resource_target: IdleResourceTarget) {
        if !self.selected_units_are_only_pawns() {
            return;
        }

        let selected_units = self.selected_units.clone();
        let started_ms = self.elapsed_ms();
        let mut pawn_clip_changes = Vec::new();
        let mut path_requests = Vec::new();

        for index in selected_units {
            let Some(unit) = self.units.get(index) else {
                continue;
            };
            let position = unit.position;
            let has_tool = unit.idle.source_tag.trim() == resource_target.kind.tool_idle_tag();
            let house = self.nearest_idle_house(position);

            let (phase, target) = if has_tool {
                (IdleResourcePhase::ToResource, resource_target.position)
            } else if let Some(house) = house {
                (IdleResourcePhase::ToolPickup, house)
            } else {
                pawn_clip_changes.push((index, resource_target.kind.tool_idle_tag()));
                (IdleResourcePhase::ToResource, resource_target.position)
            };

            if let Some(unit) = self.units.get_mut(index) {
                let dx = target.x - unit.position.x;
                if dx.abs() > 0.5 {
                    unit.facing_left = dx < 0.0;
                }
                let generation = Self::next_unit_path_generation(unit);
                unit.movement_path.clear();
                let task = IdleResourceTask {
                    kind: resource_target.kind,
                    resource: resource_target.position,
                    house,
                };
                path_requests.push(IdlePathRequest {
                    unit_index: index,
                    generation,
                    from: unit.position,
                    to: target,
                    intent: IdlePathIntent::Resource {
                        task,
                        phase,
                        running: true,
                        started_ms,
                    },
                });
                unit.state = IdleWorldUnitState::ResourceMoving {
                    task,
                    phase,
                    target,
                    running: true,
                    started_ms,
                };
            }
        }

        for (index, idle_tag) in pawn_clip_changes {
            self.apply_pawn_idle_tag(index, idle_tag);
        }
        self.queue_idle_path_requests(path_requests);
        self.last_move_command = None;
    }

    fn apply_pawn_idle_tag(&mut self, index: usize, idle_tag: &str) {
        let Some(template) = self
            .pawn_templates
            .iter()
            .find(|template| template.idle_tag.trim() == idle_tag)
            .cloned()
        else {
            return;
        };
        let Some(unit) = self.units.get_mut(index) else {
            return;
        };
        if !unit.is_pawn {
            return;
        }
        unit.idle = template.idle;
        unit.movement = template.movement;
        unit.attacks = template.attacks;
    }

    fn nearest_idle_house(&self, from: Point) -> Option<Point> {
        self.world
            .buildings
            .iter()
            .filter(|building| building.kind.is_house())
            .map(|building| self.idle_building_foot_target(*building))
            .min_by(|a, b| {
                point_distance(from, *a)
                    .partial_cmp(&point_distance(from, *b))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    fn idle_building_foot_target(&self, building: PlacedBuilding) -> Point {
        let (x2, y2, w2, h2) = building_footprint_rect2(building);
        Point {
            x: signed_half_grid_to_px(x2) + half_grid_to_px(w2) * 0.5,
            y: signed_half_grid_to_px(y2) + half_grid_to_px(h2),
        }
    }

    fn spawn_right_click_indicator(&mut self) {
        self.right_click_indicators.push(RightClickIndicator {
            position: self.mouse,
            started: Instant::now(),
        });
    }

    fn update_right_click_indicators(&mut self) {
        self.right_click_indicators.retain(|indicator| {
            indicator.started.elapsed().as_millis() < IDLE_RIGHT_CLICK_INDICATOR_MS
        });
    }

    fn selection_drag_is_large_enough(&self) -> bool {
        let Some(start) = self.selection_start else {
            return false;
        };
        (self.mouse.x - start.x).abs() >= IDLE_WORLD_MIN_SELECTION_DRAG_PX
            || (self.mouse.y - start.y).abs() >= IDLE_WORLD_MIN_SELECTION_DRAG_PX
    }

    fn selection_rect(&self) -> Option<TableRect> {
        let start = self.selection_start?;
        let x0 = start.x.min(self.mouse.x);
        let y0 = start.y.min(self.mouse.y);
        let x1 = start.x.max(self.mouse.x);
        let y1 = start.y.max(self.mouse.y);
        Some(TableRect {
            x: x0,
            y: y0,
            w: (x1 - x0).max(1.0),
            h: (y1 - y0).max(1.0),
        })
    }

    fn active_selection_rect(&self) -> Option<TableRect> {
        self.selection_drag_is_large_enough()
            .then(|| self.selection_rect())
            .flatten()
    }

    fn units_in_selection_rect(&self, elapsed_ms: u32) -> Vec<usize> {
        let Some(selection) = self.selection_rect() else {
            return Vec::new();
        };
        self.unit_draws(elapsed_ms)
            .into_iter()
            .filter(|draw| rects_intersect(selection, draw.rect))
            .map(|draw| draw.index)
            .collect()
    }

    fn idle_cursor_tool(&self, elapsed_ms: u32) -> Option<IdleCursorTool> {
        if self.selection_start.is_some() || !self.selected_units_are_only_pawns() {
            return None;
        }

        self.idle_resource_target_at_mouse(elapsed_ms)
            .map(|target| target.kind.tool())
    }

    fn idle_resource_target_at_mouse(&self, elapsed_ms: u32) -> Option<IdleResourceTarget> {
        if let Some(index) = self
            .unit_at_point(self.mouse, elapsed_ms)
            .filter(|&index| self.idle_unit_is_sheep(index))
        {
            return Some(IdleResourceTarget {
                kind: IdleResourceKind::Meat,
                position: self.units.get(index)?.position,
            });
        }

        self.idle_resource_prop_at_mouse()
    }

    fn idle_unit_is_sheep(&self, index: usize) -> bool {
        self.units
            .get(index)
            .is_some_and(|unit| !unit.is_pawn && unit.idle.name.starts_with("Sheep"))
    }

    fn idle_resource_prop_at_mouse(&self) -> Option<IdleResourceTarget> {
        for prop in self.world.props.iter().rev() {
            let Some((kind, image, rect)) = self.idle_resource_prop_hit_rect(*prop) else {
                continue;
            };
            if !inside_rect(self.mouse.x, self.mouse.y, rect.x, rect.y, rect.w, rect.h) {
                continue;
            }
            if image_point_has_alpha(image, self.mouse, rect) {
                return Some(IdleResourceTarget {
                    kind,
                    position: self.idle_resource_prop_target(*prop),
                });
            }
        }

        None
    }

    fn idle_resource_prop_hit_rect(
        &self,
        prop: PlacedProp,
    ) -> Option<(IdleResourceKind, &ImageAsset, TableRect)> {
        match prop.kind {
            PropKind::Plant(kind) if kind.is_tree() => {
                let image = self.plant_props[kind.index()].first_frame()?;
                Some((
                    IdleResourceKind::Wood,
                    image,
                    self.idle_bottom_aligned_image_half_rect(
                        image,
                        prop.x2,
                        prop.y2,
                        kind.render_offset_y(),
                        kind.render_scale(),
                    ),
                ))
            }
            PropKind::Gold(kind) => {
                let image = self.gold_props[kind.index()].first_frame()?;
                Some((
                    IdleResourceKind::Ore,
                    image,
                    self.idle_bottom_aligned_image_half_rect(image, prop.x2, prop.y2, 0.0, 1.0),
                ))
            }
            PropKind::Rock(kind) => {
                let image = &self.rock_props[kind.index()];
                Some((
                    IdleResourceKind::Ore,
                    image,
                    self.idle_bottom_aligned_image_half_rect(image, prop.x2, prop.y2, 0.0, 1.0),
                ))
            }
            _ => None,
        }
    }

    fn idle_resource_prop_target(&self, prop: PlacedProp) -> Point {
        self.world.idle_resource_prop_target(prop)
    }

    fn idle_bottom_aligned_image_half_rect(
        &self,
        image: &ImageAsset,
        x2: usize,
        y2: usize,
        offset_y: f32,
        scale: f32,
    ) -> TableRect {
        let w = image.width as f32 * BUILDING_SCALE * scale;
        let h = image.height as f32 * BUILDING_SCALE * scale;
        TableRect {
            x: half_grid_to_px(x2) - self.camera.x + (TILE_SIZE - w) * 0.5,
            y: half_grid_to_px(y2 + BUILDING_GRID_DIVISIONS) - self.camera.y - h + offset_y,
            w: w.max(1.0),
            h: h.max(1.0),
        }
    }

    fn cursor_image(&self, elapsed_ms: u32) -> &ImageAsset {
        if self.selection_start.is_some() {
            &self.cursor_default
        } else if self.unit_at_point(self.mouse, elapsed_ms).is_some() {
            &self.cursor_hover
        } else {
            &self.cursor_default
        }
    }

    fn draw_world_background(&mut self, adapter: &mut Adapter, elapsed_ms: u32) {
        let _ = adapter.set_texture_effect(TextureEffect::World);
        self.terrain_cache.rebuild_if_dirty(&self.world);
        let mut water = SolidBatch::new(self.window_width, self.window_height);
        let mut backgrounds = SpriteBatch::new(self.window_width, self.window_height);
        let mut under_foregrounds = SpriteBatch::new(self.window_width, self.window_height);
        let mut foregrounds = SpriteBatch::new(self.window_width, self.window_height);
        let start_col = (self.camera.x / TILE_SIZE).floor().max(0.0) as usize;
        let start_row = (self.camera.y / TILE_SIZE).floor().max(0.0) as usize;
        let end_col = ((self.camera.x + self.idle_view_w()) / TILE_SIZE).ceil() as usize + 1;
        let end_row = ((self.camera.y + self.idle_view_h()) / TILE_SIZE).ceil() as usize + 1;

        for row in start_row..end_row.min(self.world.rows) {
            for col in start_col..end_col.min(self.world.cols) {
                if self.world.render_background(col, row) == BackgroundTile::Water {
                    water.rect(
                        col as f32 * TILE_SIZE - self.camera.x,
                        row as f32 * TILE_SIZE - self.camera.y,
                        TILE_SIZE,
                        TILE_SIZE,
                        Rgba8::new(71, 171, 169, 255),
                    );
                }
            }
        }

        for cell in self
            .terrain_cache
            .backgrounds
            .iter()
            .filter(|cell| terrain_cell_visible(cell, start_col, start_row, end_col, end_row))
        {
            backgrounds.sprite(
                &self.terrain,
                cell.tile,
                cell.col as f32 * TILE_SIZE - self.camera.x,
                cell.row as f32 * TILE_SIZE - self.camera.y,
                TILE_SIZE,
                TILE_SIZE,
                Rgba8::WHITE,
            );
        }

        for cell in self
            .terrain_cache
            .under_foregrounds
            .iter()
            .filter(|cell| terrain_cell_visible(cell, start_col, start_row, end_col, end_row))
        {
            under_foregrounds.sprite(
                &self.terrain,
                cell.tile,
                cell.col as f32 * TILE_SIZE - self.camera.x,
                cell.row as f32 * TILE_SIZE - self.camera.y,
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
            foregrounds.sprite(
                &self.terrain,
                cell.tile,
                cell.col as f32 * TILE_SIZE - self.camera.x,
                cell.row as f32 * TILE_SIZE - self.camera.y,
                TILE_SIZE,
                TILE_SIZE,
                Rgba8::WHITE,
            );
        }

        let _ = adapter.draw_rgb_triangles_no_present(&water.bytes);
        let _ = adapter.draw_tex_triangles_no_present(self.terrain.texture_id, &backgrounds.bytes);
        self.draw_idle_water_states(adapter, elapsed_ms);
        let _ = adapter
            .draw_tex_triangles_no_present(self.terrain.texture_id, &under_foregrounds.bytes);
        let _ = adapter.draw_tex_triangles_no_present(self.terrain.texture_id, &foregrounds.bytes);
        let _ = adapter.set_texture_effect(TextureEffect::Plain);
    }

    fn draw_saved_world_assets(&self, adapter: &mut Adapter) {
        let _ = adapter.set_texture_effect(TextureEffect::World);
        self.draw_idle_props(adapter);
        self.draw_idle_buildings(adapter);
        let _ = adapter.set_texture_effect(TextureEffect::Plain);
        self.draw_idle_fog(adapter);
    }

    fn draw_idle_retile_flyout(&self, adapter: &mut Adapter) {
        let Some(transition) = &self.retile_transition else {
            return;
        };
        let elapsed_ms = transition.started.elapsed().as_millis() as u32;
        if elapsed_ms >= IDLE_RETILE_FLYOUT_MS {
            return;
        }

        let t = elapsed_ms as f32 / IDLE_RETILE_FLYOUT_MS as f32;
        let travel = ease_out_cubic(t) * IDLE_RETILE_FLYOUT_DISTANCE_PX;
        let alpha = ((1.0 - t).clamp(0.0, 1.0) * 255.0).round() as u8;
        let mut water = SolidBatch::new(self.window_width, self.window_height);
        let mut backgrounds = SpriteBatch::new(self.window_width, self.window_height);
        let mut under_foregrounds = SpriteBatch::new(self.window_width, self.window_height);
        let mut foregrounds = SpriteBatch::new(self.window_width, self.window_height);

        for tile in &transition.flyout_tiles {
            let x = tile.col as f32 * TILE_SIZE - self.camera.x + tile.dir_x * travel;
            let y = tile.row as f32 * TILE_SIZE - self.camera.y + tile.dir_y * travel;
            let tint = Rgba8::new(255, 255, 255, alpha);
            match tile.background {
                BackgroundTile::Water => {
                    water.rect(x, y, TILE_SIZE, TILE_SIZE, Rgba8::new(71, 171, 169, alpha))
                }
                BackgroundTile::Grass => backgrounds.sprite(
                    &self.terrain,
                    GRASS_BG_TILE,
                    x,
                    y,
                    TILE_SIZE,
                    TILE_SIZE,
                    tint,
                ),
            }
            if let Some(tile) = tile.under_foreground {
                under_foregrounds.sprite(&self.terrain, tile, x, y, TILE_SIZE, TILE_SIZE, tint);
            }
            if let Some(tile) = tile.foreground {
                foregrounds.sprite(&self.terrain, tile, x, y, TILE_SIZE, TILE_SIZE, tint);
            }
        }

        let _ = adapter.set_texture_effect(TextureEffect::World);
        let _ = adapter.draw_rgb_triangles_no_present(&water.bytes);
        let _ = adapter.draw_tex_triangles_no_present(self.terrain.texture_id, &backgrounds.bytes);
        let _ = adapter
            .draw_tex_triangles_no_present(self.terrain.texture_id, &under_foregrounds.bytes);
        let _ = adapter.draw_tex_triangles_no_present(self.terrain.texture_id, &foregrounds.bytes);
        let _ = adapter.set_texture_effect(TextureEffect::Plain);
    }

    fn draw_idle_retile_cover(&self, adapter: &mut Adapter) {
        let Some(transition) = &self.retile_transition else {
            return;
        };
        let elapsed_ms = transition.started.elapsed().as_millis() as u32;
        let mut batch = SpriteBatch::new(self.window_width, self.window_height);
        let bob_y = transition.cover_bob_y(elapsed_ms);
        let angle_rad = transition.cover_rotation(elapsed_ms);
        let origin_x = transition.rect.col as f32 * TILE_SIZE - self.camera.x
            + transition.rect.cols as f32 * TILE_SIZE * 0.5;
        let origin_y = transition.rect.row as f32 * TILE_SIZE - self.camera.y
            + transition.rect.rows as f32 * TILE_SIZE * 0.5
            + bob_y;

        for tile in &transition.tiles {
            let Some(reveal_elapsed_ms) = transition.cover_reveal_elapsed_ms(*tile, elapsed_ms)
            else {
                continue;
            };
            let x = tile.col as f32 * TILE_SIZE - self.camera.x;
            let y = tile.row as f32 * TILE_SIZE - self.camera.y + bob_y;
            push_idle_retile_cover_tile(
                &mut batch,
                &self.retile_cover,
                transition.cover_region(*tile),
                x,
                y,
                origin_x,
                origin_y,
                angle_rad,
                reveal_elapsed_ms,
            );
        }

        let _ = adapter.set_texture_effect(TextureEffect::Plain);
        let _ = adapter.draw_tex_triangles_no_present(self.retile_cover.texture_id, &batch.bytes);
    }

    fn draw_idle_retile_particles(&self, adapter: &mut Adapter) {
        let Some(transition) = &self.retile_transition else {
            return;
        };
        let elapsed_ms = transition.started.elapsed().as_millis() as u32;
        let mut batches = BTreeMap::new();

        for tile in &transition.tiles {
            if elapsed_ms < tile.reveal_start_ms {
                continue;
            }
            let tile_elapsed_ms = elapsed_ms - tile.reveal_start_ms;
            if tile_elapsed_ms >= self.particle_dust.total_duration_ms {
                continue;
            }
            let Some(image) = self.particle_dust.frame_once(tile_elapsed_ms) else {
                continue;
            };
            self.push_idle_centered_tile_image(&mut batches, image, tile.col, tile.row);
        }

        self.draw_idle_image_batches(adapter, batches);
    }

    fn draw_idle_water_states(&self, adapter: &mut Adapter, elapsed_ms: u32) {
        let mut batches = BTreeMap::new();
        let start_col = (self.camera.x / TILE_SIZE).floor().max(0.0) as usize;
        let start_row = (self.camera.y / TILE_SIZE).floor().max(0.0) as usize;
        let end_col = ((self.camera.x + self.idle_view_w()) / TILE_SIZE).ceil() as usize + 1;
        let end_row = ((self.camera.y + self.idle_view_h()) / TILE_SIZE).ceil() as usize + 1;

        for row in start_row..end_row.min(self.world.rows) {
            for col in start_col..end_col.min(self.world.cols) {
                if self.world.cell_accepts_wave_animation(col, row) {
                    if let Some(image) = self.active_idle_water_visual_frame(
                        WaterState::Animation,
                        col,
                        row,
                        elapsed_ms,
                    ) {
                        let w = image.width as f32 * BUILDING_SCALE;
                        let h = image.height as f32 * BUILDING_SCALE;
                        self.push_idle_image_batch(
                            &mut batches,
                            image.texture_id,
                            col as f32 * TILE_SIZE - self.camera.x + (TILE_SIZE - w) * 0.5,
                            row as f32 * TILE_SIZE - self.camera.y + (TILE_SIZE - h) * 0.5,
                            w.max(1.0),
                            h.max(1.0),
                            Rgba8::WHITE,
                        );
                    }
                }

                let Some(state) = self.world.water_state(col, row) else {
                    continue;
                };
                if state == WaterState::Nothing || state == WaterState::Animation {
                    continue;
                }
                let Some(image) = self.idle_water_visual_frame(state, col, row, elapsed_ms) else {
                    continue;
                };
                let w = image.width as f32 * BUILDING_SCALE;
                let h = image.height as f32 * BUILDING_SCALE;
                self.push_idle_image_batch(
                    &mut batches,
                    image.texture_id,
                    col as f32 * TILE_SIZE - self.camera.x + (TILE_SIZE - w) * 0.5,
                    row as f32 * TILE_SIZE - self.camera.y + (TILE_SIZE - h) * 0.5,
                    w.max(1.0),
                    h.max(1.0),
                    Rgba8::WHITE,
                );
            }
        }

        self.draw_idle_image_batches(adapter, batches);
    }

    fn active_idle_water_visual_frame(
        &self,
        state: WaterState,
        col: usize,
        row: usize,
        elapsed_ms: u32,
    ) -> Option<&ImageAsset> {
        let animation = self.idle_water_visual_animation(state)?;
        self.water_animations
            .iter()
            .find(|active| active.col == col && active.row == row && active.state == state)
            .and_then(|active| animation.frame_once(elapsed_ms.saturating_sub(active.started_ms)))
    }

    fn idle_water_visual_frame(
        &self,
        state: WaterState,
        col: usize,
        row: usize,
        elapsed_ms: u32,
    ) -> Option<&ImageAsset> {
        let animation = self.idle_water_visual_animation(state)?;
        let active = self
            .water_animations
            .iter()
            .find(|active| active.col == col && active.row == row && active.state == state);
        active
            .and_then(|active| animation.frame_once(elapsed_ms.saturating_sub(active.started_ms)))
            .or_else(|| animation.first_frame())
    }

    fn idle_water_visual_animation(&self, state: WaterState) -> Option<&SpriteAnimation> {
        let animation = match state {
            WaterState::Nothing => return None,
            WaterState::Stone1 => &self.water_visuals.stones[0],
            WaterState::Stone2 => &self.water_visuals.stones[1],
            WaterState::Stone3 => &self.water_visuals.stones[2],
            WaterState::Stone4 => &self.water_visuals.stones[3],
            WaterState::Animation => &self.water_visuals.animation,
            WaterState::Duck => &self.water_visuals.duck,
        };
        Some(animation)
    }

    fn draw_idle_props(&self, adapter: &mut Adapter) {
        let mut terrain_batch = SpriteBatch::new(self.window_width, self.window_height);
        let mut image_batches = BTreeMap::new();

        for prop in &self.world.props {
            match prop.kind {
                PropKind::Pillar(tile) => {
                    terrain_batch.sprite(
                        &self.terrain,
                        tile,
                        half_grid_to_px(prop.x2) - self.camera.x,
                        half_grid_to_px(prop.y2) - self.camera.y,
                        TILE_SIZE,
                        TILE_SIZE,
                        Rgba8::WHITE,
                    );
                }
                PropKind::Plant(kind) => {
                    let instance_count = kind.visual_instance_count(prop.x2, prop.y2);
                    for instance in 0..instance_count {
                        let Some(image) = self.idle_plant_frame(kind, prop.x2, prop.y2, instance)
                        else {
                            continue;
                        };
                        let offset = kind.visual_instance_offset(instance_count, instance);
                        self.push_idle_bottom_aligned_image_half(
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
                    let Some(image) = self.idle_gold_frame(kind, prop.x2, prop.y2) else {
                        continue;
                    };
                    self.push_idle_bottom_aligned_image_half(
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
                    let instance_count = kind.visual_instance_count(prop.x2, prop.y2);
                    for instance in 0..instance_count {
                        let offset = kind.visual_instance_offset(instance_count, instance);
                        self.push_idle_bottom_aligned_image_half(
                            &mut image_batches,
                            image,
                            prop.x2,
                            prop.y2,
                            offset.x,
                            offset.y,
                            1.0,
                            Rgba8::WHITE,
                        );
                    }
                }
            }
        }

        let _ =
            adapter.draw_tex_triangles_no_present(self.terrain.texture_id, &terrain_batch.bytes);
        self.draw_idle_image_batches(adapter, image_batches);
    }

    fn idle_plant_frame(
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

    fn idle_gold_frame(&self, kind: GoldKind, x2: usize, y2: usize) -> Option<&ImageAsset> {
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

    fn draw_idle_buildings(&self, adapter: &mut Adapter) {
        let mut batches = BTreeMap::new();
        let mut icon_batches = BTreeMap::new();
        let elapsed_ms = self.started_at.elapsed().as_millis() as u32;
        for building in &self.world.buildings {
            let image = &self.buildings[building.kind.index()];
            let x = signed_half_grid_to_px(building.x2) - self.camera.x;
            let y = signed_half_grid_to_px(building.y2) - self.camera.y;
            let w = image.width as f32 * BUILDING_SCALE;
            let h = image.height as f32 * BUILDING_SCALE;
            if x + w < 0.0
                || y + h < 0.0
                || x > self.window_width as f32
                || y > self.window_height as f32
            {
                continue;
            }

            self.push_idle_image_batch(
                &mut batches,
                image.texture_id,
                x,
                y,
                w.max(1.0),
                h.max(1.0),
                Rgba8::WHITE,
            );

            if let Some(icon_index) = Self::idle_house_icon_index(building.kind) {
                let icon = &self.house_icon_inlays[icon_index];
                let icon_size = Self::idle_house_icon_size(elapsed_ms, icon_index);
                self.push_idle_image_batch(
                    &mut icon_batches,
                    icon.texture_id,
                    (x + w - icon_size * 0.5).floor(),
                    (y + (h - icon_size) * 0.5).floor(),
                    icon_size,
                    icon_size,
                    Rgba8::new(255, 255, 255, 191),
                );
            }
        }
        self.draw_idle_image_batches(adapter, batches);
        self.draw_idle_image_batches(adapter, icon_batches);
    }

    fn idle_build_hotkey_kind(digit: u8) -> Option<BuildingKind> {
        match digit {
            1 => Some(BuildingKind::House1),
            2 => Some(BuildingKind::House2),
            3 => Some(BuildingKind::House3),
            _ => None,
        }
    }

    fn idle_house_icon_size(elapsed_ms: u32, index: usize) -> f32 {
        let base_icon_size = 56.0;
        let pulse = 0.85 + 0.05 * ((elapsed_ms as f32 * 0.004) + index as f32 * 0.55).sin();
        (base_icon_size * pulse).floor().max(1.0)
    }

    fn idle_house_icon_index(kind: BuildingKind) -> Option<usize> {
        match kind {
            BuildingKind::House1 => Some(0),
            BuildingKind::House2 => Some(1),
            BuildingKind::House3 => Some(2),
            _ => None,
        }
    }

    fn idle_world_half_cell_at(&self, x: f32, y: f32) -> Option<(usize, usize)> {
        if x < 0.0 || y < 0.0 {
            return None;
        }

        let x = x + self.camera.x;
        let y = y + self.camera.y;
        let half_tile = TILE_SIZE / BUILDING_GRID_DIVISIONS as f32;
        let x2 = (x / half_tile).floor() as usize;
        let y2 = (y / half_tile).floor() as usize;
        if x2 < self.world.cols * BUILDING_GRID_DIVISIONS
            && y2 < self.world.rows * BUILDING_GRID_DIVISIONS
        {
            Some((x2, y2))
        } else {
            None
        }
    }

    fn idle_building_anchor_half_cell(kind: BuildingKind, x2: usize, y2: usize) -> (isize, isize) {
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

    fn place_idle_building_at_mouse(&mut self) {
        let Some(kind) = self.placement_building else {
            return;
        };
        let Some((x2, y2)) = self.idle_world_half_cell_at(self.mouse.x, self.mouse.y) else {
            return;
        };

        let (anchor_x2, anchor_y2) = Self::idle_building_anchor_half_cell(kind, x2, y2);
        self.world.paint_building(kind, anchor_x2, anchor_y2);
    }

    fn draw_idle_building_preview(&self, adapter: &mut Adapter) {
        let Some(kind) = self.placement_building else {
            return;
        };
        let Some((x2, y2)) = self.idle_world_half_cell_at(self.mouse.x, self.mouse.y) else {
            return;
        };

        let (x2, y2) = Self::idle_building_anchor_half_cell(kind, x2, y2);
        let image = &self.buildings[kind.index()];
        let can_place = self.world.can_place_building_kind(kind, x2, y2);
        let tint = if can_place {
            Rgba8::new(255, 255, 255, 145)
        } else {
            Rgba8::new(255, 96, 96, 155)
        };
        let (footprint_x2, footprint_y2, footprint_w2, footprint_h2) =
            building_spec_footprint_rect2(building_spec(kind), x2, y2);
        let mut overlay = SolidBatch::new(self.window_width, self.window_height);
        outline_rect(
            &mut overlay,
            signed_half_grid_to_px(footprint_x2) - self.camera.x,
            signed_half_grid_to_px(footprint_y2) - self.camera.y,
            half_grid_to_px(footprint_w2),
            half_grid_to_px(footprint_h2),
            2.0,
            if can_place {
                Rgba8::new(255, 225, 118, 210)
            } else {
                Rgba8::new(255, 86, 86, 230)
            },
        );
        let _ = adapter.draw_rgb_triangles_no_present(&overlay.bytes);

        let mut sprite = SpriteBatch::new(self.window_width, self.window_height);
        sprite.image(
            signed_half_grid_to_px(x2) - self.camera.x,
            signed_half_grid_to_px(y2) - self.camera.y,
            image.width as f32 * BUILDING_SCALE,
            image.height as f32 * BUILDING_SCALE,
            tint,
        );
        let _ = adapter.draw_tex_triangles_no_present(image.texture_id, &sprite.bytes);

        if let Some(icon_index) = Self::idle_house_icon_index(kind) {
            let icon = &self.house_icon_inlays[icon_index];
            let elapsed_ms = self.started_at.elapsed().as_millis() as u32;
            let icon_size = Self::idle_house_icon_size(elapsed_ms, icon_index);
            let mut inlay = SpriteBatch::new(self.window_width, self.window_height);
            let x = signed_half_grid_to_px(x2) - self.camera.x;
            let y = signed_half_grid_to_px(y2) - self.camera.y;
            let w = image.width as f32 * BUILDING_SCALE;
            let h = image.height as f32 * BUILDING_SCALE;
            inlay.image(
                (x + w - icon_size * 0.5).floor(),
                (y + (h - icon_size) * 0.5).floor(),
                icon_size,
                icon_size,
                Rgba8::new(255, 255, 255, 191),
            );
            let _ = adapter.draw_tex_triangles_no_present(icon.texture_id, &inlay.bytes);
        }
    }

    fn draw_idle_fog(&self, adapter: &mut Adapter) {
        let mut fog = SpriteBatch::new(self.window_width, self.window_height);
        let start_col = (self.camera.x / TILE_SIZE).floor().max(0.0) as usize;
        let start_row = (self.camera.y / TILE_SIZE).floor().max(0.0) as usize;
        let end_col = ((self.camera.x + self.idle_view_w()) / TILE_SIZE).ceil() as usize + 1;
        let end_row = ((self.camera.y + self.idle_view_h()) / TILE_SIZE).ceil() as usize + 1;

        for row in start_row..end_row.min(self.world.rows) {
            for col in start_col..end_col.min(self.world.cols) {
                if self.world.fog(col, row) {
                    fog.image(
                        col as f32 * TILE_SIZE - self.camera.x,
                        row as f32 * TILE_SIZE - self.camera.y,
                        TILE_SIZE,
                        TILE_SIZE,
                        Rgba8::new(255, 255, 255, 190),
                    );
                }
            }
        }

        let _ = adapter.draw_tex_triangles_no_present(self.fog.texture_id, &fog.bytes);
    }

    fn push_idle_bottom_aligned_image_half(
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
        let x = half_grid_to_px(x2) - self.camera.x + (TILE_SIZE - w) * 0.5 + offset_x;
        let y = half_grid_to_px(y2 + BUILDING_GRID_DIVISIONS) - self.camera.y - h + offset_y;
        self.push_idle_image_batch(
            batches,
            image.texture_id,
            x.floor(),
            y.floor(),
            w.max(1.0),
            h.max(1.0),
            tint,
        );
    }

    fn push_idle_centered_tile_image(
        &self,
        batches: &mut BTreeMap<u32, SpriteBatch>,
        image: &ImageAsset,
        col: usize,
        row: usize,
    ) {
        let w = image.width as f32 * BUILDING_SCALE;
        let h = image.height as f32 * BUILDING_SCALE;
        self.push_idle_image_batch(
            batches,
            image.texture_id,
            col as f32 * TILE_SIZE - self.camera.x + (TILE_SIZE - w) * 0.5,
            row as f32 * TILE_SIZE - self.camera.y + (TILE_SIZE - h) * 0.5,
            w.max(1.0),
            h.max(1.0),
            Rgba8::WHITE,
        );
    }

    fn push_idle_image_batch(
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

    fn draw_idle_image_batches(&self, adapter: &mut Adapter, batches: BTreeMap<u32, SpriteBatch>) {
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
            let wrap_w = self.world.cols as f32 * TILE_SIZE + image.width as f32 * scale;
            let wrap_h = self.world.rows as f32 * TILE_SIZE + image.height as f32 * scale;
            let x = (cloud.x + cloud.drift_x * elapsed).rem_euclid(wrap_w)
                - image.width as f32 * scale * 0.5;
            let y = (cloud.y + cloud.drift_y * elapsed).rem_euclid(wrap_h)
                - image.height as f32 * scale * 0.5;
            let screen_x = x - self.camera.x;
            let screen_y = y - self.camera.y;
            let w = image.width as f32 * scale;
            let h = image.height as f32 * scale;
            if screen_x + w < 0.0
                || screen_y + h < 0.0
                || screen_x > self.window_width as f32
                || screen_y > self.window_height as f32
            {
                continue;
            }

            batches
                .entry(image.texture_id)
                .or_insert_with(|| SpriteBatch::new(self.window_width, self.window_height))
                .image(
                    screen_x.floor(),
                    screen_y.floor(),
                    w.floor().max(1.0),
                    h.floor().max(1.0),
                    Rgba8::new(255, 255, 255, (alpha * 255.0).round() as u8),
                );
        }

        for (texture_id, batch) in batches {
            let _ = adapter.draw_tex_triangles_no_present(texture_id, &batch.bytes);
        }
    }

    fn draw_selection_corners(&self, adapter: &mut Adapter, rect: TableRect) {
        let mut corners = SpriteBatch::new(self.window_width, self.window_height);
        let [top_left, top_right, bottom_left, bottom_right] = SELECT_CORNER_SOURCES;
        let corner_w = top_left.width as f32;
        let corner_h = top_left.height as f32;
        let x = rect.x.floor() - 4.0;
        let y = rect.y.floor() - 4.0;
        let w = rect.w.ceil() + 8.0;
        let h = rect.h.ceil() + 8.0;

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

    fn draw_selected_unit_ui(&self, adapter: &mut Adapter, elapsed_ms: u32) {
        let mut health_bars = ts_ui::SmallBarBatch::new(self.window_width, self.window_height);
        for &selected in &self.selected_units {
            let Some((rect, _)) = self.unit_rect(selected, elapsed_ms) else {
                continue;
            };

            let bar_w = 42.0;
            let bar_h = 8.0;
            health_bars.small_bar(
                (rect.x + (rect.w - bar_w) * 0.5).floor(),
                (rect.y - bar_h - 4.0).floor(),
                bar_w,
                bar_h,
                1.0,
                DemoUnitTeam::for_unit_index(selected).health_bar_color(),
                Rgba8::new(255, 255, 255, 245),
            );
        }
        if health_bars.base_bytes.is_empty() {
            return;
        }
        let _ = adapter
            .draw_tex_triangles_no_present(ts_ui::SMALL_BAR_BASE_TEXTURE, &health_bars.base_bytes);
        let _ = adapter.draw_rgb_triangles_no_present(&health_bars.fill_solid_bytes);
    }

    fn selected_units_include_pawn(&self) -> bool {
        self.selected_units
            .iter()
            .any(|&index| self.units.get(index).is_some_and(|unit| unit.is_pawn))
    }

    fn selected_units_are_only_pawns(&self) -> bool {
        !self.selected_units.is_empty()
            && self
                .selected_units
                .iter()
                .all(|&index| self.units.get(index).is_some_and(|unit| unit.is_pawn))
    }

    fn draw_pawn_build_ui(&self, adapter: &mut Adapter, elapsed_ms: u32) {
        if !self.selected_units_include_pawn() {
            return;
        }

        let tile = 64.0;
        let panel_w = tile * 7.0;
        let panel_h = tile;
        let panel_x = ((self.window_width as f32 - panel_w) * 0.5).round();
        let panel_y = (self.window_height as f32 - panel_h - 24.0)
            .max(0.0)
            .round();
        let mut paper = ts_ui::UiBatch::new(self.window_width, self.window_height);
        paper.paper_panel_tiles(panel_x, panel_y, 7, 1, tile, Rgba8::new(255, 255, 255, 128));
        let _ = adapter
            .draw_tex_triangles_no_present(self.regular_paper.texture_id, &paper.texture_bytes);

        let slot_w = panel_w / 3.0;
        let mut labels = ts_ui::UiBatch::new(self.window_width, self.window_height);
        for (index, image) in self.house_previews.iter().enumerate() {
            let max_w = 88.0;
            let max_h = 102.0;
            let scale = (max_w / image.width as f32)
                .min(max_h / image.height as f32)
                .min(1.0);
            let w = (image.width as f32 * scale).floor().max(1.0);
            let h = (image.height as f32 * scale).floor().max(1.0);
            let slot_x = panel_x + index as f32 * slot_w;
            let x = (slot_x + (slot_w - w) * 0.5).floor();
            let y = (panel_y + panel_h - h - 16.0).floor();
            let key = (index + 1).to_string();
            labels.text(
                &key,
                (x - 28.0).max(slot_x + 10.0),
                panel_y + 24.0,
                2.0,
                Rgba8::new(36, 44, 44, 255),
            );

            let mut preview = SpriteBatch::new(self.window_width, self.window_height);
            preview.image(x, y, w, h, Rgba8::WHITE);
            let _ = adapter.draw_tex_triangles_no_present(image.texture_id, &preview.bytes);

            let icon = &self.house_icon_inlays[index];
            let base_icon_size = 56.0;
            let pulse = 0.85 + 0.05 * ((elapsed_ms as f32 * 0.004) + index as f32 * 0.55).sin();
            let icon_size = (base_icon_size * pulse).floor().max(1.0);
            let icon_x = (x + w - icon_size * 0.5).floor();
            let icon_y = (y + (h - icon_size) * 0.5).floor();
            let mut inlay = SpriteBatch::new(self.window_width, self.window_height);
            inlay.image(
                icon_x,
                icon_y,
                icon_size,
                icon_size,
                Rgba8::new(255, 255, 255, 191),
            );
            let _ = adapter.draw_tex_triangles_no_present(icon.texture_id, &inlay.bytes);
        }
        let _ = adapter.draw_rgb_triangles_no_present(&labels.solid_bytes);
    }

    fn draw_hotkey_menu(&self, adapter: &mut Adapter) {
        if !self.show_hotkey_menu {
            return;
        }

        let rect = self.hotkey_menu_rect();
        let mut menu = ts_ui::UiBatch::new(self.window_width, self.window_height);
        menu.hotkey_menu_page("HOTKEYS", rect.x, rect.y, &IDLE_HOTKEY_ENTRIES);
        let _ = adapter.draw_tex_triangles_no_present(ts_ui::BANNER_TEXTURE, &menu.texture_bytes);
        let _ =
            adapter.draw_tex_triangles_no_present(ts_ui::BIG_RIBBONS_TEXTURE, &menu.ribbon_bytes);
        let _ = adapter.draw_tex_triangles_no_present(
            ts_ui::SMALL_BLUE_SQUARE_BUTTON_TEXTURE,
            &menu.button_bytes,
        );
        let _ = adapter.draw_rgb_triangles_no_present(&menu.solid_bytes);
    }

    fn draw_right_click_indicators(&self, adapter: &mut Adapter) {
        if self.right_click_indicators.is_empty() {
            return;
        }

        let mut batch = SpriteBatch::new(self.window_width, self.window_height);
        for indicator in &self.right_click_indicators {
            let elapsed = indicator.started.elapsed().as_millis();
            if elapsed >= IDLE_RIGHT_CLICK_INDICATOR_MS {
                continue;
            }
            let t = elapsed as f32 / IDLE_RIGHT_CLICK_INDICATOR_MS as f32;
            let size = (IDLE_RIGHT_CLICK_INDICATOR_SIZE * (1.0 - t)).max(0.0);
            if size <= 0.5 {
                continue;
            }
            batch.image(
                (indicator.position.x - size * 0.5).floor(),
                (indicator.position.y - size * 0.5).floor(),
                size.floor().max(1.0),
                size.floor().max(1.0),
                Rgba8::WHITE,
            );
        }

        let _ = adapter
            .draw_tex_triangles_no_present(self.right_click_indicator.texture_id, &batch.bytes);
    }

    fn draw_cursor(&self, adapter: &mut Adapter, elapsed_ms: u32) {
        if self.selection_drag_is_large_enough() {
            return;
        }

        if let Some(tool) = self.idle_cursor_tool(elapsed_ms) {
            let image = self.idle_cursor_tool_image(tool);
            let mut cursor = SpriteBatch::new(self.window_width, self.window_height);
            let phase_ms = elapsed_ms % 800;
            let mirror_x = phase_ms >= 400;
            let local_ms = if mirror_x { phase_ms - 400 } else { phase_ms };
            let swing = ((local_ms as f32 / 400.0) * std::f32::consts::TAU).sin();
            let angle = swing * 15.0_f32.to_radians();
            let size = 32.0;
            cursor.image_rotated_mirror_x(
                self.mouse.x - size * 0.15,
                self.mouse.y - size * 0.15,
                size,
                size,
                angle,
                mirror_x,
                Rgba8::WHITE,
            );
            let _ = adapter.draw_tex_triangles_no_present(image.texture_id, &cursor.bytes);
            return;
        }

        let image = self.cursor_image(elapsed_ms);
        let mut cursor = SpriteBatch::new(self.window_width, self.window_height);
        cursor.image(self.mouse.x, self.mouse.y, 28.0, 28.0, Rgba8::WHITE);
        let _ = adapter.draw_tex_triangles_no_present(image.texture_id, &cursor.bytes);
    }

    fn idle_cursor_tool_image(&self, tool: IdleCursorTool) -> &ImageAsset {
        match tool {
            IdleCursorTool::Pickaxe => &self.cursor_pickaxe,
            IdleCursorTool::Axe => &self.cursor_axe,
            IdleCursorTool::Dagger => &self.cursor_dagger,
        }
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
    load_unit_clip_from_tags(
        spec.label.to_string(),
        spec.path,
        &[spec.preferred_tag, "Idle"],
        spec.offset_x,
        spec.offset_y,
        next_texture_id,
    )
}

fn load_idle_world_units(next_texture_id: &mut u32) -> Vec<IdleWorldUnit> {
    let mut units = IDLE_WORLD_UNIT_SPECS
        .iter()
        .zip(IDLE_WORLD_MOVE_SPECS.iter())
        .zip(IDLE_WORLD_ATTACK_SPECS.iter())
        .enumerate()
        .filter_map(|(index, ((idle_spec, move_spec), attack_specs))| {
            let tags: &[&str] = if idle_spec.label == "Sheep Idle" {
                &["Idle", "Move"]
            } else {
                &["Idle"]
            };
            let idle = load_unit_clip_from_tags(
                idle_spec.label.to_string(),
                idle_spec.path,
                tags,
                idle_spec.offset_x,
                idle_spec.offset_y,
                next_texture_id,
            )?;
            let movement = load_unit_walk_clip(*move_spec, next_texture_id).unwrap_or_else(|| {
                load_unit_clip_from_tags(
                    move_spec.label.to_string(),
                    move_spec.path,
                    &["Idle"],
                    move_spec.offset_x,
                    move_spec.offset_y,
                    next_texture_id,
                )
                .expect("idle world movement fallback should load")
            });
            let attacks = attack_specs
                .iter()
                .filter_map(|spec| load_unit_walk_clip(*spec, next_texture_id))
                .collect();
            let is_monk = idle_spec.label.starts_with("Monk");
            Some(IdleWorldUnit {
                idle,
                movement,
                attacks,
                position: idle_world_unit_foot_position(index, 13),
                movement_path: Vec::new(),
                path_generation: 0,
                facing_left: false,
                is_pawn: false,
                is_monk,
                state: IdleWorldUnitState::Idle,
            })
        })
        .collect::<Vec<_>>();
    units.extend(load_pawn_idle_world_units(next_texture_id, units.len(), 13));
    units
}

fn load_idle_retile_particle_dust() -> SpriteAnimation {
    let set = ase_assets::load_tinted_aseprite_set(
        PARTICLE_FX_ASEPRITE_PATH,
        [255, 255, 255, 255],
        ase_assets::TintMode::Multiply,
    )
    .expect("particle fx aseprite should decode");

    let frames = ["particle_dust_1", "Dust 1", "dust_1"]
        .iter()
        .find_map(|tag| set.frames_for_tag(tag))
        .unwrap_or(set.frames.as_slice())
        .iter()
        .enumerate()
        .map(|(index, frame)| {
            (
                ImageAsset::from_rgba_cropped(
                    IDLE_RETILE_PARTICLE_TEXTURE_BASE + index as u32,
                    frame.width,
                    frame.height,
                    frame.rgba.clone(),
                ),
                frame.duration_ms.unwrap_or(80).max(1),
            )
        })
        .filter(|(image, _)| image.rgba.chunks_exact(4).any(|pixel| pixel[3] != 0))
        .collect::<Vec<_>>();
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
}

fn load_unit_clip_from_tags(
    label: String,
    path: &str,
    preferred_tags: &[&str],
    offset_x: f32,
    offset_y: f32,
    next_texture_id: &mut u32,
) -> Option<UnitWalkClip> {
    let set = ase_assets::load_tinted_aseprite_set(
        path,
        [255, 255, 255, 255],
        ase_assets::TintMode::Multiply,
    )
    .ok()?;
    let tag = preferred_tags
        .iter()
        .find_map(|name| set.tags.iter().find(|tag| tag.name.trim() == *name))?;
    unit_clip_from_frames(
        label,
        tag.name.trim().to_string(),
        set.frames
            .get(tag.from_frame as usize..=tag.to_frame as usize)?,
        offset_x,
        offset_y,
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

fn load_pawn_idle_world_units(
    next_texture_id: &mut u32,
    start_index: usize,
    count: usize,
) -> Vec<IdleWorldUnit> {
    let Ok(set) = ase_assets::load_tinted_aseprite_set(
        PAWN_ASEPRITE_PATH,
        [255, 255, 255, 255],
        ase_assets::TintMode::Multiply,
    ) else {
        return Vec::new();
    };

    PAWN_IDLE_TAGS
        .iter()
        .enumerate()
        .filter_map(|(offset, idle_tag)| {
            let idle = load_pawn_clip_from_set(
                &set,
                &format!("Pawn {idle_tag}"),
                idle_tag,
                next_texture_id,
            )?;
            let run_tag = pawn_run_tag(idle_tag);
            let movement =
                load_pawn_clip_from_set(&set, &format!("Pawn {run_tag}"), run_tag, next_texture_id)
                    .unwrap_or_else(|| {
                        load_pawn_clip_from_set(
                            &set,
                            &format!("Pawn {idle_tag}"),
                            idle_tag,
                            next_texture_id,
                        )
                        .expect("pawn idle movement fallback should load")
                    });
            let attack_tag = pawn_attack_tag(idle_tag);
            let attacks = load_pawn_clip_from_set(
                &set,
                &format!("Pawn {attack_tag}"),
                attack_tag,
                next_texture_id,
            )
            .into_iter()
            .collect();
            Some(IdleWorldUnit {
                idle,
                movement,
                attacks,
                position: idle_world_unit_foot_position(start_index + offset, count),
                movement_path: Vec::new(),
                path_generation: 0,
                facing_left: false,
                is_pawn: true,
                is_monk: false,
                state: IdleWorldUnitState::Idle,
            })
        })
        .collect()
}

fn load_pawn_clip_from_set(
    set: &ase_assets::AsepriteSet,
    label: &str,
    tag_name: &str,
    next_texture_id: &mut u32,
) -> Option<UnitWalkClip> {
    let tag = if tag_name == "Idle" {
        set.tags.iter().rfind(|tag| tag.name.trim() == tag_name)?
    } else {
        set.tags.iter().find(|tag| tag.name.trim() == tag_name)?
    };
    let frames = set
        .frames
        .get(tag.from_frame as usize..=tag.to_frame as usize)?;
    unit_clip_from_frames(
        label.to_string(),
        tag.name.trim().to_string(),
        frames,
        0.0,
        0.0,
        next_texture_id,
    )
}

fn pawn_run_tag(idle_tag: &str) -> &str {
    match idle_tag {
        "Idle" => "Run",
        "Idle Wood" => "Run Wood",
        "Idle Meat" => "Run Meat",
        "Idle Gold" => "Run Gold",
        "Idle Hammer" => "Run Hammer",
        "Idle Axe" => "Run Axe",
        "Idle Knife" => "Run Knife",
        "Idle Pickaxe" => "Run Pickaxe",
        _ => "Run",
    }
}

fn pawn_attack_tag(idle_tag: &str) -> &str {
    match idle_tag {
        "Idle Hammer" => "Interact Hammer",
        "Idle Axe" => "Interact Axe",
        "Idle Knife" => "Interact Knife",
        "Idle Pickaxe" => "Interact Pickaxe",
        _ => "Interact",
    }
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

fn idle_world_unit_foot_position(index: usize, count: usize) -> Point {
    let count = count.max(1);
    let angle = -std::f32::consts::FRAC_PI_2 + index as f32 / count as f32 * std::f32::consts::TAU;
    let virtual_cols = IDLE_WORLD_VIRTUAL_WIDTH / TILE_SIZE;
    let virtual_rows = IDLE_WORLD_VIRTUAL_HEIGHT / TILE_SIZE;
    let radius_tiles = virtual_cols.min(virtual_rows) * 0.32;

    Point {
        x: (virtual_cols * 0.5 + angle.cos() * radius_tiles) * TILE_SIZE,
        y: (virtual_rows * 0.5 + angle.sin() * radius_tiles) * TILE_SIZE,
    }
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

fn clamp_idle_retile_rect(
    rect: IdleRetileRect,
    world_cols: usize,
    world_rows: usize,
) -> Option<IdleRetileRect> {
    if world_cols == 0 || world_rows == 0 || rect.cols == 0 || rect.rows == 0 {
        return None;
    }
    let cols = rect.cols.min(world_cols);
    let rows = rect.rows.min(world_rows);
    Some(IdleRetileRect {
        col: rect.col.min(world_cols.saturating_sub(cols)),
        row: rect.row.min(world_rows.saturating_sub(rows)),
        cols,
        rows,
    })
}

fn generated_idle_retile_patch(
    rect: IdleRetileRect,
    world_cols: usize,
    world_rows: usize,
    seed: u64,
) -> (TileWorld, usize, usize) {
    let left_pad = IDLE_RETILE_PATCH_PADDING_TILES.min(rect.col);
    let top_pad = IDLE_RETILE_PATCH_PADDING_TILES.min(rect.row);
    let right_pad = IDLE_RETILE_PATCH_PADDING_TILES
        .min(world_cols.saturating_sub(rect.col.saturating_add(rect.cols)));
    let bottom_pad = IDLE_RETILE_PATCH_PADDING_TILES
        .min(world_rows.saturating_sub(rect.row.saturating_add(rect.rows)));
    let patch_cols = rect.cols.saturating_add(left_pad).saturating_add(right_pad);
    let patch_rows = rect.rows.saturating_add(top_pad).saturating_add(bottom_pad);

    let mut generator = wldgenerator::RunningGenerator::new(patch_cols, patch_rows, seed);
    generator.complete_initial_seeds();
    generator.fill_visual_voids_once(patch_cols.saturating_mul(patch_rows));
    (
        generator.world().to_visual_tile_world().tiles,
        left_pad,
        top_pad,
    )
}

fn idle_retile_rect_tiles(rect: IdleRetileRect) -> Vec<(usize, usize)> {
    let mut tiles = Vec::with_capacity(rect.cols.saturating_mul(rect.rows));
    for row in rect.row..rect.row + rect.rows {
        for col in rect.col..rect.col + rect.cols {
            tiles.push((col, row));
        }
    }
    tiles
}

fn reroll_mask_bounding_rect(
    cols: usize,
    rows: usize,
    mask: &[bool],
    padding: usize,
) -> Option<IdleRetileRect> {
    let mut min_col = cols;
    let mut min_row = rows;
    let mut max_col = 0;
    let mut max_row = 0;
    let mut found = false;

    for row in 0..rows {
        for col in 0..cols {
            if !mask.get(row * cols + col).copied().unwrap_or(false) {
                continue;
            }
            found = true;
            min_col = min_col.min(col);
            min_row = min_row.min(row);
            max_col = max_col.max(col);
            max_row = max_row.max(row);
        }
    }

    if !found {
        return None;
    }
    min_col = min_col.saturating_sub(padding);
    min_row = min_row.saturating_sub(padding);
    max_col = max_col.saturating_add(padding).min(cols.saturating_sub(1));
    max_row = max_row.saturating_add(padding).min(rows.saturating_sub(1));

    Some(IdleRetileRect {
        col: min_col,
        row: min_row,
        cols: max_col - min_col + 1,
        rows: max_row - min_row + 1,
    })
}

fn idle_retile_flyout_tiles(
    world: &TileWorld,
    rect: IdleRetileRect,
    seed: u64,
) -> Vec<IdleRetileFlyoutTile> {
    let mut rng = SeededRng::new(seed ^ 0x1D1E_F17E_0A17_2026);
    let mut tiles = Vec::with_capacity(rect.cols.saturating_mul(rect.rows));
    for row in rect.row..rect.row + rect.rows {
        for col in rect.col..rect.col + rect.cols {
            if col >= world.cols || row >= world.rows {
                continue;
            }
            let angle = rng.next_f32() * std::f32::consts::TAU;
            tiles.push(IdleRetileFlyoutTile {
                col,
                row,
                background: world.background(col, row),
                under_foreground: world.under_foreground(col, row),
                foreground: world.foreground(col, row),
                dir_x: angle.cos(),
                dir_y: angle.sin(),
            });
        }
    }
    tiles
}

fn shuffled_idle_retile_tiles(mut tiles: Vec<(usize, usize)>, seed: u64) -> Vec<(usize, usize)> {
    let mut rng = SeededRng::new(seed ^ 0x1D1E_51DE_5A1F_2026);
    for index in (1..tiles.len()).rev() {
        let swap_index = rng.range_usize(0, index + 1);
        tiles.swap(index, swap_index);
    }
    tiles
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn idle_retile_cover_region_for_rect(
    local_col: usize,
    local_row: usize,
    rect_cols: usize,
    rect_rows: usize,
) -> ImageRegion {
    let source_col = idle_retile_cover_slice_index(local_col, rect_cols);
    let source_row = idle_retile_cover_slice_index(local_row, rect_rows);
    ImageRegion::new(
        source_col * IDLE_RETILE_COVER_TILE_PX,
        source_row * IDLE_RETILE_COVER_TILE_PX,
        IDLE_RETILE_COVER_TILE_PX,
        IDLE_RETILE_COVER_TILE_PX,
    )
}

fn idle_retile_cover_slice_index(local_index: usize, rect_len: usize) -> u32 {
    if local_index == 0 {
        0
    } else if local_index + 1 >= rect_len {
        2
    } else {
        1
    }
}

fn push_idle_retile_cover_tile(
    batch: &mut SpriteBatch,
    image: &ImageAsset,
    region: ImageRegion,
    x: f32,
    y: f32,
    origin_x: f32,
    origin_y: f32,
    angle_rad: f32,
    reveal_elapsed_ms: Option<u32>,
) {
    let Some(reveal_elapsed_ms) = reveal_elapsed_ms else {
        batch.image_region_rotated_around(
            image,
            region,
            x,
            y,
            TILE_SIZE,
            TILE_SIZE,
            origin_x,
            origin_y,
            angle_rad,
            Rgba8::WHITE,
        );
        return;
    };

    let reveal_px = 1
        + (IDLE_RETILE_COVER_TILE_PX - 1)
            .saturating_mul(reveal_elapsed_ms.min(IDLE_RETILE_REVEAL_MS))
            / IDLE_RETILE_REVEAL_MS.max(1);
    let left = (IDLE_RETILE_COVER_TILE_PX - reveal_px) / 2;
    let right = left + reveal_px;
    let top = (IDLE_RETILE_COVER_TILE_PX - reveal_px) / 2;
    let bottom = top + reveal_px;

    push_idle_retile_cover_segment(
        batch,
        image,
        region,
        0,
        0,
        IDLE_RETILE_COVER_TILE_PX,
        top,
        x,
        y,
        origin_x,
        origin_y,
        angle_rad,
    );
    push_idle_retile_cover_segment(
        batch,
        image,
        region,
        0,
        bottom,
        IDLE_RETILE_COVER_TILE_PX,
        IDLE_RETILE_COVER_TILE_PX - bottom,
        x,
        y,
        origin_x,
        origin_y,
        angle_rad,
    );
    push_idle_retile_cover_segment(
        batch, image, region, 0, top, left, reveal_px, x, y, origin_x, origin_y, angle_rad,
    );
    push_idle_retile_cover_segment(
        batch,
        image,
        region,
        right,
        top,
        IDLE_RETILE_COVER_TILE_PX - right,
        reveal_px,
        x,
        y,
        origin_x,
        origin_y,
        angle_rad,
    );
}

fn push_idle_retile_cover_segment(
    batch: &mut SpriteBatch,
    image: &ImageAsset,
    region: ImageRegion,
    source_x: u32,
    source_y: u32,
    width: u32,
    height: u32,
    tile_x: f32,
    tile_y: f32,
    origin_x: f32,
    origin_y: f32,
    angle_rad: f32,
) {
    if width == 0 || height == 0 {
        return;
    }
    batch.image_region_rotated_around(
        image,
        ImageRegion::new(region.x + source_x, region.y + source_y, width, height),
        tile_x + source_x as f32,
        tile_y + source_y as f32,
        width as f32,
        height as f32,
        origin_x,
        origin_y,
        angle_rad,
        Rgba8::WHITE,
    );
}

fn lighten_rgba_toward_white(rgba: &mut [u8], percent: u8) {
    let percent = percent.min(100) as u16;
    for pixel in rgba.chunks_exact_mut(4) {
        for channel in &mut pixel[..3] {
            let value = *channel as u16;
            *channel = (value + ((255 - value) * percent + 50) / 100) as u8;
        }
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
            adapter.draw_tex_triangles_no_present(ts_ui::BIG_RIBBONS_TEXTURE, &title.ribbon_bytes);
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
        let mut health_bars = ts_ui::SmallBarBatch::new(self.window_width, self.window_height);
        let mut labels = ts_ui::UiBatch::new(self.window_width, self.window_height);

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

            let bar_w = cell_w.min(42.0).max(1.0).floor();
            let bar_h = (bar_w / 5.0).clamp(2.0, 8.0).floor().max(1.0);
            let bar_x = (image_x + (cell_w - bar_w) * 0.5).floor();
            let bar_y = (y - bar_h - 2.0).max(image_y + 2.0).floor();
            let team = DemoUnitTeam::for_unit_index(index);
            health_bars.small_bar(
                bar_x,
                bar_y,
                bar_w,
                bar_h,
                1.0,
                team.health_bar_color(),
                Rgba8::new(255, 255, 255, 245),
            );

            let label_text = unit_viewer_label(unit);
            let label_w = ui_text_width(&label_text, label_scale);
            labels.text(
                &label_text,
                image_x + (cell_w - label_w) * 0.5,
                image_y + image_h + 2.0,
                label_scale,
                Rgba8::new(220, 238, 232, 255),
            );
        }

        let _ = adapter
            .draw_tex_triangles_no_present(ts_ui::SMALL_BAR_BASE_TEXTURE, &health_bars.base_bytes);
        let _ = adapter.draw_rgb_triangles_no_present(&health_bars.fill_solid_bytes);
        let _ = adapter.draw_rgb_triangles_no_present(&labels.solid_bytes);

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

impl FrameProducer for IdleWorldViewer {
    fn cursor_visible(&self) -> bool {
        false
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.resize_view(width, height);
    }

    fn handle_input(&mut self, event: InputEvent) {
        match event {
            InputEvent::CursorMoved { x, y } => {
                if self.panning {
                    self.scroll_idle_camera(self.mouse.x - x, self.mouse.y - y);
                }
                self.mouse = Point { x, y };
            }
            InputEvent::MouseButton {
                button: InputMouseButton::Middle,
                state: InputButtonState::Pressed,
            } => {
                self.panning = true;
                self.selection_start = None;
            }
            InputEvent::MouseButton {
                button: InputMouseButton::Middle,
                state: InputButtonState::Released,
            } => {
                self.panning = false;
            }
            InputEvent::DigitPressed(digit) => {
                if !self.show_hotkey_menu && self.selected_units_include_pawn() {
                    if let Some(kind) = Self::idle_build_hotkey_kind(digit) {
                        self.placement_building = Some(kind);
                        self.selection_start = None;
                        return;
                    }
                }

                if digit == 2 {
                    self.show_hotkey_menu = !self.show_hotkey_menu;
                    self.selection_start = None;
                }
            }
            InputEvent::MouseButton {
                button: InputMouseButton::Left,
                state: InputButtonState::Pressed,
            } => {
                if self.show_hotkey_menu {
                    if !self.mouse_inside_hotkey_menu() {
                        self.show_hotkey_menu = false;
                    }
                    self.selection_start = None;
                } else if self.placement_building.is_some() {
                    self.place_idle_building_at_mouse();
                    self.selection_start = None;
                } else {
                    self.start_selection();
                }
            }
            InputEvent::MouseButton {
                button: InputMouseButton::Left,
                state: InputButtonState::Released,
            } => {
                if !self.show_hotkey_menu && self.placement_building.is_none() {
                    self.finish_selection();
                }
            }
            InputEvent::MouseButton {
                button: InputMouseButton::Right,
                state: InputButtonState::Pressed,
            } => {
                if self.placement_building.is_some() {
                    self.placement_building = None;
                    self.selection_start = None;
                } else if !self.show_hotkey_menu {
                    if !self.selected_units.is_empty() {
                        self.spawn_right_click_indicator();
                    }
                    if self.selected_units_are_only_pawns() {
                        if let Some(resource_target) =
                            self.idle_resource_target_at_mouse(self.elapsed_ms())
                        {
                            self.issue_resource_order(resource_target);
                            return;
                        }
                    }
                    self.issue_move_order(self.idle_world_point_at(self.mouse));
                }
            }
            InputEvent::EscapePressed => {
                if self.show_hotkey_menu {
                    self.show_hotkey_menu = false;
                    self.selection_start = None;
                } else if self.placement_building.is_some() {
                    self.placement_building = None;
                    self.selection_start = None;
                } else {
                    self.cancel_selection();
                }
            }
            _ => {}
        }
    }

    fn build_frame(&mut self, adapter: &mut Adapter) {
        self.upload_assets(adapter);
        self.update_units();

        let _ = adapter.begin_frame(WATER_BG);
        let _ = adapter.set_sampler_raw(0, 0, 0, 0);
        let _ = adapter.set_blend_raw(1, 0x0302, 0x0303);

        let elapsed_ms = self.started_at.elapsed().as_millis() as u32;
        self.draw_world_background(adapter, elapsed_ms);
        self.draw_saved_world_assets(adapter);
        self.draw_idle_retile_flyout(adapter);
        self.draw_idle_retile_cover(adapter);
        self.draw_idle_retile_particles(adapter);
        self.draw_idle_building_preview(adapter);

        for draw in self.unit_draws(elapsed_ms) {
            let mut image = SpriteBatch::new(self.window_width, self.window_height);
            let uv = if draw.flip_x {
                [1.0, 0.0, 0.0, 1.0]
            } else {
                [0.0, 0.0, 1.0, 1.0]
            };
            image.image_uv(
                draw.rect.x.floor(),
                draw.rect.y.floor(),
                draw.rect.w.floor().max(1.0),
                draw.rect.h.floor().max(1.0),
                uv,
                Rgba8::WHITE,
            );
            let _ = adapter.draw_tex_triangles_no_present(draw.texture_id, &image.bytes);
        }

        self.draw_clouds(adapter);
        self.draw_right_click_indicators(adapter);
        self.draw_selected_unit_ui(adapter, elapsed_ms);
        self.draw_pawn_build_ui(adapter, elapsed_ms);
        self.draw_hotkey_menu(adapter);
        if let Some(rect) = self.active_selection_rect() {
            self.draw_selection_corners(adapter, rect);
        }
        self.draw_cursor(adapter, elapsed_ms);

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    generated_source: Option<wldgenerator::GeneratedWorld>,
    backgrounds: Vec<BackgroundTile>,
    water_states: Vec<WaterState>,
    #[serde(default)]
    under_foregrounds: Vec<Option<AtlasTile>>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    hp: Option<u8>,
}

impl PlacedProp {
    fn new(kind: PropKind, x2: usize, y2: usize) -> Self {
        Self {
            kind,
            x2,
            y2,
            hp: None,
        }
    }

    fn effective_tree_hp(self) -> Option<u8> {
        let PropKind::Plant(kind) = self.kind else {
            return None;
        };
        kind.is_tree().then(|| {
            self.hp
                .unwrap_or_else(|| initial_tree_hp(kind, self.x2, self.y2))
                .clamp(1, IDLE_TREE_MAX_HP)
        })
    }
}

impl TileWorld {
    fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols,
            rows,
            generated_source: None,
            backgrounds: vec![BackgroundTile::Grass; cols * rows],
            water_states: vec![WaterState::Nothing; cols * rows],
            under_foregrounds: vec![None; cols * rows],
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
                self.under_foregrounds.splice(0..0, vec![None; self.cols]);
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
                self.under_foregrounds
                    .extend(std::iter::repeat_n(None, self.cols));
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
                self.under_foregrounds.drain(0..self.cols);
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
                self.under_foregrounds.drain(start..start + self.cols);
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
        self.under_foregrounds =
            insert_layer_column(&self.under_foregrounds, old_cols, rows, at, None);
        self.foregrounds = insert_layer_column(&self.foregrounds, old_cols, rows, at, None);
        self.fog = insert_layer_column(&self.fog, old_cols, rows, at, false);
        self.cols += 1;
    }

    fn remove_column(&mut self, at: usize) {
        let old_cols = self.cols;
        let rows = self.rows;
        self.backgrounds = remove_layer_column(&self.backgrounds, old_cols, rows, at);
        self.water_states = remove_layer_column(&self.water_states, old_cols, rows, at);
        self.under_foregrounds = remove_layer_column(&self.under_foregrounds, old_cols, rows, at);
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

    fn normalize_prop_hp(&mut self) {
        for prop in &mut self.props {
            match prop.kind {
                PropKind::Plant(kind) if kind.is_tree() => {
                    if prop.hp == Some(0) {
                        if let Some(stump) = kind.stump() {
                            prop.kind = PropKind::Plant(stump);
                            prop.hp = None;
                        }
                    } else if let Some(hp) = prop.hp {
                        prop.hp = Some(hp.clamp(1, IDLE_TREE_MAX_HP));
                    }
                }
                _ => prop.hp = None,
            }
        }
    }

    fn idle_resource_available(&self, task: IdleResourceTask) -> bool {
        match task.kind {
            IdleResourceKind::Wood => self.idle_tree_prop_index_at_target(task.resource).is_some(),
            IdleResourceKind::Meat => true,
            IdleResourceKind::Ore => self.props.iter().any(|prop| {
                matches!(prop.kind, PropKind::Gold(_) | PropKind::Rock(_))
                    && point_distance(self.idle_resource_prop_target(*prop), task.resource) <= 1.0
            }),
        }
    }

    fn collect_idle_resource(&mut self, task: IdleResourceTask) -> Option<IdleWorldEvent> {
        match task.kind {
            IdleResourceKind::Wood => self.collect_idle_tree_resource(task.resource),
            IdleResourceKind::Meat | IdleResourceKind::Ore => self
                .idle_resource_available(task)
                .then_some(IdleWorldEvent::ResourceCollected(IdleResourceCollected {
                    kind: task.kind,
                    position: task.resource,
                    depleted: false,
                })),
        }
    }

    fn collect_idle_tree_resource(&mut self, target: Point) -> Option<IdleWorldEvent> {
        let index = self.idle_tree_prop_index_at_target(target)?;
        let prop = self.props[index];
        let PropKind::Plant(kind) = prop.kind else {
            return None;
        };
        let hp = prop.effective_tree_hp()?;
        let next_hp = hp.saturating_sub(1);
        let depleted = next_hp == 0;

        if depleted {
            let stump = kind.stump()?;
            self.props[index].kind = PropKind::Plant(stump);
            self.props[index].hp = None;
        } else {
            self.props[index].hp = Some(next_hp);
        }

        Some(IdleWorldEvent::ResourceCollected(IdleResourceCollected {
            kind: IdleResourceKind::Wood,
            position: target,
            depleted,
        }))
    }

    fn idle_tree_prop_index_at_target(&self, target: Point) -> Option<usize> {
        self.props.iter().position(|prop| {
            matches!(prop.kind, PropKind::Plant(kind) if kind.is_tree())
                && point_distance(self.idle_resource_prop_target(*prop), target) <= 1.0
        })
    }

    fn idle_resource_prop_target(&self, prop: PlacedProp) -> Point {
        idle_resource_prop_target_in_world(self.cols, self.rows, prop)
    }

    fn replace_with_generated_rect(&mut self, rect: IdleRetileRect, seed: u64) {
        let Some(rect) = clamp_idle_retile_rect(rect, self.cols, self.rows) else {
            return;
        };
        if self.replace_with_generated_source_rect(rect, seed) {
            return;
        }

        let (patch, patch_col_offset, patch_row_offset) =
            generated_idle_retile_patch(rect, self.cols, self.rows, seed);
        let inner_x2 = patch_col_offset * BUILDING_GRID_DIVISIONS;
        let inner_y2 = patch_row_offset * BUILDING_GRID_DIVISIONS;
        let inner_w2 = rect.cols * BUILDING_GRID_DIVISIONS;
        let inner_h2 = rect.rows * BUILDING_GRID_DIVISIONS;
        let rect2 = (
            (rect.col * BUILDING_GRID_DIVISIONS) as isize,
            (rect.row * BUILDING_GRID_DIVISIONS) as isize,
            rect.cols * BUILDING_GRID_DIVISIONS,
            rect.rows * BUILDING_GRID_DIVISIONS,
        );
        self.remove_objects_overlapping(rect2);

        for local_row in 0..rect.rows {
            for local_col in 0..rect.cols {
                let target = self.index(rect.col + local_col, rect.row + local_row);
                let source =
                    patch.index(local_col + patch_col_offset, local_row + patch_row_offset);
                self.backgrounds[target] = patch.backgrounds[source];
                self.water_states[target] = patch.water_states[source];
                self.under_foregrounds[target] = patch.under_foregrounds[source];
                self.foregrounds[target] = patch.foregrounds[source];
                self.fog[target] = patch.fog[source];
            }
        }

        let offset_x2 = (rect.col * BUILDING_GRID_DIVISIONS) as isize - inner_x2 as isize;
        let offset_y2 = (rect.row * BUILDING_GRID_DIVISIONS) as isize - inner_y2 as isize;
        for building in patch.buildings {
            let footprint = building_footprint_rect2(building);
            if !rects_overlap(
                (inner_x2 as isize, inner_y2 as isize, inner_w2, inner_h2),
                footprint,
            ) {
                continue;
            }
            self.buildings.push(PlacedBuilding {
                kind: building.kind,
                x2: building.x2 + offset_x2,
                y2: building.y2 + offset_y2,
            });
        }
        for prop in patch.props {
            let footprint = prop_footprint_rect2(prop);
            if !rects_overlap(
                (inner_x2 as isize, inner_y2 as isize, inner_w2, inner_h2),
                footprint,
            ) {
                continue;
            }
            let Ok(x2) = usize::try_from(prop.x2 as isize + offset_x2) else {
                continue;
            };
            let Ok(y2) = usize::try_from(prop.y2 as isize + offset_y2) else {
                continue;
            };
            if x2 < self.cols * BUILDING_GRID_DIVISIONS && y2 < self.rows * BUILDING_GRID_DIVISIONS
            {
                self.props.push(PlacedProp::new(prop.kind, x2, y2));
            }
        }
        self.sort_props();
    }

    fn replace_with_generated_source_rect(&mut self, rect: IdleRetileRect, seed: u64) -> bool {
        let Some(mut generated) = self.generated_source.clone() else {
            return false;
        };
        if generated.cols != self.cols
            || generated.rows != self.rows
            || generated.validate().is_err()
        {
            self.generated_source = None;
            return false;
        }

        let reroll_mask =
            generated.feature_reroll_mask_for_rect(rect.col, rect.row, rect.cols, rect.rows);
        if !reroll_mask.iter().any(|&reroll| reroll) {
            return false;
        }
        generated.reroll_features_in_mask(&reroll_mask, seed);
        let mut pending_generator = wldgenerator::RunningGenerator::from_world(generated, seed);
        pending_generator.fill_visual_voids_once(self.cols.saturating_mul(self.rows));
        let generated = pending_generator.world().clone();
        let visual_world = generated.to_visual_tile_world();
        let copy_rect =
            reroll_mask_bounding_rect(self.cols, self.rows, &reroll_mask, 1).unwrap_or(rect);

        self.copy_visual_rect_from_generated_world(&visual_world.tiles, copy_rect);
        self.generated_source = Some(generated);
        true
    }

    fn copy_visual_rect_from_generated_world(&mut self, source: &TileWorld, rect: IdleRetileRect) {
        let Some(rect) = clamp_idle_retile_rect(rect, self.cols, self.rows) else {
            return;
        };
        let rect2 = (
            (rect.col * BUILDING_GRID_DIVISIONS) as isize,
            (rect.row * BUILDING_GRID_DIVISIONS) as isize,
            rect.cols * BUILDING_GRID_DIVISIONS,
            rect.rows * BUILDING_GRID_DIVISIONS,
        );
        self.remove_objects_overlapping(rect2);

        for row in rect.row..rect.row + rect.rows {
            for col in rect.col..rect.col + rect.cols {
                let target = self.index(col, row);
                let source_index = source.index(col, row);
                self.backgrounds[target] = source.backgrounds[source_index];
                self.water_states[target] = source.water_states[source_index];
                self.under_foregrounds[target] = source.under_foregrounds[source_index];
                self.foregrounds[target] = source.foregrounds[source_index];
            }
        }

        for building in &source.buildings {
            if rects_overlap(rect2, building_footprint_rect2(*building)) {
                self.buildings.push(*building);
            }
        }
        for prop in &source.props {
            if rects_overlap(rect2, prop_footprint_rect2(*prop)) {
                self.props.push(*prop);
            }
        }
        self.sort_props();
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
        world.normalize_loaded_state();
        world.validate()?;
        Ok(world)
    }

    fn load_for_editor_from_path(path: impl AsRef<Path>) -> Result<Self, String> {
        let mut world = Self::load_from_path(path)?;
        world.align_visuals_with_generated_source();
        world.validate()?;
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
            ("under_foregrounds", self.under_foregrounds.len()),
            ("foregrounds", self.foregrounds.len()),
            ("fog", self.fog.len()),
        ] {
            if len != cells {
                return Err(format!("{name} length {len} does not match {cells} cells"));
            }
        }
        if let Some(generated) = &self.generated_source {
            if generated.cols != self.cols || generated.rows != self.rows {
                return Err("generated source dimensions do not match visual world".to_string());
            }
            generated.validate()?;
        }
        Ok(())
    }

    fn normalize_loaded_state(&mut self) {
        let cells = self.cols.saturating_mul(self.rows);
        if self.under_foregrounds.len() != cells {
            self.under_foregrounds = vec![None; cells];
        }
        for state in &mut self.water_states {
            if *state == WaterState::Animation {
                *state = WaterState::Nothing;
            }
        }
        if self.generated_source.as_ref().is_some_and(|generated| {
            generated.cols != self.cols
                || generated.rows != self.rows
                || generated.validate().is_err()
        }) {
            self.generated_source = None;
        }
        self.normalize_prop_hp();
        self.sort_props();
    }

    fn align_visuals_with_generated_source(&mut self) -> bool {
        let Some(generated) = self.generated_source.clone() else {
            return false;
        };
        if generated.cols != self.cols
            || generated.rows != self.rows
            || generated.validate().is_err()
        {
            self.generated_source = None;
            return false;
        }

        let visual = generated.to_visual_tile_world().tiles;
        self.backgrounds = visual.backgrounds;
        self.water_states = visual.water_states;
        self.under_foregrounds = visual.under_foregrounds;
        self.foregrounds = visual.foregrounds;
        self.sort_props();
        true
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

    fn cell_accepts_wave_animation(&self, col: usize, row: usize) -> bool {
        self.background(col, row) == BackgroundTile::Water
            && self
                .foreground(col, row)
                .is_some_and(shoreline_tile_accepts_wave)
    }

    fn foreground(&self, col: usize, row: usize) -> Option<AtlasTile> {
        self.foregrounds[self.index(col, row)]
    }

    fn under_foreground(&self, col: usize, row: usize) -> Option<AtlasTile> {
        self.under_foregrounds[self.index(col, row)]
    }

    fn has_tile_overlay(&self, col: usize, row: usize) -> bool {
        let index = self.index(col, row);
        self.under_foregrounds[index].is_some() || self.foregrounds[index].is_some()
    }

    fn render_background(&self, col: usize, row: usize) -> BackgroundTile {
        let background = self.background(col, row);
        if self
            .foreground(col, row)
            .is_some_and(|tile| shoreline_tile_forces_water_background(tile, background))
        {
            BackgroundTile::Water
        } else {
            background
        }
    }

    fn idle_move_path(&self, from: Point, to: Point) -> Vec<Point> {
        let Some(start) = self.idle_path_cell_at(from) else {
            return vec![to];
        };
        let Some(goal) = self.idle_path_cell_at(to) else {
            return vec![to];
        };
        let Some(cells) = self.idle_astar_cells(start, goal) else {
            return vec![to];
        };
        if cells.len() <= 1 {
            return vec![to];
        }

        let mut points = Vec::with_capacity(cells.len() - 1);
        for (index, cell) in cells.iter().skip(1).enumerate() {
            if index + 2 == cells.len() {
                points.push(to);
            } else {
                points.push(Self::idle_path_cell_center(*cell));
            }
        }
        points
    }

    fn idle_astar_cells(
        &self,
        start: IdlePathCell,
        goal: IdlePathCell,
    ) -> Option<Vec<IdlePathCell>> {
        let cells = self.cols.checked_mul(self.rows)?;
        let start_index = self.index(start.col, start.row);
        let goal_index = self.index(goal.col, goal.row);
        let mut costs = vec![u32::MAX; cells];
        let mut came_from = vec![None; cells];
        let mut frontier = BinaryHeap::new();

        costs[start_index] = 0;
        frontier.push(IdlePathQueueEntry {
            estimated_total: Self::idle_path_heuristic(start, goal),
            cost: 0,
            index: start_index,
        });

        while let Some(entry) = frontier.pop() {
            if entry.cost != costs[entry.index] {
                continue;
            }
            if entry.index == goal_index {
                return Some(self.reconstruct_idle_path(start_index, goal_index, &came_from));
            }

            let current = self.idle_path_cell_for_index(entry.index);
            for row_delta in -1..=1 {
                for col_delta in -1..=1 {
                    if col_delta == 0 && row_delta == 0 {
                        continue;
                    }
                    let Some(next) = self.idle_neighbor_cell(current, col_delta, row_delta) else {
                        continue;
                    };
                    let next_index = self.index(next.col, next.row);
                    let step_cost = if col_delta != 0 && row_delta != 0 {
                        IDLE_PATH_DIAGONAL_COST
                    } else {
                        IDLE_PATH_CARDINAL_COST
                    };
                    let next_cost = entry
                        .cost
                        .saturating_add(step_cost)
                        .saturating_add(self.idle_path_cell_penalty(next));
                    if next_cost >= costs[next_index] {
                        continue;
                    }

                    costs[next_index] = next_cost;
                    came_from[next_index] = Some(entry.index);
                    frontier.push(IdlePathQueueEntry {
                        estimated_total: next_cost + Self::idle_path_heuristic(next, goal),
                        cost: next_cost,
                        index: next_index,
                    });
                }
            }
        }

        None
    }

    fn reconstruct_idle_path(
        &self,
        start_index: usize,
        goal_index: usize,
        came_from: &[Option<usize>],
    ) -> Vec<IdlePathCell> {
        let mut current = goal_index;
        let mut path = vec![self.idle_path_cell_for_index(current)];
        while current != start_index {
            let Some(previous) = came_from[current] else {
                break;
            };
            current = previous;
            path.push(self.idle_path_cell_for_index(current));
        }
        path.reverse();
        path
    }

    fn idle_path_cell_penalty(&self, cell: IdlePathCell) -> u32 {
        let mut penalty = 0;
        if self.render_background(cell.col, cell.row) == BackgroundTile::Water {
            penalty += IDLE_PATH_WATER_PENALTY;
        }
        if self
            .foreground(cell.col, cell.row)
            .is_some_and(is_idle_path_cliff_tile)
            || self
                .under_foreground(cell.col, cell.row)
                .is_some_and(is_idle_path_cliff_tile)
        {
            penalty += IDLE_PATH_CLIFF_PENALTY;
        }
        if self
            .foreground(cell.col, cell.row)
            .is_some_and(is_idle_path_stone_tile)
        {
            penalty += IDLE_PATH_OBJECT_PENALTY;
        }
        if self.idle_path_cell_overlaps_object(cell) {
            penalty += IDLE_PATH_OBJECT_PENALTY;
        }
        penalty
    }

    fn idle_path_cell_overlaps_object(&self, cell: IdlePathCell) -> bool {
        let rect = (
            (cell.col * BUILDING_GRID_DIVISIONS) as isize,
            (cell.row * BUILDING_GRID_DIVISIONS) as isize,
            BUILDING_GRID_DIVISIONS,
            BUILDING_GRID_DIVISIONS,
        );
        self.props.iter().any(|prop| {
            matches!(
                prop.kind,
                PropKind::Plant(_) | PropKind::Gold(_) | PropKind::Rock(_)
            ) && rects_overlap(rect, prop_footprint_rect2(*prop))
        }) || self
            .buildings
            .iter()
            .any(|building| rects_overlap(rect, building_footprint_rect2(*building)))
    }

    fn idle_path_cell_at(&self, point: Point) -> Option<IdlePathCell> {
        if self.cols == 0 || self.rows == 0 {
            return None;
        }
        let col = (point.x / TILE_SIZE).floor() as isize;
        let row = (point.y / TILE_SIZE).floor() as isize;
        Some(IdlePathCell {
            col: col.clamp(0, self.cols as isize - 1) as usize,
            row: row.clamp(0, self.rows as isize - 1) as usize,
        })
    }

    fn idle_neighbor_cell(
        &self,
        cell: IdlePathCell,
        col_delta: isize,
        row_delta: isize,
    ) -> Option<IdlePathCell> {
        let col = cell.col as isize + col_delta;
        let row = cell.row as isize + row_delta;
        (col >= 0 && row >= 0 && col < self.cols as isize && row < self.rows as isize).then_some(
            IdlePathCell {
                col: col as usize,
                row: row as usize,
            },
        )
    }

    fn idle_path_cell_for_index(&self, index: usize) -> IdlePathCell {
        IdlePathCell {
            col: index % self.cols,
            row: index / self.cols,
        }
    }

    fn idle_path_cell_center(cell: IdlePathCell) -> Point {
        Point {
            x: cell.col as f32 * TILE_SIZE + TILE_SIZE * 0.5,
            y: cell.row as f32 * TILE_SIZE + TILE_SIZE * 0.5,
        }
    }

    fn idle_path_heuristic(a: IdlePathCell, b: IdlePathCell) -> u32 {
        let dx = a.col.abs_diff(b.col) as u32;
        let dy = a.row.abs_diff(b.row) as u32;
        let diagonal = dx.min(dy);
        let cardinal = dx.max(dy) - diagonal;
        diagonal * IDLE_PATH_DIAGONAL_COST + cardinal * IDLE_PATH_CARDINAL_COST
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
                self.under_foregrounds[index] = None;
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
        self.under_foregrounds[index] = None;
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
            self.props.push(PlacedProp::new(kind, x2, y2));
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

    fn image_region_rotated_around(
        &mut self,
        image: &ImageAsset,
        region: ImageRegion,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        origin_x: f32,
        origin_y: f32,
        angle_rad: f32,
        color: Rgba8,
    ) {
        self.image_uv_rotated_around(
            x,
            y,
            w,
            h,
            region.uv(image),
            origin_x,
            origin_y,
            angle_rad,
            color,
        );
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

    fn image_uv_rotated_around(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        uv: [f32; 4],
        origin_x: f32,
        origin_y: f32,
        angle_rad: f32,
        color: Rgba8,
    ) {
        let [u0, v0, u1, v1] = uv;
        let sin = angle_rad.sin();
        let cos = angle_rad.cos();
        let corners = [
            (x, y, u0, v0),
            (x + w, y, u1, v0),
            (x + w, y + h, u1, v1),
            (x, y + h, u0, v1),
        ];
        let vertices = corners.map(|(x, y, u, v)| {
            let dx = x - origin_x;
            let dy = y - origin_y;
            let (x, y) = (
                origin_x + dx * cos - dy * sin,
                origin_y + dx * sin + dy * cos,
            );
            let (x, y) = self.to_clip(x, y);
            TexVertex { x, y, u, v, color }
        });

        for index in [0, 1, 2, 0, 2, 3] {
            self.vertex(vertices[index]);
        }
    }

    fn image_rotated_mirror_x(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        angle_rad: f32,
        mirror_x: bool,
        color: Rgba8,
    ) {
        let (u0, u1) = if mirror_x { (1.0, 0.0) } else { (0.0, 1.0) };
        let cx = x + w * 0.5;
        let cy = y + h * 0.5;
        let sin = angle_rad.sin();
        let cos = angle_rad.cos();
        let corners = [
            (-w * 0.5, -h * 0.5, u0, 0.0),
            (w * 0.5, -h * 0.5, u1, 0.0),
            (w * 0.5, h * 0.5, u1, 1.0),
            (-w * 0.5, h * 0.5, u0, 1.0),
        ];
        let vertices = corners.map(|(dx, dy, u, v)| {
            let (x, y) = (cx + dx * cos - dy * sin, cy + dx * sin + dy * cos);
            let (x, y) = self.to_clip(x, y);
            TexVertex { x, y, u, v, color }
        });

        for index in [0, 1, 2, 0, 2, 3] {
            self.vertex(vertices[index]);
        }
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

fn rects_intersect(a: TableRect, b: TableRect) -> bool {
    a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y
}

fn point_distance(a: Point, b: Point) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
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

fn scroll_strength(distance_from_edge: f32) -> f32 {
    ((EDGE_SCROLL_ZONE - distance_from_edge.max(0.0)) / EDGE_SCROLL_ZONE).clamp(0.0, 1.0)
}

fn is_ramp_part(tile: AtlasTile) -> bool {
    tile == RAMP_A.top || tile == RAMP_A.bottom || tile == RAMP_B.top || tile == RAMP_B.bottom
}

fn is_pillar_tile(tile: AtlasTile) -> bool {
    PILLAR_TILES.contains(&tile)
}

fn is_idle_path_stone_tile(tile: AtlasTile) -> bool {
    is_pillar_tile(tile)
        || tile == wldgenerator::STANDALONE_GRASS_PILLAR_TILE
        || tile == wldgenerator::STANDALONE_WATER_PILLAR_TILE
}

fn is_idle_path_cliff_tile(tile: AtlasTile) -> bool {
    wldgenerator::PLATFORM_GRASS_CLIFF_CAP_TILES.contains(&tile)
        || wldgenerator::VERTICAL_GRASS_CLIFF_TILES.contains(&tile)
        || wldgenerator::PLATFORM_GRASS_ROW_TILES.contains(&tile)
        || wldgenerator::PLATFORM_WATER_ROW_TILES.contains(&tile)
        || wldgenerator::PLATFORM_GRASS_BORDER_TILES
            .iter()
            .flatten()
            .any(|&border| border == tile)
        || tile == wldgenerator::CLIFF_GRASS_CAP_TILE
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
        Brush::RockResource if anchor_x2 >= 0 && anchor_y2 >= 0 => prop_kind_footprint_rect2(
            PropKind::Rock(RockKind::Rock1),
            anchor_x2 as usize,
            anchor_y2 as usize,
        ),
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
        PropKind::Rock(rock) if rock.uses_half_height_footprint() => {
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

fn idle_resource_prop_target_in_world(cols: usize, rows: usize, prop: PlacedProp) -> Point {
    let (x2, y2, w2, h2) = prop_footprint_rect2(prop);
    let front_y2 = y2 + h2 as isize;
    if x2 >= 0
        && front_y2 >= 0
        && x2 + w2 as isize <= (cols * BUILDING_GRID_DIVISIONS) as isize
        && front_y2 + BUILDING_GRID_DIVISIONS as isize <= (rows * BUILDING_GRID_DIVISIONS) as isize
    {
        return Point {
            x: signed_half_grid_to_px(x2) + half_grid_to_px(w2) * 0.5,
            y: signed_half_grid_to_px(front_y2) + TILE_SIZE * 0.5,
        };
    }

    Point {
        x: half_grid_to_px(prop.x2) + TILE_SIZE * 0.5,
        y: half_grid_to_px(prop.y2 + BUILDING_GRID_DIVISIONS),
    }
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

fn shoreline_tile_forces_water_background(tile: AtlasTile, background: BackgroundTile) -> bool {
    is_shoreline_tile(tile)
        && !(tile == SHORE_SINGLE_IN_GRASS && background == BackgroundTile::Grass)
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

fn image_point_has_alpha(image: &ImageAsset, point: Point, rect: TableRect) -> bool {
    if rect.w <= 0.0 || rect.h <= 0.0 {
        return false;
    }
    let px = ((point.x - rect.x) / rect.w * image.width as f32).floor() as usize;
    let py = ((point.y - rect.y) / rect.h * image.height as f32).floor() as usize;
    if px >= image.width as usize || py >= image.height as usize {
        return false;
    }
    image.rgba[(py * image.width as usize + px) * 4 + 3] != 0
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

fn initial_tree_hp(kind: PlantKind, x2: usize, y2: usize) -> u8 {
    let mut rng = SeededRng::new(
        0x7AEE_5A9E_EC05_2026
            ^ ((kind.index() as u64).wrapping_mul(0x94D0_49BB_1331_11EB))
            ^ ((x2 as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
            ^ ((y2 as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9)),
    );
    IDLE_TREE_MIN_HP + (rng.next_u64() % u64::from(IDLE_TREE_MAX_HP - IDLE_TREE_MIN_HP + 1)) as u8
}

fn rock_visual_roll(kind: RockKind, col: usize, row: usize) -> u8 {
    let mut rng = SeededRng::new(
        0x5702_E51E_EC05_2026
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
mod idle_world_tests {
    use super::*;

    #[test]
    fn warrior_has_both_arrival_attacks() {
        let viewer = IdleWorldViewer::new();
        let warrior = &viewer.units[3];
        let attack_names = warrior
            .attacks
            .iter()
            .map(|clip| clip.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(warrior.idle.name, "Warrior Idle");
        assert_eq!(attack_names, vec!["Warrior Attack 1", "Warrior Attack 2"]);
    }

    #[test]
    fn warrior_arrival_attack_coinflip_can_pick_either_attack() {
        let mut viewer = IdleWorldViewer::new();
        let warrior_index = 3;
        let target = viewer.units[warrior_index].position;
        let mut seen = HashSet::new();

        for _ in 0..16 {
            viewer.units[warrior_index].state = IdleWorldUnitState::Idle;
            viewer.units[warrior_index].position = target;
            viewer.selected_units = vec![warrior_index];
            viewer.issue_move_order(target);
            viewer.issue_move_order(target);
            viewer.update_units();
            if let IdleWorldUnitState::Attacking { attack_index, .. } =
                viewer.units[warrior_index].state
            {
                seen.insert(attack_index);
            }
        }

        assert_eq!(seen, HashSet::from([0, 1]));
    }

    #[test]
    fn moving_units_face_their_horizontal_vector() {
        let mut viewer = IdleWorldViewer::new();
        let unit_index = 0;
        let start = viewer.units[unit_index].position;

        viewer.selected_units = vec![unit_index];
        viewer.issue_move_order(Point {
            x: start.x - 64.0,
            y: start.y,
        });
        assert!(viewer.units[unit_index].facing_left);
        assert!(viewer.unit_draws(0)[0].flip_x);

        viewer.issue_move_order(Point {
            x: start.x + 64.0,
            y: start.y,
        });
        assert!(!viewer.units[unit_index].facing_left);
    }

    #[test]
    fn repeated_nearby_move_command_switches_from_walk_to_run() {
        let mut viewer = IdleWorldViewer::new();
        let unit_index = 0;
        let target = Point {
            x: viewer.units[unit_index].position.x + 80.0,
            y: viewer.units[unit_index].position.y,
        };

        viewer.selected_units = vec![unit_index];
        viewer.issue_move_order(target);
        let IdleWorldUnitState::Moving {
            running: false,
            started_ms,
            ..
        } = viewer.units[unit_index].state
        else {
            panic!("first right-click should walk");
        };
        assert_eq!(
            IdleWorldViewer::unit_clip_and_elapsed(&viewer.units[unit_index], started_ms + 100).1,
            50
        );

        viewer.issue_move_order(Point {
            x: target.x + 3.0,
            y: target.y + 4.0,
        });
        let IdleWorldUnitState::Moving {
            running: true,
            started_ms,
            ..
        } = viewer.units[unit_index].state
        else {
            panic!("nearby repeated right-click should run");
        };
        assert_eq!(
            IdleWorldViewer::unit_clip_and_elapsed(&viewer.units[unit_index], started_ms + 100).1,
            100
        );

        viewer.issue_move_order(Point {
            x: target.x + 10.0,
            y: target.y,
        });
        assert!(matches!(
            viewer.units[unit_index].state,
            IdleWorldUnitState::Moving { running: false, .. }
        ));
    }

    #[test]
    fn idle_move_path_avoids_water_when_grass_detour_exists() {
        let mut world = TileWorld::new(5, 3);
        world.set_background(2, 1, BackgroundTile::Water);

        let path = world.idle_move_path(path_cell_center(0, 1), path_cell_center(4, 1));
        let cells = path
            .iter()
            .filter_map(|&point| world.idle_path_cell_at(point))
            .collect::<Vec<_>>();

        assert!(!cells.contains(&IdlePathCell { col: 2, row: 1 }));
        assert_eq!(cells.last(), Some(&IdlePathCell { col: 4, row: 1 }));
    }

    #[test]
    fn idle_move_path_can_cross_water_when_no_detour_exists() {
        let mut world = TileWorld::new(3, 1);
        world.set_background(1, 0, BackgroundTile::Water);

        let path = world.idle_move_path(path_cell_center(0, 0), path_cell_center(2, 0));
        let cells = path
            .iter()
            .filter_map(|&point| world.idle_path_cell_at(point))
            .collect::<Vec<_>>();

        assert!(cells.contains(&IdlePathCell { col: 1, row: 0 }));
        assert_eq!(cells.last(), Some(&IdlePathCell { col: 2, row: 0 }));
    }

    #[test]
    fn idle_resource_prop_target_stops_in_front_tile_center() {
        let mut viewer = IdleWorldViewer::new();
        viewer.world = TileWorld::new(4, 4);
        let prop = PlacedProp::new(
            PropKind::Gold(GoldKind::Stone1),
            BUILDING_GRID_DIVISIONS,
            BUILDING_GRID_DIVISIONS,
        );

        assert_eq!(
            viewer.idle_resource_prop_target(prop),
            path_cell_center(1, 2)
        );
    }

    #[test]
    fn untouched_trees_have_deterministic_hp_between_five_and_seven() {
        let prop = PlacedProp::new(PropKind::Plant(PlantKind::Tree2), 2, 4);
        let hp = prop.effective_tree_hp().expect("tree should have hp");

        assert!((IDLE_TREE_MIN_HP..=IDLE_TREE_MAX_HP).contains(&hp));
        assert_eq!(prop.hp, None);
        assert_eq!(prop.effective_tree_hp(), Some(hp));
    }

    #[test]
    fn wood_collection_decrements_tree_hp_and_turns_it_into_stump() {
        let mut world = TileWorld::new(4, 4);
        world.paint_prop_half(PropKind::Plant(PlantKind::Tree1), 0, 0);
        let target = world.idle_resource_prop_target(world.props[0]);
        let task = IdleResourceTask {
            kind: IdleResourceKind::Wood,
            resource: target,
            house: None,
        };
        let full_hp = world.props[0]
            .effective_tree_hp()
            .expect("tree should start with hp");

        for expected_hp in (1..full_hp).rev() {
            let event = world
                .collect_idle_resource(task)
                .expect("tree collection should succeed before depletion");
            assert_eq!(
                event,
                IdleWorldEvent::ResourceCollected(IdleResourceCollected {
                    kind: IdleResourceKind::Wood,
                    position: target,
                    depleted: false,
                })
            );
            assert_eq!(world.props[0].kind, PropKind::Plant(PlantKind::Tree1));
            assert_eq!(world.props[0].hp, Some(expected_hp));
        }

        let event = world
            .collect_idle_resource(task)
            .expect("last tree collection should succeed");
        assert_eq!(
            event,
            IdleWorldEvent::ResourceCollected(IdleResourceCollected {
                kind: IdleResourceKind::Wood,
                position: target,
                depleted: true,
            })
        );
        assert_eq!(world.props[0].kind, PropKind::Plant(PlantKind::Stump1));
        assert_eq!(world.props[0].hp, None);
        assert!(!world.idle_resource_available(task));
        assert_eq!(world.collect_idle_resource(task), None);
    }

    #[test]
    fn idle_retile_patch_supports_minimum_rect_size() {
        let mut world = TileWorld::new(4, 4);
        world.replace_with_generated_rect(
            IdleRetileRect {
                col: 1,
                row: 1,
                cols: IDLE_MONK_RETILE_MIN_SIZE,
                rows: IDLE_MONK_RETILE_MIN_SIZE,
            },
            DEFAULT_SEED,
        );

        assert_eq!(world.cols, 4);
        assert_eq!(world.rows, 4);
        assert_eq!(world.backgrounds.len(), 16);
        assert_eq!(world.foregrounds.len(), 16);
    }

    #[test]
    fn generated_source_round_trips_with_tile_world_json() {
        let mut generator = wldgenerator::RunningGenerator::new(12, 12, DEFAULT_SEED);
        generator.complete_initial_seeds();
        generator.fill_visual_voids_once(12 * 12);
        let mut world = generator.world().to_visual_tile_world().tiles;
        world.generated_source = Some(generator.world().clone());

        let json = serde_json::to_string(&world).expect("tile world should serialize");
        assert!(json.contains("generated_source"));
        let mut decoded =
            serde_json::from_str::<TileWorld>(&json).expect("tile world should deserialize");
        decoded.normalize_loaded_state();
        decoded
            .validate()
            .expect("generated metadata should validate");
        assert!(decoded.generated_source.is_some());
    }

    #[test]
    fn editor_load_realigns_generated_visuals_from_metadata() {
        let mut generator = wldgenerator::RunningGenerator::new(12, 12, DEFAULT_SEED);
        generator.complete_initial_seeds();
        generator.fill_visual_voids_once(12 * 12);
        let generated = generator.world().clone();
        let expected = generated.to_visual_tile_world().tiles;
        let mut saved = expected.clone();
        saved.generated_source = Some(generated.clone());
        saved.backgrounds[0] = match saved.backgrounds[0] {
            BackgroundTile::Grass => BackgroundTile::Water,
            BackgroundTile::Water => BackgroundTile::Grass,
        };
        saved.buildings.push(PlacedBuilding {
            kind: BuildingKind::House1,
            x2: 0,
            y2: 0,
        });

        let path = std::env::temp_dir().join(format!(
            "tactics_editor_generated_{}_{}.json",
            std::process::id(),
            DEFAULT_SEED
        ));
        let _ = std::fs::remove_file(&path);
        saved
            .save_to_path(&path)
            .expect("generated test world should save");
        let loaded = TileWorld::load_for_editor_from_path(&path)
            .expect("generated editor world should load");
        let _ = std::fs::remove_file(&path);

        assert_eq!(loaded.generated_source.as_ref(), Some(&generated));
        assert_eq!(loaded.backgrounds, expected.backgrounds);
        assert_eq!(loaded.water_states, expected.water_states);
        assert_eq!(loaded.under_foregrounds, expected.under_foregrounds);
        assert_eq!(loaded.foregrounds, expected.foregrounds);
        assert_eq!(loaded.buildings, saved.buildings);
        assert_eq!(loaded.props, saved.props);
    }

    #[test]
    fn object_brushes_keep_generated_source_usable() {
        assert!(Brush::Background(BackgroundTile::Water).invalidates_generated_source());
        assert!(Brush::Foreground(GRASS_BG_TILE).invalidates_generated_source());
        assert!(Brush::Ramp(RAMP_A).invalidates_generated_source());

        assert!(!Brush::Building(BuildingKind::House1).invalidates_generated_source());
        assert!(!Brush::Prop(PropKind::Pillar(PILLAR_TILES[0])).invalidates_generated_source());
        assert!(!Brush::GoldResource.invalidates_generated_source());
        assert!(!Brush::RockResource.invalidates_generated_source());
        assert!(!Brush::FogRect.invalidates_generated_source());
        assert!(!Brush::ClearForeground.invalidates_generated_source());
    }

    #[test]
    fn generated_source_drives_idle_retile_replacement() {
        let mut generator = wldgenerator::RunningGenerator::new(18, 18, DEFAULT_SEED);
        generator.complete_initial_seeds();
        generator.fill_visual_voids_once(18 * 18);
        let mut world = generator.world().to_visual_tile_world().tiles;
        world.generated_source = Some(generator.world().clone());

        world.replace_with_generated_rect(
            IdleRetileRect {
                col: 4,
                row: 4,
                cols: 5,
                rows: 5,
            },
            DEFAULT_SEED ^ 0xA11E_2026,
        );

        assert!(world.generated_source.is_some());
        world
            .validate()
            .expect("retiled generated source should stay valid");
    }

    #[test]
    fn monk_retile_procs_when_move_arrival_starts_attack() {
        let mut viewer = IdleWorldViewer::new();
        let monk_index = 2;
        let target = viewer.units[monk_index].position;

        viewer.update_units();
        assert!(viewer.retile_transition.is_none());

        viewer.selected_units = vec![monk_index];
        viewer.issue_move_order(target);
        viewer.update_units();

        assert!(viewer.retile_transition.is_some());
        assert!(matches!(
            viewer.units[monk_index].state,
            IdleWorldUnitState::Attacking { .. }
        ));
    }

    #[test]
    fn monk_retile_has_no_cooldown_between_attack_procs() {
        let mut viewer = IdleWorldViewer::new();
        let monk_index = 2;
        let target = viewer.units[monk_index].position;

        for _ in 0..2 {
            viewer.retile_transition = None;
            viewer.units[monk_index].state = IdleWorldUnitState::Idle;
            viewer.selected_units = vec![monk_index];
            viewer.issue_move_order(target);
            viewer.update_units();
            assert!(viewer.retile_transition.is_some());
        }
    }

    fn path_cell_center(col: usize, row: usize) -> Point {
        Point {
            x: col as f32 * TILE_SIZE + TILE_SIZE * 0.5,
            y: row as f32 * TILE_SIZE + TILE_SIZE * 0.5,
        }
    }
}
