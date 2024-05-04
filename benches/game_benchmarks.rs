use std::sync::Arc;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use life_rust::game_impls::{compute, cpu, cpu_ndarray};
use life_rust::{Gol, VulkanContext};

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

fn run_life_compute(size: usize, iters: usize, vulkan_context: Arc<VulkanContext>) {
    let mut game: Box<dyn Gol> = Box::new(compute::GameState::from_random(vulkan_context, size));
    for _ in 0..iters {
        game = game.to_next();
    }
}

pub fn cpu_benchmark(c: &mut Criterion) {
    c.bench_function("life 128x64", |b| {
        b.iter(|| run_life(black_box(128), black_box(64)))
    });
}

pub fn cpu_ndarray_benchmark(c: &mut Criterion) {
    c.bench_function("life 128x64 (ndarray)", |b| {
        b.iter(|| run_life_ndarray(black_box(128), black_box(64)))
    });
}

pub fn compute_benchmark(c: &mut Criterion) {
    let context = Arc::new(life_rust::VulkanContext::try_create().unwrap());

    c.bench_with_input(
        BenchmarkId::new("life 128x64 (compute)", &context),
        &context.clone(),
        |b, c| b.iter(|| run_life_compute(black_box(128), black_box(64), c.clone())),
    );
}

criterion_group!(
    benches,
    cpu_benchmark,
    cpu_ndarray_benchmark,
    compute_benchmark
);
criterion_main!(benches);
