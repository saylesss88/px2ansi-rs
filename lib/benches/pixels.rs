use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

use px2ansi::simd::{compute_charset_indices, find_luma_range_rgba_bytes, luma_scalar};

// Synthetic RGBA buffers — generated once, reused across all benchmarks.

// Using a deterministic pattern so results are reproducible.

//

fn make_rgba_buf(num_pixels: usize) -> Vec<u8> {
    (0..num_pixels)
        .flat_map(|i| {
            let v = u8::try_from(i % 256).expect("i % 256 always fits in u8");

            // Mix of opaque and transparent pixels (roughly 80% opaque)

            let alpha = if i % 5 == 0 { 0u8 } else { 255u8 };

            [v, v.wrapping_add(30), v.wrapping_add(60), alpha]
        })
        .collect()
}

fn make_chunk() -> [u8; 32] {
    let buf = make_rgba_buf(8);

    buf.try_into().unwrap_or([0u8; 32])
}

// --- find_luma_range_rgba_bytes ---

fn bench_luma_range(c: &mut Criterion) {
    let mut group = c.benchmark_group("find_luma_range_rgba_bytes");

    for num_pixels in [256usize, 1024, 4096, 65536] {
        let buf = make_rgba_buf(num_pixels);

        // Report throughput in bytes so Criterion shows MB/s

        group.throughput(Throughput::Bytes(buf.len() as u64));

        group.bench_with_input(BenchmarkId::from_parameter(num_pixels), &buf, |b, buf| {
            b.iter(|| find_luma_range_rgba_bytes(std::hint::black_box(buf)));
        });
    }

    group.finish();
}

// --- compute_charset_indices ---

fn bench_compute_charset_indices(c: &mut Criterion) {
    let chunk = make_chunk();

    let luma_min = 10u32;

    let luma_range = 200u32;

    let num_chars_minus_1 = 91u32; // typical ASCII charset size - 1

    c.bench_function("compute_charset_indices", |b| {
        b.iter(|| {
            compute_charset_indices(
                std::hint::black_box(&chunk),
                std::hint::black_box(luma_min),
                std::hint::black_box(luma_range),
                std::hint::black_box(num_chars_minus_1),
            )
        });
    });
}

// --- luma_scalar (baseline scalar cost) ---

fn bench_luma_scalar(c: &mut Criterion) {
    c.bench_function("luma_scalar", |b| {
        b.iter(|| {
            luma_scalar(
                std::hint::black_box(123),
                std::hint::black_box(200),
                std::hint::black_box(45),
            )
        });
    });
}

criterion_group!(
    benches,
    bench_luma_range,
    bench_compute_charset_indices,
    bench_luma_scalar,
);

criterion_main!(benches);
