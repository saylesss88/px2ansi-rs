#![allow(clippy::panic, clippy::unwrap_used)]

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use image::open;

use std::hint::black_box;
use std::io::sink;
use std::path::PathBuf;

use px2ansi::{RenderOptions, RenderStylePreset, ResizeFilter};

fn get_asset_path(filename: &str) -> PathBuf {
    // Start from the library directory and go up to project root
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // Up to px2ansi-rs/
    path.push("tests");
    path.push(filename);
    path
}

fn bench_performance_features(c: &mut Criterion) {
    let mut group = c.benchmark_group("Engine Performance");

    // Load actual asset
    let nixos_path = get_asset_path("nixos.png");
    let img = open(&nixos_path)
        .unwrap_or_else(|_| panic!("Failed to load asset at {}", nixos_path.display()));

    let configs = [
        (
            "Fastest_Nearest",
            RenderOptions::builder()
                .preset(RenderStylePreset::Ansi)
                .filter(ResizeFilter::Nearest)
                .width(100)
                .build(),
        ),
        (
            "HighQuality_Lanczos3",
            RenderOptions::builder()
                .preset(RenderStylePreset::Ansi)
                .filter(ResizeFilter::Lanczos3)
                .width(100)
                .build(),
        ),
    ];

    for (label, opts) in configs {
        group.bench_with_input(BenchmarkId::new("Configuration", label), &img, |b, i| {
            b.iter(|| {
                let mut writer = sink();
                // Using std::hint::black_box here
                opts.render(black_box(i), &mut writer).unwrap();
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_performance_features);
criterion_main!(benches);
