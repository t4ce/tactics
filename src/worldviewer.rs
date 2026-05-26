use super::wldgenerator::{
    CLIFF_GRASS_CAP_TILE, PLATFORM_GRASS_BORDER_TILES, PLATFORM_GRASS_ROW_TILES,
    PLATFORM_WATER_ROW_TILES, STANDALONE_GRASS_PILLAR_TILE, STANDALONE_WATER_PILLAR_TILE,
    platform_row_tile,
};
use super::*;

const GENERATED_EMPTY_BG: u32 = 0x263A38;
const GENERATED_WATER_COLOR: Rgba8 = Rgba8 {
    r: 71,
    g: 171,
    b: 169,
    a: 255,
};
const SELECTION_FILL: Rgba8 = Rgba8 {
    r: 255,
    g: 244,
    b: 130,
    a: 42,
};
const SELECTION_BORDER: Rgba8 = Rgba8 {
    r: 255,
    g: 244,
    b: 130,
    a: 190,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TileRect {
    col: usize,
    row: usize,
    cols: usize,
    rows: usize,
}

pub(super) struct WorldViewer {
    terrain: TextureAtlas,
    generator: wldgenerator::RunningGenerator,
    shoreline_world: Option<TileWorld>,
    shoreline_cache: TerrainDrawCache,
    waiting_for_platform_seed_click: bool,
    waiting_for_shoreline_replace_click: bool,
    seed: u64,
    camera: Point,
    mouse: Point,
    panning: bool,
    selection_start_screen: Option<Point>,
    selection_start: Option<(usize, usize)>,
    selection_end: Option<(usize, usize)>,
    uploaded: bool,
    window_width: u32,
    window_height: u32,
}

impl WorldViewer {
    pub(super) fn new() -> Self {
        Self {
            terrain: TextureAtlas::from_png_bytes(TERRAIN_TEXTURE, TERRAIN_BYTES, TERRAIN_TILE_PX),
            generator: wldgenerator::RunningGenerator::new(WORLD_COLS, WORLD_ROWS, DEFAULT_SEED),
            shoreline_world: None,
            shoreline_cache: TerrainDrawCache::new(),
            waiting_for_platform_seed_click: true,
            waiting_for_shoreline_replace_click: false,
            seed: DEFAULT_SEED,
            camera: Point::default(),
            mouse: Point::default(),
            panning: false,
            selection_start_screen: None,
            selection_start: None,
            selection_end: None,
            uploaded: false,
            window_width: WINDOW_WIDTH,
            window_height: WINDOW_HEIGHT,
        }
    }

    fn resize_view(&mut self, width: u32, height: u32) {
        self.window_width = width.max(1);
        self.window_height = height.max(1);
        self.clamp_camera();
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
        assert_eq!(rc, 0, "failed to upload world viewer terrain texture");

        self.uploaded = true;
    }

    fn view_w(&self) -> f32 {
        self.window_width as f32
    }

    fn view_h(&self) -> f32 {
        self.window_height as f32
    }

    fn clamp_camera(&mut self) {
        let world = self.generator.world();
        let max_x = (world.width_px() - self.view_w()).max(0.0);
        let max_y = (world.height_px() - self.view_h()).max(0.0);
        self.camera.x = self.camera.x.clamp(0.0, max_x);
        self.camera.y = self.camera.y.clamp(0.0, max_y);
    }

    fn scroll(&mut self, dx: f32, dy: f32) {
        self.camera.x += dx;
        self.camera.y += dy;
        self.clamp_camera();
    }

    fn step_seed(&mut self, delta: i64) {
        if delta >= 0 {
            self.seed = self.seed.wrapping_add(delta as u64);
        } else {
            self.seed = self.seed.wrapping_sub(delta.unsigned_abs());
        }
        self.generator = wldgenerator::RunningGenerator::new(WORLD_COLS, WORLD_ROWS, self.seed);
        self.shoreline_world = None;
        self.shoreline_cache.mark_dirty();
        self.waiting_for_platform_seed_click = true;
        self.waiting_for_shoreline_replace_click = false;
        self.selection_start_screen = None;
        self.selection_start = None;
        self.selection_end = None;
        self.clamp_camera();
        eprintln!("viewer generated world seed {}", self.seed);
    }

    fn advance_generation(&mut self) {
        if self.waiting_for_platform_seed_click {
            return;
        }

        if self.generator.is_complete() {
            return;
        }

        let was_complete = self.generator.is_complete();
        while !self.generator.is_complete() {
            if self.generator.step(WORLD_COLS * WORLD_ROWS) == 0 {
                break;
            }
        }
        if !was_complete && self.generator.is_complete() && self.shoreline_world.is_none() {
            self.shoreline_world = Some(collapse_generated_world(self.generator.world()));
            self.waiting_for_shoreline_replace_click = true;
            self.shoreline_cache.mark_dirty();
        }
    }

    fn shoreline_replace_final(&mut self) {
        let Some(world) = self.shoreline_world.as_mut() else {
            return;
        };
        wldgenerator::shoreline_replace_final(world);
        self.waiting_for_shoreline_replace_click = false;
        self.shoreline_cache.mark_dirty();
    }

    fn draw(&mut self, adapter: &mut Adapter) {
        let _ = adapter.begin_frame(GENERATED_EMPTY_BG);
        let _ = adapter.set_sampler_raw(0, 0, 0, 0);
        let _ = adapter.set_blend_raw(1, 0x0302, 0x0303);
        let _ = adapter.set_scissor(Some(ScissorRect {
            x: 0,
            y: 0,
            width: self.window_width,
            height: self.window_height,
        }));

        self.draw_world(adapter);
        self.draw_selection(adapter);

        let _ = adapter.set_scissor(None);
        let _ = adapter.end_frame();
    }

    fn draw_world(&mut self, adapter: &mut Adapter) {
        if self.shoreline_world.is_some() {
            self.draw_shoreline_world(adapter);
            return;
        }

        let visual_world = self.generator.world().to_visual_tile_world();
        let world = &visual_world.tiles;
        let mut water = SolidBatch::new(self.window_width, self.window_height);
        let mut backgrounds = SpriteBatch::new(self.window_width, self.window_height);
        let mut foregrounds = SpriteBatch::new(self.window_width, self.window_height);
        let start_col = (self.camera.x / TILE_SIZE).floor().max(0.0) as usize;
        let start_row = (self.camera.y / TILE_SIZE).floor().max(0.0) as usize;
        let end_col = ((self.camera.x + self.view_w()) / TILE_SIZE).ceil() as usize + 1;
        let end_row = ((self.camera.y + self.view_h()) / TILE_SIZE).ceil() as usize + 1;

        for row in start_row..end_row.min(world.rows) {
            for col in start_col..end_col.min(world.cols) {
                let index = world.index(col, row);
                if !visual_world.visible[index] {
                    continue;
                }
                let x = col as f32 * TILE_SIZE - self.camera.x;
                let y = row as f32 * TILE_SIZE - self.camera.y;
                match world.background(col, row) {
                    BackgroundTile::Water => {
                        water.rect(x, y, TILE_SIZE, TILE_SIZE, GENERATED_WATER_COLOR)
                    }
                    BackgroundTile::Grass => backgrounds.sprite(
                        &self.terrain,
                        GRASS_BG_TILE,
                        x,
                        y,
                        TILE_SIZE,
                        TILE_SIZE,
                        Rgba8::WHITE,
                    ),
                }
                if let Some(tile) = world.foreground(col, row) {
                    foregrounds.sprite(
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

        let _ = adapter.set_texture_effect(TextureEffect::World);
        let _ = adapter.draw_rgb_triangles_no_present(&water.bytes);
        let _ = adapter.draw_tex_triangles_no_present(self.terrain.texture_id, &backgrounds.bytes);
        let _ = adapter.draw_tex_triangles_no_present(self.terrain.texture_id, &foregrounds.bytes);
        let _ = adapter.set_texture_effect(TextureEffect::Plain);
    }

    fn draw_shoreline_world(&mut self, adapter: &mut Adapter) {
        let Some(world) = self.shoreline_world.as_ref() else {
            return;
        };
        self.shoreline_cache.rebuild_if_dirty(world);

        let mut water = SolidBatch::new(self.window_width, self.window_height);
        let mut backgrounds = SpriteBatch::new(self.window_width, self.window_height);
        let mut foregrounds = SpriteBatch::new(self.window_width, self.window_height);
        let start_col = (self.camera.x / TILE_SIZE).floor().max(0.0) as usize;
        let start_row = (self.camera.y / TILE_SIZE).floor().max(0.0) as usize;
        let end_col = ((self.camera.x + self.view_w()) / TILE_SIZE).ceil() as usize + 1;
        let end_row = ((self.camera.y + self.view_h()) / TILE_SIZE).ceil() as usize + 1;

        for row in start_row..end_row.min(world.rows) {
            for col in start_col..end_col.min(world.cols) {
                if world.render_background(col, row) == BackgroundTile::Water {
                    water.rect(
                        col as f32 * TILE_SIZE - self.camera.x,
                        row as f32 * TILE_SIZE - self.camera.y,
                        TILE_SIZE,
                        TILE_SIZE,
                        GENERATED_WATER_COLOR,
                    );
                }
            }
        }

        for cell in self
            .shoreline_cache
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
            .shoreline_cache
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

        let _ = adapter.set_texture_effect(TextureEffect::World);
        let _ = adapter.draw_rgb_triangles_no_present(&water.bytes);
        let _ = adapter.draw_tex_triangles_no_present(self.terrain.texture_id, &backgrounds.bytes);
        let _ = adapter.draw_tex_triangles_no_present(self.terrain.texture_id, &foregrounds.bytes);
        let _ = adapter.set_texture_effect(TextureEffect::Plain);
    }

    fn draw_selection(&self, adapter: &mut Adapter) {
        let Some(rect) = self.selection_rect() else {
            return;
        };

        let x = rect.col as f32 * TILE_SIZE - self.camera.x;
        let y = rect.row as f32 * TILE_SIZE - self.camera.y;
        let w = rect.cols as f32 * TILE_SIZE;
        let h = rect.rows as f32 * TILE_SIZE;
        let mut batch = SolidBatch::new(self.window_width, self.window_height);
        batch.rect(x, y, w, h, SELECTION_FILL);
        outline_rect(&mut batch, x, y, w, h, 2.0, SELECTION_BORDER);

        let _ = adapter.set_texture_effect(TextureEffect::Plain);
        let _ = adapter.draw_rgb_triangles_no_present(&batch.bytes);
    }

    fn world_cell_at(&self, x: f32, y: f32) -> Option<(usize, usize)> {
        let col = ((x + self.camera.x) / TILE_SIZE).floor() as isize;
        let row = ((y + self.camera.y) / TILE_SIZE).floor() as isize;
        let world = self.generator.world();
        if col < 0 || row < 0 || col >= world.cols as isize || row >= world.rows as isize {
            return None;
        }

        Some((col as usize, row as usize))
    }

    fn selection_rect(&self) -> Option<TileRect> {
        let (start_col, start_row) = self.selection_start?;
        let (end_col, end_row) = self.selection_end?;
        let col = start_col.min(end_col);
        let row = start_row.min(end_row);
        Some(TileRect {
            col,
            row,
            cols: start_col.max(end_col) - col + 1,
            rows: start_row.max(end_row) - row + 1,
        })
    }

    fn selection_drag_is_large_enough(&self) -> bool {
        const MIN_SELECTION_DRAG_PX: f32 = 5.0;
        let Some(start) = self.selection_start_screen else {
            return false;
        };
        let dx = self.mouse.x - start.x;
        let dy = self.mouse.y - start.y;
        dx.hypot(dy) >= MIN_SELECTION_DRAG_PX
    }

    fn retile_selection(&mut self) {
        if !self.selection_drag_is_large_enough() {
            return;
        }
        let Some(rect) = self.selection_rect() else {
            return;
        };
        if self.shoreline_world.is_none() {
            return;
        };
        let reroll_mask = self
            .generator
            .world()
            .feature_reroll_mask_for_rect(rect.col, rect.row, rect.cols, rect.rows);

        let mut generated = self.generator.world().clone();
        let reroll_seed = rect_regeneration_seed(self.seed, rect);
        generated.reroll_features_in_mask(&reroll_mask, reroll_seed);
        self.generator = wldgenerator::RunningGenerator::from_world(generated, reroll_seed);
        while !self.generator.is_complete() {
            if self.generator.step(WORLD_COLS * WORLD_ROWS) == 0 {
                break;
            }
        }
        self.shoreline_world = Some(collapse_generated_world(self.generator.world()));
        self.waiting_for_shoreline_replace_click = true;
        self.shoreline_cache.mark_dirty();
    }
}

impl FrameProducer for WorldViewer {
    fn resize(&mut self, width: u32, height: u32) {
        self.resize_view(width, height);
    }

    fn handle_input(&mut self, event: InputEvent) {
        match event {
            InputEvent::CursorMoved { x, y } => {
                if self.panning {
                    self.scroll(self.mouse.x - x, self.mouse.y - y);
                }
                self.mouse = Point { x, y };
                if self.selection_start.is_some() {
                    if let Some(cell) = self.world_cell_at(x, y) {
                        self.selection_end = Some(cell);
                    }
                }
            }
            InputEvent::MouseButton {
                button: InputMouseButton::Middle,
                state: InputButtonState::Pressed,
            } => {
                self.panning = true;
            }
            InputEvent::MouseButton {
                button: InputMouseButton::Middle,
                state: InputButtonState::Released,
            } => {
                self.panning = false;
            }
            InputEvent::MouseButton {
                button: InputMouseButton::Left,
                state: InputButtonState::Pressed,
            } => {
                if self.waiting_for_platform_seed_click {
                    self.waiting_for_platform_seed_click = false;
                    self.advance_generation();
                    return;
                }
                if self.waiting_for_shoreline_replace_click {
                    self.shoreline_replace_final();
                    return;
                }

                let cell = self.world_cell_at(self.mouse.x, self.mouse.y);
                self.selection_start_screen = Some(self.mouse);
                self.selection_start = cell;
                self.selection_end = cell;
            }
            InputEvent::MouseButton {
                button: InputMouseButton::Left,
                state: InputButtonState::Released,
            } => {
                if let Some(cell) = self.world_cell_at(self.mouse.x, self.mouse.y) {
                    self.selection_end = Some(cell);
                }
                self.retile_selection();
                self.selection_start_screen = None;
                self.selection_start = None;
                self.selection_end = None;
            }
            InputEvent::MouseWheel { x, y } => {
                if y != 0.0 {
                    self.step_seed(seed_steps_from_wheel(y));
                } else if x != 0.0 {
                    self.step_seed(seed_steps_from_wheel(x));
                }
            }
            InputEvent::KeyPressed(InputKey::H) => self.scroll(-TILE_SIZE, 0.0),
            InputEvent::KeyPressed(InputKey::K) => self.scroll(TILE_SIZE, 0.0),
            InputEvent::KeyPressed(InputKey::U) => self.scroll(0.0, -TILE_SIZE),
            InputEvent::KeyPressed(InputKey::J) => self.scroll(0.0, TILE_SIZE),
            InputEvent::EscapePressed => {
                self.panning = false;
                self.selection_start_screen = None;
                self.selection_start = None;
                self.selection_end = None;
            }
            _ => {}
        }
    }

    fn build_frame(&mut self, adapter: &mut Adapter) {
        self.upload_assets(adapter);
        self.advance_generation();
        self.draw(adapter);
    }
}

fn seed_steps_from_wheel(delta: f32) -> i64 {
    let magnitude = delta.abs().round().max(1.0) as i64;
    if delta > 0.0 { magnitude } else { -magnitude }
}

fn collapse_generated_world(generated: &wldgenerator::GeneratedWorld) -> TileWorld {
    generated.to_visual_tile_world().tiles
}

fn rect_regeneration_seed(seed: u64, rect: TileRect) -> u64 {
    seed ^ 0xA11E_57EC_7A11_2026
        ^ ((rect.col as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
        ^ ((rect.row as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9))
        ^ ((rect.cols as u64).wrapping_mul(0x94D0_49BB_1331_11EB))
        ^ ((rect.rows as u64).wrapping_mul(0xD6E8_FD9D_5A64_4A03))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_rows_use_middle_tile_for_single_width_rects() {
        assert_eq!(
            platform_row_tile(BackgroundTile::Water, 0, 1),
            PLATFORM_WATER_ROW_TILES[1]
        );
        assert_eq!(
            platform_row_tile(BackgroundTile::Grass, 0, 1),
            PLATFORM_GRASS_ROW_TILES[1]
        );
    }

    #[test]
    fn platform_rows_use_left_middle_right_caps() {
        assert_eq!(
            platform_row_tile(BackgroundTile::Water, 0, 4),
            PLATFORM_WATER_ROW_TILES[0]
        );
        assert_eq!(
            platform_row_tile(BackgroundTile::Water, 1, 4),
            PLATFORM_WATER_ROW_TILES[1]
        );
        assert_eq!(
            platform_row_tile(BackgroundTile::Water, 2, 4),
            PLATFORM_WATER_ROW_TILES[1]
        );
        assert_eq!(
            platform_row_tile(BackgroundTile::Water, 3, 4),
            PLATFORM_WATER_ROW_TILES[2]
        );
    }

    #[test]
    fn terrain_cross_overlay_draws_top_cap_and_center_pillar_variant() {
        let generated = wldgenerator::GeneratedWorld {
            cols: 5,
            rows: 5,
            cells: vec![wldgenerator::GeneratedCell::Unknown; 25],
            platforms: Vec::new(),
            terrain_crosses: vec![
                wldgenerator::GeneratedTerrainCross {
                    col: 1,
                    center_row: 1,
                    background: BackgroundTile::Grass,
                },
                wldgenerator::GeneratedTerrainCross {
                    col: 3,
                    center_row: 3,
                    background: BackgroundTile::Water,
                },
            ],
        };
        let world = generated.to_visual_tile_world().tiles;

        assert_eq!(world.foreground(1, 0), Some(CLIFF_GRASS_CAP_TILE));
        assert_eq!(world.foreground(1, 1), Some(STANDALONE_GRASS_PILLAR_TILE));
        assert_eq!(world.foreground(3, 2), Some(CLIFF_GRASS_CAP_TILE));
        assert_eq!(world.foreground(3, 3), Some(STANDALONE_WATER_PILLAR_TILE));
        assert_eq!(world.foreground(0, 1), None);
        assert_eq!(world.foreground(2, 1), None);
        assert_eq!(world.foreground(1, 2), None);
    }

    #[test]
    fn terrain_cross_seed_mask_keeps_empty_cross_arms_visible_inside_platforms() {
        let generated = wldgenerator::GeneratedWorld {
            cols: 5,
            rows: 5,
            cells: vec![wldgenerator::GeneratedCell::Grass; 25],
            platforms: vec![wldgenerator::GeneratedPlatform {
                col: 0,
                row: 0,
                cols: 5,
                rows: 5,
                background: BackgroundTile::Grass,
                kind: wldgenerator::GeneratedPlatformKind::Large,
            }],
            terrain_crosses: vec![wldgenerator::GeneratedTerrainCross {
                col: 2,
                center_row: 2,
                background: BackgroundTile::Grass,
            }],
        };

        let visual = generated.to_visual_tile_world();

        for (col, row) in [(2, 1), (1, 2), (2, 2), (3, 2), (2, 3)] {
            assert!(visual.visible[row * generated.cols + col]);
        }
    }

    #[test]
    fn generated_visual_world_shows_platform_frame_but_not_inner_ring() {
        let generated = wldgenerator::GeneratedWorld {
            cols: 5,
            rows: 5,
            cells: vec![wldgenerator::GeneratedCell::Unknown; 25],
            platforms: vec![wldgenerator::GeneratedPlatform {
                col: 0,
                row: 0,
                cols: 5,
                rows: 5,
                background: BackgroundTile::Grass,
                kind: wldgenerator::GeneratedPlatformKind::Large,
            }],
            terrain_crosses: Vec::new(),
        };

        let visual = generated.to_visual_tile_world();

        assert!(visual.visible[generated.cols]);
        assert!(visual.visible[2]);
        assert!(visual.visible[2 * generated.cols]);
        assert!(visual.visible[2 * generated.cols + 4]);
        assert!(!visual.visible[generated.cols + 1]);
        assert!(!visual.visible[generated.cols + 2]);
        assert!(!visual.visible[generated.cols + 3]);
        assert!(!visual.visible[2 * generated.cols + 1]);
        assert!(!visual.visible[2 * generated.cols + 2]);
    }

    #[test]
    fn generated_platform_seed_world_keeps_platform_interiors_from_generated_cells() {
        let generated = wldgenerator::GeneratedWorld {
            cols: 5,
            rows: 5,
            cells: vec![wldgenerator::GeneratedCell::Water; 25],
            platforms: vec![wldgenerator::GeneratedPlatform {
                col: 1,
                row: 1,
                cols: 3,
                rows: 3,
                background: BackgroundTile::Grass,
                kind: wldgenerator::GeneratedPlatformKind::Large,
            }],
            terrain_crosses: Vec::new(),
        };

        let world = generated.to_visual_tile_world().tiles;

        assert_eq!(world.background(1, 1), BackgroundTile::Grass);
        assert_eq!(world.background(2, 2), BackgroundTile::Water);
        assert_eq!(world.foreground(2, 2), None);
        assert_eq!(world.background(2, 4), BackgroundTile::Grass);
        assert_eq!(world.foreground(2, 4), Some(PLATFORM_GRASS_ROW_TILES[1]));
    }

    #[test]
    fn applying_platform_rect_sets_backgrounds_and_foregrounds() {
        let mut world = TileWorld::new(4, 4);
        for row in 0..world.rows {
            for col in 0..world.cols {
                world.set_background(col, row, BackgroundTile::Water);
            }
        }

        wldgenerator::apply_platform_rect(&mut world, 0, 0, 4, 3, BackgroundTile::Grass);

        assert_eq!(world.background(0, 0), BackgroundTile::Grass);
        assert_eq!(world.background(0, 3), BackgroundTile::Grass);
        assert_eq!(world.background(3, 3), BackgroundTile::Grass);
        assert_eq!(
            world.foreground(0, 0),
            Some(PLATFORM_GRASS_BORDER_TILES[0][0])
        );
        assert_eq!(
            world.foreground(1, 0),
            Some(PLATFORM_GRASS_BORDER_TILES[0][1])
        );
        assert_eq!(
            world.foreground(3, 0),
            Some(PLATFORM_GRASS_BORDER_TILES[0][2])
        );
        assert_eq!(
            world.foreground(0, 1),
            Some(PLATFORM_GRASS_BORDER_TILES[1][0])
        );
        assert_eq!(world.foreground(1, 1), None);
        assert_eq!(world.foreground(2, 1), None);
        assert_eq!(
            world.foreground(3, 1),
            Some(PLATFORM_GRASS_BORDER_TILES[1][2])
        );

        assert_eq!(
            world.foreground(0, 2),
            Some(PLATFORM_GRASS_BORDER_TILES[2][0])
        );
        assert_eq!(
            world.foreground(1, 2),
            Some(PLATFORM_GRASS_BORDER_TILES[2][1])
        );
        assert_eq!(
            world.foreground(2, 2),
            Some(PLATFORM_GRASS_BORDER_TILES[2][1])
        );
        assert_eq!(
            world.foreground(3, 2),
            Some(PLATFORM_GRASS_BORDER_TILES[2][2])
        );

        assert_eq!(world.foreground(0, 3), Some(PLATFORM_GRASS_ROW_TILES[0]));
        assert_eq!(world.foreground(1, 3), Some(PLATFORM_GRASS_ROW_TILES[1]));
        assert_eq!(world.foreground(2, 3), Some(PLATFORM_GRASS_ROW_TILES[1]));
        assert_eq!(world.foreground(3, 3), Some(PLATFORM_GRASS_ROW_TILES[2]));
    }

    #[test]
    fn applying_platform_rects_tiles_overlapping_virtual_shape_once() {
        let mut world = TileWorld::new(8, 4);
        wldgenerator::apply_platform_rects(
            &mut world,
            [
                (0, 0, 5, 3, BackgroundTile::Water),
                (3, 0, 5, 2, BackgroundTile::Water),
            ],
        );

        assert_eq!(
            world.foreground(0, 0),
            Some(PLATFORM_GRASS_BORDER_TILES[0][0])
        );
        for col in 1..7 {
            assert_eq!(
                world.foreground(col, 0),
                Some(PLATFORM_GRASS_BORDER_TILES[0][1])
            );
        }
        assert_eq!(
            world.foreground(7, 0),
            Some(PLATFORM_GRASS_BORDER_TILES[0][2])
        );

        assert_eq!(
            world.foreground(0, 2),
            Some(PLATFORM_GRASS_BORDER_TILES[2][0])
        );
        assert_eq!(
            world.foreground(1, 2),
            Some(PLATFORM_GRASS_BORDER_TILES[2][1])
        );
        assert_eq!(
            world.foreground(4, 2),
            Some(PLATFORM_GRASS_BORDER_TILES[2][2])
        );

        assert_eq!(world.foreground(5, 2), Some(PLATFORM_WATER_ROW_TILES[0]));
        assert_eq!(world.foreground(6, 2), Some(PLATFORM_WATER_ROW_TILES[1]));
        assert_eq!(world.foreground(7, 2), Some(PLATFORM_WATER_ROW_TILES[2]));
        assert_eq!(world.background(0, 3), BackgroundTile::Water);
        assert_eq!(world.background(4, 3), BackgroundTile::Water);
        assert_eq!(world.foreground(0, 3), Some(PLATFORM_WATER_ROW_TILES[0]));
        assert_eq!(world.foreground(1, 3), Some(PLATFORM_WATER_ROW_TILES[1]));
        assert_eq!(world.foreground(4, 3), Some(PLATFORM_WATER_ROW_TILES[2]));
    }

    #[test]
    fn platform_seed_click_fills_raw_world() {
        let mut viewer = WorldViewer::new();

        viewer.advance_generation();

        assert!(!viewer.generator.is_complete());
        assert!(viewer.shoreline_world.is_none());
        assert!(viewer.waiting_for_platform_seed_click);

        viewer.handle_input(InputEvent::MouseButton {
            button: InputMouseButton::Left,
            state: InputButtonState::Pressed,
        });

        assert!(!viewer.waiting_for_platform_seed_click);
        assert!(viewer.generator.is_complete());
        assert!(viewer.shoreline_world.is_some());
        assert!(viewer.waiting_for_shoreline_replace_click);
        assert_eq!(viewer.selection_start, None);
        assert_eq!(viewer.selection_end, None);
    }

    #[test]
    fn generated_visual_world_keeps_seed_foregrounds() {
        let generated = wldgenerator::generate_world(WORLD_COLS, WORLD_ROWS, DEFAULT_SEED);
        let world = generated.to_visual_tile_world().tiles;

        assert!(
            world
                .foregrounds
                .iter()
                .any(|foreground| foreground.is_some())
        );
    }

    #[test]
    fn final_generated_fill_matches_visual_world() {
        let generated = wldgenerator::GeneratedWorld {
            cols: 5,
            rows: 5,
            cells: vec![wldgenerator::GeneratedCell::Grass; 25],
            platforms: vec![wldgenerator::GeneratedPlatform {
                col: 0,
                row: 0,
                cols: 2,
                rows: 2,
                background: BackgroundTile::Grass,
                kind: wldgenerator::GeneratedPlatformKind::Large,
            }],
            terrain_crosses: vec![wldgenerator::GeneratedTerrainCross {
                col: 2,
                center_row: 2,
                background: BackgroundTile::Grass,
            }],
        };

        let world = collapse_generated_world(&generated);
        let expected = generated.to_visual_tile_world().tiles;

        assert_eq!(world.backgrounds, expected.backgrounds);
        assert_eq!(world.foregrounds, expected.foregrounds);
    }

    #[test]
    fn second_click_after_fill_applies_final_shoreline_replace() {
        let mut viewer = WorldViewer::new();

        viewer.handle_input(InputEvent::MouseButton {
            button: InputMouseButton::Left,
            state: InputButtonState::Pressed,
        });
        assert!(viewer.waiting_for_shoreline_replace_click);

        viewer.handle_input(InputEvent::MouseButton {
            button: InputMouseButton::Left,
            state: InputButtonState::Pressed,
        });

        assert!(!viewer.waiting_for_shoreline_replace_click);
    }

    #[test]
    fn click_without_five_pixel_drag_does_not_retile_final_world() {
        let mut viewer = WorldViewer::new();

        viewer.handle_input(InputEvent::MouseButton {
            button: InputMouseButton::Left,
            state: InputButtonState::Pressed,
        });
        viewer.handle_input(InputEvent::MouseButton {
            button: InputMouseButton::Left,
            state: InputButtonState::Pressed,
        });
        let before = viewer.shoreline_world.as_ref().unwrap().foregrounds.clone();

        viewer.handle_input(InputEvent::MouseButton {
            button: InputMouseButton::Left,
            state: InputButtonState::Pressed,
        });
        viewer.handle_input(InputEvent::MouseButton {
            button: InputMouseButton::Left,
            state: InputButtonState::Released,
        });

        assert_eq!(viewer.shoreline_world.as_ref().unwrap().foregrounds, before);
        assert!(!viewer.waiting_for_shoreline_replace_click);
    }
}
