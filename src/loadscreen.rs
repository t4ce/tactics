use super::cli::{self, Lobby};
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver, RecvTimeoutError, Sender, TryRecvError},
};
use std::thread;
use std::time::{Duration, Instant};

const WOOD_TABLE_TEXTURE: u32 = 15;
const WOOD_TABLE_BYTES: &[u8] =
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Wood Table/WoodTable.png");
const WOOD_TABLE_TOP_LEFT: ImageRegion = ImageRegion::new(45, 43, 83, 85);
const WOOD_TABLE_TOP_EDGE: ImageRegion = ImageRegion::new(192, 49, 64, 24);
const WOOD_TABLE_TOP_RIGHT: ImageRegion = ImageRegion::new(320, 43, 83, 85);
const WOOD_TABLE_LEFT_EDGE: ImageRegion = ImageRegion::new(49, 192, 18, 64);
const WOOD_TABLE_FILL: ImageRegion = ImageRegion::new(192, 196, 64, 56);
const WOOD_TABLE_RIGHT_EDGE: ImageRegion = ImageRegion::new(383, 192, 16, 64);
const WOOD_TABLE_BOTTOM_LEFT: ImageRegion = ImageRegion::new(44, 360, 84, 63);
const WOOD_TABLE_BOTTOM_EDGE: ImageRegion = ImageRegion::new(192, 384, 64, 39);
const WOOD_TABLE_BOTTOM_RIGHT: ImageRegion = ImageRegion::new(320, 360, 84, 63);
const WOOD_TABLE_TOP_LEFT_OUTSET_X: f32 = 4.0;
const WOOD_TABLE_TOP_RIGHT_OUTSET_X: f32 = 4.0;
const WOOD_TABLE_BOTTOM_LEFT_OUTSET_X: f32 = 5.0;
const WOOD_TABLE_BOTTOM_RIGHT_OUTSET_X: f32 = 5.0;
const WOOD_TABLE_TOP_CORNER_OUTSET_Y: f32 = 6.0;
const LOADSCREEN_ARROW_TEXTURE: u32 = 15_000;
const LOADSCREEN_ICON_TEXTURE_BASE: u32 = 15_010;
const LOADSCREEN_CLOSE_TEXTURE: u32 = 15_020;
const LOADSCREEN_BUTTON_HOVER_TEXTURE: u32 = 15_021;
const LOADSCREEN_CREATE_GAME_TEXTURE: u32 = 15_022;
const LOADSCREEN_FRAME_TEXTURE_BASE: u32 = 15_030;
const LOADSCREEN_ARROW_BYTES: &[u8] =
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Icons/Icon_08.png");
const LOADSCREEN_CLOSE_BYTES: &[u8] =
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Icons/Icon_09.png");
const LOADSCREEN_CREATE_GAME_BYTES: &[u8] =
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Icons/Icon_07.png");
const LOADSCREEN_BUTTON_HOVER_BYTES: &[u8] = include_bytes!(
    "../ts_freepack/UI Elements/UI Elements/Buttons/SmallBlueSquareButton_Hover.png"
);
const LOADSCREEN_FRAME_TOP_LEFT_BYTES: &[u8] = include_bytes!(
    "../ts_freepack/UI Elements/UI Elements/Wood Table/WoodTable_Slots_topleft_corner.png"
);
const LOADSCREEN_FRAME_TOP_RIGHT_BYTES: &[u8] = include_bytes!(
    "../ts_freepack/UI Elements/UI Elements/Wood Table/WoodTable_Slots_topright_corner.png"
);
const LOADSCREEN_FRAME_BOTTOM_LEFT_BYTES: &[u8] = include_bytes!(
    "../ts_freepack/UI Elements/UI Elements/Wood Table/WoodTable_Slots_bottomleft_corner.png"
);
const LOADSCREEN_FRAME_BOTTOM_RIGHT_BYTES: &[u8] = include_bytes!(
    "../ts_freepack/UI Elements/UI Elements/Wood Table/WoodTable_Slots_bottomright_corner.png"
);
const LOADSCREEN_FRAME_HORIZONTAL_BYTES: &[u8] = include_bytes!(
    "../ts_freepack/UI Elements/UI Elements/Wood Table/WoodTable_Slots_middle_up_down.png"
);
const LOADSCREEN_FRAME_VERTICAL_BYTES: &[u8] = include_bytes!(
    "../ts_freepack/UI Elements/UI Elements/Wood Table/WoodTable_Slots_middle_left_right.png"
);
const LOADSCREEN_TOP_ICON_BYTES: [&[u8]; 3] = [
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Icons/Icon_10.png"),
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Icons/Icon_11.png"),
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Icons/Icon_12.png"),
];
const LOBBY_CARD_W: f32 = 256.0;
const LOBBY_CARD_H: f32 = 128.0;
const LOBBY_CARD_GAP: f32 = 20.0;
const LOBBY_CARD_TILE: f32 = 64.0;
const LOBBY_CARD_PAD_X: f32 = 26.0;
const LOBBY_ARROW_SIZE: f32 = 34.0;
const PAPER_TITLE_Y: f32 = 32.0;
const TOP_BUTTON_COUNT: usize = 6;
const TOP_BUTTON_SIZE: f32 = 42.0;
const TOP_BUTTON_GAP: f32 = 8.0;
const TOP_ICON_SIZE: f32 = 27.0;
const TOP_WORLD_BUTTON_INDEX: usize = 0;
const TOP_WORLD_VIEWER_BUTTON_INDEX: usize = 1;
const TOP_EVENT_BUTTON_INDEX: usize = 2;
const TOP_ICON_BUTTON_INDEX: usize = 3;
const TOP_INFO_BUTTON_INDEX: usize = 4;
const TOP_MUSIC_BUTTON_INDEX: usize = 5;
const CLOSE_BUTTON_SIZE: f32 = 46.0;
const CLOSE_ICON_SIZE: f32 = 34.0;
const CREATE_GAME_ICON_SIZE: f32 = 34.0;
const FRAME_DRAW_TILE: f32 = 16.0;
const FRAME_CORNER_SIZE: f32 = FRAME_DRAW_TILE * 2.0;
const LOADSCREEN_BACKGROUND_ALPHA: u8 = 191;
const LOADSCREEN_BACKGROUND_SCALE: f32 = 2.0 / 3.0;
const LOADSCREEN_BACKGROUND_RESEED_SECS: u64 = 10;
const LOADSCREEN_BACKGROUND_PAN_X: f32 = 14.0;
const LOADSCREEN_BACKGROUND_PAN_Y: f32 = 5.0;
const LOADSCREEN_TABLE_DRAW_SCALE: f32 = 0.90;
const CHAT_SERVER_BASE: &str = "https://trueos.eu:3";
const CHAT_ROOM: &str = "lobby";
const CHAT_USER: &str = "Loadscreen";
const CHAT_POLL_MS: u64 = 1_000;
const CHAT_CONNECT_TIMEOUT_MS: u64 = 2_000;
const CHAT_MESSAGE_LIMIT: usize = 96;
const CHAT_INPUT_LIMIT: usize = 160;
const LOBBY_TEXT: Rgba8 = Rgba8 {
    r: 235,
    g: 226,
    b: 206,
    a: 255,
};
const LOBBY_MUTED_TEXT: Rgba8 = Rgba8 {
    r: 177,
    g: 188,
    b: 196,
    a: 255,
};

pub(super) struct LoadScreen {
    terrain: TextureAtlas,
    wood_table: ImageAsset,
    water_visuals: WaterVisualAssets,
    plant_props: [SpriteAnimation; PLANT_PROP_COUNT],
    special_paper: ImageAsset,
    small_blue_square_button: ImageAsset,
    small_blue_square_button_hover: ImageAsset,
    arrow_icon: ImageAsset,
    close_icon: ImageAsset,
    create_game_icon: ImageAsset,
    frame_top_left: ImageAsset,
    frame_top_right: ImageAsset,
    frame_bottom_left: ImageAsset,
    frame_bottom_right: ImageAsset,
    frame_horizontal: ImageAsset,
    frame_vertical: ImageAsset,
    clouds: Vec<ImageAsset>,
    top_icons: [ImageAsset; TOP_BUTTON_COUNT],
    cursor_default: ImageAsset,
    background_world: TileWorld,
    background_visible: Vec<bool>,
    background_cloud_instances: Vec<CloudInstance>,
    background_scene_index: u64,
    started_at: Instant,
    lobbies: Vec<Lobby>,
    lobby_error: Option<String>,
    lobby_rx: Option<Receiver<Result<Vec<Lobby>, String>>>,
    lobby_request_kind: LobbyRequestKind,
    chat_tx: Sender<ChatCommand>,
    chat_rx: Receiver<ChatClientEvent>,
    chat_messages: Vec<ChatMessage>,
    chat_error: Option<String>,
    chat_input: String,
    chat_focused: bool,
    world_editor_request: Arc<AtomicBool>,
    world_viewer_request: Arc<AtomicBool>,
    unit_walk_viewer_request: Arc<AtomicBool>,
    icon_viewer_request: Arc<AtomicBool>,
    event_editor_request: Arc<AtomicBool>,
    idle_viewer_request: Arc<AtomicBool>,
    exit_request: Arc<AtomicBool>,
    mouse: Point,
    layout_offset: Point,
    uploaded: bool,
    window_width: u32,
    window_height: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LobbyRequestKind {
    Refresh,
    CreateGame,
}

#[derive(Clone, Debug, Deserialize)]
struct ChatMessage {
    id: u64,
    user: String,
    text: String,
}

#[derive(Deserialize)]
struct ChatMessagesResponse {
    messages: Vec<ChatMessage>,
}

#[derive(Serialize)]
struct ChatPost<'a> {
    user: &'a str,
    text: &'a str,
}

enum ChatCommand {
    Send(String),
}

enum ChatClientEvent {
    Messages(Vec<ChatMessage>),
    Error(String),
}

impl LoadScreen {
    pub(super) fn new(
        world_editor_request: Arc<AtomicBool>,
        world_viewer_request: Arc<AtomicBool>,
        unit_walk_viewer_request: Arc<AtomicBool>,
        icon_viewer_request: Arc<AtomicBool>,
        event_editor_request: Arc<AtomicBool>,
        idle_viewer_request: Arc<AtomicBool>,
        exit_request: Arc<AtomicBool>,
    ) -> Self {
        let lobby_rx = spawn_lobby_refresh();
        let (chat_tx, chat_rx) = spawn_chat_client();
        let terrain = TextureAtlas::from_png_bytes(TERRAIN_TEXTURE, TERRAIN_BYTES, TERRAIN_TILE_PX);
        let water_visuals = load_water_visual_assets();
        let plant_props = load_plant_prop_assets();
        let clouds = load_cloud_assets();
        let background_scene_index = 0;
        let (background_world, background_visible, background_cloud_instances) =
            generate_loadscreen_background_scene(background_scene_index, &clouds);

        Self {
            terrain,
            wood_table: ImageAsset::from_png_bytes(WOOD_TABLE_TEXTURE, WOOD_TABLE_BYTES),
            water_visuals,
            plant_props,
            special_paper: ImageAsset::from_png_bytes(
                ts_ui::SPECIAL_PAPER_TEXTURE,
                ts_ui::SPECIAL_PAPER_BYTES,
            ),
            small_blue_square_button: ImageAsset::from_png_bytes(
                ts_ui::SMALL_BLUE_SQUARE_BUTTON_TEXTURE,
                ts_ui::SMALL_BLUE_SQUARE_BUTTON_BYTES,
            ),
            small_blue_square_button_hover: ImageAsset::from_png_bytes(
                LOADSCREEN_BUTTON_HOVER_TEXTURE,
                LOADSCREEN_BUTTON_HOVER_BYTES,
            ),
            arrow_icon: ImageAsset::from_png_bytes_cropped(
                LOADSCREEN_ARROW_TEXTURE,
                LOADSCREEN_ARROW_BYTES,
            ),
            close_icon: ImageAsset::from_png_bytes_cropped(
                LOADSCREEN_CLOSE_TEXTURE,
                LOADSCREEN_CLOSE_BYTES,
            ),
            create_game_icon: ImageAsset::from_png_bytes_cropped(
                LOADSCREEN_CREATE_GAME_TEXTURE,
                LOADSCREEN_CREATE_GAME_BYTES,
            ),
            frame_top_left: ImageAsset::from_png_bytes(
                LOADSCREEN_FRAME_TEXTURE_BASE,
                LOADSCREEN_FRAME_TOP_LEFT_BYTES,
            ),
            frame_top_right: ImageAsset::from_png_bytes(
                LOADSCREEN_FRAME_TEXTURE_BASE + 1,
                LOADSCREEN_FRAME_TOP_RIGHT_BYTES,
            ),
            frame_bottom_left: ImageAsset::from_png_bytes(
                LOADSCREEN_FRAME_TEXTURE_BASE + 2,
                LOADSCREEN_FRAME_BOTTOM_LEFT_BYTES,
            ),
            frame_bottom_right: ImageAsset::from_png_bytes(
                LOADSCREEN_FRAME_TEXTURE_BASE + 3,
                LOADSCREEN_FRAME_BOTTOM_RIGHT_BYTES,
            ),
            frame_horizontal: ImageAsset::from_png_bytes(
                LOADSCREEN_FRAME_TEXTURE_BASE + 4,
                LOADSCREEN_FRAME_HORIZONTAL_BYTES,
            ),
            frame_vertical: ImageAsset::from_png_bytes(
                LOADSCREEN_FRAME_TEXTURE_BASE + 5,
                LOADSCREEN_FRAME_VERTICAL_BYTES,
            ),
            clouds,
            top_icons: [
                ImageAsset::from_png_bytes_cropped(
                    LOADSCREEN_ICON_TEXTURE_BASE,
                    LOADSCREEN_TOP_ICON_BYTES[0],
                ),
                ImageAsset::from_png_bytes_cropped(
                    LOADSCREEN_ICON_TEXTURE_BASE + 1,
                    LOADSCREEN_TOP_ICON_BYTES[0],
                ),
                ImageAsset::from_png_bytes_cropped(
                    LOADSCREEN_ICON_TEXTURE_BASE + 2,
                    LOADSCREEN_TOP_ICON_BYTES[0],
                ),
                ImageAsset::from_png_bytes_cropped(
                    LOADSCREEN_ICON_TEXTURE_BASE + 3,
                    LOADSCREEN_TOP_ICON_BYTES[0],
                ),
                ImageAsset::from_png_bytes_cropped(
                    LOADSCREEN_ICON_TEXTURE_BASE + 4,
                    LOADSCREEN_TOP_ICON_BYTES[1],
                ),
                ImageAsset::from_png_bytes_cropped(
                    LOADSCREEN_ICON_TEXTURE_BASE + 5,
                    LOADSCREEN_TOP_ICON_BYTES[2],
                ),
            ],
            cursor_default: ImageAsset::from_png_bytes_cropped(
                CURSOR_DEFAULT_TEXTURE,
                CURSOR_DEFAULT_BYTES,
            ),
            background_world,
            background_visible,
            background_cloud_instances,
            background_scene_index,
            started_at: Instant::now(),
            lobbies: Vec::new(),
            lobby_error: None,
            lobby_rx: Some(lobby_rx),
            lobby_request_kind: LobbyRequestKind::Refresh,
            chat_tx,
            chat_rx,
            chat_messages: Vec::new(),
            chat_error: None,
            chat_input: String::new(),
            chat_focused: false,
            world_editor_request,
            world_viewer_request,
            unit_walk_viewer_request,
            icon_viewer_request,
            event_editor_request,
            idle_viewer_request,
            exit_request,
            mouse: Point::default(),
            layout_offset: Point::default(),
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
            self.terrain.texture_id,
            self.terrain.width,
            self.terrain.height,
            &self.terrain.rgba,
        );
        assert_eq!(rc, 0, "failed to upload loadscreen terrain texture");
        let rc = adapter.upload_texture_rgba_image(
            self.wood_table.texture_id,
            self.wood_table.width,
            self.wood_table.height,
            &self.wood_table.rgba,
        );
        assert_eq!(rc, 0, "failed to upload loadscreen table texture");
        let rc = adapter.upload_texture_rgba_image(
            self.special_paper.texture_id,
            self.special_paper.width,
            self.special_paper.height,
            &self.special_paper.rgba,
        );
        assert_eq!(rc, 0, "failed to upload special paper texture");
        let rc = adapter.upload_texture_rgba_image(
            self.small_blue_square_button.texture_id,
            self.small_blue_square_button.width,
            self.small_blue_square_button.height,
            &self.small_blue_square_button.rgba,
        );
        assert_eq!(rc, 0, "failed to upload loadscreen button texture");
        let rc = adapter.upload_texture_rgba_image(
            self.small_blue_square_button_hover.texture_id,
            self.small_blue_square_button_hover.width,
            self.small_blue_square_button_hover.height,
            &self.small_blue_square_button_hover.rgba,
        );
        assert_eq!(rc, 0, "failed to upload loadscreen hover button texture");
        let rc = adapter.upload_texture_rgba_image(
            self.arrow_icon.texture_id,
            self.arrow_icon.width,
            self.arrow_icon.height,
            &self.arrow_icon.rgba,
        );
        assert_eq!(rc, 0, "failed to upload loadscreen arrow texture");
        let rc = adapter.upload_texture_rgba_image(
            self.close_icon.texture_id,
            self.close_icon.width,
            self.close_icon.height,
            &self.close_icon.rgba,
        );
        assert_eq!(rc, 0, "failed to upload loadscreen close texture");
        let rc = adapter.upload_texture_rgba_image(
            self.create_game_icon.texture_id,
            self.create_game_icon.width,
            self.create_game_icon.height,
            &self.create_game_icon.rgba,
        );
        assert_eq!(rc, 0, "failed to upload loadscreen create game texture");
        for image in self
            .water_visuals
            .stones
            .iter()
            .flat_map(|animation| animation.frames.iter())
            .chain(self.water_visuals.animation.frames.iter())
            .chain(self.water_visuals.duck.frames.iter())
            .chain(
                self.plant_props
                    .iter()
                    .flat_map(|animation| animation.frames.iter()),
            )
        {
            let rc = adapter.upload_texture_rgba_image(
                image.texture_id,
                image.width,
                image.height,
                &image.rgba,
            );
            assert_eq!(
                rc, 0,
                "failed to upload loadscreen background asset texture {}",
                image.texture_id
            );
        }
        for frame in [
            &self.frame_top_left,
            &self.frame_top_right,
            &self.frame_bottom_left,
            &self.frame_bottom_right,
            &self.frame_horizontal,
            &self.frame_vertical,
        ] {
            let rc = adapter.upload_texture_rgba_image(
                frame.texture_id,
                frame.width,
                frame.height,
                &frame.rgba,
            );
            assert_eq!(rc, 0, "failed to upload loadscreen frame texture");
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
                "failed to upload loadscreen cloud texture {}",
                image.texture_id
            );
        }
        for icon in &self.top_icons {
            let rc = adapter.upload_texture_rgba_image(
                icon.texture_id,
                icon.width,
                icon.height,
                &icon.rgba,
            );
            assert_eq!(rc, 0, "failed to upload loadscreen icon texture");
        }
        let rc = adapter.upload_texture_rgba_image(
            self.cursor_default.texture_id,
            self.cursor_default.width,
            self.cursor_default.height,
            &self.cursor_default.rgba,
        );
        assert_eq!(rc, 0, "failed to upload loadscreen cursor texture");
        self.uploaded = true;
    }

    fn draw_cursor(&self, adapter: &mut Adapter) {
        let mut cursor = SpriteBatch::new(self.window_width, self.window_height);
        cursor.image(
            self.mouse.x,
            self.mouse.y,
            self.cursor_default.width as f32,
            self.cursor_default.height as f32,
            Rgba8::WHITE,
        );
        let _ =
            adapter.draw_tex_triangles_no_present(self.cursor_default.texture_id, &cursor.bytes);
    }

    fn poll_lobbies(&mut self) {
        let Some(rx) = &self.lobby_rx else {
            return;
        };

        match rx.try_recv() {
            Ok(Ok(lobbies)) => {
                self.lobbies = lobbies;
                self.lobby_error = None;
                self.lobby_rx = None;
            }
            Ok(Err(error)) => {
                self.lobbies.clear();
                self.lobby_error = Some(error);
                self.lobby_rx = None;
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.lobby_error = Some("lobby request stopped".to_owned());
                self.lobby_rx = None;
            }
        }
    }

    fn poll_chat(&mut self) {
        loop {
            match self.chat_rx.try_recv() {
                Ok(ChatClientEvent::Messages(messages)) => {
                    self.chat_error = None;
                    for message in messages {
                        if self
                            .chat_messages
                            .iter()
                            .any(|existing| existing.id == message.id)
                        {
                            continue;
                        }
                        self.chat_messages.push(message);
                    }
                    self.chat_messages.sort_by_key(|message| message.id);
                    let overflow = self.chat_messages.len().saturating_sub(CHAT_MESSAGE_LIMIT);
                    if overflow > 0 {
                        self.chat_messages.drain(0..overflow);
                    }
                }
                Ok(ChatClientEvent::Error(error)) => {
                    self.chat_error = Some(error);
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.chat_error = Some("chat stopped".to_owned());
                    break;
                }
            }
        }
    }

    fn start_create_game(&mut self) {
        if self.lobby_rx.is_some() {
            return;
        }

        self.lobby_error = None;
        self.lobby_rx = Some(spawn_lobby_create_game());
        self.lobby_request_kind = LobbyRequestKind::CreateGame;
    }

    fn send_chat_input(&mut self) {
        let text = self.chat_input.trim();
        if text.is_empty() {
            return;
        }

        let text = text.chars().take(CHAT_INPUT_LIMIT).collect::<String>();
        if self.chat_tx.send(ChatCommand::Send(text)).is_ok() {
            self.chat_input.clear();
            self.chat_error = None;
        } else {
            self.chat_error = Some("chat stopped".to_owned());
        }
    }

    fn push_chat_text(&mut self, text: &str) {
        for ch in text.chars() {
            if self.chat_input.chars().count() >= CHAT_INPUT_LIMIT {
                break;
            }
            if is_chat_input_char(ch) {
                self.chat_input.push(ch);
            }
        }
    }

    fn update_background_scene(&mut self) {
        let scene_index = self.started_at.elapsed().as_secs() / LOADSCREEN_BACKGROUND_RESEED_SECS;
        if scene_index == self.background_scene_index {
            return;
        }

        let (world, visible, clouds) =
            generate_loadscreen_background_scene(scene_index, &self.clouds);
        self.background_world = world;
        self.background_visible = visible;
        self.background_cloud_instances = clouds;
        self.background_scene_index = scene_index;
    }

    fn background_camera(&self) -> Point {
        let elapsed = self.started_at.elapsed().as_secs_f32();
        let view_w = self.window_width as f32 / LOADSCREEN_BACKGROUND_SCALE;
        let view_h = self.window_height as f32 / LOADSCREEN_BACKGROUND_SCALE;
        let max_x = (self.background_world.width_px() - view_w).max(0.0);
        let max_y = (self.background_world.height_px() - view_h).max(0.0);
        let seed_offset = self.background_scene_index as f32 * 173.0;
        Point {
            x: if max_x > 0.0 {
                (elapsed * LOADSCREEN_BACKGROUND_PAN_X + seed_offset).rem_euclid(max_x)
            } else {
                0.0
            },
            y: if max_y > 0.0 {
                (elapsed * LOADSCREEN_BACKGROUND_PAN_Y + seed_offset * 0.43).rem_euclid(max_y)
            } else {
                0.0
            },
        }
    }

    fn draw_background_scene(&self, adapter: &mut Adapter) {
        let camera = self.background_camera();
        let tile_size = TILE_SIZE * LOADSCREEN_BACKGROUND_SCALE;
        let view_w = self.window_width as f32 / LOADSCREEN_BACKGROUND_SCALE;
        let view_h = self.window_height as f32 / LOADSCREEN_BACKGROUND_SCALE;
        let start_col = (camera.x / TILE_SIZE).floor().max(0.0) as usize;
        let start_row = (camera.y / TILE_SIZE).floor().max(0.0) as usize;
        let end_col = ((camera.x + view_w) / TILE_SIZE).ceil() as usize + 1;
        let end_row = ((camera.y + view_h) / TILE_SIZE).ceil() as usize + 1;
        let tint = Rgba8::new(255, 255, 255, LOADSCREEN_BACKGROUND_ALPHA);
        let mut water = SolidBatch::new(self.window_width, self.window_height);
        let mut backgrounds = SpriteBatch::new(self.window_width, self.window_height);
        let mut under_foregrounds = SpriteBatch::new(self.window_width, self.window_height);
        let mut foregrounds = SpriteBatch::new(self.window_width, self.window_height);

        for row in start_row..end_row.min(self.background_world.rows) {
            for col in start_col..end_col.min(self.background_world.cols) {
                let index = self.background_world.index(col, row);
                if !self.background_visible.get(index).copied().unwrap_or(true) {
                    continue;
                }

                let x = (col as f32 * TILE_SIZE - camera.x) * LOADSCREEN_BACKGROUND_SCALE;
                let y = (row as f32 * TILE_SIZE - camera.y) * LOADSCREEN_BACKGROUND_SCALE;
                match self.background_world.render_background(col, row) {
                    BackgroundTile::Water => {
                        water.rect(
                            x,
                            y,
                            tile_size,
                            tile_size,
                            Rgba8::new(71, 171, 169, LOADSCREEN_BACKGROUND_ALPHA),
                        );
                    }
                    BackgroundTile::Grass => {
                        backgrounds.sprite(
                            &self.terrain,
                            GRASS_BG_TILE,
                            x,
                            y,
                            tile_size,
                            tile_size,
                            tint,
                        );
                    }
                }

                if let Some(tile) = self.background_world.under_foreground(col, row) {
                    under_foregrounds.sprite(&self.terrain, tile, x, y, tile_size, tile_size, tint);
                }
                if let Some(tile) = self.background_world.foreground(col, row) {
                    foregrounds.sprite(&self.terrain, tile, x, y, tile_size, tile_size, tint);
                }
            }
        }

        let _ = adapter.set_texture_effect(TextureEffect::World);
        let _ = adapter.draw_rgb_triangles_no_present(&water.bytes);
        let _ = adapter.draw_tex_triangles_no_present(self.terrain.texture_id, &backgrounds.bytes);
        let _ = adapter
            .draw_tex_triangles_no_present(self.terrain.texture_id, &under_foregrounds.bytes);
        let _ = adapter.draw_tex_triangles_no_present(self.terrain.texture_id, &foregrounds.bytes);
        let _ = adapter.set_texture_effect(TextureEffect::Plain);

        self.draw_background_world_assets(adapter, camera, start_col, start_row, end_col, end_row);
        self.draw_background_clouds(adapter, camera);
    }

    fn draw_background_world_assets(
        &self,
        adapter: &mut Adapter,
        camera: Point,
        start_col: usize,
        start_row: usize,
        end_col: usize,
        end_row: usize,
    ) {
        let mut batches = BTreeMap::new();

        for row in start_row..end_row.min(self.background_world.rows) {
            for col in start_col..end_col.min(self.background_world.cols) {
                let index = self.background_world.index(col, row);
                if !self.background_visible.get(index).copied().unwrap_or(true) {
                    continue;
                }
                let state = self.background_world.water_states[index];
                if state == WaterState::Nothing || state == WaterState::Animation {
                    continue;
                }
                let Some(image) = self.loadscreen_water_visual_frame(state) else {
                    continue;
                };
                self.push_background_centered_tile_image(
                    &mut batches,
                    image,
                    col,
                    row,
                    camera,
                    Rgba8::new(255, 255, 255, LOADSCREEN_BACKGROUND_ALPHA),
                );
            }
        }

        for prop in &self.background_world.props {
            let col = prop.x2 / BUILDING_GRID_DIVISIONS;
            let row = prop.y2 / BUILDING_GRID_DIVISIONS;
            if col >= self.background_world.cols || row >= self.background_world.rows {
                continue;
            }
            let index = self.background_world.index(col, row);
            if !self.background_visible.get(index).copied().unwrap_or(true) {
                continue;
            }

            let PropKind::Plant(kind) = prop.kind else {
                continue;
            };
            let Some(image) = self.plant_props[kind.index()].first_frame() else {
                continue;
            };
            let instance_count = kind.visual_instance_count(prop.x2, prop.y2);
            for instance in 0..instance_count {
                let offset = kind.visual_instance_offset(instance_count, instance);
                self.push_background_bottom_aligned_image_half(
                    &mut batches,
                    image,
                    prop.x2,
                    prop.y2,
                    camera,
                    offset.x,
                    kind.render_offset_y() + offset.y,
                    kind.render_scale(),
                    Rgba8::new(255, 255, 255, LOADSCREEN_BACKGROUND_ALPHA),
                );
            }
        }

        for (texture_id, batch) in batches {
            let _ = adapter.draw_tex_triangles_no_present(texture_id, &batch.bytes);
        }
    }

    fn loadscreen_water_visual_frame(&self, state: WaterState) -> Option<&ImageAsset> {
        match state {
            WaterState::Nothing | WaterState::Animation => None,
            WaterState::Stone1 => self.water_visuals.stones[0].first_frame(),
            WaterState::Stone2 => self.water_visuals.stones[1].first_frame(),
            WaterState::Stone3 => self.water_visuals.stones[2].first_frame(),
            WaterState::Stone4 => self.water_visuals.stones[3].first_frame(),
            WaterState::Duck => self.water_visuals.duck.first_frame(),
        }
    }

    fn push_background_centered_tile_image(
        &self,
        batches: &mut BTreeMap<u32, SpriteBatch>,
        image: &ImageAsset,
        col: usize,
        row: usize,
        camera: Point,
        tint: Rgba8,
    ) {
        let w = image.width as f32 * BUILDING_SCALE * LOADSCREEN_BACKGROUND_SCALE;
        let h = image.height as f32 * BUILDING_SCALE * LOADSCREEN_BACKGROUND_SCALE;
        let tile_size = TILE_SIZE * LOADSCREEN_BACKGROUND_SCALE;
        let x = (col as f32 * TILE_SIZE - camera.x) * LOADSCREEN_BACKGROUND_SCALE
            + (tile_size - w) * 0.5;
        let y = (row as f32 * TILE_SIZE - camera.y) * LOADSCREEN_BACKGROUND_SCALE
            + (tile_size - h) * 0.5;
        self.push_background_image_batch(batches, image.texture_id, x, y, w, h, tint);
    }

    fn push_background_bottom_aligned_image_half(
        &self,
        batches: &mut BTreeMap<u32, SpriteBatch>,
        image: &ImageAsset,
        x2: usize,
        y2: usize,
        camera: Point,
        offset_x: f32,
        offset_y: f32,
        scale: f32,
        tint: Rgba8,
    ) {
        let w = image.width as f32 * BUILDING_SCALE * scale * LOADSCREEN_BACKGROUND_SCALE;
        let h = image.height as f32 * BUILDING_SCALE * scale * LOADSCREEN_BACKGROUND_SCALE;
        let tile_size = TILE_SIZE * LOADSCREEN_BACKGROUND_SCALE;
        let x = (half_grid_to_px(x2) - camera.x) * LOADSCREEN_BACKGROUND_SCALE
            + (tile_size - w) * 0.5
            + offset_x * LOADSCREEN_BACKGROUND_SCALE;
        let y = (half_grid_to_px(y2 + BUILDING_GRID_DIVISIONS) - camera.y)
            * LOADSCREEN_BACKGROUND_SCALE
            - h
            + offset_y * LOADSCREEN_BACKGROUND_SCALE;
        self.push_background_image_batch(batches, image.texture_id, x, y, w, h, tint);
    }

    fn push_background_image_batch(
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
            .image(
                x.floor(),
                y.floor(),
                w.floor().max(1.0),
                h.floor().max(1.0),
                tint,
            );
    }

    fn draw_background_clouds(&self, adapter: &mut Adapter, camera: Point) {
        if self.clouds.is_empty() {
            return;
        }

        let elapsed = self.started_at.elapsed().as_secs_f32();
        let world_w = self.background_world.width_px();
        let world_h = self.background_world.height_px();
        let mut batches = BTreeMap::new();

        for cloud in &self.background_cloud_instances {
            let image = &self.clouds[cloud.asset_index % self.clouds.len()];
            let fade = ((elapsed * 0.12 + cloud.phase).sin() + 1.0) * 0.5;
            let alpha =
                (cloud.alpha_min + (cloud.alpha_max - cloud.alpha_min) * fade).clamp(0.0, 1.0);
            let scale =
                cloud.scale * (1.0 + cloud.scale_wobble * (elapsed * 0.18 + cloud.phase).sin());
            let wrap_w = world_w + image.width as f32 * scale;
            let wrap_h = world_h + image.height as f32 * scale;
            let world_x = (cloud.x + cloud.drift_x * elapsed).rem_euclid(wrap_w)
                - image.width as f32 * scale * 0.5;
            let world_y = (cloud.y + cloud.drift_y * elapsed).rem_euclid(wrap_h)
                - image.height as f32 * scale * 0.5;
            let x = (world_x - camera.x) * LOADSCREEN_BACKGROUND_SCALE;
            let y = (world_y - camera.y) * LOADSCREEN_BACKGROUND_SCALE;
            let w = image.width as f32 * scale * LOADSCREEN_BACKGROUND_SCALE;
            let h = image.height as f32 * scale * LOADSCREEN_BACKGROUND_SCALE;

            if x + w < 0.0
                || y + h < 0.0
                || x > self.window_width as f32
                || y > self.window_height as f32
            {
                continue;
            }

            batches
                .entry(image.texture_id)
                .or_insert_with(|| SpriteBatch::new(self.window_width, self.window_height))
                .image(
                    x.floor(),
                    y.floor(),
                    w.floor().max(1.0),
                    h.floor().max(1.0),
                    Rgba8::new(
                        255,
                        255,
                        255,
                        (alpha * LOADSCREEN_BACKGROUND_ALPHA as f32).round() as u8,
                    ),
                );
        }

        for (texture_id, batch) in batches {
            let _ = adapter.draw_tex_triangles_no_present(texture_id, &batch.bytes);
        }
    }

    fn draw_lobby_cards(&self, adapter: &mut Adapter, table: TableRect) {
        let start_x = table.x.round() + 72.0;
        let start_y = table.y.round() + 70.0;
        let available_w = (table.w - 144.0).max(LOBBY_CARD_W);
        let available_h = (table.h - 128.0).max(LOBBY_CARD_H);
        let cols = ((available_w + LOBBY_CARD_GAP) / (LOBBY_CARD_W + LOBBY_CARD_GAP))
            .floor()
            .max(1.0) as usize;
        let rows = ((available_h + LOBBY_CARD_GAP) / (LOBBY_CARD_H + LOBBY_CARD_GAP))
            .floor()
            .max(1.0) as usize;
        let card_w = ((available_w - LOBBY_CARD_GAP * (cols.saturating_sub(1) as f32))
            / cols as f32)
            .clamp(192.0, 320.0);
        let max_cards = cols.saturating_mul(rows).max(1);

        let mut papers = ts_ui::UiBatch::new(self.window_width, self.window_height);
        let mut labels = ts_ui::UiBatch::new(self.window_width, self.window_height);
        let mut arrows = SpriteBatch::new(self.window_width, self.window_height);
        let status = self.status_card_text();

        if self.lobbies.is_empty() {
            draw_lobby_card_panel(&mut papers, start_x, start_y, card_w, LOBBY_CARD_H);
            draw_status_card_text(
                &mut labels,
                status.0,
                status.1,
                status.2,
                start_x,
                start_y,
                card_w,
            );
        } else {
            for (index, lobby) in self.lobbies.iter().take(max_cards).enumerate() {
                let col = index % cols;
                let row = index / cols;
                let x = start_x + col as f32 * (card_w + LOBBY_CARD_GAP);
                let y = start_y + row as f32 * (LOBBY_CARD_H + LOBBY_CARD_GAP);
                draw_lobby_card_panel(&mut papers, x, y, card_w, LOBBY_CARD_H);
                draw_lobby_card_text(&mut labels, lobby, x, y, card_w);
                arrows.image(
                    x + card_w - LOBBY_CARD_PAD_X - LOBBY_ARROW_SIZE,
                    y + (LOBBY_CARD_H - LOBBY_ARROW_SIZE) * 0.5,
                    LOBBY_ARROW_SIZE,
                    LOBBY_ARROW_SIZE,
                    Rgba8::WHITE,
                );
            }
        }

        let _ = adapter
            .draw_tex_triangles_no_present(self.special_paper.texture_id, &papers.texture_bytes);
        let _ = adapter.draw_rgb_triangles_no_present(&labels.solid_bytes);
        let _ = adapter.draw_tex_triangles_no_present(self.arrow_icon.texture_id, &arrows.bytes);
    }

    fn draw_create_game_button(&self, adapter: &mut Adapter, table: TableRect) {
        let rect = create_game_button_rect(table);
        let hovered = inside_rect(self.mouse.x, self.mouse.y, rect.x, rect.y, rect.w, rect.h);
        let tint = if self.lobby_rx.is_some() {
            Rgba8 {
                r: 165,
                g: 180,
                b: 190,
                a: 255,
            }
        } else {
            Rgba8::WHITE
        };
        let button_asset = if hovered && self.lobby_rx.is_none() {
            &self.small_blue_square_button_hover
        } else {
            &self.small_blue_square_button
        };
        let mut button = SpriteBatch::new(self.window_width, self.window_height);
        button.image(rect.x, rect.y, rect.w, rect.h, tint);
        let _ = adapter.draw_tex_triangles_no_present(button_asset.texture_id, &button.bytes);

        let icon_size = CREATE_GAME_ICON_SIZE;
        let mut icon = SpriteBatch::new(self.window_width, self.window_height);
        icon.image(
            rect.x + (rect.w - icon_size) * 0.5,
            rect.y + (rect.h - icon_size) * 0.5,
            icon_size,
            icon_size,
            tint,
        );
        let _ =
            adapter.draw_tex_triangles_no_present(self.create_game_icon.texture_id, &icon.bytes);

        if hovered {
            let mut label = ts_ui::UiBatch::new(self.window_width, self.window_height);
            draw_centered_text(
                &mut label,
                if self.lobby_rx.is_some() {
                    "PLEASE WAIT"
                } else {
                    "CREATE GAME"
                },
                table.x,
                rect.y + rect.h + 12.0,
                table.w,
                1.0,
                LOBBY_TEXT,
            );
            let _ = adapter.draw_rgb_triangles_no_present(&label.solid_bytes);
        }
    }

    fn draw_top_table_buttons(&self, adapter: &mut Adapter, table: TableRect) {
        let (start_x, button_y) = top_table_button_origin(table);
        let hovered = top_table_button_at(self.mouse, table);
        let mut buttons = SpriteBatch::new(self.window_width, self.window_height);
        let mut hover_buttons = SpriteBatch::new(self.window_width, self.window_height);

        for index in 0..TOP_BUTTON_COUNT {
            let batch = if Some(index) == hovered {
                &mut hover_buttons
            } else {
                &mut buttons
            };
            batch.image(
                start_x + index as f32 * (TOP_BUTTON_SIZE + TOP_BUTTON_GAP),
                button_y,
                TOP_BUTTON_SIZE,
                TOP_BUTTON_SIZE,
                Rgba8::WHITE,
            );
        }

        let _ = adapter.draw_tex_triangles_no_present(
            self.small_blue_square_button.texture_id,
            &buttons.bytes,
        );
        let _ = adapter.draw_tex_triangles_no_present(
            self.small_blue_square_button_hover.texture_id,
            &hover_buttons.bytes,
        );

        for (index, icon) in self.top_icons.iter().enumerate() {
            let icon_x = start_x
                + index as f32 * (TOP_BUTTON_SIZE + TOP_BUTTON_GAP)
                + (TOP_BUTTON_SIZE - TOP_ICON_SIZE) * 0.5;
            let icon_y = button_y + (TOP_BUTTON_SIZE - TOP_ICON_SIZE) * 0.5;
            let mut icon_batch = SpriteBatch::new(self.window_width, self.window_height);
            icon_batch.image(icon_x, icon_y, TOP_ICON_SIZE, TOP_ICON_SIZE, Rgba8::WHITE);
            let _ = adapter.draw_tex_triangles_no_present(icon.texture_id, &icon_batch.bytes);
        }

        let mut status = ts_ui::UiBatch::new(self.window_width, self.window_height);
        if let Some(index) = hovered {
            draw_centered_text(
                &mut status,
                top_table_button_label(index),
                table.x,
                button_y + TOP_BUTTON_SIZE + 16.0,
                table.w,
                1.0,
                LOBBY_TEXT,
            );
        }
        draw_centered_text(
            &mut status,
            self.server_status_text(),
            table.x,
            button_y + TOP_BUTTON_SIZE + 38.0,
            table.w,
            1.0,
            LOBBY_MUTED_TEXT,
        );
        let _ = adapter.draw_rgb_triangles_no_present(&status.solid_bytes);
    }

    fn draw_chat(&self, adapter: &mut Adapter, table: TableRect, input: TableRect) {
        let panel = chat_panel_rect(table);
        let mut paper = ts_ui::UiBatch::new(self.window_width, self.window_height);
        paper.paper_panel(
            panel.x,
            panel.y,
            panel.w,
            panel.h,
            LOBBY_CARD_TILE,
            Rgba8::WHITE,
        );
        let _ = adapter
            .draw_tex_triangles_no_present(self.special_paper.texture_id, &paper.texture_bytes);

        let mut solid = SolidBatch::new(self.window_width, self.window_height);
        solid.rect(
            input.x,
            input.y,
            input.w,
            input.h,
            Rgba8::new(52, 64, 78, 236),
        );
        outline_rect(
            &mut solid,
            input.x,
            input.y,
            input.w,
            input.h,
            2.0,
            if self.chat_focused {
                Rgba8::new(235, 226, 206, 255)
            } else {
                Rgba8::new(121, 138, 150, 255)
            },
        );
        let _ = adapter.draw_rgb_triangles_no_present(&solid.bytes);

        let mut labels = ts_ui::UiBatch::new(self.window_width, self.window_height);
        draw_centered_text(
            &mut labels,
            "CHAT",
            panel.x,
            panel.y + PAPER_TITLE_Y,
            panel.w,
            2.0,
            LOBBY_TEXT,
        );

        let message_x = panel.x + 18.0;
        let message_y = panel.y + 62.0;
        let message_w = panel.w - 36.0;
        let line_h = 15.0;
        let max_lines = ((panel.y + panel.h - message_y - 32.0) / line_h)
            .floor()
            .max(1.0) as usize;
        if self.chat_messages.is_empty() {
            let status = if self.chat_error.is_some() {
                "CHAT OFFLINE"
            } else {
                "NO MESSAGES"
            };
            draw_centered_text(
                &mut labels,
                status,
                panel.x,
                message_y + line_h,
                panel.w,
                1.0,
                LOBBY_MUTED_TEXT,
            );
        } else {
            let first = self.chat_messages.len().saturating_sub(max_lines);
            for (index, message) in self.chat_messages[first..].iter().enumerate() {
                let text = format!("{}: {}", message.user, message.text);
                labels.text(
                    &fit_chat_text(&text, message_w, 1.0),
                    message_x,
                    message_y + index as f32 * line_h,
                    1.0,
                    LOBBY_TEXT,
                );
            }
        }

        if let Some(error) = &self.chat_error {
            labels.text(
                &fit_chat_text(error, panel.w - 30.0, 1.0),
                panel.x + 15.0,
                panel.y + panel.h - 22.0,
                1.0,
                Rgba8::new(222, 145, 128, 255),
            );
        }

        let input_text = if self.chat_input.is_empty() && !self.chat_focused {
            "MESSAGE".to_owned()
        } else {
            self.chat_input.clone()
        };
        labels.text(
            &fit_chat_input_text(&input_text, input.w - 24.0, 1.0),
            input.x + 12.0,
            input.y + 11.0,
            1.0,
            if self.chat_input.is_empty() && !self.chat_focused {
                LOBBY_MUTED_TEXT
            } else {
                LOBBY_TEXT
            },
        );

        if self.chat_focused {
            let cursor_x = input.x
                + 12.0
                + ts_ui::text_width(&fit_chat_input_text(&input_text, input.w - 24.0, 1.0), 1.0)
                + 3.0;
            labels.text(
                "-",
                cursor_x.min(input.x + input.w - 14.0),
                input.y + 11.0,
                1.0,
                LOBBY_TEXT,
            );
        }

        let _ = adapter.draw_rgb_triangles_no_present(&labels.solid_bytes);
    }

    fn draw_frame(&self, adapter: &mut Adapter) {
        let window_w = self.window_width as f32;
        let window_h = self.window_height as f32;
        draw_tiled_asset(
            adapter,
            &self.frame_horizontal,
            self.window_width,
            self.window_height,
            FRAME_CORNER_SIZE,
            0.0,
            (window_w - FRAME_CORNER_SIZE * 2.0).max(0.0),
            FRAME_CORNER_SIZE,
            FRAME_DRAW_TILE,
            FRAME_CORNER_SIZE,
        );
        draw_tiled_asset(
            adapter,
            &self.frame_horizontal,
            self.window_width,
            self.window_height,
            FRAME_CORNER_SIZE,
            (window_h - FRAME_CORNER_SIZE).max(0.0),
            (window_w - FRAME_CORNER_SIZE * 2.0).max(0.0),
            FRAME_CORNER_SIZE,
            FRAME_DRAW_TILE,
            FRAME_CORNER_SIZE,
        );
        draw_tiled_asset(
            adapter,
            &self.frame_vertical,
            self.window_width,
            self.window_height,
            0.0,
            FRAME_CORNER_SIZE,
            FRAME_CORNER_SIZE,
            (window_h - FRAME_CORNER_SIZE * 2.0).max(0.0),
            FRAME_CORNER_SIZE,
            FRAME_DRAW_TILE,
        );
        draw_tiled_asset(
            adapter,
            &self.frame_vertical,
            self.window_width,
            self.window_height,
            (window_w - FRAME_CORNER_SIZE).max(0.0),
            FRAME_CORNER_SIZE,
            FRAME_CORNER_SIZE,
            (window_h - FRAME_CORNER_SIZE * 2.0).max(0.0),
            FRAME_CORNER_SIZE,
            FRAME_DRAW_TILE,
        );
        draw_asset_rect(
            adapter,
            &self.frame_top_left,
            self.window_width,
            self.window_height,
            0.0,
            0.0,
            FRAME_CORNER_SIZE,
            FRAME_CORNER_SIZE,
        );
        draw_asset_rect(
            adapter,
            &self.frame_top_right,
            self.window_width,
            self.window_height,
            (window_w - FRAME_CORNER_SIZE).max(0.0),
            0.0,
            FRAME_CORNER_SIZE,
            FRAME_CORNER_SIZE,
        );
        draw_asset_rect(
            adapter,
            &self.frame_bottom_left,
            self.window_width,
            self.window_height,
            0.0,
            (window_h - FRAME_CORNER_SIZE).max(0.0),
            FRAME_CORNER_SIZE,
            FRAME_CORNER_SIZE,
        );
        draw_asset_rect(
            adapter,
            &self.frame_bottom_right,
            self.window_width,
            self.window_height,
            (window_w - FRAME_CORNER_SIZE).max(0.0),
            (window_h - FRAME_CORNER_SIZE).max(0.0),
            FRAME_CORNER_SIZE,
            FRAME_CORNER_SIZE,
        );
    }

    fn window_drag_region_at(&self, point: Point) -> bool {
        let table_layout = offset_table_layout(
            loadscreen_table_layout(self.window_width as f32, self.window_height as f32),
            self.layout_offset,
        );
        let close = close_button_rect(self.window_width as f32);
        let create_game = create_game_button_rect(table_layout[0]);
        let chat_input = chat_input_rect(table_layout[1], table_layout[2]);
        if inside_rect(point.x, point.y, close.x, close.y, close.w, close.h)
            || inside_rect(
                point.x,
                point.y,
                create_game.x,
                create_game.y,
                create_game.w,
                create_game.h,
            )
            || inside_rect(
                point.x,
                point.y,
                chat_input.x,
                chat_input.y,
                chat_input.w,
                chat_input.h,
            )
            || top_table_button_at(point, table_layout[1]).is_some()
        {
            return false;
        }

        inside_rect(
            point.x,
            point.y,
            0.0,
            0.0,
            self.window_width as f32,
            self.window_height as f32,
        )
    }

    fn draw_close_button(&self, adapter: &mut Adapter) {
        let rect = close_button_rect(self.window_width as f32);
        let button_asset =
            if inside_rect(self.mouse.x, self.mouse.y, rect.x, rect.y, rect.w, rect.h) {
                &self.small_blue_square_button_hover
            } else {
                &self.small_blue_square_button
            };
        let mut button = SpriteBatch::new(self.window_width, self.window_height);
        button.image(rect.x, rect.y, rect.w, rect.h, Rgba8::WHITE);
        let _ = adapter.draw_tex_triangles_no_present(button_asset.texture_id, &button.bytes);

        let mut close = SpriteBatch::new(self.window_width, self.window_height);
        close.image(
            rect.x + (rect.w - CLOSE_ICON_SIZE) * 0.5,
            rect.y + (rect.h - CLOSE_ICON_SIZE) * 0.5,
            CLOSE_ICON_SIZE,
            CLOSE_ICON_SIZE,
            Rgba8::WHITE,
        );
        let _ = adapter.draw_tex_triangles_no_present(self.close_icon.texture_id, &close.bytes);
    }

    fn status_card_text(&self) -> (&str, &str, &str) {
        if self.lobby_rx.is_some() {
            match self.lobby_request_kind {
                LobbyRequestKind::Refresh => ("GAME LIST", "SERVER CONNECTING", "LOADING"),
                LobbyRequestKind::CreateGame => ("GAME LIST", "SERVER CONNECTED", "CREATING GAME"),
            }
        } else if self.lobby_error.is_some() {
            ("GAME LIST", "SERVER OFFLINE", "TRY AGAIN")
        } else {
            ("GAME LIST", "SERVER CONNECTED", "NO GAMES")
        }
    }

    fn server_status_text(&self) -> &str {
        if self.lobby_rx.is_some() {
            match self.lobby_request_kind {
                LobbyRequestKind::Refresh => "SERVER CONNECTING",
                LobbyRequestKind::CreateGame => "CREATING GAME",
            }
        } else if self.lobby_error.is_some() {
            "SERVER OFFLINE"
        } else {
            "SERVER CONNECTED"
        }
    }

    fn handle_left_click(&mut self) {
        let table_layout = offset_table_layout(
            loadscreen_table_layout(self.window_width as f32, self.window_height as f32),
            self.layout_offset,
        );
        let close = close_button_rect(self.window_width as f32);
        if inside_rect(
            self.mouse.x,
            self.mouse.y,
            close.x,
            close.y,
            close.w,
            close.h,
        ) {
            self.chat_focused = false;
            self.exit_request.store(true, Ordering::Relaxed);
            return;
        }

        let create_game = create_game_button_rect(table_layout[0]);
        if inside_rect(
            self.mouse.x,
            self.mouse.y,
            create_game.x,
            create_game.y,
            create_game.w,
            create_game.h,
        ) {
            self.chat_focused = false;
            self.start_create_game();
            return;
        }

        let chat_input = chat_input_rect(table_layout[1], table_layout[2]);
        if inside_rect(
            self.mouse.x,
            self.mouse.y,
            chat_input.x,
            chat_input.y,
            chat_input.w,
            chat_input.h,
        ) {
            self.chat_focused = true;
            return;
        }

        self.chat_focused = false;
        match top_table_button_at(self.mouse, table_layout[1]) {
            Some(TOP_WORLD_BUTTON_INDEX) => {
                self.world_editor_request.store(true, Ordering::Relaxed);
            }
            Some(TOP_WORLD_VIEWER_BUTTON_INDEX) => {
                self.world_viewer_request.store(true, Ordering::Relaxed);
            }
            Some(TOP_EVENT_BUTTON_INDEX) => {
                self.event_editor_request.store(true, Ordering::Relaxed);
            }
            Some(TOP_ICON_BUTTON_INDEX) => {
                self.icon_viewer_request.store(true, Ordering::Relaxed);
            }
            Some(TOP_INFO_BUTTON_INDEX) => {
                self.idle_viewer_request.store(true, Ordering::Relaxed);
            }
            Some(TOP_MUSIC_BUTTON_INDEX) => {
                self.unit_walk_viewer_request.store(true, Ordering::Relaxed);
            }
            _ => {}
        }
    }
}

impl FrameProducer for LoadScreen {
    fn cursor_visible(&self) -> bool {
        false
    }

    fn window_decorations(&self) -> bool {
        false
    }

    fn window_resizable(&self) -> bool {
        false
    }

    fn window_drag_region(&self) -> bool {
        self.window_drag_region_at(self.mouse)
    }

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
            InputEvent::MouseButton {
                button: InputMouseButton::Left,
                state: InputButtonState::Released,
            } => {}
            InputEvent::TextInput(text) if self.chat_focused => self.push_chat_text(&text),
            InputEvent::DigitPressed(digit) if self.chat_focused => {
                self.push_chat_text(&digit.to_string());
            }
            InputEvent::KeyPressed(key) if self.chat_focused => match key {
                InputKey::U => self.push_chat_text("u"),
                InputKey::J => self.push_chat_text("j"),
                InputKey::H => self.push_chat_text("h"),
                InputKey::K => self.push_chat_text("k"),
            },
            InputEvent::BackspacePressed if self.chat_focused => {
                self.chat_input.pop();
            }
            InputEvent::EnterPressed if self.chat_focused => {
                self.send_chat_input();
            }
            InputEvent::EscapePressed if self.chat_focused => {
                self.chat_focused = false;
            }
            _ => {}
        }
    }

    fn build_frame(&mut self, adapter: &mut Adapter) {
        self.poll_lobbies();
        self.poll_chat();
        self.update_background_scene();
        self.upload_assets(adapter);

        let _ = adapter.begin_frame(0xFFFFFF);
        let _ = adapter.set_sampler_raw(0, 0, 0, 0);
        let _ = adapter.set_blend_raw(1, 0x0302, 0x0303);

        self.draw_background_scene(adapter);
        let table_layout = offset_table_layout(
            loadscreen_table_layout(self.window_width as f32, self.window_height as f32),
            self.layout_offset,
        );
        self.draw_frame(adapter);
        let mut tables = SpriteBatch::new(self.window_width, self.window_height);
        for table in table_layout {
            let table = scaled_table_rect(table, LOADSCREEN_TABLE_DRAW_SCALE);
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

        let large_table = table_layout[0];
        self.draw_lobby_cards(adapter, large_table);
        self.draw_create_game_button(adapter, large_table);
        self.draw_top_table_buttons(adapter, table_layout[1]);
        self.draw_chat(
            adapter,
            table_layout[2],
            chat_input_rect(table_layout[1], table_layout[2]),
        );
        self.draw_close_button(adapter);

        self.draw_cursor(adapter);
        let _ = adapter.end_frame();
    }
}

fn draw_lobby_card_panel(batch: &mut ts_ui::UiBatch, x: f32, y: f32, w: f32, h: f32) {
    batch.paper_panel(x, y, w, h, LOBBY_CARD_TILE, Rgba8::WHITE);
}

fn draw_lobby_card_text(batch: &mut ts_ui::UiBatch, lobby: &Lobby, x: f32, y: f32, w: f32) {
    let text_x = x + LOBBY_CARD_PAD_X;
    let max_text_w = w - LOBBY_CARD_PAD_X * 3.0 - LOBBY_ARROW_SIZE;
    let name = fit_lobby_text(&lobby.name, max_text_w, 2.0);
    let players = if lobby.max_players == 0 {
        format!("{} PLAYERS", lobby.players)
    } else {
        format!("{} OF {} PLAYERS", lobby.players, lobby.max_players)
    };
    let status = fit_lobby_text(&lobby.status, max_text_w, 1.0);

    batch.text(&name, text_x, y + 30.0, 2.0, LOBBY_TEXT);
    batch.text(&players, text_x, y + 65.0, 1.0, LOBBY_MUTED_TEXT);
    batch.text(&status, text_x, y + 88.0, 1.0, LOBBY_MUTED_TEXT);
}

fn draw_status_card_text(
    batch: &mut ts_ui::UiBatch,
    title: &str,
    server: &str,
    detail: &str,
    x: f32,
    y: f32,
    w: f32,
) {
    let title = fit_lobby_text(title, w - LOBBY_CARD_PAD_X * 2.0, 2.0);
    let server = fit_lobby_text(server, w - LOBBY_CARD_PAD_X * 2.0, 1.0);
    let detail = fit_lobby_text(detail, w - LOBBY_CARD_PAD_X * 2.0, 1.0);
    draw_centered_text(batch, &title, x, y + PAPER_TITLE_Y, w, 2.0, LOBBY_TEXT);
    draw_centered_text(batch, &server, x, y + 70.0, w, 1.0, LOBBY_MUTED_TEXT);
    draw_centered_text(batch, &detail, x, y + 94.0, w, 1.0, LOBBY_MUTED_TEXT);
}

fn draw_centered_text(
    batch: &mut ts_ui::UiBatch,
    text: &str,
    x: f32,
    y: f32,
    w: f32,
    scale: f32,
    color: Rgba8,
) {
    let text_w = ts_ui::text_width(text, scale);
    batch.text(
        text,
        (x + (w - text_w).max(0.0) * 0.5).round(),
        y.round(),
        scale,
        color,
    );
}

fn top_table_button_origin(table: TableRect) -> (f32, f32) {
    let total_w =
        TOP_BUTTON_SIZE * TOP_BUTTON_COUNT as f32 + TOP_BUTTON_GAP * (TOP_BUTTON_COUNT - 1) as f32;
    (table.x + (table.w - total_w) * 0.5, table.y + 72.0)
}

fn top_table_button_at(point: Point, table: TableRect) -> Option<usize> {
    let (start_x, button_y) = top_table_button_origin(table);
    (0..TOP_BUTTON_COUNT).find(|index| {
        let x = start_x + *index as f32 * (TOP_BUTTON_SIZE + TOP_BUTTON_GAP);
        point.x >= x
            && point.x <= x + TOP_BUTTON_SIZE
            && point.y >= button_y
            && point.y <= button_y + TOP_BUTTON_SIZE
    })
}

fn top_table_button_label(index: usize) -> &'static str {
    match index {
        TOP_WORLD_BUTTON_INDEX => "WORLD EDITOR",
        TOP_WORLD_VIEWER_BUTTON_INDEX => "WORLD VIEWER",
        TOP_EVENT_BUTTON_INDEX => "EVENT EDITOR",
        TOP_ICON_BUTTON_INDEX => "ICON VIEWER",
        TOP_INFO_BUTTON_INDEX => "IDLE WORLD",
        TOP_MUSIC_BUTTON_INDEX => "UNIT WALK",
        _ => "",
    }
}

fn create_game_button_rect(table: TableRect) -> TableRect {
    TableRect {
        x: table.x + table.w - 72.0 - CLOSE_BUTTON_SIZE,
        y: table.y + 24.0,
        w: CLOSE_BUTTON_SIZE,
        h: CLOSE_BUTTON_SIZE,
    }
}

fn chat_panel_rect(table: TableRect) -> TableRect {
    TableRect {
        x: table.x + 44.0,
        y: table.y + 48.0,
        w: (table.w - 88.0).max(120.0),
        h: (table.h - 96.0).max(120.0),
    }
}

fn chat_input_rect(top_table: TableRect, bottom_table: TableRect) -> TableRect {
    let w = (bottom_table.w - 124.0).clamp(180.0, 360.0);
    let h = 34.0;
    let gap_top = top_table.y + top_table.h;
    let gap_h = (bottom_table.y - gap_top).max(h);
    TableRect {
        x: bottom_table.x + (bottom_table.w - w) * 0.5,
        y: gap_top + (gap_h - h) * 0.5,
        w,
        h,
    }
}

fn close_button_rect(window_w: f32) -> TableRect {
    let half_button = CLOSE_BUTTON_SIZE * 0.5;
    TableRect {
        x: (window_w - FRAME_CORNER_SIZE - half_button).max(0.0),
        y: (FRAME_CORNER_SIZE - half_button).max(0.0),
        w: CLOSE_BUTTON_SIZE,
        h: CLOSE_BUTTON_SIZE,
    }
}

fn spawn_lobby_refresh() -> Receiver<Result<Vec<Lobby>, String>> {
    spawn_lobby_request(|| cli::get_lobbies().map_err(|error| error.to_string()))
}

fn spawn_lobby_create_game() -> Receiver<Result<Vec<Lobby>, String>> {
    spawn_lobby_request(|| cli::create_game_and_get_lobbies().map_err(|error| error.to_string()))
}

fn spawn_lobby_request(
    request: impl FnOnce() -> Result<Vec<Lobby>, String> + Send + 'static,
) -> Receiver<Result<Vec<Lobby>, String>> {
    let (lobby_tx, lobby_rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = lobby_tx.send(request());
    });
    lobby_rx
}

fn spawn_chat_client() -> (Sender<ChatCommand>, Receiver<ChatClientEvent>) {
    let (command_tx, command_rx) = mpsc::channel();
    let (event_tx, event_rx) = mpsc::channel();
    thread::spawn(move || {
        let mut since = 0;
        let mut last_poll = Instant::now() - Duration::from_millis(CHAT_POLL_MS);
        loop {
            match command_rx.recv_timeout(Duration::from_millis(100)) {
                Ok(ChatCommand::Send(text)) => match post_chat_message(&text) {
                    Ok(()) => {
                        last_poll = Instant::now() - Duration::from_millis(CHAT_POLL_MS);
                    }
                    Err(error) => {
                        let _ = event_tx.send(ChatClientEvent::Error(error));
                    }
                },
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }

            if last_poll.elapsed() < Duration::from_millis(CHAT_POLL_MS) {
                continue;
            }
            last_poll = Instant::now();

            match fetch_chat_messages(since) {
                Ok(messages) => {
                    if let Some(max_id) = messages.iter().map(|message| message.id).max() {
                        since = since.max(max_id);
                    }
                    if !messages.is_empty() {
                        let _ = event_tx.send(ChatClientEvent::Messages(messages));
                    }
                }
                Err(error) => {
                    let _ = event_tx.send(ChatClientEvent::Error(error));
                }
            }
        }
    });
    (command_tx, event_rx)
}

fn fetch_chat_messages(since: u64) -> Result<Vec<ChatMessage>, String> {
    let url = format!("{CHAT_SERVER_BASE}/api/rooms/{CHAT_ROOM}/messages?since={since}");
    let response = chat_http_client()?
        .get(url)
        .send()
        .map_err(|error| format!("chat get: {error}"))?
        .error_for_status()
        .map_err(|error| format!("chat status: {error}"))?
        .json::<ChatMessagesResponse>()
        .map_err(|error| format!("chat parse failed: {error}"))?;
    Ok(response.messages)
}

fn post_chat_message(text: &str) -> Result<(), String> {
    let url = format!("{CHAT_SERVER_BASE}/api/rooms/{CHAT_ROOM}/messages");
    chat_http_client()?
        .post(url)
        .json(&ChatPost {
            user: CHAT_USER,
            text,
        })
        .send()
        .map_err(|error| format!("chat post: {error}"))?
        .error_for_status()
        .map_err(|error| format!("chat status: {error}"))?;
    Ok(())
}

fn chat_http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_millis(CHAT_CONNECT_TIMEOUT_MS))
        .build()
        .map_err(|error| format!("chat client: {error}"))
}

fn generate_loadscreen_background_scene(
    scene_index: u64,
    cloud_assets: &[ImageAsset],
) -> (TileWorld, Vec<bool>, Vec<CloudInstance>) {
    let seed = DEFAULT_SEED ^ 0x10AD_5CEE_2026 ^ scene_index.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let mut generator = wldgenerator::RunningGenerator::new(WORLD_COLS, WORLD_ROWS, seed);
    generator.complete_initial_seeds();
    while generator.fill_visual_voids_once(256) != 0 {}
    let visual_world = generator.world().to_visual_tile_world();
    let clouds = generate_clouds(seed ^ 0xC10D_2026, cloud_assets, WORLD_COLS, WORLD_ROWS);
    (visual_world.tiles, visual_world.visible, clouds)
}

fn offset_table_layout(mut layout: [TableRect; 3], offset: Point) -> [TableRect; 3] {
    for table in &mut layout {
        table.x += offset.x;
        table.y += offset.y;
    }
    layout
}

fn scaled_table_rect(table: TableRect, scale: f32) -> TableRect {
    let w = table.w * scale;
    let h = table.h * scale;
    TableRect {
        x: table.x + (table.w - w) * 0.5,
        y: table.y + (table.h - h) * 0.5,
        w,
        h,
    }
}

fn fit_lobby_text(text: &str, max_w: f32, scale: f32) -> String {
    let max_chars = (max_w / (6.0 * scale)).floor().max(1.0) as usize;
    let mut cleaned = String::with_capacity(text.len());
    for ch in text.chars() {
        cleaned.push(match ch.to_ascii_uppercase() {
            'A'..='I' | 'K'..='P' | 'R'..='W' | 'Y' | '0'..='9' | '-' | '+' | ':' => {
                ch.to_ascii_uppercase()
            }
            _ => ' ',
        });
    }

    let cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    if cleaned.chars().count() <= max_chars {
        return cleaned;
    }

    let take_count = max_chars.saturating_sub(1);
    let mut shortened: String = cleaned.chars().take(take_count).collect();
    shortened.push('-');
    shortened
}

fn fit_chat_text(text: &str, max_w: f32, scale: f32) -> String {
    let max_chars = (max_w / (6.0 * scale)).floor().max(1.0) as usize;
    let cleaned = sanitize_chat_display_text(text);
    if cleaned.chars().count() <= max_chars {
        return cleaned;
    }

    let take_count = max_chars.saturating_sub(1);
    let mut shortened: String = cleaned.chars().take(take_count).collect();
    shortened.push('-');
    shortened
}

fn fit_chat_input_text(text: &str, max_w: f32, scale: f32) -> String {
    let max_chars = (max_w / (6.0 * scale)).floor().max(1.0) as usize;
    let cleaned = sanitize_chat_display_text(text);
    let char_count = cleaned.chars().count();
    if char_count <= max_chars {
        return cleaned;
    }

    cleaned.chars().skip(char_count - max_chars).collect()
}

fn sanitize_chat_display_text(text: &str) -> String {
    text.chars()
        .map(|ch| match ch.to_ascii_uppercase() {
            'A'..='I' | 'K'..='P' | 'R'..='W' | 'Y' | '0'..='9' | '-' | '+' | ':' => {
                ch.to_ascii_uppercase()
            }
            _ => ' ',
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_chat_input_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric()
        || matches!(
            ch,
            ' ' | '-' | '+' | ':' | '.' | ',' | '!' | '?' | '\'' | '"' | '/'
        )
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
    let x = x.round();
    let y = y.round();
    let w = w.round();
    let h = h.round();
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

fn draw_asset_rect(
    adapter: &mut Adapter,
    image: &ImageAsset,
    window_w: u32,
    window_h: u32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }

    let mut batch = SpriteBatch::new(window_w, window_h);
    batch.image(x, y, w, h, Rgba8::WHITE);
    let _ = adapter.draw_tex_triangles_no_present(image.texture_id, &batch.bytes);
}

fn draw_tiled_asset(
    adapter: &mut Adapter,
    image: &ImageAsset,
    window_w: u32,
    window_h: u32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    tile_w: f32,
    tile_h: f32,
) {
    if w <= 0.0 || h <= 0.0 || tile_w <= 0.0 || tile_h <= 0.0 {
        return;
    }

    let mut batch = SpriteBatch::new(window_w, window_h);
    let mut dy = 0.0;
    while dy < h {
        let draw_h = (h - dy).min(tile_h);
        let mut dx = 0.0;
        while dx < w {
            let draw_w = (w - dx).min(tile_w);
            batch.image(x + dx, y + dy, draw_w, draw_h, Rgba8::WHITE);
            dx += draw_w;
        }
        dy += draw_h;
    }
    let _ = adapter.draw_tex_triangles_no_present(image.texture_id, &batch.bytes);
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
