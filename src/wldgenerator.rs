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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum GeneratedCell {
    Unknown,
    Grass,
    Water,
}

impl GeneratedCell {
    pub(super) fn background(self) -> Option<BackgroundTile> {
        match self {
            Self::Unknown => None,
            Self::Grass => Some(BackgroundTile::Grass),
            Self::Water => Some(BackgroundTile::Water),
        }
    }
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

        let platform_seed_tiles = self.platform_seed_tile_mask();
        let mut visible = vec![false; self.cells.len()];
        for (index, cell) in self.cells.iter().enumerate() {
            visible[index] = *cell != GeneratedCell::Unknown || platform_seed_tiles[index];
        }

        GeneratedVisualWorld { tiles, visible }
    }

    pub(super) fn platform_seed_tile_mask(&self) -> Vec<bool> {
        platform_seed_mask(self)
            .into_iter()
            .map(|background| background.is_some())
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

    fn finish(mut self) -> GeneratedWorld {
        while !self.is_complete() {
            if self.step(1024) == 0 {
                break;
            }
        }
        self.world
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

pub(super) fn generate_world(cols: usize, rows: usize, seed: u64) -> GeneratedWorld {
    RunningGenerator::new(cols, rows, seed).finish()
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

fn stamp_platform_background(world: &mut GeneratedWorld, platform: GeneratedPlatform) {
    let cell = match platform.background {
        BackgroundTile::Grass => GeneratedCell::Grass,
        BackgroundTile::Water => GeneratedCell::Water,
    };

    for (col, row) in platform_seed_cells(world.cols, world.rows, platform) {
        let index = world.index(col, row);
        world.cells[index] = cell;
    }
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

fn platform_seed_cells(
    world_cols: usize,
    world_rows: usize,
    platform: GeneratedPlatform,
) -> Vec<(usize, usize)> {
    let mut cells = Vec::new();
    let start_col = platform.col.min(world_cols);
    let start_row = platform.row.min(world_rows);
    let end_col = platform.col.saturating_add(platform.cols).min(world_cols);
    let end_row = platform.row.saturating_add(platform.rows).min(world_rows);
    if start_col >= end_col || start_row >= end_row {
        return cells;
    }

    for row in start_row..end_row {
        for col in start_col..end_col {
            if row == start_row || row + 1 == end_row || col == start_col || col + 1 == end_col {
                cells.push((col, row));
            }
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

pub(super) fn apply_platform_rect(
    world: &mut TileWorld,
    col: usize,
    row: usize,
    cols: usize,
    rows: usize,
    background: BackgroundTile,
) {
    apply_platform_rects(world, [(col, row, cols, rows, background)]);
}

pub(super) fn shoreline_replace_final(world: &mut TileWorld) {
    let backgrounds = world.backgrounds.clone();
    for row in 0..world.rows {
        for col in 0..world.cols {
            let index = world.index(col, row);
            if backgrounds[index] != BackgroundTile::Grass || world.foregrounds[index].is_some() {
                continue;
            }
            if let Some(tile) = final_shore_tile(&backgrounds, world.cols, world.rows, col, row) {
                world.set_background(col, row, BackgroundTile::Water);
                world.foregrounds[index] = Some(tile);
            }
        }
    }
}

fn final_shore_tile(
    backgrounds: &[BackgroundTile],
    cols: usize,
    rows: usize,
    col: usize,
    row: usize,
) -> Option<AtlasTile> {
    let water_n = final_neighbor_is_water_or_edge(backgrounds, cols, rows, col, row, 0, -1);
    let water_e = final_neighbor_is_water_or_edge(backgrounds, cols, rows, col, row, 1, 0);
    let water_s = final_neighbor_is_water_or_edge(backgrounds, cols, rows, col, row, 0, 1);
    let water_w = final_neighbor_is_water_or_edge(backgrounds, cols, rows, col, row, -1, 0);

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

    backgrounds[next_row as usize * cols + next_col as usize] == BackgroundTile::Water
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

fn standalone_pillar_tile(background: BackgroundTile) -> AtlasTile {
    match background {
        BackgroundTile::Grass => STANDALONE_GRASS_PILLAR_TILE,
        BackgroundTile::Water => STANDALONE_WATER_PILLAR_TILE,
    }
}

pub(super) fn apply_platform_rects(
    world: &mut TileWorld,
    platforms: impl IntoIterator<Item = (usize, usize, usize, usize, BackgroundTile)>,
) {
    apply_platform_rects_with_interior_fill(world, platforms, true);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_world_places_large_platform_prestep_rectangles_when_the_map_has_room() {
        let generator = RunningGenerator::new(48, 29, 123);
        let world = generator.world();

        let large_platforms = world
            .platforms
            .iter()
            .filter(|platform| platform.kind == GeneratedPlatformKind::Large)
            .collect::<Vec<_>>();
        assert_eq!(large_platforms.len(), LARGE_PLATFORM_COUNT);
        let platform_seed_mask = platform_seed_mask(world);
        let mut seed_mask = platform_seed_mask
            .iter()
            .map(Option::is_some)
            .collect::<Vec<_>>();
        for row in 0..world.rows {
            for col in 0..world.cols {
                let Some(background) = platform_seed_mask[world.index(col, row)] else {
                    continue;
                };
                let expected = match background {
                    BackgroundTile::Grass => GeneratedCell::Grass,
                    BackgroundTile::Water => GeneratedCell::Water,
                };
                assert_eq!(world.cell(col, row), expected);
            }
        }
        for &cross in &world.terrain_crosses {
            for (col, row) in terrain_cross_cells(cross) {
                seed_mask[world.index(col, row)] = true;
            }
        }

        for platform in large_platforms {
            assert!((6..=10).contains(&platform.cols));
            assert!((6..=10).contains(&platform.rows));
            assert!(platform.col + platform.cols <= world.cols);
            assert!(platform.row + platform.rows <= world.rows);

            let mut saw_unknown_interior = false;
            for row in platform.row + 1..platform.row + platform.rows - 1 {
                for col in platform.col + 1..platform.col + platform.cols - 1 {
                    if !seed_mask[world.index(col, row)] {
                        assert_eq!(world.cell(col, row), GeneratedCell::Unknown);
                        saw_unknown_interior = true;
                    }
                }
            }
            assert!(saw_unknown_interior);
        }
    }

    #[test]
    fn terrain_crosses_can_spawn_inside_platforms_but_not_near_platform_frame() {
        let mut world = GeneratedWorld::new(12, 12);
        let platform = GeneratedPlatform {
            col: 1,
            row: 1,
            cols: 7,
            rows: 9,
            background: BackgroundTile::Grass,
            kind: GeneratedPlatformKind::Large,
        };
        stamp_platform_background(&mut world, platform);
        world.platforms.push(platform);

        let occupied = platform_frame_buffer_mask(&world);
        let inside_platform = GeneratedTerrainCross {
            col: 4,
            center_row: 5,
            background: BackgroundTile::Water,
        };
        let near_top_frame = GeneratedTerrainCross {
            col: 4,
            center_row: 2,
            background: BackgroundTile::Water,
        };
        let near_side_frame = GeneratedTerrainCross {
            col: 2,
            center_row: 5,
            background: BackgroundTile::Water,
        };
        let near_bottom_pillars = GeneratedTerrainCross {
            col: 4,
            center_row: 9,
            background: BackgroundTile::Water,
        };

        assert!(terrain_cross_fits(&world, &occupied, inside_platform));
        assert!(!terrain_cross_fits(&world, &occupied, near_top_frame));
        assert!(!terrain_cross_fits(&world, &occupied, near_side_frame));
        assert!(!terrain_cross_fits(&world, &occupied, near_bottom_pillars));
        for row in 0..=2 {
            for col in 0..=8 {
                assert!(occupied[world.index(col, row)]);
            }
        }
        for row in 8..=11 {
            for col in 0..=8 {
                assert!(occupied[world.index(col, row)]);
            }
        }
        for row in 0..=11 {
            for col in [0, 1, 2, 7, 8] {
                assert!(occupied[world.index(col, row)]);
            }
        }
    }

    #[test]
    fn feature_reroll_mask_expands_touched_platform_to_full_footprint() {
        let mut world = GeneratedWorld::new(7, 7);
        let platform = GeneratedPlatform {
            col: 1,
            row: 1,
            cols: 3,
            rows: 3,
            background: BackgroundTile::Grass,
            kind: GeneratedPlatformKind::Large,
        };
        world.platforms.push(platform);
        stamp_platform_backgrounds(&mut world);

        let mask = world.feature_reroll_mask_for_rect(2, 2, 1, 1);

        for row in 1..=4 {
            for col in 1..=3 {
                assert!(mask[world.index(col, row)]);
            }
        }
        assert!(!mask[world.index(0, 0)]);
        assert!(!mask[world.index(4, 4)]);
    }

    #[test]
    fn feature_reroll_keeps_touched_platforms_and_crosses_as_seed_features() {
        let mut world = GeneratedWorld::new(9, 9);
        world.cells.fill(GeneratedCell::Grass);
        world.platforms.push(GeneratedPlatform {
            col: 1,
            row: 1,
            cols: 4,
            rows: 4,
            background: BackgroundTile::Grass,
            kind: GeneratedPlatformKind::Large,
        });
        world.terrain_crosses.push(GeneratedTerrainCross {
            col: 7,
            center_row: 4,
            background: BackgroundTile::Water,
        });
        stamp_platform_backgrounds(&mut world);
        stamp_terrain_cross_background(
            &mut world,
            GeneratedTerrainCross {
                col: 7,
                center_row: 4,
                background: BackgroundTile::Water,
            },
        );

        let platform_mask = world.feature_reroll_mask_for_rect(2, 2, 1, 1);
        let cross_mask = world.feature_reroll_mask_for_rect(7, 3, 1, 1);
        let reroll_mask = platform_mask
            .into_iter()
            .zip(cross_mask)
            .map(|(a, b)| a || b)
            .collect::<Vec<_>>();

        world.reroll_features_in_mask(&reroll_mask, 7);

        assert!(world.platforms.iter().any(|platform| {
            platform.kind == GeneratedPlatformKind::Large
                && platform_footprint_intersects_mask(&world, *platform, &reroll_mask)
        }));
        assert!(
            world
                .terrain_crosses
                .iter()
                .any(|&cross| terrain_cross_intersects_mask(&world, cross, &reroll_mask))
        );
    }

    #[test]
    fn shoreline_replace_final_maps_cardinal_water_patterns_to_shore_tiles() {
        let patterns = [
            ((true, false, false, false), SHORE_TOP),
            ((false, true, false, false), SHORE_RIGHT),
            ((false, false, true, false), SHORE_BOTTOM),
            ((false, false, false, true), SHORE_LEFT),
            ((true, true, false, false), SHORE_TOP_RIGHT),
            ((false, true, true, false), SHORE_BOTTOM_RIGHT),
            ((false, false, true, true), SHORE_BOTTOM_LEFT),
            ((true, false, false, true), SHORE_TOP_LEFT),
            ((true, false, true, false), SHORE_NARROW_CENTER),
            ((false, true, false, true), SHORE_NARROW_MIDDLE),
            ((true, true, true, false), SHORE_NARROW_RIGHT),
            ((false, true, true, true), SHORE_NARROW_BOTTOM),
            ((true, false, true, true), SHORE_NARROW_LEFT),
            ((true, true, false, true), SHORE_NARROW_TOP),
            ((true, true, true, true), SHORE_SINGLE_IN_WATER),
        ];

        for (water_sides, expected_tile) in patterns {
            let mut world = TileWorld::new(5, 5);
            let (water_n, water_e, water_s, water_w) = water_sides;
            for (is_water, col, row) in [
                (water_n, 2, 1),
                (water_e, 3, 2),
                (water_s, 2, 3),
                (water_w, 1, 2),
            ] {
                if is_water {
                    world.set_background(col, row, BackgroundTile::Water);
                }
            }

            shoreline_replace_final(&mut world);

            assert_eq!(world.background(2, 2), BackgroundTile::Water);
            assert_eq!(world.foreground(2, 2), Some(expected_tile));
        }
    }

    #[test]
    fn shoreline_replace_final_skips_full_grass_and_existing_foregrounds() {
        let mut world = TileWorld::new(5, 5);
        let foreground_index = world.index(3, 3);
        world.foregrounds[foreground_index] = Some(CLIFF_GRASS_CAP_TILE);

        shoreline_replace_final(&mut world);

        assert_eq!(world.background(2, 2), BackgroundTile::Grass);
        assert_eq!(world.foreground(2, 2), None);
        assert_eq!(world.background(3, 3), BackgroundTile::Grass);
        assert_eq!(world.foreground(3, 3), Some(CLIFF_GRASS_CAP_TILE));
    }

    #[test]
    fn platform_seed_only_places_frame_and_bottom_pillar_row() {
        let mut world = GeneratedWorld::new(7, 7);
        let platform = GeneratedPlatform {
            col: 1,
            row: 1,
            cols: 5,
            rows: 5,
            background: BackgroundTile::Water,
            kind: GeneratedPlatformKind::Large,
        };
        world.platforms.push(platform);

        stamp_platform_backgrounds(&mut world);

        assert_eq!(world.cell(1, 1), GeneratedCell::Water);
        assert_eq!(world.cell(3, 6), GeneratedCell::Water);
        assert_eq!(world.cell(2, 2), GeneratedCell::Unknown);
        assert_eq!(world.cell(3, 3), GeneratedCell::Unknown);
    }

    #[test]
    fn running_generator_starts_with_platform_and_terrain_cross_cells_as_seed_when_the_map_has_room()
     {
        let generator = RunningGenerator::new(48, 29, 123);
        let world = generator.world();
        let mut seed_mask = vec![false; world.cells.len()];
        for (index, background) in platform_seed_mask(world).into_iter().enumerate() {
            seed_mask[index] = background.is_some();
        }
        for &cross in &world.terrain_crosses {
            for (col, row) in terrain_cross_cells(cross) {
                seed_mask[world.index(col, row)] = true;
            }
        }
        let seed_area = seed_mask.iter().filter(|&&covered| covered).count();
        let known_cells = world
            .cells
            .iter()
            .filter(|&&cell| cell != GeneratedCell::Unknown)
            .count();

        assert!(!world.platforms.is_empty());
        assert_eq!(known_cells, seed_area);
    }

    #[test]
    fn seeded_world_places_terrain_crosses_as_upfront_crosses() {
        let generator = RunningGenerator::new(48, 29, 123);
        let world = generator.world();

        assert!(!world.terrain_crosses.is_empty());
        for &cross in &world.terrain_crosses {
            assert!(cross.col > 0);
            assert!(cross.center_row > 0);
            assert!(cross.col + 1 < world.cols);
            assert!(cross.center_row + 1 < world.rows);
            let expected = match cross.background {
                BackgroundTile::Grass => GeneratedCell::Grass,
                BackgroundTile::Water => GeneratedCell::Water,
            };

            for (col, row) in terrain_cross_cells(cross) {
                assert_eq!(world.cell(col, row), expected);
            }
        }
    }

    #[test]
    fn seeded_world_places_four_to_six_small_platforms() {
        let world = generate_world(48, 29, 123);
        let small_platforms = world
            .platforms
            .iter()
            .filter(|platform| platform.kind == GeneratedPlatformKind::Small)
            .collect::<Vec<_>>();

        assert!(
            (SMALL_PLATFORM_MIN_COUNT..=SMALL_PLATFORM_MAX_COUNT).contains(&small_platforms.len())
        );
        for platform in small_platforms {
            assert!((SMALL_PLATFORM_MIN_SIZE..=SMALL_PLATFORM_MAX_SIZE).contains(&platform.cols));
            assert!((SMALL_PLATFORM_MIN_SIZE..=SMALL_PLATFORM_MAX_SIZE).contains(&platform.rows));
            assert!(platform.col + platform.cols <= world.cols);
            assert!(platform.row + platform.rows <= world.rows);
        }
    }

    #[test]
    fn each_large_platform_gets_one_centered_border_overlap_small_platform() {
        let world = generate_world(48, 29, 123);
        let large_platforms = world
            .platforms
            .iter()
            .copied()
            .filter(|platform| platform.kind == GeneratedPlatformKind::Large)
            .collect::<Vec<_>>();
        let small_platforms = world
            .platforms
            .iter()
            .copied()
            .filter(|platform| platform.kind == GeneratedPlatformKind::Small)
            .collect::<Vec<_>>();

        for large in large_platforms {
            let attached_count = small_platforms
                .iter()
                .filter(|&&small| {
                    platforms_overlap(large, small) && platform_center_on_border_of(large, small)
                })
                .count();
            assert_eq!(attached_count, 1);
        }
    }

    #[test]
    fn platform_prestep_only_overlaps_large_platforms_with_their_attached_small_platforms() {
        let world = generate_world(48, 29, 123);

        for (index, &platform) in world.platforms.iter().enumerate() {
            for &other in &world.platforms[index + 1..] {
                if platforms_overlap(platform, other) {
                    assert_ne!(platform.kind, other.kind);
                    assert!(
                        platform_center_on_border_of(platform, other)
                            || platform_center_on_border_of(other, platform)
                    );
                    continue;
                }

                assert!(!platforms_overlap_with_margin(platform, other, 1));
                assert!(!platforms_overlap_with_margin(other, platform, 1));
            }
        }
    }
}
