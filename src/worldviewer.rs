use super::*;
use std::time::{Duration, Instant};

const GENERATED_EMPTY_BG: u32 = 0x263A38;
const GENERATOR_STEP_INTERVAL: Duration = Duration::from_millis(30);
const GENERATOR_CELLS_PER_STEP: usize = 8;
const GENERATED_WATER_COLOR: Rgba8 = Rgba8 {
    r: 71,
    g: 171,
    b: 169,
    a: 255,
};

pub(super) struct WorldViewer {
    terrain: TextureAtlas,
    generator: wldgenerator::RunningGenerator,
    shoreline_world: Option<TileWorld>,
    shoreline_cache: TerrainDrawCache,
    last_generation_step: Instant,
    seed: u64,
    camera: Point,
    mouse: Point,
    dragging: bool,
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
            last_generation_step: Instant::now(),
            seed: DEFAULT_SEED,
            camera: Point::default(),
            mouse: Point::default(),
            dragging: false,
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
        self.last_generation_step = Instant::now();
        self.clamp_camera();
        eprintln!("viewer generated world seed {}", self.seed);
    }

    fn advance_generation(&mut self) {
        if self.generator.is_complete() {
            return;
        }

        let now = Instant::now();
        let elapsed = now.duration_since(self.last_generation_step);
        if elapsed < GENERATOR_STEP_INTERVAL {
            return;
        }

        let ticks = (elapsed.as_millis() / GENERATOR_STEP_INTERVAL.as_millis()).clamp(1, 4);
        for _ in 0..ticks {
            if self.generator.is_complete() {
                break;
            }
            self.generator.step(GENERATOR_CELLS_PER_STEP);
        }
        self.last_generation_step = now;
        self.refresh_shorelines_if_complete();
    }

    fn refresh_shorelines_if_complete(&mut self) {
        if !self.generator.is_complete() || self.shoreline_world.is_some() {
            return;
        }

        let mut world = self.generator.world().to_tile_world();
        world.collapse_shorelines();
        self.shoreline_world = Some(world);
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

        let _ = adapter.set_scissor(None);
        let _ = adapter.end_frame();
    }

    fn draw_world(&mut self, adapter: &mut Adapter) {
        if self.shoreline_world.is_some() {
            self.draw_shoreline_world(adapter);
            return;
        }

        let world = self.generator.world();
        let mut water = SolidBatch::new(self.window_width, self.window_height);
        let mut grass = SpriteBatch::new(self.window_width, self.window_height);
        let start_col = (self.camera.x / TILE_SIZE).floor().max(0.0) as usize;
        let start_row = (self.camera.y / TILE_SIZE).floor().max(0.0) as usize;
        let end_col = ((self.camera.x + self.view_w()) / TILE_SIZE).ceil() as usize + 1;
        let end_row = ((self.camera.y + self.view_h()) / TILE_SIZE).ceil() as usize + 1;

        for row in start_row..end_row.min(world.rows) {
            for col in start_col..end_col.min(world.cols) {
                let x = col as f32 * TILE_SIZE - self.camera.x;
                let y = row as f32 * TILE_SIZE - self.camera.y;
                match world.cell(col, row) {
                    wldgenerator::GeneratedCell::Unknown => {}
                    wldgenerator::GeneratedCell::Water => {
                        water.rect(x, y, TILE_SIZE, TILE_SIZE, GENERATED_WATER_COLOR);
                    }
                    wldgenerator::GeneratedCell::Grass => grass.sprite(
                        &self.terrain,
                        GRASS_BG_TILE,
                        x,
                        y,
                        TILE_SIZE,
                        TILE_SIZE,
                        Rgba8::WHITE,
                    ),
                }
            }
        }

        let _ = adapter.set_texture_effect(TextureEffect::Plain);
        let _ = adapter.draw_rgb_triangles_no_present(&water.bytes);
        let _ = adapter.set_texture_effect(TextureEffect::World);
        let _ = adapter.draw_tex_triangles_no_present(self.terrain.texture_id, &grass.bytes);
        let _ = adapter.set_texture_effect(TextureEffect::Plain);
    }

    fn draw_shoreline_world(&mut self, adapter: &mut Adapter) {
        let Some(world) = self.shoreline_world.as_ref() else {
            return;
        };
        self.shoreline_cache.rebuild_if_dirty(world);

        let mut backgrounds = SpriteBatch::new(self.window_width, self.window_height);
        let mut foregrounds = SpriteBatch::new(self.window_width, self.window_height);
        let start_col = (self.camera.x / TILE_SIZE).floor().max(0.0) as usize;
        let start_row = (self.camera.y / TILE_SIZE).floor().max(0.0) as usize;
        let end_col = ((self.camera.x + self.view_w()) / TILE_SIZE).ceil() as usize + 1;
        let end_row = ((self.camera.y + self.view_h()) / TILE_SIZE).ceil() as usize + 1;

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
        let _ = adapter.draw_tex_triangles_no_present(self.terrain.texture_id, &backgrounds.bytes);
        let _ = adapter.draw_tex_triangles_no_present(self.terrain.texture_id, &foregrounds.bytes);
        let _ = adapter.set_texture_effect(TextureEffect::Plain);
    }
}

impl FrameProducer for WorldViewer {
    fn resize(&mut self, width: u32, height: u32) {
        self.resize_view(width, height);
    }

    fn handle_input(&mut self, event: InputEvent) {
        match event {
            InputEvent::CursorMoved { x, y } => {
                if self.dragging {
                    self.scroll(self.mouse.x - x, self.mouse.y - y);
                }
                self.mouse = Point { x, y };
            }
            InputEvent::MouseButton {
                button: InputMouseButton::Left | InputMouseButton::Middle,
                state: InputButtonState::Pressed,
            } => {
                self.dragging = true;
            }
            InputEvent::MouseButton {
                button: InputMouseButton::Left | InputMouseButton::Middle,
                state: InputButtonState::Released,
            } => {
                self.dragging = false;
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
                self.dragging = false;
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
