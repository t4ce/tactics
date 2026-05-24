use super::{BackgroundTile, Brush, RAMP_A, RAMP_B, SeededRng, TileWorld};

pub(super) const GENERATED_RAMP_COUNT: usize = 8;

pub(super) fn generate_world(cols: usize, rows: usize, seed: u64) -> TileWorld {
    let mut world = TileWorld::new(cols, rows);
    let mut rng = SeededRng::new(seed);
    let blob_count = 5 + rng.range_usize(0, 5);

    for _ in 0..blob_count {
        let center_col = rng.range_f32(4.0, cols as f32 - 4.0);
        let center_row = rng.range_f32(4.0, rows as f32 - 4.0);
        let radius_x = rng.range_f32(3.5, 10.0);
        let radius_y = rng.range_f32(2.5, 7.0);
        let wobble = rng.range_f32(-0.16, 0.08);

        for row in 0..rows {
            for col in 0..cols {
                let dx = (col as f32 + 0.5 - center_col) / radius_x;
                let dy = (row as f32 + 0.5 - center_row) / radius_y;
                let noise = seeded_cell_noise(seed, col, row) * 0.28;
                if dx * dx + dy * dy + noise + wobble < 1.0 {
                    world.set_background(col, row, BackgroundTile::Water);
                }
            }
        }
    }

    world.collapse_shorelines();
    scatter_ramps(&mut world, &mut rng, GENERATED_RAMP_COUNT);
    world
}

fn scatter_ramps(world: &mut TileWorld, rng: &mut SeededRng, count: usize) {
    let mut placed = 0;
    let max_attempts = count * 80;

    for _ in 0..max_attempts {
        if placed >= count || world.cols == 0 || world.rows < 2 {
            break;
        }

        let col = rng.range_usize(0, world.cols);
        let row = rng.range_usize(0, world.rows - 1);
        if !world.can_place_ramp(col, row) {
            continue;
        }

        let ramp = if rng.next_u64() & 1 == 0 {
            RAMP_A
        } else {
            RAMP_B
        };
        world.paint(col, row, Brush::Ramp(ramp));
        placed += 1;
    }
}

fn seeded_cell_noise(seed: u64, col: usize, row: usize) -> f32 {
    let mut rng = SeededRng::new(
        seed ^ ((col as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
            ^ ((row as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9)),
    );
    rng.range_f32(-1.0, 1.0)
}
