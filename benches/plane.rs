//! Performance benchmarks for `Plane` operations.

#![allow(missing_docs, reason = "benchmark file")]
#![allow(clippy::unwrap_used, reason = "benchmark file")]

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::num::{NonZeroU8, NonZeroUsize};
use v_frame::{chroma::ChromaSubsampling, frame::FrameBuilder, plane::Plane};

/// Standard HD resolution for benchmarks (1920x1080)
const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;

/// Creates a test plane for 8-bit benchmarks
fn create_plane_u8() -> Plane<u8> {
    let width = NonZeroUsize::new(WIDTH).unwrap();
    let height = NonZeroUsize::new(HEIGHT).unwrap();
    let bit_depth = NonZeroU8::new(8).unwrap();

    let frame = FrameBuilder::new(width, height, ChromaSubsampling::Yuv420, bit_depth)
        .build::<u8>()
        .unwrap();

    frame.y_plane
}

/// Creates a test plane for 10-bit benchmarks (using u16)
fn create_plane_u16() -> Plane<u16> {
    let width = NonZeroUsize::new(WIDTH).unwrap();
    let height = NonZeroUsize::new(HEIGHT).unwrap();
    let bit_depth = NonZeroU8::new(10).unwrap();

    let frame = FrameBuilder::new(width, height, ChromaSubsampling::Yuv420, bit_depth)
        .build::<u16>()
        .unwrap();

    frame.y_plane
}

/// Creates source data for copy benchmarks (u8)
fn create_source_u8() -> Vec<u8> {
    vec![42u8; WIDTH * HEIGHT]
}

/// Creates source data for copy benchmarks (u16)
fn create_source_u16() -> Vec<u16> {
    vec![512u16; WIDTH * HEIGHT]
}

/// Creates byte source data for u8 copy benchmarks
fn create_byte_source_u8() -> Vec<u8> {
    vec![42u8; WIDTH * HEIGHT]
}

/// Creates byte source data for u16 copy benchmarks (little-endian)
fn create_byte_source_u16() -> Vec<u8> {
    let mut bytes = Vec::with_capacity(WIDTH * HEIGHT * 2);
    for _ in 0..WIDTH * HEIGHT {
        bytes.extend_from_slice(&512u16.to_le_bytes());
    }
    bytes
}

/// Creates byte source data with stride for u8 benchmarks
fn create_strided_byte_source_u8(stride: usize) -> Vec<u8> {
    vec![42u8; stride * HEIGHT]
}

/// Creates byte source data with stride for u16 benchmarks
fn create_strided_byte_source_u16(stride: usize) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(stride * HEIGHT * 2);
    for _ in 0..stride * HEIGHT {
        bytes.extend_from_slice(&512u16.to_le_bytes());
    }
    bytes
}

fn bench_pixels_u8(c: &mut Criterion) {
    let plane = create_plane_u8();

    c.bench_function("pixels_u8", |b| {
        b.iter(|| {
            let sum: u64 = black_box(&plane).pixels().map(|p| p as u64).sum();
            black_box(sum);
        });
    });
}

fn bench_pixels_u16(c: &mut Criterion) {
    let plane = create_plane_u16();

    c.bench_function("pixels_u16", |b| {
        b.iter(|| {
            let sum: u64 = black_box(&plane).pixels().map(|p| p as u64).sum();
            black_box(sum);
        });
    });
}

fn bench_byte_data_u8(c: &mut Criterion) {
    let plane = create_plane_u8();

    c.bench_function("byte_data_u8", |b| {
        b.iter(|| {
            let sum: u64 = black_box(&plane).byte_data().map(|b| b as u64).sum();
            black_box(sum);
        });
    });
}

fn bench_byte_data_u16(c: &mut Criterion) {
    let plane = create_plane_u16();

    c.bench_function("byte_data_u16", |b| {
        b.iter(|| {
            let sum: u64 = black_box(&plane).byte_data().map(|b| b as u64).sum();
            black_box(sum);
        });
    });
}

fn bench_copy_from_slice_u8(c: &mut Criterion) {
    let mut plane = create_plane_u8();
    let source = create_source_u8();

    c.bench_function("copy_from_slice_u8", |b| {
        b.iter(|| {
            black_box(&mut plane)
                .copy_from_slice(black_box(&source))
                .unwrap();
        });
    });
}

fn bench_copy_from_slice_u16(c: &mut Criterion) {
    let mut plane = create_plane_u16();
    let source = create_source_u16();

    c.bench_function("copy_from_slice_u16", |b| {
        b.iter(|| {
            black_box(&mut plane)
                .copy_from_slice(black_box(&source))
                .unwrap();
        });
    });
}

fn bench_copy_from_u8_slice_u8(c: &mut Criterion) {
    let mut plane = create_plane_u8();
    let source = create_byte_source_u8();

    c.bench_function("copy_from_u8_slice_u8", |b| {
        b.iter(|| {
            black_box(&mut plane)
                .copy_from_u8_slice(black_box(&source))
                .unwrap();
        });
    });
}

fn bench_copy_from_u8_slice_u16(c: &mut Criterion) {
    let mut plane = create_plane_u16();
    let source = create_byte_source_u16();

    c.bench_function("copy_from_u8_slice_u16", |b| {
        b.iter(|| {
            black_box(&mut plane)
                .copy_from_u8_slice(black_box(&source))
                .unwrap();
        });
    });
}

fn bench_copy_from_u8_slice_with_stride_u8(c: &mut Criterion) {
    let mut plane = create_plane_u8();
    // Add 64 pixels of padding per row for a realistic strided scenario
    let stride = WIDTH + 64;
    let stride_nz = NonZeroUsize::new(stride).unwrap();
    let source = create_strided_byte_source_u8(stride);

    c.bench_function("copy_from_u8_slice_with_stride_u8", |b| {
        b.iter(|| {
            black_box(&mut plane)
                .copy_from_u8_slice_with_stride(black_box(&source), stride_nz)
                .unwrap();
        });
    });
}

fn bench_copy_from_u8_slice_with_stride_u16(c: &mut Criterion) {
    let mut plane = create_plane_u16();
    // Add 64 pixels of padding per row for a realistic strided scenario
    let stride = WIDTH + 64;
    let stride_bytes = NonZeroUsize::new(stride * 2).unwrap();
    let source = create_strided_byte_source_u16(stride);

    c.bench_function("copy_from_u8_slice_with_stride_u16", |b| {
        b.iter(|| {
            black_box(&mut plane)
                .copy_from_u8_slice_with_stride(black_box(&source), stride_bytes)
                .unwrap();
        });
    });
}

criterion_group!(
    benches,
    bench_pixels_u8,
    bench_pixels_u16,
    bench_byte_data_u8,
    bench_byte_data_u16,
    bench_copy_from_slice_u8,
    bench_copy_from_slice_u16,
    bench_copy_from_u8_slice_u8,
    bench_copy_from_u8_slice_u16,
    bench_copy_from_u8_slice_with_stride_u8,
    bench_copy_from_u8_slice_with_stride_u16
);
criterion_main!(benches);
