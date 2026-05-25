use super::{BackgroundTile, SeededRng, TILE_SIZE, TileWorld};

const INITIAL_SEED_NUMERATOR: usize = 1;
const INITIAL_SEED_DENOMINATOR: usize = 200;
const INITIAL_WATER_WEIGHT: usize = 1;
const INITIAL_GRASS_WEIGHT: usize = 8;
const WATER_WEIGHT: f32 = 0.76;
const GRASS_WEIGHT: f32 = 2.0;
const GRASS_AFFINITY_STEP: f32 = 0.95;
const WATER_AFFINITY_STEP: f32 = 0.95;

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct GeneratedWorld {
    pub(super) cols: usize,
    pub(super) rows: usize,
    pub(super) cells: Vec<GeneratedCell>,
}

impl GeneratedWorld {
    fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols,
            rows,
            cells: vec![GeneratedCell::Unknown; cols.saturating_mul(rows)],
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

    let seed_count = cell_count
        .saturating_mul(INITIAL_SEED_NUMERATOR)
        .div_ceil(INITIAL_SEED_DENOMINATOR)
        .max(1);
    let mut indices = (0..cell_count).collect::<Vec<_>>();
    for offset in 0..seed_count {
        let chosen = rng.range_usize(offset, cell_count);
        indices.swap(offset, chosen);
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
