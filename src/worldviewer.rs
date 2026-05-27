use super::*;
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const GENERATED_EMPTY_BG: u32 = 0x263A38;
const GENERATED_WORLD_SAVE_PREFIX: &str = "generated_world";
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
const WALL_PREVIEW_TINT: Rgba8 = Rgba8 {
    r: 255,
    g: 255,
    b: 255,
    a: 128,
};
const RETILE_COVER_TEXTURE: u32 = 11_000;
const RETILE_COVER_BYTES: &[u8] =
    include_bytes!("../ts_freepack/UI Elements/UI Elements/Wood Table/WoodTable_Slots.png");
const RETILE_COVER_TILE_PX: u32 = 64;
const RETILE_COVER_LIGHTEN_PERCENT: u8 = 8;
const RETILE_PARTICLE_TEXTURE_BASE: u32 = 11_100;
const RETILE_FLYOUT_MS: u32 = 500;
const RETILE_FLYOUT_DISTANCE_PX: f32 = 192.0;
const RETILE_COVER_HOLD_MS: u32 = 250;
const RETILE_REVEAL_STAGGER_MS: u32 = 35;
const RETILE_SEQUENTIAL_REVEAL_TILES: usize = 50;
const RETILE_REVEAL_MS: u32 = 50;
const RETILE_BOB_AMPLITUDE_PX: f32 = 2.25;
const RETILE_BOB_PERIOD_MS: f32 = 720.0;
const RETILE_ROTATION_AMPLITUDE_RAD: f32 = 0.0125;
const RETILE_ROTATION_PERIOD_MS: f32 = 960.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TileRect {
    col: usize,
    row: usize,
    cols: usize,
    rows: usize,
}

struct RetileTransition {
    rect: TileRect,
    tiles: Vec<RetileTile>,
    flyout_tiles: Vec<RetileFlyoutTile>,
    started: Instant,
    finish_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RetileTile {
    col: usize,
    row: usize,
    reveal_start_ms: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct RetileFlyoutTile {
    col: usize,
    row: usize,
    background: BackgroundTile,
    under_foreground: Option<AtlasTile>,
    foreground: Option<AtlasTile>,
    dir_x: f32,
    dir_y: f32,
}

impl RetileTransition {
    fn new(
        rect: TileRect,
        seed: u64,
        dust_duration_ms: u32,
        flyout_tiles: Vec<RetileFlyoutTile>,
    ) -> Self {
        let mut tiles = shuffled_retile_tiles(rect_tiles(rect), seed);
        let dust_duration_ms = dust_duration_ms.max(1);
        let mut finish_ms =
            RETILE_FLYOUT_MS.max(RETILE_COVER_HOLD_MS.saturating_add(dust_duration_ms));

        let tiles = tiles
            .drain(..)
            .enumerate()
            .map(|(order, (col, row))| {
                let reveal_order = order.min(RETILE_SEQUENTIAL_REVEAL_TILES);
                let reveal_start_ms = RETILE_COVER_HOLD_MS
                    .saturating_add((reveal_order as u32).saturating_mul(RETILE_REVEAL_STAGGER_MS));
                finish_ms = finish_ms.max(reveal_start_ms.saturating_add(dust_duration_ms));
                RetileTile {
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

    fn cover_reveal_elapsed_ms(&self, tile: RetileTile, elapsed_ms: u32) -> Option<Option<u32>> {
        if elapsed_ms < tile.reveal_start_ms {
            return Some(None);
        }
        let elapsed = elapsed_ms - tile.reveal_start_ms;
        (elapsed < RETILE_REVEAL_MS).then_some(Some(elapsed))
    }

    fn cover_bob_y(&self, elapsed_ms: u32) -> f32 {
        let radians = elapsed_ms as f32 / RETILE_BOB_PERIOD_MS * std::f32::consts::TAU;
        radians.sin() * RETILE_BOB_AMPLITUDE_PX
    }

    fn cover_rotation(&self, elapsed_ms: u32) -> f32 {
        let radians = elapsed_ms as f32 / RETILE_ROTATION_PERIOD_MS * std::f32::consts::TAU;
        radians.sin() * RETILE_ROTATION_AMPLITUDE_RAD
    }

    fn cover_region(&self, tile: RetileTile) -> ImageRegion {
        retile_cover_region_for_rect(
            tile.col.saturating_sub(self.rect.col),
            tile.row.saturating_sub(self.rect.row),
            self.rect.cols,
            self.rect.rows,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WorldViewerTool {
    Select,
    HorizontalWall,
    VerticalWall,
}

pub(super) struct WorldViewer {
    terrain: TextureAtlas,
    retile_cover: ImageAsset,
    water_visuals: WaterVisualAssets,
    plant_props: [SpriteAnimation; PLANT_PROP_COUNT],
    particle_dust: SpriteAnimation,
    generator: wldgenerator::RunningGenerator,
    retile_transition: Option<RetileTransition>,
    seed: u64,
    camera: Point,
    mouse: Point,
    panning: bool,
    selection_start_screen: Option<Point>,
    selection_start: Option<(usize, usize)>,
    selection_end: Option<(usize, usize)>,
    tool: WorldViewerTool,
    horizontal_wall_len: usize,
    vertical_wall_len: usize,
    waiting_for_initial_platform_click: bool,
    uploaded: bool,
    window_width: u32,
    window_height: u32,
}

impl WorldViewer {
    pub(super) fn new() -> Self {
        let mut retile_cover = ImageAsset::from_png_bytes(RETILE_COVER_TEXTURE, RETILE_COVER_BYTES);
        lighten_rgba_toward_white(&mut retile_cover.rgba, RETILE_COVER_LIGHTEN_PERCENT);

        Self {
            terrain: TextureAtlas::from_png_bytes(TERRAIN_TEXTURE, TERRAIN_BYTES, TERRAIN_TILE_PX),
            retile_cover,
            water_visuals: load_water_visual_assets(),
            plant_props: load_plant_prop_assets(),
            particle_dust: load_retile_particle_dust(),
            generator: wldgenerator::RunningGenerator::new(WORLD_COLS, WORLD_ROWS, DEFAULT_SEED),
            retile_transition: None,
            seed: DEFAULT_SEED,
            camera: Point::default(),
            mouse: Point::default(),
            panning: false,
            selection_start_screen: None,
            selection_start: None,
            selection_end: None,
            tool: WorldViewerTool::Select,
            horizontal_wall_len: 1,
            vertical_wall_len: 2,
            waiting_for_initial_platform_click: true,
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

        let rc = adapter.upload_texture_rgba_image(
            self.retile_cover.texture_id,
            self.retile_cover.width,
            self.retile_cover.height,
            &self.retile_cover.rgba,
        );
        assert_eq!(rc, 0, "failed to upload world viewer retile cover");

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
                "failed to upload generated world water texture {}",
                image.texture_id
            );
        }

        for image in self
            .plant_props
            .iter()
            .flat_map(|animation| animation.frames.iter())
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
                "failed to upload generated world plant texture {}",
                image.texture_id
            );
        }

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
        self.selection_start_screen = None;
        self.selection_start = None;
        self.selection_end = None;
        self.retile_transition = None;
        self.tool = WorldViewerTool::Select;
        self.horizontal_wall_len = 1;
        self.vertical_wall_len = 2;
        self.waiting_for_initial_platform_click = true;
        self.clamp_camera();
        eprintln!("viewer generated world seed {}", self.seed);
    }

    fn advance_generation(&mut self) {
        // Intentionally frozen: the world viewer is only showing the seeded
        // platform layout now. The old post-click collapse/fill path stays off.
    }

    fn draw(&mut self, adapter: &mut Adapter) {
        self.update_retile_transition();
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
        self.draw_generated_assets(adapter);
        self.draw_retile_flyout(adapter);
        self.draw_retile_cover(adapter);
        self.draw_retile_particles(adapter);
        self.draw_selection(adapter);
        self.draw_wall_preview(adapter);

        let _ = adapter.set_scissor(None);
        let _ = adapter.end_frame();
    }

    fn draw_world(&mut self, adapter: &mut Adapter) {
        let visual_world = self.generator.world().to_visual_tile_world();
        let world = &visual_world.tiles;
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
                if let Some(tile) = world.under_foreground(col, row) {
                    under_foregrounds.sprite(
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
        let _ = adapter
            .draw_tex_triangles_no_present(self.terrain.texture_id, &under_foregrounds.bytes);
        let _ = adapter.draw_tex_triangles_no_present(self.terrain.texture_id, &foregrounds.bytes);
        let _ = adapter.set_texture_effect(TextureEffect::Plain);
    }

    fn draw_generated_assets(&self, adapter: &mut Adapter) {
        let visual_world = self.generator.world().to_visual_tile_world();
        let world = &visual_world.tiles;
        self.draw_generated_water_states(adapter, world, &visual_world.visible);
        self.draw_generated_props(adapter, world, &visual_world.visible);
    }

    fn update_retile_transition(&mut self) {
        let Some(transition) = &self.retile_transition else {
            return;
        };
        let elapsed_ms = transition.started.elapsed().as_millis() as u32;
        if elapsed_ms >= transition.finish_ms {
            self.retile_transition = None;
        }
    }

    fn draw_retile_flyout(&self, adapter: &mut Adapter) {
        let Some(transition) = &self.retile_transition else {
            return;
        };
        let elapsed_ms = transition.started.elapsed().as_millis() as u32;
        if elapsed_ms >= RETILE_FLYOUT_MS {
            return;
        }

        let t = elapsed_ms as f32 / RETILE_FLYOUT_MS as f32;
        let travel = ease_out_cubic(t) * RETILE_FLYOUT_DISTANCE_PX;
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
                BackgroundTile::Water => water.rect(
                    x,
                    y,
                    TILE_SIZE,
                    TILE_SIZE,
                    Rgba8::new(
                        GENERATED_WATER_COLOR.r,
                        GENERATED_WATER_COLOR.g,
                        GENERATED_WATER_COLOR.b,
                        alpha,
                    ),
                ),
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

    fn draw_retile_cover(&self, adapter: &mut Adapter) {
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
            push_retile_cover_tile(
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

    fn draw_retile_particles(&self, adapter: &mut Adapter) {
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
            self.push_centered_tile_image(&mut batches, image, tile.col, tile.row);
        }

        self.draw_image_batches(adapter, batches);
    }

    fn draw_generated_water_states(
        &self,
        adapter: &mut Adapter,
        world: &TileWorld,
        visible: &[bool],
    ) {
        let start_col = (self.camera.x / TILE_SIZE).floor().max(0.0) as usize;
        let start_row = (self.camera.y / TILE_SIZE).floor().max(0.0) as usize;
        let end_col = ((self.camera.x + self.view_w()) / TILE_SIZE).ceil() as usize + 1;
        let end_row = ((self.camera.y + self.view_h()) / TILE_SIZE).ceil() as usize + 1;
        let mut batches = BTreeMap::new();

        for row in start_row..end_row.min(world.rows) {
            for col in start_col..end_col.min(world.cols) {
                let index = world.index(col, row);
                if !visible[index] {
                    continue;
                }
                let state = world.water_states[index];
                if state == WaterState::Nothing || state == WaterState::Animation {
                    continue;
                }
                let Some(image) = self.water_visual_frame(state) else {
                    continue;
                };
                self.push_centered_tile_image(&mut batches, image, col, row);
            }
        }

        self.draw_image_batches(adapter, batches);
    }

    fn draw_generated_props(&self, adapter: &mut Adapter, world: &TileWorld, visible: &[bool]) {
        let mut batches = BTreeMap::new();
        for prop in &world.props {
            let col = prop.x2 / BUILDING_GRID_DIVISIONS;
            let row = prop.y2 / BUILDING_GRID_DIVISIONS;
            if col >= world.cols || row >= world.rows {
                continue;
            }
            let index = world.index(col, row);
            if !visible[index] {
                continue;
            }

            let PropKind::Plant(kind) = prop.kind else {
                continue;
            };
            let instance_count = kind.visual_instance_count(prop.x2, prop.y2);
            for instance in 0..instance_count {
                let Some(image) = self.plant_props[kind.index()].first_frame() else {
                    continue;
                };
                let offset = kind.visual_instance_offset(instance_count, instance);
                self.push_bottom_aligned_image_half(
                    &mut batches,
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

        self.draw_image_batches(adapter, batches);
    }

    fn water_visual_frame(&self, state: WaterState) -> Option<&ImageAsset> {
        match state {
            WaterState::Nothing | WaterState::Animation => None,
            WaterState::Stone1 => self.water_visuals.stones[0].first_frame(),
            WaterState::Stone2 => self.water_visuals.stones[1].first_frame(),
            WaterState::Stone3 => self.water_visuals.stones[2].first_frame(),
            WaterState::Stone4 => self.water_visuals.stones[3].first_frame(),
            WaterState::Duck => self.water_visuals.duck.first_frame(),
        }
    }

    fn push_centered_tile_image(
        &self,
        batches: &mut BTreeMap<u32, SpriteBatch>,
        image: &ImageAsset,
        col: usize,
        row: usize,
    ) {
        let w = image.width as f32 * BUILDING_SCALE;
        let h = image.height as f32 * BUILDING_SCALE;
        let x = col as f32 * TILE_SIZE - self.camera.x + (TILE_SIZE - w) * 0.5;
        let y = row as f32 * TILE_SIZE - self.camera.y + (TILE_SIZE - h) * 0.5;
        self.push_image_batch(
            batches,
            image.texture_id,
            x.floor(),
            y.floor(),
            w,
            h,
            Rgba8::WHITE,
        );
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
        let x = half_grid_to_px(x2) - self.camera.x + (TILE_SIZE - w) * 0.5 + offset_x;
        let y = half_grid_to_px(y2 + BUILDING_GRID_DIVISIONS) - self.camera.y - h + offset_y;
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

    fn draw_wall_preview(&self, adapter: &mut Adapter) {
        if self.waiting_for_initial_platform_click {
            return;
        }
        let Some((col, row)) = self.world_cell_at(self.mouse.x, self.mouse.y) else {
            return;
        };
        let tiles =
            match self.tool {
                WorldViewerTool::Select => return,
                WorldViewerTool::HorizontalWall => self
                    .generator
                    .horizontal_wall_preview_tiles_centered(col, row, self.horizontal_wall_len),
                WorldViewerTool::VerticalWall => self
                    .generator
                    .vertical_wall_preview_tiles_centered(col, row, self.vertical_wall_len),
            };
        let Some(tiles) = tiles else {
            return;
        };

        let mut batch = SpriteBatch::new(self.window_width, self.window_height);
        for preview in tiles {
            batch.sprite(
                &self.terrain,
                preview.tile,
                preview.col as f32 * TILE_SIZE - self.camera.x,
                preview.row as f32 * TILE_SIZE - self.camera.y,
                TILE_SIZE,
                TILE_SIZE,
                WALL_PREVIEW_TINT,
            );
        }

        let _ = adapter.set_texture_effect(TextureEffect::World);
        let _ = adapter.draw_tex_triangles_no_present(self.terrain.texture_id, &batch.bytes);
        let _ = adapter.set_texture_effect(TextureEffect::Plain);
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
        let world = self.generator.world();
        let large_enough = self.selection_drag_is_large_enough();
        let min_cols = if large_enough { 2 } else { 1 };
        let min_rows = if large_enough { 2 } else { 1 };
        let (screen_dx, screen_dy) = self
            .selection_start_screen
            .map(|start| (self.mouse.x - start.x, self.mouse.y - start.y))
            .unwrap_or((0.0, 0.0));
        Some(selection_rect_with_min_size(
            start_col, start_row, end_col, end_row, screen_dx, screen_dy, min_cols, min_rows,
            world.cols, world.rows,
        ))
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
        if self.retile_transition.is_some() {
            return;
        }
        if !self.selection_drag_is_large_enough() {
            return;
        }
        let Some(rect) = self.selection_rect() else {
            return;
        };

        let reroll_mask = self
            .generator
            .world()
            .feature_reroll_mask_for_rect(rect.col, rect.row, rect.cols, rect.rows);
        let affected_tiles = reroll_mask_tiles(
            self.generator.world().cols,
            self.generator.world().rows,
            &reroll_mask,
        );
        if affected_tiles.is_empty() {
            return;
        }

        let reroll_seed = rect_regeneration_seed(self.seed, rect);
        let old_visual_world = self.generator.world().to_visual_tile_world();
        let flyout_tiles = retile_flyout_tiles(
            &old_visual_world.tiles,
            &old_visual_world.visible,
            rect,
            reroll_seed,
        );
        let mut generated = self.generator.world().clone();
        generated.reroll_features_in_mask(&reroll_mask, reroll_seed);
        let mut pending_generator =
            wldgenerator::RunningGenerator::from_world(generated, reroll_seed);
        pending_generator.fill_visual_voids_once(WORLD_COLS * WORLD_ROWS);
        let transition = RetileTransition::new(
            rect,
            reroll_seed,
            self.particle_dust.total_duration_ms,
            flyout_tiles,
        );
        self.generator = pending_generator;
        self.retile_transition = Some(transition);
    }

    fn place_wall_at_mouse(&mut self) {
        let Some((col, row)) = self.world_cell_at(self.mouse.x, self.mouse.y) else {
            return;
        };
        match self.tool {
            WorldViewerTool::Select => {}
            WorldViewerTool::HorizontalWall => {
                self.generator
                    .add_horizontal_wall_centered(col, row, self.horizontal_wall_len);
            }
            WorldViewerTool::VerticalWall => {
                self.generator
                    .add_vertical_wall_centered(col, row, self.vertical_wall_len);
            }
        }
    }

    fn cycle_tool(&mut self, tool: WorldViewerTool) {
        self.selection_start_screen = None;
        self.selection_start = None;
        self.selection_end = None;
        if self.tool == tool {
            match tool {
                WorldViewerTool::Select => {}
                WorldViewerTool::HorizontalWall => {
                    self.horizontal_wall_len = next_wall_len(self.horizontal_wall_len);
                }
                WorldViewerTool::VerticalWall => {
                    self.vertical_wall_len = next_vertical_wall_len(self.vertical_wall_len);
                }
            }
        } else {
            self.tool = tool;
        }
    }

    fn save_generated_world(&self) {
        match self.write_generated_world_to_root() {
            Ok(path) => eprintln!("saved generated world to {}", path.display()),
            Err(error) => eprintln!("failed to save generated world: {error}"),
        }
    }

    fn write_generated_world_to_root(&self) -> Result<PathBuf, String> {
        let visual_world = self.generator.world().to_visual_tile_world();
        let mut tiles = visual_world.tiles;
        tiles.generated_source = Some(self.generator.world().clone());
        let path = generated_world_save_path()?;
        save_tile_world_without_overwrite(&tiles, &path)?;
        Ok(path)
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
                if self.waiting_for_initial_platform_click {
                    self.waiting_for_initial_platform_click = false;
                    self.generator.complete_initial_seeds();
                    self.generator
                        .fill_visual_voids_once(WORLD_COLS * WORLD_ROWS);
                    return;
                }
                if self.tool != WorldViewerTool::Select {
                    self.place_wall_at_mouse();
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
            InputEvent::DigitPressed(1) => self.cycle_tool(WorldViewerTool::HorizontalWall),
            InputEvent::DigitPressed(2) => self.cycle_tool(WorldViewerTool::VerticalWall),
            InputEvent::DigitPressed(0) => self.save_generated_world(),
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
                self.tool = WorldViewerTool::Select;
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

fn selection_rect_with_min_size(
    start_col: usize,
    start_row: usize,
    end_col: usize,
    end_row: usize,
    screen_dx: f32,
    screen_dy: f32,
    min_cols: usize,
    min_rows: usize,
    world_cols: usize,
    world_rows: usize,
) -> TileRect {
    let grow_right = end_col > start_col || (end_col == start_col && screen_dx >= 0.0);
    let grow_down = end_row > start_row || (end_row == start_row && screen_dy >= 0.0);
    let cols = start_col
        .abs_diff(end_col)
        .saturating_add(1)
        .max(min_cols)
        .min(world_cols);
    let rows = start_row
        .abs_diff(end_row)
        .saturating_add(1)
        .max(min_rows)
        .min(world_rows);

    let col = if grow_right {
        start_col
    } else {
        start_col.saturating_add(1).saturating_sub(cols)
    }
    .min(world_cols.saturating_sub(cols));
    let row = if grow_down {
        start_row
    } else {
        start_row.saturating_add(1).saturating_sub(rows)
    }
    .min(world_rows.saturating_sub(rows));

    TileRect {
        col,
        row,
        cols,
        rows,
    }
}

fn next_wall_len(len: usize) -> usize {
    if len >= 3 { 1 } else { len + 1 }
}

fn next_vertical_wall_len(len: usize) -> usize {
    if len >= 3 { 2 } else { 3 }
}

fn rect_regeneration_seed(seed: u64, rect: TileRect) -> u64 {
    seed ^ 0xA11E_57EC_7A11_2026
        ^ ((rect.col as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
        ^ ((rect.row as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9))
        ^ ((rect.cols as u64).wrapping_mul(0x94D0_49BB_1331_11EB))
        ^ ((rect.rows as u64).wrapping_mul(0xD6E8_FD9D_5A64_4A03))
}

fn reroll_mask_tiles(cols: usize, rows: usize, reroll_mask: &[bool]) -> Vec<(usize, usize)> {
    let mut tiles = Vec::new();
    for row in 0..rows {
        for col in 0..cols {
            let index = row * cols + col;
            if reroll_mask.get(index).copied().unwrap_or(false) {
                tiles.push((col, row));
            }
        }
    }
    tiles
}

fn rect_tiles(rect: TileRect) -> Vec<(usize, usize)> {
    let mut tiles = Vec::with_capacity(rect.cols.saturating_mul(rect.rows));
    for row in rect.row..rect.row + rect.rows {
        for col in rect.col..rect.col + rect.cols {
            tiles.push((col, row));
        }
    }
    tiles
}

fn retile_flyout_tiles(
    world: &TileWorld,
    visible: &[bool],
    rect: TileRect,
    seed: u64,
) -> Vec<RetileFlyoutTile> {
    let mut rng = SeededRng::new(seed ^ 0xF17E_0A17_2026);
    let mut tiles = Vec::with_capacity(rect.cols.saturating_mul(rect.rows));
    for row in rect.row..rect.row + rect.rows {
        for col in rect.col..rect.col + rect.cols {
            if col >= world.cols || row >= world.rows {
                continue;
            }
            let index = world.index(col, row);
            if !visible.get(index).copied().unwrap_or(false) {
                continue;
            }
            let angle = rng.next_f32() * std::f32::consts::TAU;
            tiles.push(RetileFlyoutTile {
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

fn shuffled_retile_tiles(mut tiles: Vec<(usize, usize)>, seed: u64) -> Vec<(usize, usize)> {
    let mut rng = SeededRng::new(seed ^ 0x51DE_5A1F_1EAF_2026);
    for index in (1..tiles.len()).rev() {
        let swap_index = rng.range_usize(0, index + 1);
        tiles.swap(index, swap_index);
    }
    tiles
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn retile_cover_region_for_rect(
    local_col: usize,
    local_row: usize,
    rect_cols: usize,
    rect_rows: usize,
) -> ImageRegion {
    let source_col = retile_cover_slice_index(local_col, rect_cols);
    let source_row = retile_cover_slice_index(local_row, rect_rows);
    ImageRegion::new(
        source_col * RETILE_COVER_TILE_PX,
        source_row * RETILE_COVER_TILE_PX,
        RETILE_COVER_TILE_PX,
        RETILE_COVER_TILE_PX,
    )
}

fn retile_cover_slice_index(local_index: usize, rect_len: usize) -> u32 {
    if local_index == 0 {
        0
    } else if local_index + 1 >= rect_len {
        2
    } else {
        1
    }
}

fn push_retile_cover_tile(
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
        + (RETILE_COVER_TILE_PX - 1).saturating_mul(reveal_elapsed_ms.min(RETILE_REVEAL_MS))
            / RETILE_REVEAL_MS.max(1);
    let left = (RETILE_COVER_TILE_PX - reveal_px) / 2;
    let right = left + reveal_px;
    let top = (RETILE_COVER_TILE_PX - reveal_px) / 2;
    let bottom = top + reveal_px;

    push_retile_cover_segment(
        batch,
        image,
        region,
        0,
        0,
        RETILE_COVER_TILE_PX,
        top,
        x,
        y,
        origin_x,
        origin_y,
        angle_rad,
    );
    push_retile_cover_segment(
        batch,
        image,
        region,
        0,
        bottom,
        RETILE_COVER_TILE_PX,
        RETILE_COVER_TILE_PX - bottom,
        x,
        y,
        origin_x,
        origin_y,
        angle_rad,
    );
    push_retile_cover_segment(
        batch, image, region, 0, top, left, reveal_px, x, y, origin_x, origin_y, angle_rad,
    );
    push_retile_cover_segment(
        batch,
        image,
        region,
        right,
        top,
        RETILE_COVER_TILE_PX - right,
        reveal_px,
        x,
        y,
        origin_x,
        origin_y,
        angle_rad,
    );
}

fn push_retile_cover_segment(
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

fn load_retile_particle_dust() -> SpriteAnimation {
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
                    RETILE_PARTICLE_TEXTURE_BASE + index as u32,
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

fn generated_world_save_path() -> Result<PathBuf, String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("system clock before unix epoch: {error}"))?
        .as_secs();

    for suffix in 0..1000 {
        let file_name = if suffix == 0 {
            format!("{GENERATED_WORLD_SAVE_PREFIX}_{timestamp}.json")
        } else {
            format!("{GENERATED_WORLD_SAVE_PREFIX}_{timestamp}_{suffix}.json")
        };
        let path = PathBuf::from(file_name);
        if !path.exists() {
            return Ok(path);
        }
    }

    Err(format!(
        "could not find a free {GENERATED_WORLD_SAVE_PREFIX}_{timestamp}_*.json path"
    ))
}

fn save_tile_world_without_overwrite(world: &TileWorld, path: &Path) -> Result<(), String> {
    let json = serde_json::to_string_pretty(world)
        .map_err(|error| format!("world json encode failed: {error}"))?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("create {} failed: {error}", path.display()))?;
    file.write_all(json.as_bytes())
        .map_err(|error| format!("write {} failed: {error}", path.display()))
}
