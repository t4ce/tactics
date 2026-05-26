use adapterlibgfx::vertex::{Rgba8, TexVertex};

pub const BANNER_TEXTURE: u32 = 21;
pub const BIG_RIBBONS_TEXTURE: u32 = 22;
pub const SMALL_RIBBONS_TEXTURE: u32 = 23;
pub const SMALL_BAR_BASE_TEXTURE: u32 = 24;
pub const BANNER_BYTES: &[u8] =
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Banners/Banner.png");
pub const BIG_RIBBONS_BYTES: &[u8] =
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Ribbons/BigRibbons.png");
pub const SMALL_RIBBONS_BYTES: &[u8] =
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Ribbons/SmallRibbons.png");
pub const SMALL_BAR_BASE_BYTES: &[u8] =
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Bars/SmallBar_Base.png");

const BANNER_SOURCE_PX: f32 = 448.0;
const BANNER_TILE_PX: f32 = 64.0;
const RIBBON_ROWS: usize = 5;
const RIBBON_ROW_PX: f32 = 128.0;
const RIBBON_CAP_PX: f32 = 64.0;
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
    pub solid_bytes: Vec<u8>,
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
            solid_bytes: Vec::new(),
        }
    }

    pub fn banner_panel(&mut self, x: f32, y: f32, w: f32, h: f32, tile: f32, color: Rgba8) {
        let tile = tile.min(w * 0.5).min(h * 0.5).max(1.0);
        let center_w = (w - tile * 2.0).max(0.0);
        let center_h = (h - tile * 2.0).max(0.0);

        self.banner_piece(0.0, 0.0, x, y, tile, tile, color);
        self.banner_piece(
            BANNER_SOURCE_PX - BANNER_TILE_PX,
            0.0,
            x + w - tile,
            y,
            tile,
            tile,
            color,
        );
        self.banner_piece(
            0.0,
            BANNER_SOURCE_PX - BANNER_TILE_PX,
            x,
            y + h - tile,
            tile,
            tile,
            color,
        );
        self.banner_piece(
            BANNER_SOURCE_PX - BANNER_TILE_PX,
            BANNER_SOURCE_PX - BANNER_TILE_PX,
            x + w - tile,
            y + h - tile,
            tile,
            tile,
            color,
        );

        self.tiled_banner_piece(
            BANNER_TILE_PX,
            0.0,
            x + tile,
            y,
            center_w,
            tile,
            tile,
            color,
        );
        self.tiled_banner_piece(
            BANNER_TILE_PX,
            BANNER_SOURCE_PX - BANNER_TILE_PX,
            x + tile,
            y + h - tile,
            center_w,
            tile,
            tile,
            color,
        );
        self.tiled_banner_piece(
            0.0,
            BANNER_TILE_PX,
            x,
            y + tile,
            tile,
            center_h,
            tile,
            color,
        );
        self.tiled_banner_piece(
            BANNER_SOURCE_PX - BANNER_TILE_PX,
            BANNER_TILE_PX,
            x + w - tile,
            y + tile,
            tile,
            center_h,
            tile,
            color,
        );
        self.tiled_banner_piece(
            BANNER_TILE_PX,
            BANNER_TILE_PX,
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
        let cap = RIBBON_CAP_PX.min(w * 0.5).max(1.0);
        let center_w = (w - cap * 2.0).max(0.0);
        let source_y = (row % RIBBON_ROWS) as f32 * RIBBON_ROW_PX;
        let center_source_w = source_w - RIBBON_CAP_PX * 2.0;

        self.image_uv(
            x,
            y,
            cap,
            h,
            [
                0.0,
                source_y / RIBBON_H,
                RIBBON_CAP_PX / source_w,
                (source_y + RIBBON_ROW_PX) / RIBBON_H,
            ],
            color,
        );
        self.image_uv(
            x + cap,
            y,
            center_w,
            h,
            [
                RIBBON_CAP_PX / source_w,
                source_y / RIBBON_H,
                (RIBBON_CAP_PX + center_source_w) / source_w,
                (source_y + RIBBON_ROW_PX) / RIBBON_H,
            ],
            color,
        );
        self.image_uv(
            x + w - cap,
            y,
            cap,
            h,
            [
                (source_w - RIBBON_CAP_PX) / source_w,
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
        color: Rgba8,
    ) {
        let mut yy = 0.0;
        while yy < h {
            let piece_h = (h - yy).min(tile);
            let mut xx = 0.0;
            while xx < w {
                let piece_w = (w - xx).min(tile);
                self.banner_piece(source_x, source_y, x + xx, y + yy, piece_w, piece_h, color);
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
                (source_x + BANNER_TILE_PX) / BANNER_SOURCE_PX,
                (source_y + BANNER_TILE_PX) / BANNER_SOURCE_PX,
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
        'G' => [
            0b01111, 0b10000, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110,
        ],
        'I' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'N' => [
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
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
        'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001,
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
        ':' => [0, 0b00100, 0b00100, 0, 0b00100, 0b00100, 0],
        _ => [0; 7],
    }
}

fn push_f32(out: &mut Vec<u8>, value: f32) {
    out.extend_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn banner_panel_and_text_emit_batches() {
        let mut ui = UiBatch::new(320, 200);

        ui.banner_panel(10.0, 10.0, 120.0, 48.0, 16.0, Rgba8::WHITE);
        ui.text("BUILDINGS", 22.0, 26.0, 2.0, Rgba8::WHITE);

        assert!(!ui.texture_bytes.is_empty());
        assert!(!ui.solid_bytes.is_empty());
    }

    #[test]
    fn ribbons_emit_horizontal_batches() {
        let mut ui = UiBatch::new(320, 200);

        ui.big_ribbon(0, 12.0, 18.0, 180.0, 32.0, Rgba8::WHITE);
        ui.small_ribbon(1, 12.0, 58.0, 140.0, 24.0, Rgba8::WHITE);

        assert_eq!(ui.texture_bytes.len(), 6 * 3 * 2 * 20);
        assert!(ui.solid_bytes.is_empty());
    }

    #[test]
    fn small_bar_emits_base_and_fill_batches() {
        let mut bars = SmallBarBatch::new(320, 200);

        bars.small_bar(12.0, 18.0, 80.0, 16.0, 0.5, Rgba8::WHITE, Rgba8::WHITE);

        assert_eq!(bars.base_bytes.len(), 6 * 3 * 20);
        assert_eq!(bars.fill_solid_bytes.len(), 6 * 12);
    }
}
