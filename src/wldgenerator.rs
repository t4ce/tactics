use super::{
    AtlasTile, BackgroundTile, SHORE_BOTTOM, SHORE_BOTTOM_LEFT, SHORE_BOTTOM_RIGHT, SHORE_LEFT,
    SHORE_NARROW_BOTTOM, SHORE_NARROW_CENTER, SHORE_NARROW_LEFT, SHORE_NARROW_MIDDLE,
    SHORE_NARROW_RIGHT, SHORE_NARROW_TOP, SHORE_RIGHT, SHORE_SINGLE_IN_WATER, SHORE_TOP,
    SHORE_TOP_LEFT, SHORE_TOP_RIGHT, SeededRng, TILE_SIZE, TileWorld,
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

pub(super) struct GeneratedVisualWorld {
    pub(super) tiles: TileWorld,
    pub(super) visible: Vec<bool>,
    pub(super) levels: Vec<GeneratedTileLevel>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum GeneratedTileLevel {
    Ground,
    Platform,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct GeneratedWorld {
    pub(super) cols: usize,
    pub(super) rows: usize,
    pub(super) cells: Vec<GeneratedCell>,
    pub(super) platforms: Vec<GeneratedPlatform>,
    pub(super) terrain_crosses: Vec<GeneratedTerrainCross>,
}

impl GeneratedWorld {
    fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols,
            rows,
            cells: vec![GeneratedCell::Unknown; cols.saturating_mul(rows)],
            platforms: Vec::new(),
            terrain_crosses: Vec::new(),
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
        let mut tiles = self.to_tile_world();
        apply_generated_platforms(&mut tiles, self);
        apply_generated_terrain_crosses(&mut tiles, self);
        let levels = generated_tile_levels(self);

        let platform_seed_mask = platform_seed_mask(self);
        let platform_seed_tiles = platform_seed_mask
            .iter()
            .map(|background| background.is_some())
            .collect::<Vec<_>>();
        if self
            .cells
            .iter()
            .any(|&cell| cell == GeneratedCell::Unknown)
        {
            apply_generated_platform_seed_frame(&mut tiles, &platform_seed_mask);
            apply_generated_terrain_crosses(&mut tiles, self);
        }
        let mut visible = vec![false; self.cells.len()];
        for (index, cell) in self.cells.iter().enumerate() {
            visible[index] = *cell != GeneratedCell::Unknown || platform_seed_tiles[index];
        }

        GeneratedVisualWorld {
            tiles,
            visible,
            levels,
        }
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
    buckets: [Vec<usize>; 5],
    in_frontier: Vec<bool>,
    remaining_unknown: usize,
}

impl RunningGenerator {
    pub(super) fn new(cols: usize, rows: usize, seed: u64) -> Self {
        let mut rng = SeededRng::new(seed);
        let world = seeded_world(cols, rows, &mut rng);
        Self::from_world_with_rng(world, rng)
    }

    pub(super) fn from_world(world: GeneratedWorld, seed: u64) -> Self {
        Self::from_world_with_rng(world, SeededRng::new(seed))
    }

    fn from_world_with_rng(world: GeneratedWorld, rng: SeededRng) -> Self {
        let mut generator = Self {
            remaining_unknown: world
                .cells
                .iter()
                .filter(|&&cell| cell == GeneratedCell::Unknown)
                .count(),
            in_frontier: vec![false; world.cells.len()],
            buckets: std::array::from_fn(|_| Vec::new()),
            world,
            rng,
        };
        generator.rebuild_frontier();
        generator
    }

    pub(super) fn world(&self) -> &GeneratedWorld {
        &self.world
    }

    pub(super) fn is_complete(&self) -> bool {
        self.remaining_unknown == 0
    }

    pub(super) fn step(&mut self, max_cells: usize) -> usize {
        let mut collapsed = 0;
        while collapsed < max_cells && self.remaining_unknown > 0 {
            let Some(index) = next_frontier_cell(
                &self.world,
                &mut self.buckets,
                &mut self.in_frontier,
                &mut self.rng,
            )
            .or_else(|| {
                self.world
                    .cells
                    .iter()
                    .position(|&cell| cell == GeneratedCell::Unknown)
            }) else {
                self.remaining_unknown = 0;
                break;
            };
            self.world.cells[index] = affinity_fitting_cell(&self.world, index, &mut self.rng);
            self.remaining_unknown -= 1;
            collapsed += 1;
            add_unknown_neighbors_to_frontier(
                &self.world,
                index,
                &mut self.buckets,
                &mut self.in_frontier,
            );
        }
        collapsed
    }

    fn rebuild_frontier(&mut self) {
        for index in 0..self.world.cells.len() {
            if self.world.cells[index] != GeneratedCell::Unknown {
                add_unknown_neighbors_to_frontier(
                    &self.world,
                    index,
                    &mut self.buckets,
                    &mut self.in_frontier,
                );
            }
        }
    }
}

fn seeded_world(cols: usize, rows: usize, rng: &mut SeededRng) -> GeneratedWorld {
    let mut world = GeneratedWorld::new(cols, rows);
    let cell_count = world.cells.len();
    if cell_count == 0 {
        return world;
    }

    seed_platforms(&mut world, rng);
    seed_terrain_crosses(&mut world, rng);
    if !world.platforms.is_empty() {
        return world;
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

    world
}

fn seed_platforms(world: &mut GeneratedWorld, rng: &mut SeededRng) {
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

fn generated_tile_levels(world: &GeneratedWorld) -> Vec<GeneratedTileLevel> {
    let mut levels = vec![GeneratedTileLevel::Ground; world.cells.len()];
    for &platform in &world.platforms {
        for (col, row) in platform_footprint_cells(world.cols, world.rows, platform) {
            if col < world.cols && row < world.rows {
                levels[world.index(col, row)] = GeneratedTileLevel::Platform;
            }
        }
    }
    levels
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
    let mask = platform_seed_mask(world);
    for row in 0..world.rows {
        for col in 0..world.cols {
            let Some(background) = mask[world.index(col, row)] else {
                continue;
            };
            let index = world.index(col, row);
            world.cells[index] = match background {
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

pub(super) fn shoreline_replace_final(world: &mut TileWorld, levels: &[GeneratedTileLevel]) {
    let backgrounds = world.backgrounds.clone();
    let foregrounds = world.foregrounds.clone();
    for row in 0..world.rows {
        for col in 0..world.cols {
            let index = world.index(col, row);
            if backgrounds[index] != BackgroundTile::Grass
                || foreground_blocks_final_shore(foregrounds[index])
            {
                continue;
            }
            let use_underlay = foreground_accepts_final_shore_underlay(foregrounds[index]);
            if let Some(tile) = final_shore_tile(
                &backgrounds,
                &foregrounds,
                levels,
                use_underlay,
                world.cols,
                world.rows,
                col,
                row,
            ) {
                world.set_background(col, row, BackgroundTile::Water);
                if use_underlay {
                    world.under_foregrounds[index] = Some(tile);
                } else {
                    world.foregrounds[index] = Some(tile);
                }
            }
        }
    }
    platform_frame_constraint_replace_final(world, levels);
}

fn platform_frame_constraint_replace_final(world: &mut TileWorld, levels: &[GeneratedTileLevel]) {
    let backgrounds = world.backgrounds.clone();
    let foregrounds = world.foregrounds.clone();
    for row in 0..world.rows {
        for col in 0..world.cols {
            let index = world.index(col, row);
            if levels[index] != GeneratedTileLevel::Platform {
                continue;
            }
            if let Some((tile, inward_col, inward_row)) =
                platform_frame_inner_water_replacement(foregrounds[index], col, row)
                && inward_col < world.cols
                && inward_row < world.rows
            {
                let inward_index = world.index(inward_col, inward_row);
                if levels[inward_index] == GeneratedTileLevel::Platform
                    && backgrounds[inward_index] == BackgroundTile::Water
                    && foregrounds[inward_index].is_none()
                {
                    world.foregrounds[index] = Some(tile);
                }
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

fn foreground_blocks_final_shore(foreground: Option<AtlasTile>) -> bool {
    foreground.is_some_and(|tile| !foreground_accepts_final_shore_underlay(Some(tile)))
}

fn foreground_accepts_final_shore_underlay(foreground: Option<AtlasTile>) -> bool {
    foreground.is_some_and(|tile| {
        tile == CLIFF_GRASS_CAP_TILE
            || PLATFORM_GRASS_ROW_TILES.contains(&tile)
            || PLATFORM_WATER_ROW_TILES.contains(&tile)
            || platform_border_tiles_contain(tile)
    })
}

fn platform_border_tiles_contain(tile: AtlasTile) -> bool {
    PLATFORM_GRASS_BORDER_TILES
        .iter()
        .flatten()
        .any(|&border_tile| border_tile == tile)
}

fn final_shore_tile(
    backgrounds: &[BackgroundTile],
    foregrounds: &[Option<AtlasTile>],
    levels: &[GeneratedTileLevel],
    use_ground_underlay: bool,
    cols: usize,
    rows: usize,
    col: usize,
    row: usize,
) -> Option<AtlasTile> {
    let center_level = levels[row * cols + col];
    let water_n = final_neighbor_is_water_or_edge(
        backgrounds,
        foregrounds,
        levels,
        center_level,
        use_ground_underlay,
        cols,
        rows,
        col,
        row,
        0,
        -1,
    );
    let water_e = final_neighbor_is_water_or_edge(
        backgrounds,
        foregrounds,
        levels,
        center_level,
        use_ground_underlay,
        cols,
        rows,
        col,
        row,
        1,
        0,
    );
    let water_s = final_neighbor_is_water_or_edge(
        backgrounds,
        foregrounds,
        levels,
        center_level,
        use_ground_underlay,
        cols,
        rows,
        col,
        row,
        0,
        1,
    );
    let water_w = final_neighbor_is_water_or_edge(
        backgrounds,
        foregrounds,
        levels,
        center_level,
        use_ground_underlay,
        cols,
        rows,
        col,
        row,
        -1,
        0,
    );

    Some(match (water_n, water_e, water_s, water_w) {
        (false, false, false, false) => return None,
        (true, true, false, true) => SHORE_NARROW_TOP,
        (false, true, false, true) => SHORE_NARROW_MIDDLE,
        (false, true, true, true) => SHORE_NARROW_BOTTOM,
        (true, false, true, true) => SHORE_NARROW_LEFT,
        (true, false, true, false) => SHORE_NARROW_CENTER,
        (true, true, true, false) => SHORE_NARROW_RIGHT,
        (true, true, true, true) => SHORE_SINGLE_IN_WATER,
        (true, false, false, true) => SHORE_TOP_LEFT,
        (true, true, false, false) => SHORE_TOP_RIGHT,
        (false, false, true, true) => SHORE_BOTTOM_LEFT,
        (false, true, true, false) => SHORE_BOTTOM_RIGHT,
        (true, _, _, _) => SHORE_TOP,
        (_, _, true, _) => SHORE_BOTTOM,
        (_, true, _, _) => SHORE_RIGHT,
        (_, _, _, true) => SHORE_LEFT,
    })
}

fn final_neighbor_is_water_or_edge(
    backgrounds: &[BackgroundTile],
    foregrounds: &[Option<AtlasTile>],
    levels: &[GeneratedTileLevel],
    center_level: GeneratedTileLevel,
    use_ground_underlay: bool,
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
        return true;
    }

    let index = next_row as usize * cols + next_col as usize;
    if center_level == GeneratedTileLevel::Ground || use_ground_underlay {
        backgrounds[index] == BackgroundTile::Water
    } else {
        levels[index] == center_level
            && backgrounds[index] == BackgroundTile::Water
            && foregrounds[index].is_none()
    }
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

fn apply_generated_platform_seed_frame(
    world: &mut TileWorld,
    platform_seed_mask: &[Option<BackgroundTile>],
) {
    let mut mask = vec![None; world.cols * world.rows];
    for (index, &background) in platform_seed_mask.iter().enumerate() {
        mask[index] = background;
    }
    apply_platform_mask(world, &mask, false);
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
                world.set_background(col, row, background);
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
    row: usize,
    start_col: usize,
    cols: usize,
    background: BackgroundTile,
) {
    for offset in 0..cols {
        let col = start_col + offset;
        let index = world.index(col, row);
        world.foregrounds[index] = Some(platform_row_tile(background, offset, cols));
        world.set_background(col, row, background);
    }
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
                tile_platform_row_run(world, row + 1, start_col, col - start_col, background);
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
    background: BackgroundTile,
    offset: usize,
    cols: usize,
) -> AtlasTile {
    let tiles = match background {
        BackgroundTile::Grass => PLATFORM_GRASS_ROW_TILES,
        BackgroundTile::Water => PLATFORM_WATER_ROW_TILES,
    };

    match (offset, cols) {
        (_, 1) => tiles[1],
        (0, _) => tiles[0],
        (offset, cols) if offset + 1 == cols => tiles[2],
        _ => tiles[1],
    }
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

fn add_unknown_neighbors_to_frontier(
    world: &GeneratedWorld,
    index: usize,
    buckets: &mut [Vec<usize>; 5],
    in_frontier: &mut [bool],
) {
    for neighbor in cardinal_neighbor_indices(world, index) {
        if world.cells[neighbor] != GeneratedCell::Unknown || in_frontier[neighbor] {
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

fn next_frontier_cell(
    world: &GeneratedWorld,
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

        if world.cells[index] != GeneratedCell::Unknown {
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
