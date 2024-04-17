use criterion::{black_box, criterion_group, criterion_main, Criterion};
use life_rust::game_impls::cpu;

fn run_life(size: usize, iters: usize) {
    let mut game = cpu::GameState::from_random(size);
    for _ in 0..iters {
        game = cpu::GameState::from_previous(&game)
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("life 64x64", |b| {
        b.iter(|| run_life(black_box(64), black_box(64)))
    });
    c.bench_function("life 128x64", |b| {
        b.iter(|| run_life(black_box(128), black_box(64)))
    });
    c.bench_function("life 512x64", |b| {
        b.iter(|| run_life(black_box(512), black_box(64)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
