use criterion::{black_box, criterion_group, criterion_main, Criterion};
use life_rust::game_impls::cpu;

fn run_life(size: usize, iters: usize, parallel: bool) {
    let mut game = cpu::GameState::from_random(size);
    for _ in 0..iters {
        if parallel {
            game = cpu::GameState::from_previous_parallel(&game, 4)
        } else {
            game = cpu::GameState::from_previous(&game)
        }
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("life 128x64", |b| {
        b.iter(|| run_life(black_box(128), black_box(64), false))
    });
    c.bench_function("life 512x64", |b| {
        b.iter(|| run_life(black_box(512), black_box(64), false))
    });
    c.bench_function("life128x64p", |b| {
        b.iter(|| run_life(black_box(128), black_box(64), true))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
