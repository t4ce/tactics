use super::cli::{self, Lobby};
use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver, TryRecvError},
};
use std::thread;

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
const LOADSCREEN_CURSOR_SIZE: f32 = 64.0;
const LOADSCREEN_ARROW_TEXTURE: u32 = 16;
const LOADSCREEN_ICON_TEXTURE_BASE: u32 = 17;
const LOADSCREEN_ARROW_BYTES: &[u8] =
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Icons/Icon_08.png");
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
const TOP_BUTTON_SIZE: f32 = 64.0;
const TOP_BUTTON_GAP: f32 = 22.0;
const TOP_ICON_SIZE: f32 = 38.0;
const TOP_INFO_BUTTON_INDEX: usize = 1;
const TOP_MUSIC_BUTTON_INDEX: usize = 2;
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
    wood_table: ImageAsset,
    special_paper: ImageAsset,
    small_blue_square_button: ImageAsset,
    arrow_icon: ImageAsset,
    top_icons: [ImageAsset; 3],
    cursor_default: ImageAsset,
    lobbies: Vec<Lobby>,
    lobby_error: Option<String>,
    lobby_rx: Option<Receiver<Result<Vec<Lobby>, String>>>,
    unit_walk_viewer_request: Arc<AtomicBool>,
    idle_viewer_request: Arc<AtomicBool>,
    mouse: Point,
    uploaded: bool,
    window_width: u32,
    window_height: u32,
}

impl LoadScreen {
    pub(super) fn new(
        unit_walk_viewer_request: Arc<AtomicBool>,
        idle_viewer_request: Arc<AtomicBool>,
    ) -> Self {
        let (lobby_tx, lobby_rx) = mpsc::channel();
        thread::spawn(move || {
            let result = cli::get_lobbies().map_err(|error| error.to_string());
            let _ = lobby_tx.send(result);
        });

        Self {
            wood_table: ImageAsset::from_png_bytes(WOOD_TABLE_TEXTURE, WOOD_TABLE_BYTES),
            special_paper: ImageAsset::from_png_bytes(
                ts_ui::SPECIAL_PAPER_TEXTURE,
                ts_ui::SPECIAL_PAPER_BYTES,
            ),
            small_blue_square_button: ImageAsset::from_png_bytes(
                ts_ui::SMALL_BLUE_SQUARE_BUTTON_TEXTURE,
                ts_ui::SMALL_BLUE_SQUARE_BUTTON_BYTES,
            ),
            arrow_icon: ImageAsset::from_png_bytes_cropped(
                LOADSCREEN_ARROW_TEXTURE,
                LOADSCREEN_ARROW_BYTES,
            ),
            top_icons: [
                ImageAsset::from_png_bytes_cropped(
                    LOADSCREEN_ICON_TEXTURE_BASE,
                    LOADSCREEN_TOP_ICON_BYTES[0],
                ),
                ImageAsset::from_png_bytes_cropped(
                    LOADSCREEN_ICON_TEXTURE_BASE + 1,
                    LOADSCREEN_TOP_ICON_BYTES[1],
                ),
                ImageAsset::from_png_bytes_cropped(
                    LOADSCREEN_ICON_TEXTURE_BASE + 2,
                    LOADSCREEN_TOP_ICON_BYTES[2],
                ),
            ],
            cursor_default: ImageAsset::from_png_bytes(
                CURSOR_DEFAULT_TEXTURE,
                CURSOR_DEFAULT_BYTES,
            ),
            lobbies: Vec::new(),
            lobby_error: None,
            lobby_rx: Some(lobby_rx),
            unit_walk_viewer_request,
            idle_viewer_request,
            mouse: Point::default(),
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
            self.arrow_icon.texture_id,
            self.arrow_icon.width,
            self.arrow_icon.height,
            &self.arrow_icon.rgba,
        );
        assert_eq!(rc, 0, "failed to upload loadscreen arrow texture");
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
            LOADSCREEN_CURSOR_SIZE,
            LOADSCREEN_CURSOR_SIZE,
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

    fn draw_top_table_buttons(&self, adapter: &mut Adapter, table: TableRect) {
        let (start_x, button_y) = top_table_button_origin(table);
        let mut buttons = SpriteBatch::new(self.window_width, self.window_height);

        for index in 0..3 {
            buttons.image(
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
        draw_centered_text(
            &mut status,
            self.server_status_text(),
            table.x,
            button_y + TOP_BUTTON_SIZE + 26.0,
            table.w,
            1.0,
            LOBBY_TEXT,
        );
        let _ = adapter.draw_rgb_triangles_no_present(&status.solid_bytes);
    }

    fn status_card_text(&self) -> (&str, &str, &str) {
        if self.lobby_rx.is_some() {
            ("GAME LIST", "SERVER CONNECTING", "LOADING")
        } else if self.lobby_error.is_some() {
            ("GAME LIST", "SERVER OFFLINE", "TRY AGAIN")
        } else {
            ("GAME LIST", "SERVER CONNECTED", "NO GAMES")
        }
    }

    fn server_status_text(&self) -> &str {
        if self.lobby_rx.is_some() {
            "SERVER CONNECTING"
        } else if self.lobby_error.is_some() {
            "SERVER OFFLINE"
        } else {
            "SERVER CONNECTED"
        }
    }

    fn handle_left_click(&mut self) {
        let table_layout =
            loadscreen_table_layout(self.window_width as f32, self.window_height as f32);
        match top_table_button_at(self.mouse, table_layout[1]) {
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
        self.poll_lobbies();
        self.upload_assets(adapter);

        let _ = adapter.begin_frame(WATER_BG);
        let _ = adapter.set_sampler_raw(0, 0, 0, 0);
        let _ = adapter.set_blend_raw(1, 0x0302, 0x0303);

        let table_layout =
            loadscreen_table_layout(self.window_width as f32, self.window_height as f32);
        let mut tables = SpriteBatch::new(self.window_width, self.window_height);
        for table in table_layout {
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
        self.draw_top_table_buttons(adapter, table_layout[1]);

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
    draw_centered_text(batch, &title, x, y + 26.0, w, 2.0, LOBBY_TEXT);
    draw_centered_text(batch, &server, x, y + 64.0, w, 1.0, LOBBY_MUTED_TEXT);
    draw_centered_text(batch, &detail, x, y + 88.0, w, 1.0, LOBBY_MUTED_TEXT);
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
    batch.text(text, x + (w - text_w).max(0.0) * 0.5, y, scale, color);
}

fn top_table_button_origin(table: TableRect) -> (f32, f32) {
    let total_w = TOP_BUTTON_SIZE * 3.0 + TOP_BUTTON_GAP * 2.0;
    (table.x + (table.w - total_w) * 0.5, table.y + 72.0)
}

fn top_table_button_at(point: Point, table: TableRect) -> Option<usize> {
    let (start_x, button_y) = top_table_button_origin(table);
    (0..3).find(|index| {
        let x = start_x + *index as f32 * (TOP_BUTTON_SIZE + TOP_BUTTON_GAP);
        point.x >= x
            && point.x <= x + TOP_BUTTON_SIZE
            && point.y >= button_y
            && point.y <= button_y + TOP_BUTTON_SIZE
    })
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
