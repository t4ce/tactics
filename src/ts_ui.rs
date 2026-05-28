use adapterlibgfx::vertex::{Rgba8, TexVertex};

pub const BANNER_TEXTURE: u32 = 21;
pub const BIG_RIBBONS_TEXTURE: u32 = 22;
pub const SMALL_RIBBONS_TEXTURE: u32 = 23;
pub const SMALL_BAR_BASE_TEXTURE: u32 = 24;
pub const REGULAR_PAPER_TEXTURE: u32 = 25;
pub const SPECIAL_PAPER_TEXTURE: u32 = 26;
pub const SMALL_BLUE_SQUARE_BUTTON_TEXTURE: u32 = 27;
pub const SMALL_BLUE_ROUND_BUTTON_TEXTURE: u32 = 28;
pub const BANNER_SLOTS_TEXTURE: u32 = 29;
pub const BANNER_BYTES: &[u8] =
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Banners/Banner.png");
pub const BANNER_SLOTS_BYTES: &[u8] =
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Banners/Banner_Slots.png");
pub const SMALL_BLUE_SQUARE_BUTTON_BYTES: &[u8] = include_bytes!(
    "../ts_freepack/UI Elements/UI Elements/Buttons/SmallBlueSquareButton_Regular.png"
);
pub const SMALL_BLUE_ROUND_BUTTON_BYTES: &[u8] = include_bytes!(
    "../ts_freepack/UI Elements/UI Elements/Buttons/SmallBlueRoundButton_Regular.png"
);
pub const BIG_RIBBONS_BYTES: &[u8] =
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Ribbons/BigRibbons.png");
pub const SMALL_RIBBONS_BYTES: &[u8] =
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Ribbons/SmallRibbons.png");
pub const SMALL_BAR_BASE_BYTES: &[u8] =
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Bars/SmallBar_Base.png");
pub const REGULAR_PAPER_BYTES: &[u8] =
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Papers/RegularPaper.png");
pub const SPECIAL_PAPER_BYTES: &[u8] =
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Papers/SpecialPaper.png");

const BANNER_SOURCE_PX: f32 = 448.0;
const BANNER_SIDE_SOURCE_PX: f32 = 128.0;
const BANNER_CENTER_SOURCE_X: f32 = 192.0;
const BANNER_CENTER_SOURCE_PX: f32 = 64.0;
const BANNER_RIGHT_SOURCE_X: f32 = 320.0;
const MENU_PAGE_TILE: f32 = 64.0;
const HOTKEY_MENU_COLS: usize = 4;
const HOTKEY_MENU_PAD_X: f32 = 48.0;
const HOTKEY_MENU_PAD_TOP: f32 = 64.0;
const HOTKEY_MENU_PAD_BOTTOM: f32 = 36.0;
const HOTKEY_CELL_W: f32 = 236.0;
const HOTKEY_CELL_H: f32 = 70.0;
const HOTKEY_PREFIX_SLOT_W: f32 = 62.0;
const HOTKEY_BUTTON_SIZE: f32 = 44.0;
const HOTKEY_TEXT_SCALE: f32 = 2.0;
const HOTKEY_BODY_TEXT: Rgba8 = Rgba8 {
    r: 34,
    g: 38,
    b: 42,
    a: 255,
};
const PAPER_SOURCE_PX: f32 = 320.0;
const PAPER_TILE_PX: f32 = 64.0;
const PAPER_CENTER_SOURCE_PX: f32 = 128.0;
const PAPER_RIGHT_SOURCE_PX: f32 = 256.0;
const PAPER_BOTTOM_SOURCE_PX: f32 = 256.0;
const RIBBON_ROWS: usize = 5;
const RIBBON_ROW_PX: f32 = 128.0;
const RIBBON_SIDE_SOURCE_PX: f32 = 128.0;
const RIBBON_CENTER_SOURCE_X: f32 = 192.0;
const RIBBON_CENTER_SOURCE_PX: f32 = 64.0;
const RIBBON_RIGHT_SOURCE_X: f32 = 320.0;
const BIG_RIBBON_W: f32 = 448.0;
#[allow(dead_code)]
const SMALL_RIBBON_W: f32 = 320.0;
const RIBBON_H: f32 = 640.0;
const SMALL_BAR_BASE_W: f32 = 320.0;
const SMALL_BAR_TEXTURE_H: f32 = 64.0;
const SMALL_BAR_FRAME_TOP: f32 = 22.0;
const SMALL_BAR_FRAME_BOTTOM: f32 = 41.0;
const SMALL_BAR_FRAME_H: f32 = SMALL_BAR_FRAME_BOTTOM - SMALL_BAR_FRAME_TOP;
const SMALL_BAR_LEFT_X: f32 = 49.0;
const SMALL_BAR_LEFT_W: f32 = 15.0;
const SMALL_BAR_CENTER_X: f32 = 160.0;
const SMALL_BAR_RIGHT_X: f32 = 256.0;
const SMALL_BAR_RIGHT_W: f32 = 15.0;
const SMALL_BAR_FILL_TOP: f32 = 30.0;
const SMALL_BAR_FILL_BOTTOM: f32 = 33.0;

pub struct UiBatch {
    window_width: u32,
    window_height: u32,
    pub texture_bytes: Vec<u8>,
    pub ribbon_bytes: Vec<u8>,
    pub button_bytes: Vec<u8>,
    pub solid_bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenuPage {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HotkeyEntry<'a> {
    pub prefix: Option<&'a str>,
    pub key: &'a str,
    pub label: &'a str,
}

pub struct SmallBarBatch {
    window_width: u32,
    window_height: u32,
    pub base_bytes: Vec<u8>,
    pub fill_solid_bytes: Vec<u8>,
}

impl SmallBarBatch {
    pub fn new(window_width: u32, window_height: u32) -> Self {
        Self {
            window_width,
            window_height,
            base_bytes: Vec::new(),
            fill_solid_bytes: Vec::new(),
        }
    }

    pub fn small_bar(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        value: f32,
        fill_color: Rgba8,
        base_color: Rgba8,
    ) {
        let value = value.clamp(0.0, 1.0);
        if value <= 0.0 || w <= 0.0 || h <= 0.0 {
            return;
        }

        let scaled_w = w * value;
        let scale = h / SMALL_BAR_FRAME_H;
        let left_w = SMALL_BAR_LEFT_W * scale;
        let right_w = SMALL_BAR_RIGHT_W * scale;
        let cap_w = left_w.min(right_w).min(scaled_w * 0.5);
        let center_w = (scaled_w - cap_w * 2.0).max(0.0);
        let fill_h = (SMALL_BAR_FILL_BOTTOM - SMALL_BAR_FILL_TOP) * scale;
        let fill_y = y + (SMALL_BAR_FILL_TOP - SMALL_BAR_FRAME_TOP) * scale;

        if center_w > 0.0 && fill_h > 0.0 {
            self.solid_rect(x + cap_w, fill_y, center_w, fill_h, fill_color);
        }

        self.base_piece(
            SMALL_BAR_LEFT_X,
            SMALL_BAR_LEFT_W,
            x,
            y,
            cap_w,
            h,
            base_color,
        );
        if center_w > 0.0 {
            self.base_piece(
                SMALL_BAR_CENTER_X,
                1.0,
                x + cap_w,
                y,
                center_w,
                h,
                base_color,
            );
        }
        self.base_piece(
            SMALL_BAR_RIGHT_X,
            SMALL_BAR_RIGHT_W,
            x + scaled_w - cap_w,
            y,
            cap_w,
            h,
            base_color,
        );
    }

    fn base_piece(
        &mut self,
        source_x: f32,
        source_w: f32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: Rgba8,
    ) {
        if w <= 0.0 || h <= 0.0 {
            return;
        }

        self.image_uv(
            x,
            y,
            w,
            h,
            [
                source_x / SMALL_BAR_BASE_W,
                SMALL_BAR_FRAME_TOP / SMALL_BAR_TEXTURE_H,
                (source_x + source_w) / SMALL_BAR_BASE_W,
                SMALL_BAR_FRAME_BOTTOM / SMALL_BAR_TEXTURE_H,
            ],
            color,
        );
    }

    fn image_uv(&mut self, x: f32, y: f32, w: f32, h: f32, uv: [f32; 4], color: Rgba8) {
        let [u0, v0, u1, v1] = uv;
        let (x0, y0) = self.to_clip(x, y);
        let (x1, y1) = self.to_clip(x + w, y + h);
        self.tex_vertex(x0, y0, u0, v0, color);
        self.tex_vertex(x1, y0, u1, v0, color);
        self.tex_vertex(x1, y1, u1, v1, color);
        self.tex_vertex(x0, y0, u0, v0, color);
        self.tex_vertex(x1, y1, u1, v1, color);
        self.tex_vertex(x0, y1, u0, v1, color);
    }

    fn solid_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Rgba8) {
        let (x0, y0) = self.to_clip(x, y);
        let (x1, y1) = self.to_clip(x + w, y + h);
        self.solid_vertex(x0, y0, color);
        self.solid_vertex(x1, y0, color);
        self.solid_vertex(x1, y1, color);
        self.solid_vertex(x0, y0, color);
        self.solid_vertex(x1, y1, color);
        self.solid_vertex(x0, y1, color);
    }

    fn tex_vertex(&mut self, x: f32, y: f32, u: f32, v: f32, color: Rgba8) {
        let vertex = TexVertex { x, y, u, v, color };
        push_f32(&mut self.base_bytes, vertex.x);
        push_f32(&mut self.base_bytes, vertex.y);
        push_f32(&mut self.base_bytes, vertex.u);
        push_f32(&mut self.base_bytes, vertex.v);
        self.base_bytes.extend_from_slice(&[
            vertex.color.r,
            vertex.color.g,
            vertex.color.b,
            vertex.color.a,
        ]);
    }

    fn solid_vertex(&mut self, x: f32, y: f32, color: Rgba8) {
        push_f32(&mut self.fill_solid_bytes, x);
        push_f32(&mut self.fill_solid_bytes, y);
        self.fill_solid_bytes
            .extend_from_slice(&[color.r, color.g, color.b, color.a]);
    }

    fn to_clip(&self, x: f32, y: f32) -> (f32, f32) {
        (
            (x / self.window_width as f32) * 2.0 - 1.0,
            1.0 - (y / self.window_height as f32) * 2.0,
        )
    }
}

impl UiBatch {
    pub fn new(window_width: u32, window_height: u32) -> Self {
        Self {
            window_width,
            window_height,
            texture_bytes: Vec::new(),
            ribbon_bytes: Vec::new(),
            button_bytes: Vec::new(),
            solid_bytes: Vec::new(),
        }
    }

    pub fn menu_page(
        &mut self,
        title: &str,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: Rgba8,
    ) -> MenuPage {
        self.banner_panel(x, y, w, h, MENU_PAGE_TILE, color);
        let ribbon_max_w = (w - 96.0).max(160.0);
        let ribbon_w = (w * 0.54).clamp(160.0, ribbon_max_w);
        let ribbon_h = 54.0;
        let ribbon_x = x + (w - ribbon_w) * 0.5;
        let ribbon_y = y + 19.0;
        self.big_ribbon(
            RIBBON_ROWS - 1,
            ribbon_x,
            ribbon_y,
            ribbon_w,
            ribbon_h,
            Rgba8::WHITE,
        );
        let title_scale = 2.0;
        let title_w = text_width(title, title_scale);
        self.text(
            title,
            x + (w - title_w) * 0.5,
            ribbon_y + 17.0,
            title_scale,
            Rgba8::WHITE,
        );
        MenuPage { x, y, w, h }
    }

    pub fn hotkey_menu_page(
        &mut self,
        title: &str,
        x: f32,
        y: f32,
        entries: &[HotkeyEntry<'_>],
    ) -> MenuPage {
        let (w, h) = hotkey_menu_page_size(entries.len());
        let page = self.menu_page(title, x, y, w, h, Rgba8::new(255, 255, 255, 245));
        let start_x = x + HOTKEY_MENU_PAD_X;
        let start_y = y + HOTKEY_MENU_PAD_TOP;

        for (index, entry) in entries.iter().enumerate() {
            let col = index % HOTKEY_MENU_COLS;
            let row = index / HOTKEY_MENU_COLS;
            let cell_x = start_x + col as f32 * HOTKEY_CELL_W;
            let cell_y = start_y + row as f32 * HOTKEY_CELL_H;
            let button_x = cell_x + HOTKEY_PREFIX_SLOT_W;
            if let Some(prefix) = entry.prefix {
                let prefix_text = format!("{prefix} +");
                let prefix_w = text_width(&prefix_text, HOTKEY_TEXT_SCALE);
                self.text(
                    &prefix_text,
                    button_x - prefix_w - 2.0,
                    cell_y + 16.0,
                    HOTKEY_TEXT_SCALE,
                    HOTKEY_BODY_TEXT,
                );
            }
            self.hotkey_button(button_x, cell_y, entry.key, HOTKEY_TEXT_SCALE, Rgba8::WHITE);
            self.text(
                entry.label,
                button_x + HOTKEY_BUTTON_SIZE + 12.0,
                cell_y + 16.0,
                HOTKEY_TEXT_SCALE,
                HOTKEY_BODY_TEXT,
            );
        }

        page
    }

    pub fn banner_panel(&mut self, x: f32, y: f32, w: f32, h: f32, tile: f32, color: Rgba8) {
        let tile = tile.min(w * 0.5).min(h * 0.5).max(1.0);
        let center_w = (w - tile * 2.0).max(0.0);
        let center_h = (h - tile * 2.0).max(0.0);

        self.banner_piece(
            0.0,
            0.0,
            x,
            y,
            tile,
            tile,
            BANNER_SIDE_SOURCE_PX,
            BANNER_SIDE_SOURCE_PX,
            color,
        );
        self.banner_piece(
            BANNER_RIGHT_SOURCE_X,
            0.0,
            x + w - tile,
            y,
            tile,
            tile,
            BANNER_SIDE_SOURCE_PX,
            BANNER_SIDE_SOURCE_PX,
            color,
        );
        self.banner_piece(
            0.0,
            BANNER_RIGHT_SOURCE_X,
            x,
            y + h - tile,
            tile,
            tile,
            BANNER_SIDE_SOURCE_PX,
            BANNER_SIDE_SOURCE_PX,
            color,
        );
        self.banner_piece(
            BANNER_RIGHT_SOURCE_X,
            BANNER_RIGHT_SOURCE_X,
            x + w - tile,
            y + h - tile,
            tile,
            tile,
            BANNER_SIDE_SOURCE_PX,
            BANNER_SIDE_SOURCE_PX,
            color,
        );

        self.tiled_banner_piece(
            BANNER_CENTER_SOURCE_X,
            0.0,
            x + tile,
            y,
            center_w,
            tile,
            tile,
            BANNER_CENTER_SOURCE_PX,
            BANNER_SIDE_SOURCE_PX,
            color,
        );
        self.tiled_banner_piece(
            BANNER_CENTER_SOURCE_X,
            BANNER_RIGHT_SOURCE_X,
            x + tile,
            y + h - tile,
            center_w,
            tile,
            tile,
            BANNER_CENTER_SOURCE_PX,
            BANNER_SIDE_SOURCE_PX,
            color,
        );
        self.tiled_banner_piece(
            0.0,
            BANNER_CENTER_SOURCE_X,
            x,
            y + tile,
            tile,
            center_h,
            tile,
            BANNER_SIDE_SOURCE_PX,
            BANNER_CENTER_SOURCE_PX,
            color,
        );
        self.tiled_banner_piece(
            BANNER_RIGHT_SOURCE_X,
            BANNER_CENTER_SOURCE_X,
            x + w - tile,
            y + tile,
            tile,
            center_h,
            tile,
            BANNER_SIDE_SOURCE_PX,
            BANNER_CENTER_SOURCE_PX,
            color,
        );
        self.tiled_banner_piece(
            BANNER_CENTER_SOURCE_X,
            BANNER_CENTER_SOURCE_X,
            x + tile,
            y + tile,
            center_w,
            center_h,
            tile,
            BANNER_CENTER_SOURCE_PX,
            BANNER_CENTER_SOURCE_PX,
            color,
        );
    }

    #[allow(dead_code)]
    pub fn paper_panel_tiles(
        &mut self,
        x: f32,
        y: f32,
        cols: usize,
        rows: usize,
        tile: f32,
        color: Rgba8,
    ) {
        if cols == 0 || rows == 0 {
            return;
        }

        self.paper_panel(x, y, cols as f32 * tile, rows as f32 * tile, tile, color);
    }

    pub fn paper_panel(&mut self, x: f32, y: f32, w: f32, h: f32, tile: f32, color: Rgba8) {
        if w <= 0.0 || h <= 0.0 {
            return;
        }

        let tile = tile.min(w * 0.5).min(h * 0.5).max(1.0);
        let center_w = (w - tile * 2.0).max(0.0);
        let center_h = (h - tile * 2.0).max(0.0);

        self.paper_piece(0.0, 0.0, x, y, tile, tile, color);
        self.paper_piece(
            PAPER_RIGHT_SOURCE_PX,
            0.0,
            x + w - tile,
            y,
            tile,
            tile,
            color,
        );
        self.paper_piece(
            0.0,
            PAPER_BOTTOM_SOURCE_PX,
            x,
            y + h - tile,
            tile,
            tile,
            color,
        );
        self.paper_piece(
            PAPER_RIGHT_SOURCE_PX,
            PAPER_BOTTOM_SOURCE_PX,
            x + w - tile,
            y + h - tile,
            tile,
            tile,
            color,
        );

        self.tiled_paper_piece(
            PAPER_CENTER_SOURCE_PX,
            0.0,
            x + tile,
            y,
            center_w,
            tile,
            tile,
            color,
        );
        self.tiled_paper_piece(
            PAPER_CENTER_SOURCE_PX,
            PAPER_BOTTOM_SOURCE_PX,
            x + tile,
            y + h - tile,
            center_w,
            tile,
            tile,
            color,
        );
        self.tiled_paper_piece(
            0.0,
            PAPER_CENTER_SOURCE_PX,
            x,
            y + tile,
            tile,
            center_h,
            tile,
            color,
        );
        self.tiled_paper_piece(
            PAPER_RIGHT_SOURCE_PX,
            PAPER_CENTER_SOURCE_PX,
            x + w - tile,
            y + tile,
            tile,
            center_h,
            tile,
            color,
        );
        self.tiled_paper_piece(
            PAPER_CENTER_SOURCE_PX,
            PAPER_CENTER_SOURCE_PX,
            x + tile,
            y + tile,
            center_w,
            center_h,
            tile,
            color,
        );
    }

    pub fn big_ribbon(&mut self, row: usize, x: f32, y: f32, w: f32, h: f32, color: Rgba8) {
        self.ribbon(BIG_RIBBON_W, row, x, y, w, h, color);
    }

    #[allow(dead_code)]
    pub fn small_ribbon(&mut self, row: usize, x: f32, y: f32, w: f32, h: f32, color: Rgba8) {
        self.ribbon(SMALL_RIBBON_W, row, x, y, w, h, color);
    }

    pub fn text(&mut self, text: &str, x: f32, y: f32, scale: f32, color: Rgba8) {
        let mut cursor_x = x;
        for ch in text.chars() {
            if ch == '\n' {
                cursor_x = x;
                continue;
            }
            self.glyph(ch, cursor_x, y, scale, color);
            cursor_x += 6.0 * scale;
        }
    }

    fn ribbon(&mut self, source_w: f32, row: usize, x: f32, y: f32, w: f32, h: f32, color: Rgba8) {
        let cap = h.min(w * 0.5).max(1.0);
        let center_w = (w - cap * 2.0).max(0.0);
        let source_y = (row % RIBBON_ROWS) as f32 * RIBBON_ROW_PX;

        self.ribbon_image_uv(
            x,
            y,
            cap,
            h,
            [
                0.0,
                source_y / RIBBON_H,
                RIBBON_SIDE_SOURCE_PX / source_w,
                (source_y + RIBBON_ROW_PX) / RIBBON_H,
            ],
            color,
        );
        self.ribbon_image_uv(
            x + cap,
            y,
            center_w,
            h,
            [
                RIBBON_CENTER_SOURCE_X / source_w,
                source_y / RIBBON_H,
                (RIBBON_CENTER_SOURCE_X + RIBBON_CENTER_SOURCE_PX) / source_w,
                (source_y + RIBBON_ROW_PX) / RIBBON_H,
            ],
            color,
        );
        self.ribbon_image_uv(
            x + w - cap,
            y,
            cap,
            h,
            [
                RIBBON_RIGHT_SOURCE_X / source_w,
                source_y / RIBBON_H,
                1.0,
                (source_y + RIBBON_ROW_PX) / RIBBON_H,
            ],
            color,
        );
    }

    fn tiled_banner_piece(
        &mut self,
        source_x: f32,
        source_y: f32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        tile: f32,
        source_tile_w: f32,
        source_tile_h: f32,
        color: Rgba8,
    ) {
        let mut yy = 0.0;
        while yy < h {
            let piece_h = (h - yy).min(tile);
            let mut xx = 0.0;
            while xx < w {
                let piece_w = (w - xx).min(tile);
                let source_piece_w = source_tile_w * (piece_w / tile);
                let source_piece_h = source_tile_h * (piece_h / tile);
                self.banner_piece(
                    source_x,
                    source_y,
                    x + xx,
                    y + yy,
                    piece_w,
                    piece_h,
                    source_piece_w,
                    source_piece_h,
                    color,
                );
                xx += tile;
            }
            yy += tile;
        }
    }

    fn banner_piece(
        &mut self,
        source_x: f32,
        source_y: f32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        source_w: f32,
        source_h: f32,
        color: Rgba8,
    ) {
        self.image_uv(
            x,
            y,
            w,
            h,
            [
                source_x / BANNER_SOURCE_PX,
                source_y / BANNER_SOURCE_PX,
                (source_x + source_w) / BANNER_SOURCE_PX,
                (source_y + source_h) / BANNER_SOURCE_PX,
            ],
            color,
        );
    }

    fn tiled_paper_piece(
        &mut self,
        source_x: f32,
        source_y: f32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        tile: f32,
        color: Rgba8,
    ) {
        let mut yy = 0.0;
        while yy < h {
            let piece_h = (h - yy).min(tile);
            let mut xx = 0.0;
            while xx < w {
                let piece_w = (w - xx).min(tile);
                self.paper_piece(source_x, source_y, x + xx, y + yy, piece_w, piece_h, color);
                xx += tile;
            }
            yy += tile;
        }
    }

    fn paper_piece(
        &mut self,
        source_x: f32,
        source_y: f32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: Rgba8,
    ) {
        self.image_uv(
            x,
            y,
            w,
            h,
            [
                source_x / PAPER_SOURCE_PX,
                source_y / PAPER_SOURCE_PX,
                (source_x + PAPER_TILE_PX) / PAPER_SOURCE_PX,
                (source_y + PAPER_TILE_PX) / PAPER_SOURCE_PX,
            ],
            color,
        );
    }

    fn image_uv(&mut self, x: f32, y: f32, w: f32, h: f32, uv: [f32; 4], color: Rgba8) {
        let [u0, v0, u1, v1] = uv;
        let (x0, y0) = self.to_clip(x, y);
        let (x1, y1) = self.to_clip(x + w, y + h);
        self.tex_vertex(x0, y0, u0, v0, color);
        self.tex_vertex(x1, y0, u1, v0, color);
        self.tex_vertex(x1, y1, u1, v1, color);
        self.tex_vertex(x0, y0, u0, v0, color);
        self.tex_vertex(x1, y1, u1, v1, color);
        self.tex_vertex(x0, y1, u0, v1, color);
    }

    fn ribbon_image_uv(&mut self, x: f32, y: f32, w: f32, h: f32, uv: [f32; 4], color: Rgba8) {
        let [u0, v0, u1, v1] = uv;
        let (x0, y0) = self.to_clip(x, y);
        let (x1, y1) = self.to_clip(x + w, y + h);
        self.ribbon_vertex(x0, y0, u0, v0, color);
        self.ribbon_vertex(x1, y0, u1, v0, color);
        self.ribbon_vertex(x1, y1, u1, v1, color);
        self.ribbon_vertex(x0, y0, u0, v0, color);
        self.ribbon_vertex(x1, y1, u1, v1, color);
        self.ribbon_vertex(x0, y1, u0, v1, color);
    }

    fn hotkey_button(&mut self, x: f32, y: f32, label: &str, text_scale: f32, text_color: Rgba8) {
        self.blue_square_button(x, y, HOTKEY_BUTTON_SIZE, HOTKEY_BUTTON_SIZE, Rgba8::WHITE);
        let label_w = text_width(label, text_scale);
        self.text(
            label,
            x + (HOTKEY_BUTTON_SIZE - label_w) * 0.5,
            y + (HOTKEY_BUTTON_SIZE - 7.0 * text_scale) * 0.5,
            text_scale,
            text_color,
        );
    }

    fn blue_square_button(&mut self, x: f32, y: f32, w: f32, h: f32, color: Rgba8) {
        let (x0, y0) = self.to_clip(x, y);
        let (x1, y1) = self.to_clip(x + w, y + h);
        self.button_vertex(x0, y0, 0.0, 0.0, color);
        self.button_vertex(x1, y0, 1.0, 0.0, color);
        self.button_vertex(x1, y1, 1.0, 1.0, color);
        self.button_vertex(x0, y0, 0.0, 0.0, color);
        self.button_vertex(x1, y1, 1.0, 1.0, color);
        self.button_vertex(x0, y1, 0.0, 1.0, color);
    }

    fn glyph(&mut self, ch: char, x: f32, y: f32, scale: f32, color: Rgba8) {
        let rows = glyph_rows(ch);
        for (row, bits) in rows.iter().enumerate() {
            for col in 0..5 {
                if bits & (1 << (4 - col)) != 0 {
                    self.solid_rect(
                        x + col as f32 * scale,
                        y + row as f32 * scale,
                        scale,
                        scale,
                        color,
                    );
                }
            }
        }
    }

    fn solid_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Rgba8) {
        let (x0, y0) = self.to_clip(x, y);
        let (x1, y1) = self.to_clip(x + w, y + h);
        self.solid_vertex(x0, y0, color);
        self.solid_vertex(x1, y0, color);
        self.solid_vertex(x1, y1, color);
        self.solid_vertex(x0, y0, color);
        self.solid_vertex(x1, y1, color);
        self.solid_vertex(x0, y1, color);
    }

    fn tex_vertex(&mut self, x: f32, y: f32, u: f32, v: f32, color: Rgba8) {
        let vertex = TexVertex { x, y, u, v, color };
        push_f32(&mut self.texture_bytes, vertex.x);
        push_f32(&mut self.texture_bytes, vertex.y);
        push_f32(&mut self.texture_bytes, vertex.u);
        push_f32(&mut self.texture_bytes, vertex.v);
        self.texture_bytes.extend_from_slice(&[
            vertex.color.r,
            vertex.color.g,
            vertex.color.b,
            vertex.color.a,
        ]);
    }

    fn ribbon_vertex(&mut self, x: f32, y: f32, u: f32, v: f32, color: Rgba8) {
        let vertex = TexVertex { x, y, u, v, color };
        push_f32(&mut self.ribbon_bytes, vertex.x);
        push_f32(&mut self.ribbon_bytes, vertex.y);
        push_f32(&mut self.ribbon_bytes, vertex.u);
        push_f32(&mut self.ribbon_bytes, vertex.v);
        self.ribbon_bytes.extend_from_slice(&[
            vertex.color.r,
            vertex.color.g,
            vertex.color.b,
            vertex.color.a,
        ]);
    }

    fn button_vertex(&mut self, x: f32, y: f32, u: f32, v: f32, color: Rgba8) {
        let vertex = TexVertex { x, y, u, v, color };
        push_f32(&mut self.button_bytes, vertex.x);
        push_f32(&mut self.button_bytes, vertex.y);
        push_f32(&mut self.button_bytes, vertex.u);
        push_f32(&mut self.button_bytes, vertex.v);
        self.button_bytes.extend_from_slice(&[
            vertex.color.r,
            vertex.color.g,
            vertex.color.b,
            vertex.color.a,
        ]);
    }

    fn solid_vertex(&mut self, x: f32, y: f32, color: Rgba8) {
        push_f32(&mut self.solid_bytes, x);
        push_f32(&mut self.solid_bytes, y);
        self.solid_bytes
            .extend_from_slice(&[color.r, color.g, color.b, color.a]);
    }

    fn to_clip(&self, x: f32, y: f32) -> (f32, f32) {
        (
            (x / self.window_width as f32) * 2.0 - 1.0,
            1.0 - (y / self.window_height as f32) * 2.0,
        )
    }
}

pub fn hotkey_menu_page_size(entry_count: usize) -> (f32, f32) {
    let rows = entry_count.div_ceil(HOTKEY_MENU_COLS).max(1);
    (
        HOTKEY_MENU_PAD_X * 2.0 + HOTKEY_CELL_W * HOTKEY_MENU_COLS as f32,
        HOTKEY_MENU_PAD_TOP + HOTKEY_CELL_H * rows as f32 + HOTKEY_MENU_PAD_BOTTOM,
    )
}

pub fn text_width(text: &str, scale: f32) -> f32 {
    text.chars().count() as f32 * 6.0 * scale
}

fn glyph_rows(ch: char) -> [u8; 7] {
    match ch.to_ascii_uppercase() {
        'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'B' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
        'C' => [
            0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111,
        ],
        'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'F' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'G' => [
            0b01111, 0b10000, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110,
        ],
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'I' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        'N' => [
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'V' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
        ],
        'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001,
        ],
        'Y' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        '0' => [
            0b01110, 0b10011, 0b10101, 0b10101, 0b10101, 0b11001, 0b01110,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        '3' => [
            0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b00001, 0b00001, 0b11110,
        ],
        '6' => [
            0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110,
        ],
        '-' => [0, 0, 0, 0b11111, 0, 0, 0],
        '+' => [0, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0],
        ':' => [0, 0b00100, 0b00100, 0, 0b00100, 0b00100, 0],
        _ => [0; 7],
    }
}

fn push_f32(out: &mut Vec<u8>, value: f32) {
    out.extend_from_slice(&value.to_le_bytes());
}
