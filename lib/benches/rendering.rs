#![allow(clippy::panic, clippy::unwrap_used)]
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use image::open;
use px2ansi::{RenderOptions, RenderStylePreset, ResizeFilter};
use std::hint::black_box;
use std::io::sink;
use std::path::PathBuf;

fn get_asset_path(filename: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.push("tests");
    path.push(filename);
    path
}

fn bench_performance_features(c: &mut Criterion) {
    let mut group = c.benchmark_group("Engine Performance");

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
                opts.render(black_box(i), &mut writer).unwrap();
            });
        });
    }

    group.finish();
}

#[cfg(feature = "sixel")]
fn bench_sixel(c: &mut Criterion) {
    let mut group = c.benchmark_group("Sixel Rendering");

    let nixos_path = get_asset_path("nixos.png");
    let img = open(&nixos_path)
        .unwrap_or_else(|_| panic!("Failed to load asset at {}", nixos_path.display()));

    let pre_resized_160 = img.resize(
        160,
        (img.height() * 160) / img.width().max(1),
        image::imageops::FilterType::Lanczos3,
    );
    let opts_no_width = RenderOptions::builder()
        .preset(RenderStylePreset::Sixel)
        .build();

    group.bench_with_input(
        BenchmarkId::new("Width", "w160_preresized"),
        &pre_resized_160,
        |b, i| {
            b.iter(|| {
                let mut writer = sink();
                opts_no_width.render(black_box(i), &mut writer).unwrap();
            });
        },
    );

    let width_configs = [("w80", 80u32), ("w160", 160), ("w320", 320)];
    for (label, w) in width_configs {
        let opts = RenderOptions::builder()
            .preset(RenderStylePreset::Sixel)
            .filter(ResizeFilter::Lanczos3)
            .width(w)
            .build();
        let prepared = opts.prepare_image(&img);
        group.bench_with_input(BenchmarkId::new("Width", label), &prepared, |b, i| {
            b.iter(|| {
                let mut writer = sink();
                opts.render(black_box(i), &mut writer).unwrap();
            });
        });
    }

    let quant_configs: &[(&str, RenderOptions)] = &[
        (
            "colors256_diffusion",
            RenderOptions::builder()
                .preset(RenderStylePreset::Sixel)
                .filter(ResizeFilter::Nearest)
                .width(160)
                .max_colors(256)
                .diffusion(0.875)
                .build(),
        ),
        (
            "colors128_diffusion",
            RenderOptions::builder()
                .preset(RenderStylePreset::Sixel)
                .filter(ResizeFilter::Nearest)
                .width(160)
                .max_colors(128)
                .diffusion(0.875)
                .build(),
        ),
        (
            "colors256_nodiffusion",
            RenderOptions::builder()
                .preset(RenderStylePreset::Sixel)
                .filter(ResizeFilter::Nearest)
                .width(160)
                .max_colors(256)
                .diffusion(0.0)
                .build(),
        ),
        (
            "colors64_nodiffusion",
            RenderOptions::builder()
                .preset(RenderStylePreset::Sixel)
                .filter(ResizeFilter::Nearest)
                .width(160)
                .max_colors(64)
                .diffusion(0.0)
                .build(),
        ),
    ];
    for (label, opts) in quant_configs {
        group.bench_with_input(BenchmarkId::new("Quantization", label), &img, |b, i| {
            b.iter(|| {
                let mut writer = sink();
                opts.render(black_box(i), &mut writer).unwrap();
            });
        });
    }

    group.finish();
}

#[cfg(not(feature = "sixel"))]
fn bench_sixel(_c: &mut Criterion) {}

criterion_group!(benches, bench_performance_features, bench_sixel);
criterion_main!(benches);
