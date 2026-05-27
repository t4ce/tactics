use super::{
    AtlasTile, BUILDING_GRID_DIVISIONS, BackgroundTile, PlacedProp, PlantKind, PropKind,
    SHORE_BOTTOM, SHORE_BOTTOM_LEFT, SHORE_BOTTOM_RIGHT, SHORE_LEFT, SHORE_NARROW_BOTTOM,
    SHORE_NARROW_CENTER, SHORE_NARROW_LEFT, SHORE_NARROW_MIDDLE, SHORE_NARROW_RIGHT,
    SHORE_NARROW_TOP, SHORE_RIGHT, SHORE_SINGLE_IN_WATER, SHORE_TOP, SHORE_TOP_LEFT,
    SHORE_TOP_RIGHT, SeededRng, TILE_SIZE, TileWorld, WaterState,
};

const INITIAL_SEED_NUMERATOR: usize = 1;
const INITIAL_SEED_DENOMINATOR: usize = 200;
const INITIAL_WATER_WEIGHT: usize = 1;
const INITIAL_GRASS_WEIGHT: usize = 8;
const WATER_WEIGHT: f32 = 0.76;
const GRASS_WEIGHT: f32 = 2.0;
const GRASS_AFFINITY_STEP: f32 = 0.95;
const WATER_AFFINITY_STEP: f32 = 0.95;
const LARGE_PLATFORM_COUNT: usize = 3;
const LARGE_PLATFORM_BASE_SIZE: usize = 5;
const LARGE_PLATFORM_EXTRA_MIN: usize = 1;
const LARGE_PLATFORM_EXTRA_MAX: usize = 5;
const SMALL_PLATFORM_MIN_COUNT: usize = 4;
const SMALL_PLATFORM_MAX_COUNT: usize = 6;
const SMALL_PLATFORM_MIN_SIZE: usize = 3;
const SMALL_PLATFORM_MAX_SIZE: usize = 5;
const PLATFORM_PLACEMENT_ATTEMPTS: usize = 64;
const TERRAIN_CROSS_CHANCE: usize = 100;
const GENERATED_BUSH_CHANCE_DENOMINATOR: u64 = 32;
const GENERATED_BUSH_PAIR_CHANCE_NUMERATOR: u64 = 35;
const GENERATED_WATER_VARIATION_CHANCE_DENOMINATOR: u64 = 15;
const GENERATED_BUSH_KINDS: [PlantKind; 4] = [
    PlantKind::Bush1,
    PlantKind::Bush2,
    PlantKind::Bush3,
    PlantKind::Bush4,
];
const GENERATED_WATER_STATES: [WaterState; 5] = [
    WaterState::Stone1,
    WaterState::Stone2,
    WaterState::Stone3,
    WaterState::Stone4,
    WaterState::Duck,
];
const GENERATED_BUSH_ADJACENT_OFFSETS: [(isize, isize); 4] = [
    (BUILDING_GRID_DIVISIONS as isize, 0),
    (-(BUILDING_GRID_DIVISIONS as isize), 0),
    (0, 1),
    (0, -1),
];
pub(super) const PLATFORM_WATER_ROW_TILES: [AtlasTile; 3] = [
    AtlasTile { col: 5, row: 5 },
    AtlasTile { col: 6, row: 5 },
    AtlasTile { col: 7, row: 5 },
];
pub(super) const PLATFORM_GRASS_ROW_TILES: [AtlasTile; 3] = [
    AtlasTile { col: 5, row: 4 },
    AtlasTile { col: 6, row: 4 },
    AtlasTile { col: 7, row: 4 },
];
pub(super) const PLATFORM_GRASS_CLIFF_CAP_TILES: [AtlasTile; 3] = [
    AtlasTile { col: 5, row: 3 },
    AtlasTile { col: 6, row: 3 },
    AtlasTile { col: 7, row: 3 },
];
pub(super) const VERTICAL_GRASS_CLIFF_TILES: [AtlasTile; 3] = [
    AtlasTile { col: 8, row: 0 },
    AtlasTile { col: 8, row: 1 },
    AtlasTile { col: 8, row: 2 },
];
pub(super) const CLIFF_GRASS_CAP_TILE: AtlasTile = AtlasTile { col: 8, row: 3 };
pub(super) const STANDALONE_GRASS_PILLAR_TILE: AtlasTile = AtlasTile { col: 8, row: 4 };
pub(super) const STANDALONE_WATER_PILLAR_TILE: AtlasTile = AtlasTile { col: 8, row: 5 };
pub(super) const PLATFORM_GRASS_BORDER_TILES: [[AtlasTile; 3]; 3] = [
    [
        AtlasTile { col: 5, row: 0 },
        AtlasTile { col: 6, row: 0 },
        AtlasTile { col: 7, row: 0 },
    ],
    [
        AtlasTile { col: 5, row: 1 },
        AtlasTile { col: 6, row: 1 },
        AtlasTile { col: 7, row: 1 },
    ],
    [
        AtlasTile { col: 5, row: 2 },
        AtlasTile { col: 6, row: 2 },
        AtlasTile { col: 7, row: 2 },
    ],
];
const PLATFORM_GRASS_INNER_WATER_TOP_TILE: AtlasTile = AtlasTile { col: 4, row: 0 };
const PLATFORM_GRASS_INNER_WATER_LEFT_TILE: AtlasTile = AtlasTile { col: 4, row: 1 };
const PLATFORM_GRASS_INNER_WATER_RIGHT_TILE: AtlasTile = AtlasTile { col: 4, row: 2 };
const PLATFORM_GRASS_INNER_WATER_BOTTOM_TILE: AtlasTile = AtlasTile { col: 4, row: 3 };
const LEVEL0_SHORELINE_BY_OPEN_MASK: [Option<AtlasTile>; 16] = [
    None,
    Some(SHORE_TOP),
    Some(SHORE_RIGHT),
    Some(SHORE_TOP_RIGHT),
    Some(SHORE_BOTTOM),
    Some(SHORE_NARROW_CENTER),
    Some(SHORE_BOTTOM_RIGHT),
    Some(SHORE_NARROW_RIGHT),
    Some(SHORE_LEFT),
    Some(SHORE_TOP_LEFT),
    Some(SHORE_NARROW_MIDDLE),
    Some(SHORE_NARROW_TOP),
    Some(SHORE_BOTTOM_LEFT),
    Some(SHORE_NARROW_LEFT),
    Some(SHORE_NARROW_BOTTOM),
    Some(SHORE_SINGLE_IN_WATER),
];
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum GeneratedCell {
    Unknown,
    Grass,
    Water,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct GeneratedPlatform {
    pub(super) col: usize,
    pub(super) row: usize,
    pub(super) cols: usize,
    pub(super) rows: usize,
    pub(super) background: BackgroundTile,
    pub(super) kind: GeneratedPlatformKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum GeneratedPlatformKind {
    Large,
    Small,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct GeneratedTerrainCross {
    pub(super) col: usize,
    pub(super) center_row: usize,
    pub(super) background: BackgroundTile,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct GeneratedWallTile {
    pub(super) col: usize,
    pub(super) row: usize,
    pub(super) cols: usize,
    pub(super) rows: usize,
    pub(super) orientation: GeneratedWallOrientation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum GeneratedWallOrientation {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct GeneratedWallPreviewTile {
    pub(super) col: usize,
    pub(super) row: usize,
    pub(super) tile: AtlasTile,
}

pub(super) struct GeneratedVisualWorld {
    pub(super) tiles: TileWorld,
    pub(super) visible: Vec<bool>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct GeneratedWorld {
    pub(super) cols: usize,
    pub(super) rows: usize,
    pub(super) cells: Vec<GeneratedCell>,
    pub(super) platforms: Vec<GeneratedPlatform>,
    pub(super) terrain_crosses: Vec<GeneratedTerrainCross>,
    pub(super) wall_tiles: Vec<GeneratedWallTile>,
}

impl GeneratedWorld {
    fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols,
            rows,
            cells: vec![GeneratedCell::Unknown; cols.saturating_mul(rows)],
            platforms: Vec::new(),
            terrain_crosses: Vec::new(),
            wall_tiles: Vec::new(),
        }
    }

    pub(super) fn cell(&self, col: usize, row: usize) -> GeneratedCell {
        self.cells[self.index(col, row)]
    }

    pub(super) fn width_px(&self) -> f32 {
        self.cols as f32 * TILE_SIZE
    }

    pub(super) fn height_px(&self) -> f32 {
        self.rows as f32 * TILE_SIZE
    }

    pub(super) fn to_tile_world(&self) -> TileWorld {
        let mut world = TileWorld::new(self.cols, self.rows);
        for row in 0..self.rows {
            for col in 0..self.cols {
                let background = match self.cell(col, row) {
                    GeneratedCell::Unknown | GeneratedCell::Grass => BackgroundTile::Grass,
                    GeneratedCell::Water => BackgroundTile::Water,
                };
                world.set_background(col, row, background);
            }
        }
        world
    }

    pub(super) fn to_visual_tile_world(&self) -> GeneratedVisualWorld {
        self.to_visual_tile_world_with_shoreline(true)
    }

    fn to_visual_tile_world_with_shoreline(&self, replace_shoreline: bool) -> GeneratedVisualWorld {
        let mut tiles = self.to_tile_world();
        apply_generated_platforms(&mut tiles, self);
        apply_generated_terrain_crosses(&mut tiles, self);
        apply_generated_wall_tiles(&mut tiles, self);
        if replace_shoreline {
            let pre_shoreline_backgrounds = tiles.backgrounds.clone();
            apply_level0_shoreline_replace(&mut tiles);
            apply_platform_cliffline_replace(&mut tiles, &pre_shoreline_backgrounds);
            apply_generated_asset_pass(&mut tiles);
        }

        let platform_seed_mask = platform_seed_mask(self);
        let platform_seed_tiles = platform_seed_mask
            .iter()
            .map(|background| background.is_some())
            .collect::<Vec<_>>();
        let mut visible = vec![false; self.cells.len()];
        let has_unknown = self
            .cells
            .iter()
            .any(|&cell| cell == GeneratedCell::Unknown);
        let platform_inner_preview_tiles = if has_unknown {
            platform_inner_preview_mask(self, &platform_seed_mask)
        } else {
            vec![false; self.cells.len()]
        };
        for (index, cell) in self.cells.iter().enumerate() {
            visible[index] = if has_unknown && platform_seed_tiles[index] {
                tiles.foregrounds[index].is_some() || tiles.under_foregrounds[index].is_some()
            } else if has_unknown && platform_inner_preview_tiles[index] {
                true
            } else {
                *cell != GeneratedCell::Unknown
                    || platform_seed_tiles[index]
                    || tiles.foregrounds[index].is_some()
                    || tiles.under_foregrounds[index].is_some()
            };
        }

        GeneratedVisualWorld { tiles, visible }
    }

    fn visual_void_fill_mask(&self) -> Vec<bool> {
        let visual_world = self.to_visual_tile_world_with_shoreline(false);
        self.cells
            .iter()
            .enumerate()
            .map(|(index, &cell)| {
                cell == GeneratedCell::Unknown
                    && !visual_world.visible[index]
                    && visual_world.tiles.foregrounds[index].is_none()
                    && visual_world.tiles.under_foregrounds[index].is_none()
            })
            .collect()
    }

    pub(super) fn feature_reroll_mask_for_rect(
        &self,
        rect_col: usize,
        rect_row: usize,
        rect_cols: usize,
        rect_rows: usize,
    ) -> Vec<bool> {
        let mut mask = rect_mask(self, rect_col, rect_row, rect_cols, rect_rows);
        let mut changed = true;

        while changed {
            changed = false;

            for &platform in &self.platforms {
                if !platform_footprint_intersects_mask(self, platform, &mask) {
                    continue;
                }
                changed |= mark_platform_footprint(self, &mut mask, platform);
            }

            for &cross in &self.terrain_crosses {
                let cells = terrain_cross_cells(cross);
                if !cells.iter().any(|&(col, row)| {
                    col < self.cols && row < self.rows && mask[self.index(col, row)]
                }) {
                    continue;
                }
                for (col, row) in cells {
                    if col < self.cols && row < self.rows {
                        let index = self.index(col, row);
                        changed |= !mask[index];
                        mask[index] = true;
                    }
                }
            }
        }

        mask
    }

    pub(super) fn reroll_features_in_mask(&mut self, reroll_mask: &[bool], seed: u64) {
        let mut rng = SeededRng::new(seed);
        let touched_platforms = self
            .platforms
            .iter()
            .copied()
            .filter(|&platform| platform_footprint_intersects_mask(self, platform, reroll_mask))
            .collect::<Vec<_>>();
        let touched_crosses = self
            .terrain_crosses
            .iter()
            .copied()
            .filter(|&cross| terrain_cross_intersects_mask(self, cross, reroll_mask))
            .collect::<Vec<_>>();

        self.platforms.retain(|&platform| {
            !platform_footprint_intersects_mask_for_dims(
                self.cols,
                self.rows,
                platform,
                reroll_mask,
            )
        });
        self.terrain_crosses.retain(|&cross| {
            !terrain_cross_intersects_mask_for_dims(self.cols, self.rows, cross, reroll_mask)
        });

        for (index, reroll) in reroll_mask.iter().copied().enumerate() {
            if reroll {
                self.cells[index] = GeneratedCell::Unknown;
            }
        }

        seed_reroll_platforms(self, reroll_mask, &touched_platforms, &mut rng);
        seed_reroll_terrain_crosses(self, reroll_mask, &touched_crosses, &mut rng);
        let cols = self.cols;
        let rows = self.rows;
        self.wall_tiles.retain(|wall| {
            wall.row < rows
                && wall.col.saturating_add(wall.cols) <= cols
                && wall.row.saturating_add(wall.rows) <= rows
                && !generated_wall_cells(wall).any(|(col, row)| reroll_mask[row * cols + col])
        });
    }

    pub(super) fn can_add_horizontal_wall_centered(
        &self,
        col: usize,
        row: usize,
        cols: usize,
    ) -> bool {
        let Some(start_col) = horizontal_wall_start_col(self.cols, col, cols) else {
            return false;
        };
        if row >= self.rows {
            return false;
        }
        let platform_mask = platform_rect_mask(self);
        if !generated_wall_run_has_same_level(&platform_mask, self, start_col, row, cols) {
            return false;
        }

        let mut tiles = self.to_tile_world();
        apply_generated_platforms(&mut tiles, self);
        apply_generated_terrain_crosses(&mut tiles, self);
        apply_generated_wall_tiles(&mut tiles, self);
        generated_wall_run_has_pillar_clearance(&tiles, start_col, row, cols)
    }

    pub(super) fn add_horizontal_wall_centered(
        &mut self,
        col: usize,
        row: usize,
        cols: usize,
    ) -> bool {
        if !self.can_add_horizontal_wall_centered(col, row, cols) {
            return false;
        }
        let start_col = horizontal_wall_start_col(self.cols, col, cols).unwrap();

        let wall = GeneratedWallTile {
            col: start_col,
            row,
            cols,
            rows: 1,
            orientation: GeneratedWallOrientation::Horizontal,
        };
        if !self.wall_tiles.contains(&wall) {
            self.wall_tiles.push(wall);
        }
        true
    }

    #[cfg(test)]
    pub(super) fn add_wall_run3_centered(&mut self, col: usize, row: usize) -> bool {
        self.add_horizontal_wall_centered(col, row, 3)
    }

    pub(super) fn can_add_vertical_wall_centered(
        &self,
        col: usize,
        row: usize,
        cliff_rows: usize,
    ) -> bool {
        let Some(start_row) = vertical_wall_start_row(self.rows, row, cliff_rows) else {
            return false;
        };
        if col >= self.cols {
            return false;
        }
        let rows = cliff_rows + 1;
        let platform_mask = platform_rect_mask(self);
        if !generated_vertical_wall_run_has_same_level(&platform_mask, self, col, start_row, rows) {
            return false;
        }

        let mut tiles = self.to_tile_world();
        apply_generated_platforms(&mut tiles, self);
        apply_generated_terrain_crosses(&mut tiles, self);
        apply_generated_wall_tiles(&mut tiles, self);
        generated_vertical_wall_run_has_pillar_clearance(&tiles, col, start_row, rows)
    }

    pub(super) fn add_vertical_wall_centered(
        &mut self,
        col: usize,
        row: usize,
        cliff_rows: usize,
    ) -> bool {
        if !self.can_add_vertical_wall_centered(col, row, cliff_rows) {
            return false;
        }
        let start_row = vertical_wall_start_row(self.rows, row, cliff_rows).unwrap();

        let wall = GeneratedWallTile {
            col,
            row: start_row,
            cols: 1,
            rows: cliff_rows + 1,
            orientation: GeneratedWallOrientation::Vertical,
        };
        if !self.wall_tiles.contains(&wall) {
            self.wall_tiles.push(wall);
        }
        true
    }

    #[cfg(test)]
    pub(super) fn add_vertical_wall_run3_centered(&mut self, col: usize, row: usize) -> bool {
        self.add_vertical_wall_centered(col, row, 3)
    }

    pub(super) fn horizontal_wall_preview_tiles_centered(
        &self,
        col: usize,
        row: usize,
        cols: usize,
    ) -> Option<Vec<GeneratedWallPreviewTile>> {
        if !self.can_add_horizontal_wall_centered(col, row, cols) {
            return None;
        }

        let start_col = horizontal_wall_start_col(self.cols, col, cols).unwrap();
        let mut tiles = self.to_tile_world();
        apply_generated_platforms(&mut tiles, self);
        apply_generated_terrain_crosses(&mut tiles, self);
        apply_generated_wall_tiles(&mut tiles, self);
        let platform_mask = platform_rect_mask(self);

        let mut preview_tiles = Vec::with_capacity(cols * 2);
        if row > 0 {
            preview_tiles.extend((0..cols).map(|offset| GeneratedWallPreviewTile {
                col: start_col + offset,
                row: row - 1,
                tile: wall_cliff_cap_tile(offset, cols),
            }));
        }
        preview_tiles.extend((0..cols).map(|offset| {
            let tile_col = start_col + offset;
            let tile_background = wall_cap_tile_background(&tiles, &platform_mask, tile_col, row);
            GeneratedWallPreviewTile {
                col: tile_col,
                row,
                tile: platform_row_tile(
                    tile_background,
                    tile_background,
                    &platform_mask,
                    tiles.cols,
                    row,
                    start_col,
                    tile_col,
                    offset,
                    cols,
                ),
            }
        }));
        Some(preview_tiles)
    }

    pub(super) fn vertical_wall_preview_tiles_centered(
        &self,
        col: usize,
        row: usize,
        cliff_rows: usize,
    ) -> Option<Vec<GeneratedWallPreviewTile>> {
        if !self.can_add_vertical_wall_centered(col, row, cliff_rows) {
            return None;
        }

        let start_row = vertical_wall_start_row(self.rows, row, cliff_rows).unwrap();
        let mut tiles = self.to_tile_world();
        apply_generated_platforms(&mut tiles, self);
        apply_generated_terrain_crosses(&mut tiles, self);
        apply_generated_wall_tiles(&mut tiles, self);
        let platform_mask = platform_rect_mask(self);

        let mut preview_tiles = Vec::with_capacity(cliff_rows + 1);
        preview_tiles.extend((0..cliff_rows).map(|offset| GeneratedWallPreviewTile {
            col,
            row: start_row + offset,
            tile: vertical_cliff_tile(offset, cliff_rows),
        }));
        let pillar_row = start_row + cliff_rows;
        preview_tiles.push({
            let tile_background = wall_cap_tile_background(&tiles, &platform_mask, col, pillar_row);
            GeneratedWallPreviewTile {
                col,
                row: pillar_row,
                tile: standalone_pillar_tile(tile_background),
            }
        });
        Some(preview_tiles)
    }

    #[cfg(test)]
    pub(super) fn wall_run3_preview_tiles_centered(
        &self,
        col: usize,
        row: usize,
    ) -> Option<Vec<GeneratedWallPreviewTile>> {
        self.horizontal_wall_preview_tiles_centered(col, row, 3)
    }

    #[cfg(test)]
    pub(super) fn vertical_wall_run3_preview_tiles_centered(
        &self,
        col: usize,
        row: usize,
    ) -> Option<Vec<GeneratedWallPreviewTile>> {
        self.vertical_wall_preview_tiles_centered(col, row, 3)
    }

    fn index(&self, col: usize, row: usize) -> usize {
        debug_assert!(col < self.cols);
        debug_assert!(row < self.rows);
        row * self.cols + col
    }
}

pub(super) struct RunningGenerator {
    world: GeneratedWorld,
    rng: SeededRng,
    initial_seeds_complete: bool,
}

impl RunningGenerator {
    pub(super) fn new(cols: usize, rows: usize, seed: u64) -> Self {
        let mut rng = SeededRng::new(seed);
        let mut world = GeneratedWorld::new(cols, rows);
        seed_large_platforms(&mut world, &mut rng);
        seed_remaining_platforms(&mut world, &mut rng);
        seed_terrain_crosses(&mut world, &mut rng);
        Self::from_world_with_rng_and_stage(world, rng, false)
    }

    pub(super) fn from_world(world: GeneratedWorld, seed: u64) -> Self {
        Self::from_world_with_rng_and_stage(world, SeededRng::new(seed), true)
    }

    fn from_world_with_rng_and_stage(
        world: GeneratedWorld,
        rng: SeededRng,
        initial_seeds_complete: bool,
    ) -> Self {
        Self {
            world,
            rng,
            initial_seeds_complete,
        }
    }

    pub(super) fn world(&self) -> &GeneratedWorld {
        &self.world
    }

    pub(super) fn add_horizontal_wall_centered(
        &mut self,
        col: usize,
        row: usize,
        cols: usize,
    ) -> bool {
        self.world.add_horizontal_wall_centered(col, row, cols)
    }

    pub(super) fn add_vertical_wall_centered(
        &mut self,
        col: usize,
        row: usize,
        cliff_rows: usize,
    ) -> bool {
        self.world.add_vertical_wall_centered(col, row, cliff_rows)
    }

    pub(super) fn horizontal_wall_preview_tiles_centered(
        &self,
        col: usize,
        row: usize,
        cols: usize,
    ) -> Option<Vec<GeneratedWallPreviewTile>> {
        self.world
            .horizontal_wall_preview_tiles_centered(col, row, cols)
    }

    pub(super) fn vertical_wall_preview_tiles_centered(
        &self,
        col: usize,
        row: usize,
        cliff_rows: usize,
    ) -> Option<Vec<GeneratedWallPreviewTile>> {
        self.world
            .vertical_wall_preview_tiles_centered(col, row, cliff_rows)
    }

    pub(super) fn complete_initial_seeds(&mut self) {
        if self.initial_seeds_complete {
            return;
        }

        seed_fallback_cells_if_needed(&mut self.world, &mut self.rng);
        self.initial_seeds_complete = true;
    }

    pub(super) fn fill_visual_voids_once(&mut self, max_cells: usize) -> usize {
        let fillable = self.world.visual_void_fill_mask();
        let mut buckets = std::array::from_fn(|_| Vec::new());
        let mut in_frontier = vec![false; self.world.cells.len()];

        for index in 0..self.world.cells.len() {
            if self.world.cells[index] != GeneratedCell::Unknown {
                add_fillable_unknown_neighbors_to_frontier(
                    &self.world,
                    &fillable,
                    index,
                    &mut buckets,
                    &mut in_frontier,
                );
            }
        }

        let mut collapsed = 0;
        while collapsed < max_cells {
            let Some(index) = next_fillable_frontier_cell(
                &self.world,
                &fillable,
                &mut buckets,
                &mut in_frontier,
                &mut self.rng,
            )
            .or_else(|| {
                fillable
                    .iter()
                    .enumerate()
                    .find(|&(index, &fillable)| {
                        fillable && self.world.cells[index] == GeneratedCell::Unknown
                    })
                    .map(|(index, _)| index)
            }) else {
                break;
            };

            self.world.cells[index] = affinity_fitting_cell(&self.world, index, &mut self.rng);
            collapsed += 1;
            add_fillable_unknown_neighbors_to_frontier(
                &self.world,
                &fillable,
                index,
                &mut buckets,
                &mut in_frontier,
            );
        }

        collapsed
    }
}

fn seed_fallback_cells_if_needed(world: &mut GeneratedWorld, rng: &mut SeededRng) {
    let cell_count = world.cells.len();
    if cell_count == 0 || !world.platforms.is_empty() {
        return;
    }

    let seed_count = cell_count
        .saturating_mul(INITIAL_SEED_NUMERATOR)
        .div_ceil(INITIAL_SEED_DENOMINATOR)
        .max(1);
    let mut indices = (0..cell_count).collect::<Vec<_>>();
    for offset in 0..seed_count {
        let chosen = rng.range_usize(offset, cell_count);
        indices.swap(offset, chosen);
        if world.cells[indices[offset]] != GeneratedCell::Unknown {
            continue;
        }
        world.cells[indices[offset]] = if rng
            .range_usize(0, INITIAL_WATER_WEIGHT + INITIAL_GRASS_WEIGHT)
            < INITIAL_WATER_WEIGHT
        {
            GeneratedCell::Water
        } else {
            GeneratedCell::Grass
        };
    }
}

fn seed_large_platforms(world: &mut GeneratedWorld, rng: &mut SeededRng) {
    if world.cols < LARGE_PLATFORM_BASE_SIZE || world.rows < LARGE_PLATFORM_BASE_SIZE {
        return;
    }

    for platform_index in 0..LARGE_PLATFORM_COUNT {
        let cols = large_platform_extent(world.cols, rng);
        let rows = large_platform_extent(world.rows, rng);
        let background = if platform_index % 2 == 0 {
            BackgroundTile::Water
        } else {
            BackgroundTile::Grass
        };

        let Some(platform) = place_platform(
            world,
            cols,
            rows,
            background,
            GeneratedPlatformKind::Large,
            rng,
        ) else {
            continue;
        };
        world.platforms.push(platform);
    }
}

fn seed_remaining_platforms(world: &mut GeneratedWorld, rng: &mut SeededRng) {
    if world.cols < SMALL_PLATFORM_MIN_SIZE || world.rows < SMALL_PLATFORM_MIN_SIZE {
        stamp_platform_backgrounds(world);
        return;
    }

    let small_count = rng.range_usize(SMALL_PLATFORM_MIN_COUNT, SMALL_PLATFORM_MAX_COUNT + 1);
    let mut placed_small = 0;
    for anchor in world
        .platforms
        .iter()
        .copied()
        .filter(|platform| platform.kind == GeneratedPlatformKind::Large)
        .collect::<Vec<_>>()
    {
        if place_attached_small_platform(world, anchor, rng) {
            placed_small += 1;
        }
    }

    while placed_small < small_count {
        let cols = small_platform_extent(world.cols, rng);
        let rows = small_platform_extent(world.rows, rng);
        let background = random_background(rng);
        let Some(platform) = place_platform(
            world,
            cols,
            rows,
            background,
            GeneratedPlatformKind::Small,
            rng,
        ) else {
            break;
        };
        world.platforms.push(platform);
        placed_small += 1;
    }

    stamp_platform_backgrounds(world);
}

fn seed_terrain_crosses(world: &mut GeneratedWorld, rng: &mut SeededRng) {
    if world.cols < 3 || world.rows < 3 {
        return;
    }

    let mut occupied = platform_frame_buffer_mask(world);
    for center_row in 1..world.rows - 1 {
        for col in 1..world.cols - 1 {
            if rng.range_usize(0, TERRAIN_CROSS_CHANCE) != 0 {
                continue;
            }
            let background = random_background(rng);
            let cross = GeneratedTerrainCross {
                col,
                center_row,
                background,
            };
            if terrain_cross_fits(world, &occupied, cross) {
                for (cell_col, cell_row) in terrain_cross_cells(cross) {
                    occupied[world.index(cell_col, cell_row)] = true;
                }
                stamp_terrain_cross_background(world, cross);
                world.terrain_crosses.push(cross);
            }
        }
    }
}

fn seed_reroll_platforms(
    world: &mut GeneratedWorld,
    reroll_mask: &[bool],
    touched_platforms: &[GeneratedPlatform],
    rng: &mut SeededRng,
) {
    let reroll_cells = reroll_mask.iter().filter(|&&reroll| reroll).count();
    let full_cells = world.cells.len().max(1);
    let random_large_count =
        scaled_count_with_fraction(LARGE_PLATFORM_COUNT, reroll_cells, full_cells, rng);
    let random_small_count = scaled_count_with_fraction(
        rng.range_usize(SMALL_PLATFORM_MIN_COUNT, SMALL_PLATFORM_MAX_COUNT + 1),
        reroll_cells,
        full_cells,
        rng,
    );
    let touched_large_count = touched_platforms
        .iter()
        .filter(|platform| platform.kind == GeneratedPlatformKind::Large)
        .count();
    let touched_small_count = touched_platforms
        .iter()
        .filter(|platform| platform.kind == GeneratedPlatformKind::Small)
        .count();

    for &platform in touched_platforms {
        let replacement = GeneratedPlatform {
            background: random_background(rng),
            ..platform
        };
        place_reroll_platform(world, reroll_mask, replacement, rng);
    }

    for platform_index in 0..random_large_count.saturating_sub(touched_large_count) {
        let cols = large_platform_extent(world.cols, rng);
        let rows = large_platform_extent(world.rows, rng);
        let background = if platform_index % 2 == 0 {
            BackgroundTile::Water
        } else {
            BackgroundTile::Grass
        };
        place_reroll_platform_with_size(
            world,
            reroll_mask,
            cols,
            rows,
            background,
            GeneratedPlatformKind::Large,
            rng,
        );
    }

    for _ in 0..random_small_count.saturating_sub(touched_small_count) {
        place_reroll_platform_with_size(
            world,
            reroll_mask,
            small_platform_extent(world.cols, rng),
            small_platform_extent(world.rows, rng),
            random_background(rng),
            GeneratedPlatformKind::Small,
            rng,
        );
    }

    stamp_platform_backgrounds(world);
}

fn place_reroll_platform_with_size(
    world: &mut GeneratedWorld,
    reroll_mask: &[bool],
    cols: usize,
    rows: usize,
    background: BackgroundTile,
    kind: GeneratedPlatformKind,
    rng: &mut SeededRng,
) -> bool {
    for _ in 0..PLATFORM_PLACEMENT_ATTEMPTS {
        if cols == 0 || rows == 0 || cols > world.cols || rows > world.rows {
            return false;
        }
        let platform = GeneratedPlatform {
            col: rng.range_usize(0, world.cols - cols + 1),
            row: rng.range_usize(0, world.rows - rows + 1),
            cols,
            rows,
            background,
            kind,
        };
        if place_reroll_platform(world, reroll_mask, platform, rng) {
            return true;
        }
    }

    false
}

fn place_reroll_platform(
    world: &mut GeneratedWorld,
    reroll_mask: &[bool],
    platform: GeneratedPlatform,
    rng: &mut SeededRng,
) -> bool {
    if platform.cols == 0
        || platform.rows == 0
        || platform.cols > world.cols
        || platform.rows > world.rows
    {
        return false;
    }

    let mut candidate = platform;
    for attempt in 0..PLATFORM_PLACEMENT_ATTEMPTS {
        if attempt + 1 < PLATFORM_PLACEMENT_ATTEMPTS {
            candidate.col = rng.range_usize(0, world.cols.saturating_sub(platform.cols) + 1);
            candidate.row = rng.range_usize(0, world.rows.saturating_sub(platform.rows) + 1);
        } else {
            candidate = platform;
        }
        if platform_fits_reroll_mask(world, reroll_mask, candidate)
            && world
                .platforms
                .iter()
                .all(|&existing| !platforms_overlap_with_margin(existing, candidate, 1))
        {
            world.platforms.push(candidate);
            return true;
        }
    }

    false
}

fn seed_reroll_terrain_crosses(
    world: &mut GeneratedWorld,
    reroll_mask: &[bool],
    touched_crosses: &[GeneratedTerrainCross],
    rng: &mut SeededRng,
) {
    let reroll_cells = reroll_mask.iter().filter(|&&reroll| reroll).count();
    let random_cross_count = scaled_count_with_fraction(reroll_cells, 1, TERRAIN_CROSS_CHANCE, rng);
    let target_count = touched_crosses.len().max(random_cross_count);
    let mut occupied = platform_frame_buffer_mask(world);
    for &cross in &world.terrain_crosses {
        for (col, row) in terrain_cross_cells(cross) {
            if col < world.cols && row < world.rows {
                occupied[world.index(col, row)] = true;
            }
        }
    }

    let mut placed = 0;
    for &cross in touched_crosses {
        let replacement = GeneratedTerrainCross {
            background: random_background(rng),
            ..cross
        };
        if terrain_cross_fits_mask(world, reroll_mask, replacement)
            && terrain_cross_fits(world, &occupied, replacement)
        {
            for (cell_col, cell_row) in terrain_cross_cells(replacement) {
                occupied[world.index(cell_col, cell_row)] = true;
            }
            stamp_terrain_cross_background(world, replacement);
            world.terrain_crosses.push(replacement);
            placed += 1;
        }
    }

    let attempts = target_count
        .saturating_mul(PLATFORM_PLACEMENT_ATTEMPTS)
        .max(PLATFORM_PLACEMENT_ATTEMPTS);
    for _ in 0..attempts {
        if placed >= target_count || world.cols < 3 || world.rows < 3 {
            break;
        }
        let cross = GeneratedTerrainCross {
            col: rng.range_usize(1, world.cols - 1),
            center_row: rng.range_usize(1, world.rows - 1),
            background: random_background(rng),
        };
        if !terrain_cross_fits_mask(world, reroll_mask, cross) {
            continue;
        }
        if terrain_cross_fits(world, &occupied, cross) {
            for (cell_col, cell_row) in terrain_cross_cells(cross) {
                occupied[world.index(cell_col, cell_row)] = true;
            }
            stamp_terrain_cross_background(world, cross);
            world.terrain_crosses.push(cross);
            placed += 1;
        }
    }
}

fn scaled_count_with_fraction(
    base_count: usize,
    numerator: usize,
    denominator: usize,
    rng: &mut SeededRng,
) -> usize {
    if denominator == 0 {
        return 0;
    }
    let scaled = base_count.saturating_mul(numerator);
    let mut count = scaled / denominator;
    if rng.range_usize(0, denominator) < scaled % denominator {
        count += 1;
    }
    count
}

fn rect_mask(
    world: &GeneratedWorld,
    rect_col: usize,
    rect_row: usize,
    rect_cols: usize,
    rect_rows: usize,
) -> Vec<bool> {
    let mut mask = vec![false; world.cells.len()];
    let end_col = rect_col.saturating_add(rect_cols).min(world.cols);
    let end_row = rect_row.saturating_add(rect_rows).min(world.rows);
    for row in rect_row..end_row {
        for col in rect_col..end_col {
            mask[world.index(col, row)] = true;
        }
    }
    mask
}

fn platform_fits_reroll_mask(
    world: &GeneratedWorld,
    reroll_mask: &[bool],
    platform: GeneratedPlatform,
) -> bool {
    platform_footprint_cells(world.cols, world.rows, platform)
        .into_iter()
        .all(|(col, row)| {
            col < world.cols && row < world.rows && reroll_mask[world.index(col, row)]
        })
}

fn platform_footprint_intersects_mask(
    world: &GeneratedWorld,
    platform: GeneratedPlatform,
    mask: &[bool],
) -> bool {
    platform_footprint_intersects_mask_for_dims(world.cols, world.rows, platform, mask)
}

fn platform_footprint_intersects_mask_for_dims(
    cols: usize,
    rows: usize,
    platform: GeneratedPlatform,
    mask: &[bool],
) -> bool {
    platform_footprint_cells(cols, rows, platform)
        .into_iter()
        .any(|(col, row)| col < cols && row < rows && mask[row * cols + col])
}

fn terrain_cross_intersects_mask(
    world: &GeneratedWorld,
    cross: GeneratedTerrainCross,
    mask: &[bool],
) -> bool {
    terrain_cross_intersects_mask_for_dims(world.cols, world.rows, cross, mask)
}

fn terrain_cross_intersects_mask_for_dims(
    cols: usize,
    rows: usize,
    cross: GeneratedTerrainCross,
    mask: &[bool],
) -> bool {
    terrain_cross_cells(cross)
        .into_iter()
        .any(|(col, row)| col < cols && row < rows && mask[row * cols + col])
}

fn mark_platform_footprint(
    world: &GeneratedWorld,
    mask: &mut [bool],
    platform: GeneratedPlatform,
) -> bool {
    let mut changed = false;
    for (col, row) in platform_footprint_cells(world.cols, world.rows, platform) {
        if col >= world.cols || row >= world.rows {
            continue;
        }
        let index = world.index(col, row);
        changed |= !mask[index];
        mask[index] = true;
    }
    changed
}

fn platform_footprint_cells(
    world_cols: usize,
    world_rows: usize,
    platform: GeneratedPlatform,
) -> Vec<(usize, usize)> {
    let mut cells = Vec::new();
    let start_col = platform.col.min(world_cols);
    let start_row = platform.row.min(world_rows);
    let end_col = platform.col.saturating_add(platform.cols).min(world_cols);
    let end_row = platform.row.saturating_add(platform.rows).min(world_rows);
    for row in start_row..end_row {
        for col in start_col..end_col {
            cells.push((col, row));
        }
    }

    let pillar_row = platform.row.saturating_add(platform.rows);
    if pillar_row < world_rows {
        for col in start_col..end_col {
            cells.push((col, pillar_row));
        }
    }

    cells
}

fn terrain_cross_fits_mask(
    world: &GeneratedWorld,
    reroll_mask: &[bool],
    cross: GeneratedTerrainCross,
) -> bool {
    terrain_cross_cells(cross).into_iter().all(|(col, row)| {
        col < world.cols && row < world.rows && reroll_mask[world.index(col, row)]
    })
}

fn platform_frame_buffer_mask(world: &GeneratedWorld) -> Vec<bool> {
    let mut mask = vec![false; world.cells.len()];
    for platform in &world.platforms {
        let start_col = platform.col.min(world.cols);
        let end_col = platform.col.saturating_add(platform.cols).min(world.cols);
        let start_row = platform.row.min(world.rows);
        let end_row = platform.row.saturating_add(platform.rows).min(world.rows);
        if start_col >= end_col || start_row >= end_row {
            continue;
        };
        let frame_bottom_row = end_row - 1;
        let pillar_row = end_row;

        for row in start_row..end_row {
            for col in start_col..end_col {
                let on_platform_frame = row == start_row
                    || row == frame_bottom_row
                    || col == start_col
                    || col + 1 == end_col;
                if on_platform_frame {
                    mark_platform_frame_buffer_cell(world, &mut mask, col, row);
                }
            }
        }

        if pillar_row < world.rows {
            for col in start_col..end_col {
                mark_platform_frame_buffer_cell(world, &mut mask, col, pillar_row);
            }
        }
    }
    mask
}

fn mark_platform_frame_buffer_cell(
    world: &GeneratedWorld,
    mask: &mut [bool],
    col: usize,
    row: usize,
) {
    let start_col = col.saturating_sub(1);
    let start_row = row.saturating_sub(1);
    let end_col = (col + 1).min(world.cols - 1);
    let end_row = (row + 1).min(world.rows - 1);

    for mark_row in start_row..=end_row {
        for mark_col in start_col..=end_col {
            let index = world.index(mark_col, mark_row);
            mask[index] = true;
        }
    }
}

fn large_platform_extent(limit: usize, rng: &mut SeededRng) -> usize {
    let extra = rng.range_usize(LARGE_PLATFORM_EXTRA_MIN, LARGE_PLATFORM_EXTRA_MAX + 1);
    (LARGE_PLATFORM_BASE_SIZE + extra).min(limit)
}

fn small_platform_extent(limit: usize, rng: &mut SeededRng) -> usize {
    rng.range_usize(
        SMALL_PLATFORM_MIN_SIZE.min(limit),
        SMALL_PLATFORM_MAX_SIZE.min(limit) + 1,
    )
}

fn random_background(rng: &mut SeededRng) -> BackgroundTile {
    if rng.range_usize(0, 2) == 0 {
        BackgroundTile::Water
    } else {
        BackgroundTile::Grass
    }
}

fn place_platform(
    world: &GeneratedWorld,
    cols: usize,
    rows: usize,
    background: BackgroundTile,
    kind: GeneratedPlatformKind,
    rng: &mut SeededRng,
) -> Option<GeneratedPlatform> {
    for _ in 0..PLATFORM_PLACEMENT_ATTEMPTS {
        let col = rng.range_usize(0, world.cols.saturating_sub(cols) + 1);
        let row = rng.range_usize(0, world.rows.saturating_sub(rows) + 1);
        let platform = GeneratedPlatform {
            col,
            row,
            cols,
            rows,
            background,
            kind,
        };
        if world
            .platforms
            .iter()
            .all(|&existing| !platforms_overlap_with_margin(existing, platform, 1))
        {
            return Some(platform);
        }
    }

    None
}

fn place_attached_small_platform(
    world: &mut GeneratedWorld,
    anchor: GeneratedPlatform,
    rng: &mut SeededRng,
) -> bool {
    for _ in 0..PLATFORM_PLACEMENT_ATTEMPTS {
        let side = rng.range_usize(0, 4);
        let (cols, rows) = attached_small_platform_extent(world, side, rng);
        let Some((col, row)) = attached_platform_origin(world, anchor, cols, rows, side, rng)
        else {
            continue;
        };
        let platform = GeneratedPlatform {
            col,
            row,
            cols,
            rows,
            background: anchor.background,
            kind: GeneratedPlatformKind::Small,
        };

        if platform_fits_attached_to(world, platform, anchor) {
            world.platforms.push(platform);
            return true;
        }
    }

    false
}

fn attached_small_platform_extent(
    world: &GeneratedWorld,
    side: usize,
    rng: &mut SeededRng,
) -> (usize, usize) {
    let span = small_platform_extent(world.cols.min(world.rows), rng);
    let overlap_axis = 4;
    match side {
        0 | 2 => (span.min(world.cols), overlap_axis.min(world.rows)),
        _ => (overlap_axis.min(world.cols), span.min(world.rows)),
    }
}

fn attached_platform_origin(
    world: &GeneratedWorld,
    anchor: GeneratedPlatform,
    cols: usize,
    rows: usize,
    side: usize,
    rng: &mut SeededRng,
) -> Option<(usize, usize)> {
    match side {
        0 => {
            let row = origin_centered_on_line(anchor.row, rows, world.rows)?;
            let col = origin_with_center_in_span(anchor.col, anchor.cols, cols, world.cols, rng)?;
            Some((col, row))
        }
        1 => {
            let col = origin_centered_on_line(anchor.col + anchor.cols, cols, world.cols)?;
            let row = origin_with_center_in_span(anchor.row, anchor.rows, rows, world.rows, rng)?;
            Some((col, row))
        }
        2 => {
            let row = origin_centered_on_line(anchor.row + anchor.rows, rows, world.rows)?;
            let col = origin_with_center_in_span(anchor.col, anchor.cols, cols, world.cols, rng)?;
            Some((col, row))
        }
        3 => {
            let col = origin_centered_on_line(anchor.col, cols, world.cols)?;
            let row = origin_with_center_in_span(anchor.row, anchor.rows, rows, world.rows, rng)?;
            Some((col, row))
        }
        _ => None,
    }
}

fn origin_centered_on_line(line: usize, len: usize, world_limit: usize) -> Option<usize> {
    if len % 2 != 0 {
        return None;
    }

    let origin = line.checked_sub(len / 2)?;
    (origin + len <= world_limit).then_some(origin)
}

fn origin_with_center_in_span(
    anchor_start: usize,
    anchor_len: usize,
    len: usize,
    world_limit: usize,
    rng: &mut SeededRng,
) -> Option<usize> {
    let low = anchor_start.saturating_mul(2).max(len);
    let high = (anchor_start + anchor_len)
        .saturating_mul(2)
        .min(world_limit.saturating_mul(2).saturating_sub(len));
    let parity = len % 2;
    let first = if low % 2 == parity { low } else { low + 1 };
    if first > high {
        return None;
    }

    let center = first + rng.range_usize(0, (high - first) / 2 + 1) * 2;
    Some((center - len) / 2)
}

fn platform_fits_attached_to(
    world: &GeneratedWorld,
    platform: GeneratedPlatform,
    anchor: GeneratedPlatform,
) -> bool {
    world.platforms.iter().all(|&existing| {
        if existing == anchor {
            platforms_overlap(existing, platform)
                && platform_center_on_border_of(existing, platform)
        } else {
            !platforms_overlap(existing, platform)
                && !platforms_overlap_with_margin(existing, platform, 1)
        }
    })
}

fn stamp_platform_backgrounds(world: &mut GeneratedWorld) {
    let rect_mask = platform_rect_mask(world);
    let mask = platform_seed_mask(world);
    for row in 0..world.rows {
        for col in 0..world.cols {
            let Some(background) = mask[world.index(col, row)] else {
                continue;
            };
            let index = world.index(col, row);
            let seeded_background = if rect_mask[index].is_some() {
                BackgroundTile::Grass
            } else {
                background
            };
            world.cells[index] = match seeded_background {
                BackgroundTile::Grass => GeneratedCell::Grass,
                BackgroundTile::Water => GeneratedCell::Water,
            };
        }
    }
}

fn platform_seed_mask(world: &GeneratedWorld) -> Vec<Option<BackgroundTile>> {
    let platform_mask = platform_rect_mask(world);
    let mut seed_mask = vec![None; world.cells.len()];

    for row in 0..world.rows {
        for col in 0..world.cols {
            let index = world.index(col, row);
            let Some(background) = platform_mask[index] else {
                continue;
            };
            let open_s = row + 1 >= world.rows
                || platform_mask[world.index(col, row + 1)] != Some(background);
            if platform_mask_open_to_edge(&platform_mask, world, col, row, background) {
                seed_mask[index] = Some(background);
            }
            if open_s && row + 1 < world.rows && platform_mask[world.index(col, row + 1)].is_none()
            {
                seed_mask[world.index(col, row + 1)] = Some(background);
            }
        }
    }
    seed_mask
}

fn platform_inner_preview_mask(
    world: &GeneratedWorld,
    seed_mask: &[Option<BackgroundTile>],
) -> Vec<bool> {
    let platform_mask = platform_rect_mask(world);
    let large_mask = platform_kind_rect_mask(world, GeneratedPlatformKind::Large);
    let small_mask = platform_kind_rect_mask(world, GeneratedPlatformKind::Small);
    let mut inner_mask = vec![false; world.cells.len()];

    for row in 0..world.rows {
        for col in 0..world.cols {
            let index = world.index(col, row);
            if platform_mask[index].is_none()
                || seed_mask[index].is_some()
                || !large_mask[index]
                || !small_mask[index]
            {
                continue;
            }

            let seeded_frame_neighbors = [(0, -1), (1, 0), (0, 1), (-1, 0)]
                .iter()
                .filter(|&&(dc, dr)| {
                    let next_col = col as isize + dc;
                    let next_row = row as isize + dr;
                    if next_col < 0
                        || next_row < 0
                        || next_col >= world.cols as isize
                        || next_row >= world.rows as isize
                    {
                        return false;
                    }

                    let next_index = world.index(next_col as usize, next_row as usize);
                    platform_mask[next_index].is_some() && seed_mask[next_index].is_some()
                })
                .count();
            if seeded_frame_neighbors >= 2 {
                inner_mask[index] = true;
            }
        }
    }

    inner_mask
}

fn platform_kind_rect_mask(world: &GeneratedWorld, kind: GeneratedPlatformKind) -> Vec<bool> {
    let mut mask = vec![false; world.cells.len()];
    for platform in world
        .platforms
        .iter()
        .filter(|platform| platform.kind == kind)
    {
        let start_col = platform.col.min(world.cols);
        let start_row = platform.row.min(world.rows);
        let end_col = platform.col.saturating_add(platform.cols).min(world.cols);
        let end_row = platform.row.saturating_add(platform.rows).min(world.rows);
        for row in start_row..end_row {
            for col in start_col..end_col {
                mask[world.index(col, row)] = true;
            }
        }
    }
    mask
}

fn platform_rect_mask(world: &GeneratedWorld) -> Vec<Option<BackgroundTile>> {
    let mut mask = vec![None; world.cells.len()];
    for platform in &world.platforms {
        let start_col = platform.col.min(world.cols);
        let start_row = platform.row.min(world.rows);
        let end_col = platform.col.saturating_add(platform.cols).min(world.cols);
        let end_row = platform.row.saturating_add(platform.rows).min(world.rows);
        for row in start_row..end_row {
            for col in start_col..end_col {
                mask[world.index(col, row)] = Some(platform.background);
            }
        }
    }
    mask
}

fn platform_mask_open_to_edge(
    mask: &[Option<BackgroundTile>],
    world: &GeneratedWorld,
    col: usize,
    row: usize,
    background: BackgroundTile,
) -> bool {
    row == 0
        || mask[world.index(col, row - 1)] != Some(background)
        || col + 1 >= world.cols
        || mask[world.index(col + 1, row)] != Some(background)
        || row + 1 >= world.rows
        || mask[world.index(col, row + 1)] != Some(background)
        || col == 0
        || mask[world.index(col - 1, row)] != Some(background)
}

fn terrain_cross_fits(
    world: &GeneratedWorld,
    occupied: &[bool],
    cross: GeneratedTerrainCross,
) -> bool {
    terrain_cross_cells(cross)
        .iter()
        .all(|&(col, row)| col < world.cols && row < world.rows && !occupied[world.index(col, row)])
}

fn stamp_terrain_cross_background(world: &mut GeneratedWorld, cross: GeneratedTerrainCross) {
    let background_cell = match cross.background {
        BackgroundTile::Grass => GeneratedCell::Grass,
        BackgroundTile::Water => GeneratedCell::Water,
    };

    for (col, row) in terrain_cross_cells(cross) {
        let index = world.index(col, row);
        world.cells[index] = background_cell;
    }
}

fn terrain_cross_cells(cross: GeneratedTerrainCross) -> [(usize, usize); 5] {
    [
        (cross.col, cross.center_row - 1),
        (cross.col - 1, cross.center_row),
        (cross.col, cross.center_row),
        (cross.col + 1, cross.center_row),
        (cross.col, cross.center_row + 1),
    ]
}

fn apply_generated_platforms(world: &mut TileWorld, generated: &GeneratedWorld) {
    apply_platform_rects_with_interior_fill(
        world,
        generated.platforms.iter().map(|platform| {
            (
                platform.col,
                platform.row,
                platform.cols,
                platform.rows,
                platform.background,
            )
        }),
        false,
    );
}

fn apply_generated_terrain_crosses(world: &mut TileWorld, generated: &GeneratedWorld) {
    for cross in &generated.terrain_crosses {
        if cross.col >= world.cols || cross.center_row >= world.rows {
            continue;
        }
        apply_generated_terrain_cross_background(world, *cross);
        if cross.center_row > 0 {
            let cap_index = world.index(cross.col, cross.center_row - 1);
            world.foregrounds[cap_index] = Some(CLIFF_GRASS_CAP_TILE);
        }
        let index = world.index(cross.col, cross.center_row);
        world.foregrounds[index] = Some(standalone_pillar_tile(cross.background));
    }
}

fn apply_generated_terrain_cross_background(world: &mut TileWorld, cross: GeneratedTerrainCross) {
    for (col, row) in terrain_cross_cells(cross) {
        if col >= world.cols || row >= world.rows {
            continue;
        }
        let index = world.index(col, row);
        world.set_background(col, row, cross.background);
        world.foregrounds[index] = None;
    }
}

fn apply_generated_wall_tiles(world: &mut TileWorld, generated: &GeneratedWorld) {
    let platform_mask = platform_rect_mask(generated);
    for wall in &generated.wall_tiles {
        match wall.orientation {
            GeneratedWallOrientation::Horizontal => {
                apply_wall_cliff_cap_run(world, wall.row, wall.col, wall.cols);
                apply_wall_cap_run(world, &platform_mask, wall.row, wall.col, wall.cols);
            }
            GeneratedWallOrientation::Vertical => {
                apply_vertical_wall_run(world, &platform_mask, wall.row, wall.col, wall.rows);
            }
        }
    }
}

fn apply_generated_asset_pass(world: &mut TileWorld) {
    apply_generated_bushes(world);
    apply_generated_water_variations(world);
}

fn apply_generated_bushes(world: &mut TileWorld) {
    for row in 0..world.rows {
        for col in 0..world.cols {
            if !generated_asset_tile_is_plain_grass(world, col, row) {
                continue;
            }

            for half_row in 0..BUILDING_GRID_DIVISIONS {
                let roll = generated_asset_roll(col, row, half_row, 0xB05C_BA55_2026);
                if roll % GENERATED_BUSH_CHANCE_DENOMINATOR != 0 {
                    continue;
                }

                let x2 = col * BUILDING_GRID_DIVISIONS;
                let y2 = row * BUILDING_GRID_DIVISIONS + half_row;
                let kind = generated_bush_kind(roll >> 8);
                if !try_push_generated_bush(world, kind, x2, y2) {
                    continue;
                }

                if ((roll >> 16) % 100) >= GENERATED_BUSH_PAIR_CHANCE_NUMERATOR {
                    continue;
                }

                let (dx2, dy2) = GENERATED_BUSH_ADJACENT_OFFSETS
                    [((roll >> 24) as usize) % GENERATED_BUSH_ADJACENT_OFFSETS.len()];
                let Some(adjacent_x2) = x2.checked_add_signed(dx2) else {
                    continue;
                };
                let Some(adjacent_y2) = y2.checked_add_signed(dy2) else {
                    continue;
                };
                let adjacent_kind = generated_bush_kind(roll >> 32);
                let _ = try_push_generated_bush(world, adjacent_kind, adjacent_x2, adjacent_y2);
            }
        }
    }

    world.props.sort_by_key(|prop| (prop.y2, prop.x2));
}

fn apply_generated_water_variations(world: &mut TileWorld) {
    for row in 0..world.rows {
        for col in 0..world.cols {
            let index = world.index(col, row);
            if world.backgrounds[index] != BackgroundTile::Water
                || world.foregrounds[index].is_some()
                || world.under_foregrounds[index].is_some()
                || world.water_states[index] != WaterState::Nothing
            {
                continue;
            }

            let roll = generated_asset_roll(col, row, 0, 0x0A7E_51A7_E202_6);
            if roll % GENERATED_WATER_VARIATION_CHANCE_DENOMINATOR != 0 {
                continue;
            }

            world.water_states[index] =
                GENERATED_WATER_STATES[((roll >> 8) as usize) % GENERATED_WATER_STATES.len()];
        }
    }
}

fn generated_asset_tile_is_plain_grass(world: &TileWorld, col: usize, row: usize) -> bool {
    let index = world.index(col, row);
    world.backgrounds[index] == BackgroundTile::Grass
        && world.foregrounds[index].is_none()
        && world.under_foregrounds[index].is_none()
}

fn generated_bush_kind(roll: u64) -> PlantKind {
    GENERATED_BUSH_KINDS[(roll as usize) % GENERATED_BUSH_KINDS.len()]
}

fn try_push_generated_bush(world: &mut TileWorld, kind: PlantKind, x2: usize, y2: usize) -> bool {
    let prop = PropKind::Plant(kind);
    if !world.can_place_prop_half(prop, x2, y2)
        || !generated_bush_foundation_is_plain_grass(world, kind, x2, y2)
    {
        return false;
    }

    world.props.push(PlacedProp { kind: prop, x2, y2 });
    true
}

fn generated_bush_foundation_is_plain_grass(
    world: &TileWorld,
    kind: PlantKind,
    x2: usize,
    y2: usize,
) -> bool {
    let footprint = generated_bush_footprint_rect2(kind, x2, y2);
    let Some((start_col, start_row, cols, rows)) = generated_half_rect_to_tile_rect(footprint)
    else {
        return false;
    };
    if start_col + cols > world.cols || start_row + rows > world.rows {
        return false;
    }

    for row in start_row..start_row + rows {
        for col in start_col..start_col + cols {
            if !generated_asset_tile_is_plain_grass(world, col, row) {
                return false;
            }
        }
    }

    true
}

fn generated_bush_footprint_rect2(
    kind: PlantKind,
    x2: usize,
    y2: usize,
) -> (isize, isize, usize, usize) {
    if kind.uses_half_height_footprint() {
        (x2 as isize, y2 as isize + 1, BUILDING_GRID_DIVISIONS, 1)
    } else {
        (
            x2 as isize,
            y2 as isize,
            BUILDING_GRID_DIVISIONS,
            BUILDING_GRID_DIVISIONS,
        )
    }
}

fn generated_half_rect_to_tile_rect(
    rect: (isize, isize, usize, usize),
) -> Option<(usize, usize, usize, usize)> {
    let (x2, y2, w2, h2) = rect;
    if x2 < 0 || y2 < 0 {
        return None;
    }

    let x2 = x2 as usize;
    let y2 = y2 as usize;
    let start_col = x2 / BUILDING_GRID_DIVISIONS;
    let start_row = y2 / BUILDING_GRID_DIVISIONS;
    let end_col = (x2 + w2).div_ceil(BUILDING_GRID_DIVISIONS);
    let end_row = (y2 + h2).div_ceil(BUILDING_GRID_DIVISIONS);
    Some((
        start_col,
        start_row,
        end_col - start_col,
        end_row - start_row,
    ))
}

fn generated_asset_roll(col: usize, row: usize, slot: usize, salt: u64) -> u64 {
    let mut value = salt
        ^ ((col as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
        ^ ((row as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9))
        ^ ((slot as u64).wrapping_mul(0x94D0_49BB_1331_11EB));
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

fn apply_level0_shoreline_replace(world: &mut TileWorld) {
    let backgrounds = world.backgrounds.clone();
    let under_foregrounds = world.under_foregrounds.clone();
    let foregrounds = world.foregrounds.clone();

    for row in 0..world.rows {
        for col in 0..world.cols {
            let index = world.index(col, row);
            if backgrounds[index] != BackgroundTile::Grass || under_foregrounds[index].is_some() {
                continue;
            }

            let grass_n =
                level0_background_is_grass(&backgrounds, world.cols, world.rows, col, row, 0, -1);
            let grass_e =
                level0_background_is_grass(&backgrounds, world.cols, world.rows, col, row, 1, 0);
            let grass_s =
                level0_background_is_grass(&backgrounds, world.cols, world.rows, col, row, 0, 1);
            let grass_w =
                level0_background_is_grass(&backgrounds, world.cols, world.rows, col, row, -1, 0);

            let Some(tile) = level0_shoreline_tile(grass_n, grass_e, grass_s, grass_w) else {
                continue;
            };

            match foregrounds[index] {
                None => {
                    world.set_background(col, row, BackgroundTile::Water);
                    world.foregrounds[index] = Some(tile);
                }
                Some(foreground) if foreground_accepts_level0_shoreline_underlay(foreground) => {
                    world.set_background(col, row, BackgroundTile::Water);
                    world.under_foregrounds[index] = Some(tile);
                }
                Some(_) => {}
            }
        }
    }
}

fn foreground_accepts_level0_shoreline_underlay(foreground: AtlasTile) -> bool {
    PLATFORM_GRASS_ROW_TILES.contains(&foreground)
        || PLATFORM_WATER_ROW_TILES.contains(&foreground)
        || PLATFORM_GRASS_BORDER_TILES
            .iter()
            .flatten()
            .any(|&tile| tile == foreground)
}

fn level0_background_is_grass(
    backgrounds: &[BackgroundTile],
    cols: usize,
    rows: usize,
    col: usize,
    row: usize,
    dc: isize,
    dr: isize,
) -> bool {
    let next_col = col as isize + dc;
    let next_row = row as isize + dr;
    if next_col < 0 || next_row < 0 || next_col >= cols as isize || next_row >= rows as isize {
        return false;
    }

    backgrounds[next_row as usize * cols + next_col as usize] == BackgroundTile::Grass
}

fn level0_shoreline_tile(
    grass_n: bool,
    grass_e: bool,
    grass_s: bool,
    grass_w: bool,
) -> Option<AtlasTile> {
    let open_mask = usize::from(!grass_n)
        | (usize::from(!grass_e) << 1)
        | (usize::from(!grass_s) << 2)
        | (usize::from(!grass_w) << 3);
    LEVEL0_SHORELINE_BY_OPEN_MASK[open_mask]
}

fn apply_platform_cliffline_replace(
    world: &mut TileWorld,
    pre_shoreline_backgrounds: &[BackgroundTile],
) {
    let foregrounds = world.foregrounds.clone();

    for row in 0..world.rows {
        for col in 0..world.cols {
            let index = world.index(col, row);
            let Some((tile, inward_col, inward_row)) =
                platform_frame_inner_water_replacement(foregrounds[index], col, row)
            else {
                continue;
            };
            if inward_col >= world.cols || inward_row >= world.rows {
                continue;
            }

            let inward_index = world.index(inward_col, inward_row);
            if pre_shoreline_backgrounds[inward_index] == BackgroundTile::Water {
                world.foregrounds[index] = Some(tile);
            }
        }
    }
}

fn platform_frame_inner_water_replacement(
    foreground: Option<AtlasTile>,
    col: usize,
    row: usize,
) -> Option<(AtlasTile, usize, usize)> {
    let tile = foreground?;
    if tile == PLATFORM_GRASS_BORDER_TILES[1][0] {
        Some((PLATFORM_GRASS_INNER_WATER_LEFT_TILE, col + 1, row))
    } else if tile == PLATFORM_GRASS_BORDER_TILES[1][2] {
        Some((
            PLATFORM_GRASS_INNER_WATER_RIGHT_TILE,
            col.checked_sub(1)?,
            row,
        ))
    } else if tile == PLATFORM_GRASS_BORDER_TILES[0][1] {
        Some((PLATFORM_GRASS_INNER_WATER_TOP_TILE, col, row + 1))
    } else if tile == PLATFORM_GRASS_BORDER_TILES[2][1] {
        Some((
            PLATFORM_GRASS_INNER_WATER_BOTTOM_TILE,
            col,
            row.checked_sub(1)?,
        ))
    } else {
        None
    }
}

fn standalone_pillar_tile(background: BackgroundTile) -> AtlasTile {
    match background {
        BackgroundTile::Grass => STANDALONE_GRASS_PILLAR_TILE,
        BackgroundTile::Water => STANDALONE_WATER_PILLAR_TILE,
    }
}

fn apply_platform_rects_with_interior_fill(
    world: &mut TileWorld,
    platforms: impl IntoIterator<Item = (usize, usize, usize, usize, BackgroundTile)>,
    fill_interior_background: bool,
) {
    let mut mask = vec![None; world.cols * world.rows];

    for (col, row, cols, rows, background) in platforms {
        stamp_platform_visual_mask(world, &mut mask, col, row, cols, rows, background);
    }

    apply_platform_mask(world, &mask, fill_interior_background);
}

fn apply_platform_mask(
    world: &mut TileWorld,
    mask: &[Option<BackgroundTile>],
    fill_interior_background: bool,
) {
    for row in 0..world.rows {
        for col in 0..world.cols {
            let Some(background) = mask[world.index(col, row)] else {
                continue;
            };
            let border_tile =
                platform_border_tile(&mask, world.cols, world.rows, col, row, background);
            if fill_interior_background || border_tile.is_some() {
                let index = world.index(col, row);
                world.set_background(col, row, BackgroundTile::Grass);
                world.foregrounds[index] = None;
            }
            if let Some(tile) = border_tile {
                let index = world.index(col, row);
                world.foregrounds[index] = Some(tile);
            }
        }
    }

    tile_platform_bottom_pillars(world, &mask);
}

fn stamp_platform_visual_mask(
    world: &TileWorld,
    mask: &mut [Option<BackgroundTile>],
    rect_col: usize,
    rect_row: usize,
    rect_cols: usize,
    rect_rows: usize,
    background: BackgroundTile,
) {
    let end_row = (rect_row + rect_rows).min(world.rows);
    let end_col = (rect_col + rect_cols).min(world.cols);
    for row in rect_row..end_row {
        for col in rect_col..end_col {
            mask[world.index(col, row)] = Some(background);
        }
    }
}

fn tile_platform_row_run(
    world: &mut TileWorld,
    mask: &[Option<BackgroundTile>],
    row: usize,
    start_col: usize,
    cols: usize,
    background: BackgroundTile,
) {
    apply_wall_run(
        world,
        mask,
        row,
        start_col,
        cols,
        WallRunStyle::PlatformBottom { background },
    );
}

fn apply_wall_run(
    world: &mut TileWorld,
    foundation_mask: &[Option<BackgroundTile>],
    row: usize,
    start_col: usize,
    cols: usize,
    style: WallRunStyle,
) {
    for offset in 0..cols {
        let col = start_col + offset;
        if col >= world.cols || row >= world.rows {
            continue;
        }
        let index = world.index(col, row);
        if world.foregrounds[index].is_some_and(is_wall_pillar_tile) {
            continue;
        }

        let foundation = wall_foundation_background(world, foundation_mask, col, row);
        let tile = match style {
            WallRunStyle::PlatformBottom { background } => platform_row_tile(
                foundation,
                background,
                foundation_mask,
                world.cols,
                row,
                start_col,
                col,
                offset,
                cols,
            ),
        };
        world.set_background(col, row, foundation);
        world.foregrounds[index] = Some(tile);
    }
}

fn apply_wall_cap_run(
    world: &mut TileWorld,
    foundation_mask: &[Option<BackgroundTile>],
    row: usize,
    start_col: usize,
    cols: usize,
) {
    for offset in 0..cols {
        let col = start_col + offset;
        if col >= world.cols || row >= world.rows {
            continue;
        }
        let index = world.index(col, row);
        if world.foregrounds[index].is_some_and(is_wall_pillar_tile) {
            continue;
        }

        let foundation = wall_foundation_background(world, foundation_mask, col, row);
        let tile_background = wall_cap_tile_background(world, foundation_mask, col, row);
        world.set_background(col, row, foundation);
        world.foregrounds[index] = Some(platform_row_tile(
            tile_background,
            tile_background,
            foundation_mask,
            world.cols,
            row,
            start_col,
            col,
            offset,
            cols,
        ));
    }
}

fn apply_wall_cliff_cap_run(world: &mut TileWorld, row: usize, start_col: usize, cols: usize) {
    let Some(cap_row) = row.checked_sub(1) else {
        return;
    };

    for offset in 0..cols {
        let col = start_col + offset;
        if col >= world.cols {
            continue;
        }
        let index = world.index(col, cap_row);
        if world.foregrounds[index].is_some_and(is_wall_pillar_tile) {
            continue;
        }
        world.foregrounds[index] = Some(wall_cliff_cap_tile(offset, cols));
    }
}

fn apply_vertical_wall_run(
    world: &mut TileWorld,
    foundation_mask: &[Option<BackgroundTile>],
    start_row: usize,
    col: usize,
    rows: usize,
) {
    let cliff_rows = rows.saturating_sub(1);
    for offset in 0..rows {
        let row = start_row + offset;
        if col >= world.cols || row >= world.rows {
            continue;
        }
        let index = world.index(col, row);
        if world.foregrounds[index].is_some_and(is_wall_pillar_tile) {
            continue;
        }

        if offset < cliff_rows {
            world.foregrounds[index] = Some(vertical_cliff_tile(offset, cliff_rows));
        } else {
            let foundation = wall_foundation_background(world, foundation_mask, col, row);
            let tile_background = wall_cap_tile_background(world, foundation_mask, col, row);
            world.set_background(col, row, foundation);
            world.foregrounds[index] = Some(standalone_pillar_tile(tile_background));
        }
    }
}

fn wall_cliff_cap_tile(offset: usize, cols: usize) -> AtlasTile {
    match cols {
        1 => CLIFF_GRASS_CAP_TILE,
        2 => PLATFORM_GRASS_CLIFF_CAP_TILES[[0, 2][offset.min(1)]],
        _ => PLATFORM_GRASS_CLIFF_CAP_TILES[offset.min(2)],
    }
}

fn vertical_cliff_tile(offset: usize, cliff_rows: usize) -> AtlasTile {
    match cliff_rows {
        1 => VERTICAL_GRASS_CLIFF_TILES[2],
        2 => VERTICAL_GRASS_CLIFF_TILES[[0, 2][offset.min(1)]],
        _ => VERTICAL_GRASS_CLIFF_TILES[offset.min(2)],
    }
}

fn wall_cap_tile_background(
    world: &TileWorld,
    foundation_mask: &[Option<BackgroundTile>],
    col: usize,
    row: usize,
) -> BackgroundTile {
    if foundation_mask[world.index(col, row)].is_some() {
        BackgroundTile::Grass
    } else {
        wall_foundation_background(world, foundation_mask, col, row)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WallRunStyle {
    PlatformBottom { background: BackgroundTile },
}

fn tile_platform_bottom_pillars(world: &mut TileWorld, mask: &[Option<BackgroundTile>]) {
    for row in 0..world.rows {
        let mut col = 0;
        while col < world.cols {
            let Some(background) =
                platform_bottom_background(mask, world.cols, world.rows, col, row)
            else {
                col += 1;
                continue;
            };

            let start_col = col;
            while col < world.cols
                && platform_bottom_background(mask, world.cols, world.rows, col, row)
                    == Some(background)
            {
                col += 1;
            }
            if row + 1 < world.rows {
                tile_platform_row_run(world, mask, row + 1, start_col, col - start_col, background);
            }
        }
    }
}

fn platform_bottom_background(
    mask: &[Option<BackgroundTile>],
    cols: usize,
    rows: usize,
    col: usize,
    row: usize,
) -> Option<BackgroundTile> {
    let background = mask[row * cols + col]?;
    let below = row + 1 < rows && mask[(row + 1) * cols + col] == Some(background);
    (!below).then_some(background)
}

fn platform_border_tile(
    mask: &[Option<BackgroundTile>],
    cols: usize,
    rows: usize,
    col: usize,
    row: usize,
    background: BackgroundTile,
) -> Option<AtlasTile> {
    let open_n = row == 0 || mask[(row - 1) * cols + col] != Some(background);
    let open_e = col + 1 >= cols || mask[row * cols + col + 1] != Some(background);
    let open_s = row + 1 >= rows || mask[(row + 1) * cols + col] != Some(background);
    let open_w = col == 0 || mask[row * cols + col - 1] != Some(background);

    if !(open_n || open_e || open_s || open_w) {
        return None;
    }

    Some(match (open_n, open_e, open_s, open_w) {
        (true, _, _, true) => PLATFORM_GRASS_BORDER_TILES[0][0],
        (true, true, _, _) => PLATFORM_GRASS_BORDER_TILES[0][2],
        (_, true, true, _) => PLATFORM_GRASS_BORDER_TILES[2][2],
        (_, _, true, true) => PLATFORM_GRASS_BORDER_TILES[2][0],
        (true, _, _, _) => PLATFORM_GRASS_BORDER_TILES[0][1],
        (_, true, _, _) => PLATFORM_GRASS_BORDER_TILES[1][2],
        (_, _, true, _) => PLATFORM_GRASS_BORDER_TILES[2][1],
        (_, _, _, true) => PLATFORM_GRASS_BORDER_TILES[1][0],
        _ => PLATFORM_GRASS_BORDER_TILES[1][1],
    })
}

pub(super) fn platform_row_tile(
    tile_background: BackgroundTile,
    run_background: BackgroundTile,
    mask: &[Option<BackgroundTile>],
    cols_total: usize,
    row: usize,
    start_col: usize,
    col: usize,
    offset: usize,
    cols: usize,
) -> AtlasTile {
    let tiles = match tile_background {
        BackgroundTile::Grass => PLATFORM_GRASS_ROW_TILES,
        BackgroundTile::Water => PLATFORM_WATER_ROW_TILES,
    };

    if cols == 2 && short_platform_row_uses_standalone_pair(row, start_col, run_background) {
        return standalone_pillar_tile(tile_background);
    }

    match (offset, cols) {
        (_, 1) => single_platform_row_tile(
            tiles,
            mask,
            cols_total,
            row,
            col,
            tile_background,
            run_background,
        ),
        (0, _) => tiles[0],
        (offset, cols) if offset + 1 == cols => tiles[2],
        _ => tiles[1],
    }
}

fn single_platform_row_tile(
    tiles: [AtlasTile; 3],
    mask: &[Option<BackgroundTile>],
    cols_total: usize,
    row: usize,
    col: usize,
    tile_background: BackgroundTile,
    run_background: BackgroundTile,
) -> AtlasTile {
    if single_platform_row_uses_standalone(row, col, run_background) {
        return standalone_pillar_tile(tile_background);
    }

    let source_row = row.saturating_sub(1);
    let same_left = col > 0 && mask[source_row * cols_total + col - 1] == Some(run_background);
    let same_right =
        col + 1 < cols_total && mask[source_row * cols_total + col + 1] == Some(run_background);

    match (same_left, same_right) {
        (true, false) => tiles[2],
        (false, true) => tiles[0],
        _ => tiles[1],
    }
}

fn wall_foundation_background(
    world: &TileWorld,
    foundation_mask: &[Option<BackgroundTile>],
    col: usize,
    row: usize,
) -> BackgroundTile {
    foundation_mask
        .get(world.index(col, row))
        .copied()
        .flatten()
        .unwrap_or_else(|| world.background(col, row))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WallFoundationLevel {
    Level0,
    Level1,
}

fn generated_wall_foundation_level(
    platform_mask: &[Option<BackgroundTile>],
    world: &GeneratedWorld,
    col: usize,
    row: usize,
) -> WallFoundationLevel {
    if platform_mask[world.index(col, row)].is_some() {
        WallFoundationLevel::Level1
    } else {
        WallFoundationLevel::Level0
    }
}

fn generated_wall_run_has_same_level(
    platform_mask: &[Option<BackgroundTile>],
    world: &GeneratedWorld,
    start_col: usize,
    row: usize,
    cols: usize,
) -> bool {
    let first_level = generated_wall_foundation_level(platform_mask, world, start_col, row);
    (start_col + 1..start_col + cols)
        .all(|col| generated_wall_foundation_level(platform_mask, world, col, row) == first_level)
}

fn generated_vertical_wall_run_has_same_level(
    platform_mask: &[Option<BackgroundTile>],
    world: &GeneratedWorld,
    col: usize,
    start_row: usize,
    rows: usize,
) -> bool {
    let first_level = generated_wall_foundation_level(platform_mask, world, col, start_row);
    (start_row + 1..start_row + rows)
        .all(|row| generated_wall_foundation_level(platform_mask, world, col, row) == first_level)
}

fn horizontal_wall_start_col(world_cols: usize, col: usize, cols: usize) -> Option<usize> {
    if !(1..=3).contains(&cols) {
        return None;
    }
    let start_col = match cols {
        3 => col.checked_sub(1)?,
        _ => col,
    };
    (start_col.saturating_add(cols) <= world_cols).then_some(start_col)
}

fn vertical_wall_start_row(world_rows: usize, row: usize, cliff_rows: usize) -> Option<usize> {
    if !(1..=3).contains(&cliff_rows) {
        return None;
    }
    let start_row = row.checked_add(1)?.checked_sub(cliff_rows)?;
    (start_row.saturating_add(cliff_rows + 1) <= world_rows).then_some(start_row)
}

fn generated_wall_run_has_pillar_clearance(
    world: &TileWorld,
    start_col: usize,
    row: usize,
    cols: usize,
) -> bool {
    let check_start_col = start_col;
    let check_end_col = start_col.saturating_add(cols).min(world.cols);
    let check_start_row = row.saturating_sub(1);
    let check_end_row = row.saturating_add(2).min(world.rows);

    for check_row in check_start_row..check_end_row {
        for check_col in check_start_col..check_end_col {
            let index = world.index(check_col, check_row);
            if world.foregrounds[index].is_some_and(is_wall_pillar_tile) {
                return false;
            }
        }
    }

    true
}

fn generated_vertical_wall_run_has_pillar_clearance(
    world: &TileWorld,
    col: usize,
    start_row: usize,
    rows: usize,
) -> bool {
    let check_start_col = col.saturating_sub(1);
    let check_end_col = col.saturating_add(2).min(world.cols);
    let check_start_row = start_row;
    let check_end_row = start_row.saturating_add(rows).min(world.rows);

    for check_row in check_start_row..check_end_row {
        for check_col in check_start_col..check_end_col {
            let index = world.index(check_col, check_row);
            if world.foregrounds[index].is_some_and(is_wall_pillar_tile) {
                return false;
            }
        }
    }

    true
}

fn generated_wall_cells(wall: &GeneratedWallTile) -> impl Iterator<Item = (usize, usize)> + '_ {
    (0..wall.rows).flat_map(move |row_offset| {
        (0..wall.cols).map(move |col_offset| (wall.col + col_offset, wall.row + row_offset))
    })
}

fn is_wall_pillar_tile(tile: AtlasTile) -> bool {
    PLATFORM_GRASS_ROW_TILES.contains(&tile)
        || PLATFORM_WATER_ROW_TILES.contains(&tile)
        || VERTICAL_GRASS_CLIFF_TILES.contains(&tile)
        || tile == STANDALONE_GRASS_PILLAR_TILE
        || tile == STANDALONE_WATER_PILLAR_TILE
}

fn single_platform_row_uses_standalone(row: usize, col: usize, background: BackgroundTile) -> bool {
    let background_bit = match background {
        BackgroundTile::Grass => 0,
        BackgroundTile::Water => 1,
    };
    (row + col + background_bit) % 2 == 0
}

fn short_platform_row_uses_standalone_pair(
    row: usize,
    start_col: usize,
    background: BackgroundTile,
) -> bool {
    let background_bit = match background {
        BackgroundTile::Grass => 0,
        BackgroundTile::Water => 1,
    };
    (row + start_col + background_bit) % 4 == 0
}

fn platforms_overlap_with_margin(
    a: GeneratedPlatform,
    b: GeneratedPlatform,
    margin: usize,
) -> bool {
    let a_left = a.col.saturating_sub(margin);
    let a_top = a.row.saturating_sub(margin);
    let a_right = (a.col + a.cols + margin).min(usize::MAX);
    let a_bottom = (a.row + a.rows + margin).min(usize::MAX);
    let b_left = b.col;
    let b_top = b.row;
    let b_right = b.col + b.cols;
    let b_bottom = b.row + b.rows;

    a_left < b_right && b_left < a_right && a_top < b_bottom && b_top < a_bottom
}

fn platforms_overlap(a: GeneratedPlatform, b: GeneratedPlatform) -> bool {
    a.col < b.col + b.cols
        && b.col < a.col + a.cols
        && a.row < b.row + b.rows
        && b.row < a.row + a.rows
}

fn platform_center_on_border_of(anchor: GeneratedPlatform, platform: GeneratedPlatform) -> bool {
    let center_col2 = platform.col * 2 + platform.cols;
    let center_row2 = platform.row * 2 + platform.rows;
    let left2 = anchor.col * 2;
    let right2 = (anchor.col + anchor.cols) * 2;
    let top2 = anchor.row * 2;
    let bottom2 = (anchor.row + anchor.rows) * 2;

    ((center_col2 == left2 || center_col2 == right2)
        && center_row2 >= top2
        && center_row2 <= bottom2)
        || ((center_row2 == top2 || center_row2 == bottom2)
            && center_col2 >= left2
            && center_col2 <= right2)
}

fn add_fillable_unknown_neighbors_to_frontier(
    world: &GeneratedWorld,
    fillable: &[bool],
    index: usize,
    buckets: &mut [Vec<usize>; 5],
    in_frontier: &mut [bool],
) {
    for neighbor in cardinal_neighbor_indices(world, index) {
        if !fillable[neighbor]
            || world.cells[neighbor] != GeneratedCell::Unknown
            || in_frontier[neighbor]
        {
            continue;
        }
        let known_neighbors = known_neighbor_count(world, neighbor);
        if known_neighbors == 0 {
            continue;
        }
        buckets[known_neighbors].push(neighbor);
        in_frontier[neighbor] = true;
    }
}

fn next_fillable_frontier_cell(
    world: &GeneratedWorld,
    fillable: &[bool],
    buckets: &mut [Vec<usize>; 5],
    in_frontier: &mut [bool],
    rng: &mut SeededRng,
) -> Option<usize> {
    loop {
        let bucket_id = (1..buckets.len())
            .rev()
            .find(|&bucket_id| !buckets[bucket_id].is_empty())?;
        let pick = rng.range_usize(0, buckets[bucket_id].len());
        let index = buckets[bucket_id].swap_remove(pick);
        in_frontier[index] = false;

        if !fillable[index] || world.cells[index] != GeneratedCell::Unknown {
            continue;
        }

        let known_neighbors = known_neighbor_count(world, index);
        if known_neighbors == 0 {
            continue;
        }
        if known_neighbors != bucket_id {
            buckets[known_neighbors].push(index);
            in_frontier[index] = true;
            continue;
        }

        return Some(index);
    }
}

fn affinity_fitting_cell(
    world: &GeneratedWorld,
    index: usize,
    rng: &mut SeededRng,
) -> GeneratedCell {
    let candidates = [GeneratedCell::Grass, GeneratedCell::Water];
    let mut weighted = Vec::new();
    let mut total_weight = 0.0;

    for candidate in candidates {
        if !fits_known_neighbors(world, index, candidate) {
            continue;
        }

        let weight = base_cell_weight(candidate)
            * (1.0
                + affinity_step(candidate)
                    * same_cell_count_in_area3(world, index, candidate) as f32);
        total_weight += weight;
        weighted.push((candidate, weight));
    }

    if weighted.is_empty() {
        return GeneratedCell::Grass;
    }

    let mut roll = rng.next_f32() * total_weight;
    for (candidate, weight) in weighted {
        if roll < weight {
            return candidate;
        }
        roll -= weight;
    }

    GeneratedCell::Grass
}

fn base_cell_weight(cell: GeneratedCell) -> f32 {
    match cell {
        GeneratedCell::Unknown => 0.0,
        GeneratedCell::Grass => GRASS_WEIGHT,
        GeneratedCell::Water => WATER_WEIGHT,
    }
}

fn affinity_step(cell: GeneratedCell) -> f32 {
    match cell {
        GeneratedCell::Unknown => 0.0,
        GeneratedCell::Grass => GRASS_AFFINITY_STEP,
        GeneratedCell::Water => WATER_AFFINITY_STEP,
    }
}

fn fits_known_neighbors(world: &GeneratedWorld, index: usize, candidate: GeneratedCell) -> bool {
    cardinal_neighbor_indices(world, index).all(|neighbor| {
        let neighbor_cell = world.cells[neighbor];
        neighbor_cell == GeneratedCell::Unknown || raw_cells_can_touch(candidate, neighbor_cell)
    })
}

fn raw_cells_can_touch(_a: GeneratedCell, _b: GeneratedCell) -> bool {
    true
}

fn known_neighbor_count(world: &GeneratedWorld, index: usize) -> usize {
    cardinal_neighbor_indices(world, index)
        .filter(|&neighbor| world.cells[neighbor] != GeneratedCell::Unknown)
        .count()
}

fn same_cell_count_in_area3(
    world: &GeneratedWorld,
    index: usize,
    candidate: GeneratedCell,
) -> usize {
    let col = index % world.cols;
    let row = index / world.cols;
    let col_start = col.saturating_sub(1);
    let row_start = row.saturating_sub(1);
    let col_end = (col + 1).min(world.cols - 1);
    let row_end = (row + 1).min(world.rows - 1);
    let mut count = 0;

    for neighbor_row in row_start..=row_end {
        for neighbor_col in col_start..=col_end {
            if neighbor_col == col && neighbor_row == row {
                continue;
            }
            if world.cell(neighbor_col, neighbor_row) == candidate {
                count += 1;
            }
        }
    }

    count
}

fn cardinal_neighbor_indices(
    world: &GeneratedWorld,
    index: usize,
) -> impl Iterator<Item = usize> + '_ {
    let col = index % world.cols;
    let row = index / world.cols;
    [
        (row > 0).then(|| index - world.cols),
        (col + 1 < world.cols).then(|| index + 1),
        (row + 1 < world.rows).then(|| index + world.cols),
        (col > 0).then(|| index - 1),
    ]
    .into_iter()
    .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visual_void_fill_does_not_write_existing_platform_tiles() {
        let mut world = GeneratedWorld::new(5, 5);
        world.platforms.push(GeneratedPlatform {
            col: 1,
            row: 1,
            cols: 3,
            rows: 3,
            background: BackgroundTile::Grass,
            kind: GeneratedPlatformKind::Large,
        });

        let visual_world = world.to_visual_tile_world_with_shoreline(false);
        let protected = visual_world
            .tiles
            .foregrounds
            .iter()
            .enumerate()
            .filter_map(|(index, foreground)| foreground.is_some().then_some(index))
            .collect::<Vec<_>>();
        assert!(!protected.is_empty());

        let mut generator = RunningGenerator::from_world(world, 1);
        generator.fill_visual_voids_once(25);

        for index in protected {
            assert_eq!(generator.world.cells[index], GeneratedCell::Unknown);
        }
        assert_ne!(
            generator.world.cell(2, 2),
            GeneratedCell::Unknown,
            "platform interior without a tile should still be fillable"
        );
    }

    #[test]
    fn shoreline_replace_maps_only_plain_level0_grass_edges() {
        let mut world = GeneratedWorld::new(3, 3);
        world.cells = vec![GeneratedCell::Grass; 9];
        let north_index = world.index(1, 0);
        world.cells[north_index] = GeneratedCell::Water;

        let visual_world = world.to_visual_tile_world();
        let center = visual_world.tiles.index(1, 1);
        assert_eq!(visual_world.tiles.foregrounds[center], Some(SHORE_TOP));
        assert_eq!(
            visual_world.tiles.backgrounds[center],
            BackgroundTile::Water
        );
    }

    #[test]
    fn shoreline_replace_leaves_center_grass_plain() {
        let mut world = GeneratedWorld::new(3, 3);
        world.cells = vec![GeneratedCell::Grass; 9];
        for (col, row) in [(0, 0), (2, 0), (0, 2), (2, 2)] {
            let index = world.index(col, row);
            world.cells[index] = GeneratedCell::Water;
        }

        let visual_world = world.to_visual_tile_world();
        let center = visual_world.tiles.index(1, 1);
        assert_eq!(visual_world.tiles.foregrounds[center], None);
        assert_eq!(
            visual_world.tiles.backgrounds[center],
            BackgroundTile::Grass
        );
    }

    #[test]
    fn shoreline_replace_maps_every_non_center_side_mask() {
        for open_mask in 1..LEVEL0_SHORELINE_BY_OPEN_MASK.len() {
            assert!(
                LEVEL0_SHORELINE_BY_OPEN_MASK[open_mask].is_some(),
                "open side mask {open_mask:04b} should map to a shore tile"
            );
        }
    }

    #[test]
    fn shoreline_replace_uses_narrow_tiles_for_opposite_and_three_side_masks() {
        assert_eq!(
            LEVEL0_SHORELINE_BY_OPEN_MASK[0b0101],
            Some(SHORE_NARROW_CENTER)
        );
        assert_eq!(
            LEVEL0_SHORELINE_BY_OPEN_MASK[0b1010],
            Some(SHORE_NARROW_MIDDLE)
        );
        assert_eq!(
            LEVEL0_SHORELINE_BY_OPEN_MASK[0b0111],
            Some(SHORE_NARROW_RIGHT)
        );
        assert_eq!(
            LEVEL0_SHORELINE_BY_OPEN_MASK[0b1111],
            Some(SHORE_SINGLE_IN_WATER)
        );
    }

    #[test]
    fn cliffline_replace_polishes_platform_frame_against_inner_water() {
        let mut world = GeneratedWorld::new(5, 5);
        world.cells = vec![GeneratedCell::Grass; 25];
        world.platforms.push(GeneratedPlatform {
            col: 1,
            row: 1,
            cols: 3,
            rows: 3,
            background: BackgroundTile::Grass,
            kind: GeneratedPlatformKind::Large,
        });
        let center_index = world.index(2, 2);
        world.cells[center_index] = GeneratedCell::Water;

        let visual_world = world.to_visual_tile_world();
        let top_frame = visual_world.tiles.index(2, 1);
        let left_frame = visual_world.tiles.index(1, 2);
        let right_frame = visual_world.tiles.index(3, 2);
        let bottom_frame = visual_world.tiles.index(2, 3);

        assert_eq!(
            visual_world.tiles.foregrounds[top_frame],
            Some(PLATFORM_GRASS_INNER_WATER_TOP_TILE)
        );
        assert_eq!(
            visual_world.tiles.foregrounds[left_frame],
            Some(PLATFORM_GRASS_INNER_WATER_LEFT_TILE)
        );
        assert_eq!(
            visual_world.tiles.foregrounds[right_frame],
            Some(PLATFORM_GRASS_INNER_WATER_RIGHT_TILE)
        );
        assert_eq!(
            visual_world.tiles.foregrounds[bottom_frame],
            Some(PLATFORM_GRASS_INNER_WATER_BOTTOM_TILE)
        );
    }

    #[test]
    fn cliffline_replace_ignores_water_created_by_shoreline_pass() {
        let mut world = GeneratedWorld::new(4, 3);
        world.cells = vec![GeneratedCell::Grass; 12];
        world.platforms.push(GeneratedPlatform {
            col: 0,
            row: 0,
            cols: 2,
            rows: 3,
            background: BackgroundTile::Grass,
            kind: GeneratedPlatformKind::Large,
        });
        let east_water = world.index(3, 1);
        world.cells[east_water] = GeneratedCell::Water;

        let visual_world = world.to_visual_tile_world();
        let right_frame = visual_world.tiles.index(1, 1);

        assert_eq!(
            visual_world.tiles.foregrounds[right_frame],
            Some(PLATFORM_GRASS_BORDER_TILES[1][2])
        );
        assert_eq!(
            visual_world.tiles.foregrounds[visual_world.tiles.index(2, 1)],
            Some(SHORE_RIGHT)
        );
    }

    #[test]
    fn shoreline_replace_underlays_platform_row_tiles() {
        let mut world = TileWorld::new(3, 3);
        let center = world.index(1, 1);
        world.foregrounds[center] = Some(PLATFORM_GRASS_ROW_TILES[1]);
        world.set_background(1, 2, BackgroundTile::Water);

        apply_level0_shoreline_replace(&mut world);

        assert_eq!(world.backgrounds[center], BackgroundTile::Water);
        assert_eq!(world.foregrounds[center], Some(PLATFORM_GRASS_ROW_TILES[1]));
        assert_eq!(world.under_foregrounds[center], Some(SHORE_BOTTOM));
    }

    #[test]
    fn wall_helper_uses_level0_foundation_when_no_level1_mask_exists() {
        let mut world = TileWorld::new(3, 2);
        let mask = vec![None; 6];
        world.set_background(1, 1, BackgroundTile::Water);

        apply_wall_run(
            &mut world,
            &mask,
            1,
            1,
            1,
            WallRunStyle::PlatformBottom {
                background: BackgroundTile::Grass,
            },
        );

        assert_eq!(world.backgrounds[world.index(1, 1)], BackgroundTile::Water);
        assert_eq!(
            world.foregrounds[world.index(1, 1)],
            Some(STANDALONE_WATER_PILLAR_TILE)
        );
    }

    #[test]
    fn wall_helper_prefers_level1_foundation_mask() {
        let mut world = TileWorld::new(3, 2);
        let mut mask = vec![None; 6];
        mask[world.index(1, 1)] = Some(BackgroundTile::Water);

        apply_wall_run(
            &mut world,
            &mask,
            1,
            1,
            1,
            WallRunStyle::PlatformBottom {
                background: BackgroundTile::Grass,
            },
        );

        assert_eq!(world.backgrounds[world.index(1, 1)], BackgroundTile::Water);
        assert_eq!(
            world.foregrounds[world.index(1, 1)],
            Some(STANDALONE_WATER_PILLAR_TILE)
        );
    }

    #[test]
    fn wall_helper_does_not_overwrite_existing_pillar_tile() {
        let mut world = TileWorld::new(3, 2);
        let mask = vec![None; 6];
        let index = world.index(1, 1);
        world.foregrounds[index] = Some(PLATFORM_GRASS_ROW_TILES[0]);

        apply_wall_run(
            &mut world,
            &mask,
            1,
            1,
            1,
            WallRunStyle::PlatformBottom {
                background: BackgroundTile::Water,
            },
        );

        assert_eq!(world.backgrounds[index], BackgroundTile::Grass);
        assert_eq!(world.foregrounds[index], Some(PLATFORM_GRASS_ROW_TILES[0]));
    }

    #[test]
    fn generated_wall_tile_is_visible_and_uses_foundation_art() {
        let mut world = GeneratedWorld::new(5, 4);
        for col in 1..=3 {
            let index = world.index(col, 1);
            world.cells[index] = GeneratedCell::Water;
        }
        assert!(world.add_wall_run3_centered(2, 1));

        let visual_world = world.to_visual_tile_world();

        for (offset, col) in (1..=3).enumerate() {
            let cap_index = visual_world.tiles.index(col, 1);
            assert!(visual_world.visible[cap_index]);
            assert_eq!(
                visual_world.tiles.backgrounds[cap_index],
                BackgroundTile::Water
            );
            assert_eq!(
                visual_world.tiles.foregrounds[cap_index],
                Some(PLATFORM_WATER_ROW_TILES[offset])
            );
        }
    }

    #[test]
    fn generated_wall_tile_places_matching_cliff_caps_above() {
        let mut world = GeneratedWorld::new(5, 4);

        assert!(world.add_wall_run3_centered(2, 1));

        let visual_world = world.to_visual_tile_world_with_shoreline(false);

        for (offset, col) in (1..=3).enumerate() {
            let cap_index = visual_world.tiles.index(col, 0);
            assert_eq!(
                visual_world.tiles.foregrounds[cap_index],
                Some(PLATFORM_GRASS_CLIFF_CAP_TILES[offset])
            );
        }
    }

    #[test]
    fn generated_wall_preview_includes_cliff_caps_and_pillars() {
        let world = GeneratedWorld::new(5, 4);

        let preview = world.wall_run3_preview_tiles_centered(2, 1).unwrap();

        assert_eq!(preview.len(), 6);
        assert_eq!(
            &preview[0..3],
            &[
                GeneratedWallPreviewTile {
                    col: 1,
                    row: 0,
                    tile: PLATFORM_GRASS_CLIFF_CAP_TILES[0],
                },
                GeneratedWallPreviewTile {
                    col: 2,
                    row: 0,
                    tile: PLATFORM_GRASS_CLIFF_CAP_TILES[1],
                },
                GeneratedWallPreviewTile {
                    col: 3,
                    row: 0,
                    tile: PLATFORM_GRASS_CLIFF_CAP_TILES[2],
                },
            ]
        );
        assert_eq!(
            &preview[3..6],
            &[
                GeneratedWallPreviewTile {
                    col: 1,
                    row: 1,
                    tile: PLATFORM_GRASS_ROW_TILES[0],
                },
                GeneratedWallPreviewTile {
                    col: 2,
                    row: 1,
                    tile: PLATFORM_GRASS_ROW_TILES[1],
                },
                GeneratedWallPreviewTile {
                    col: 3,
                    row: 1,
                    tile: PLATFORM_GRASS_ROW_TILES[2],
                },
            ]
        );
    }

    #[test]
    fn generated_horizontal_wall_supports_one_and_two_tile_variants() {
        let mut one_tile_world = GeneratedWorld::new(5, 4);
        assert!(one_tile_world.add_horizontal_wall_centered(2, 1, 1));
        let one_tile_visual = one_tile_world.to_visual_tile_world_with_shoreline(false);
        assert_eq!(
            one_tile_visual.tiles.foregrounds[one_tile_visual.tiles.index(2, 0)],
            Some(CLIFF_GRASS_CAP_TILE)
        );
        assert!(
            one_tile_visual.tiles.foregrounds[one_tile_visual.tiles.index(2, 1)]
                .is_some_and(is_wall_pillar_tile)
        );

        let mut two_tile_world = GeneratedWorld::new(5, 4);
        assert!(two_tile_world.add_horizontal_wall_centered(1, 1, 2));
        let two_tile_visual = two_tile_world.to_visual_tile_world_with_shoreline(false);
        assert_eq!(
            two_tile_visual.tiles.foregrounds[two_tile_visual.tiles.index(1, 0)],
            Some(PLATFORM_GRASS_CLIFF_CAP_TILES[0])
        );
        assert_eq!(
            two_tile_visual.tiles.foregrounds[two_tile_visual.tiles.index(2, 0)],
            Some(PLATFORM_GRASS_CLIFF_CAP_TILES[2])
        );
    }

    #[test]
    fn generated_vertical_wall_tile_places_three_cliff_tiles_and_bottom_pillar() {
        let mut world = GeneratedWorld::new(5, 6);

        assert!(world.add_vertical_wall_run3_centered(2, 2));

        let visual_world = world.to_visual_tile_world_with_shoreline(false);

        for (offset, row) in (0..=2).enumerate() {
            assert_eq!(
                visual_world.tiles.foregrounds[visual_world.tiles.index(2, row)],
                Some(VERTICAL_GRASS_CLIFF_TILES[offset])
            );
        }
        assert_eq!(
            visual_world.tiles.foregrounds[visual_world.tiles.index(2, 3)],
            Some(STANDALONE_GRASS_PILLAR_TILE)
        );
    }

    #[test]
    fn generated_vertical_wall_preview_is_centered_on_clicked_tile() {
        let world = GeneratedWorld::new(5, 6);

        let preview = world
            .vertical_wall_run3_preview_tiles_centered(2, 2)
            .unwrap();

        assert_eq!(
            preview,
            vec![
                GeneratedWallPreviewTile {
                    col: 2,
                    row: 0,
                    tile: VERTICAL_GRASS_CLIFF_TILES[0],
                },
                GeneratedWallPreviewTile {
                    col: 2,
                    row: 1,
                    tile: VERTICAL_GRASS_CLIFF_TILES[1],
                },
                GeneratedWallPreviewTile {
                    col: 2,
                    row: 2,
                    tile: VERTICAL_GRASS_CLIFF_TILES[2],
                },
                GeneratedWallPreviewTile {
                    col: 2,
                    row: 3,
                    tile: STANDALONE_GRASS_PILLAR_TILE,
                },
            ]
        );
    }

    #[test]
    fn generated_vertical_wall_supports_one_and_two_cliff_tile_variants() {
        let mut one_tile_world = GeneratedWorld::new(5, 6);
        assert!(one_tile_world.add_vertical_wall_centered(2, 2, 1));
        let one_tile_visual = one_tile_world.to_visual_tile_world_with_shoreline(false);
        assert_eq!(
            one_tile_visual.tiles.foregrounds[one_tile_visual.tiles.index(2, 2)],
            Some(VERTICAL_GRASS_CLIFF_TILES[2])
        );
        assert_eq!(
            one_tile_visual.tiles.foregrounds[one_tile_visual.tiles.index(2, 3)],
            Some(STANDALONE_GRASS_PILLAR_TILE)
        );

        let mut two_tile_world = GeneratedWorld::new(5, 6);
        assert!(two_tile_world.add_vertical_wall_centered(2, 2, 2));
        let two_tile_visual = two_tile_world.to_visual_tile_world_with_shoreline(false);
        assert_eq!(
            two_tile_visual.tiles.foregrounds[two_tile_visual.tiles.index(2, 1)],
            Some(VERTICAL_GRASS_CLIFF_TILES[0])
        );
        assert_eq!(
            two_tile_visual.tiles.foregrounds[two_tile_visual.tiles.index(2, 2)],
            Some(VERTICAL_GRASS_CLIFF_TILES[2])
        );
        assert_eq!(
            two_tile_visual.tiles.foregrounds[two_tile_visual.tiles.index(2, 3)],
            Some(STANDALONE_GRASS_PILLAR_TILE)
        );
    }

    #[test]
    fn generated_vertical_wall_rejects_mixed_level_foundations() {
        let mut world = GeneratedWorld::new(5, 6);
        world.platforms.push(GeneratedPlatform {
            col: 2,
            row: 2,
            cols: 1,
            rows: 2,
            background: BackgroundTile::Grass,
            kind: GeneratedPlatformKind::Small,
        });

        assert!(!world.add_vertical_wall_run3_centered(2, 2));
        assert!(world.wall_tiles.is_empty());
    }

    #[test]
    fn generated_vertical_wall_bottom_pillar_uses_level1_surface_art_on_platform() {
        let mut world = GeneratedWorld::new(5, 7);
        world.platforms.push(GeneratedPlatform {
            col: 2,
            row: 1,
            cols: 1,
            rows: 4,
            background: BackgroundTile::Water,
            kind: GeneratedPlatformKind::Small,
        });

        assert!(world.add_vertical_wall_run3_centered(2, 3));

        let visual_world = world.to_visual_tile_world_with_shoreline(false);

        assert_eq!(
            visual_world.tiles.foregrounds[visual_world.tiles.index(2, 4)],
            Some(STANDALONE_GRASS_PILLAR_TILE)
        );
    }

    #[test]
    fn generated_wall_tile_uses_own_foundation_art_not_tile_below() {
        let mut world = GeneratedWorld::new(5, 4);
        for col in 1..=3 {
            let index = world.index(col, 2);
            world.cells[index] = GeneratedCell::Water;
        }
        assert!(world.add_wall_run3_centered(2, 1));

        let visual_world = world.to_visual_tile_world_with_shoreline(false);

        for (offset, col) in (1..=3).enumerate() {
            let cap_index = visual_world.tiles.index(col, 1);
            assert_eq!(
                visual_world.tiles.backgrounds[cap_index],
                BackgroundTile::Grass
            );
            assert_eq!(
                visual_world.tiles.foregrounds[cap_index],
                Some(PLATFORM_GRASS_ROW_TILES[offset])
            );
        }
    }

    #[test]
    fn generated_wall_tile_uses_own_foundation_over_platform_below() {
        let mut world = GeneratedWorld::new(5, 4);
        world.platforms.push(GeneratedPlatform {
            col: 1,
            row: 2,
            cols: 3,
            rows: 1,
            background: BackgroundTile::Water,
            kind: GeneratedPlatformKind::Small,
        });

        assert!(world.add_wall_run3_centered(2, 1));

        let visual_world = world.to_visual_tile_world_with_shoreline(false);

        for (offset, col) in (1..=3).enumerate() {
            let cap_index = visual_world.tiles.index(col, 1);
            assert_eq!(
                visual_world.tiles.foregrounds[cap_index],
                Some(PLATFORM_GRASS_ROW_TILES[offset])
            );
        }
    }

    #[test]
    fn generated_wall_tile_uses_level1_surface_art_on_platform() {
        let mut world = GeneratedWorld::new(5, 6);
        world.platforms.push(GeneratedPlatform {
            col: 1,
            row: 1,
            cols: 3,
            rows: 3,
            background: BackgroundTile::Water,
            kind: GeneratedPlatformKind::Small,
        });

        assert!(world.add_wall_run3_centered(2, 2));

        let visual_world = world.to_visual_tile_world_with_shoreline(false);

        for (offset, col) in (1..=3).enumerate() {
            let cap_index = visual_world.tiles.index(col, 2);
            assert_eq!(
                visual_world.tiles.foregrounds[cap_index],
                Some(PLATFORM_GRASS_ROW_TILES[offset])
            );
        }
    }

    #[test]
    fn generated_wall_run_allows_existing_pillar_next_to_side() {
        let mut world = GeneratedWorld::new(6, 4);
        world.platforms.push(GeneratedPlatform {
            col: 0,
            row: 0,
            cols: 1,
            rows: 1,
            background: BackgroundTile::Grass,
            kind: GeneratedPlatformKind::Small,
        });

        assert!(world.add_wall_run3_centered(2, 1));
        assert_eq!(world.wall_tiles.len(), 1);
    }

    #[test]
    fn generated_wall_run_rejects_existing_pillar_inside_columns() {
        let mut world = GeneratedWorld::new(6, 4);
        world.platforms.push(GeneratedPlatform {
            col: 1,
            row: 0,
            cols: 1,
            rows: 1,
            background: BackgroundTile::Grass,
            kind: GeneratedPlatformKind::Small,
        });

        assert!(!world.add_wall_run3_centered(2, 1));
        assert!(world.wall_tiles.is_empty());
    }

    #[test]
    fn single_pillar_run_can_use_standalone_pillar() {
        let mut world = TileWorld::new(3, 2);
        let mut mask = vec![None; 6];
        mask[world.index(0, 0)] = Some(BackgroundTile::Grass);
        mask[world.index(1, 0)] = Some(BackgroundTile::Grass);
        mask[world.index(0, 1)] = Some(BackgroundTile::Grass);

        tile_platform_bottom_pillars(&mut world, &mask);

        assert_eq!(
            world.foregrounds[world.index(1, 1)],
            Some(STANDALONE_GRASS_PILLAR_TILE)
        );
    }

    #[test]
    fn generated_wall_run_rejects_mixed_level_foundations() {
        let mut world = GeneratedWorld::new(5, 3);
        world.platforms.push(GeneratedPlatform {
            col: 2,
            row: 1,
            cols: 2,
            rows: 1,
            background: BackgroundTile::Grass,
            kind: GeneratedPlatformKind::Small,
        });

        assert!(!world.add_wall_run3_centered(2, 1));
        assert!(world.wall_tiles.is_empty());
    }

    #[test]
    fn single_pillar_run_falls_back_to_visible_source_side_end_cap() {
        let mut world = TileWorld::new(3, 3);
        let mut mask = vec![None; 9];
        mask[world.index(0, 1)] = Some(BackgroundTile::Grass);
        mask[world.index(1, 1)] = Some(BackgroundTile::Grass);
        mask[world.index(0, 2)] = Some(BackgroundTile::Grass);

        tile_platform_bottom_pillars(&mut world, &mask);

        assert_eq!(
            world.foregrounds[world.index(1, 2)],
            Some(PLATFORM_GRASS_ROW_TILES[2])
        );
    }

    #[test]
    fn two_pillar_run_can_use_standalone_pair() {
        let mut world = TileWorld::new(3, 4);
        let mut mask = vec![None; 12];
        mask[world.index(1, 2)] = Some(BackgroundTile::Grass);
        mask[world.index(2, 2)] = Some(BackgroundTile::Grass);

        tile_platform_bottom_pillars(&mut world, &mask);

        assert_eq!(
            world.foregrounds[world.index(1, 3)],
            Some(STANDALONE_GRASS_PILLAR_TILE)
        );
        assert_eq!(
            world.foregrounds[world.index(2, 3)],
            Some(STANDALONE_GRASS_PILLAR_TILE)
        );
    }

    #[test]
    fn two_pillar_run_falls_back_to_end_caps() {
        let mut world = TileWorld::new(3, 3);
        let mut mask = vec![None; 9];
        mask[world.index(0, 1)] = Some(BackgroundTile::Grass);
        mask[world.index(1, 1)] = Some(BackgroundTile::Grass);

        tile_platform_bottom_pillars(&mut world, &mask);

        assert_eq!(
            world.foregrounds[world.index(0, 2)],
            Some(PLATFORM_GRASS_ROW_TILES[0])
        );
        assert_eq!(
            world.foregrounds[world.index(1, 2)],
            Some(PLATFORM_GRASS_ROW_TILES[2])
        );
    }
}
