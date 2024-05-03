use criterion::{black_box, criterion_group, criterion_main, Criterion};
use life_rust::game_impls::{cpu, cpu_ndarray};
use life_rust::Gol;

fn run_life(size: usize, iters: usize) {
    let mut game = cpu::GameState::from_random(size);
    for _ in 0..iters {
        game = game.to_next();
    }
}

fn run_life_ndarray(size: usize, iters: usize) {
    let mut game: Box<dyn Gol> = cpu_ndarray::GameState::from_random(size);
    for _ in 0..iters {
        game = game.to_next();
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("life 128x64", |b| {
        b.iter(|| run_life(black_box(128), black_box(64)))
    });
    // c.bench_function("life 512x64", |b| {
    //     b.iter(|| run_life(black_box(512), black_box(64), false))
    // });
    c.bench_function("life 128x64 (ndarray)", |b| {
        b.iter(|| run_life_ndarray(black_box(128), black_box(64)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
