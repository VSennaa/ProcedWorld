use determinism_hex::{grid_for, World};
use std::time::Instant;

/// Usage: determinism-hex [seed] [turns]. Prints the state hash every 100 turns and at the end,
/// plus timing (timing is informational and never part of the hash).
fn main() {
    let mut args = std::env::args().skip(1);
    let seed: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(20261001);
    let turns: u32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(1000);

    let grid = grid_for(8);
    let started = Instant::now();
    let mut world = World::new(seed, 8, grid);
    println!("grid {}x{} tiles {} civs {}", grid.width, grid.height, grid.len(), world.civs.len());
    println!("turn 0 hash {:016x}", world.hash());
    for _ in 0..turns {
        world.step();
        if world.turn % 100 == 0 {
            println!("turn {} hash {:016x}", world.turn, world.hash());
        }
    }
    let elapsed = started.elapsed();
    println!("FINAL {:016x}", world.hash());
    let owned = world.tiles.iter().filter(|t| t.owner != 0).count();
    let pop: i32 = world.civs.iter().map(|c| c.population).sum();
    println!("owned tiles {owned} total population {pop}");
    eprintln!(
        "elapsed {:.3}s ({:.1} us/turn)",
        elapsed.as_secs_f64(),
        elapsed.as_micros() as f64 / turns.max(1) as f64
    );
}
