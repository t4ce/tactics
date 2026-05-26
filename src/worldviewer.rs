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
        self.selection_start_screen = None;
        self.selection_start = None;
        self.selection_end = None;
        self.clamp_camera();
        eprintln!("viewer generated world seed {}", self.seed);
    }

    fn advance_generation(&mut self) {
        if self.generator.is_complete() {
            if self.shoreline_world.is_none() {
                self.shoreline_world = Some(collapse_generated_world(self.generator.world()));
                self.shoreline_replace_final();
            }
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
            self.shoreline_replace_final();
            self.shoreline_cache.mark_dirty();
        }
    }

    fn shoreline_replace_final(&mut self) {
        let Some(world) = self.shoreline_world.as_mut() else {
            return;
        };
        let visual_world = self.generator.world().to_visual_tile_world();
        wldgenerator::shoreline_replace_final(world, &visual_world.levels);
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
        let mut under_foregrounds = SpriteBatch::new(self.window_width, self.window_height);
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
        let _ = adapter
            .draw_tex_triangles_no_present(self.terrain.texture_id, &under_foregrounds.bytes);
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
        self.shoreline_replace_final();
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
