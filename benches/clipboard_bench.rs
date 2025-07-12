//! Clipboard performance benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use crosscopy::clipboard::ClipboardContent;

fn bench_clipboard_content_creation(c: &mut Criterion) {
    c.bench_function("clipboard_content_text_creation", |b| {
        b.iter(|| {
            let content = ClipboardContent::new_text(
                black_box("Test clipboard content for benchmarking".to_string()),
                black_box("benchmark-device".to_string()),
            );
            black_box(content)
        })
    });
}

fn bench_clipboard_content_serialization(c: &mut Criterion) {
    let content = ClipboardContent::new_text(
        "Test clipboard content for serialization benchmark".to_string(),
        "benchmark-device".to_string(),
    );

    c.bench_function("clipboard_content_serialization", |b| {
        b.iter(|| {
            let serialized = serde_json::to_vec(black_box(&content)).unwrap();
            black_box(serialized)
        })
    });
}

fn bench_clipboard_content_deserialization(c: &mut Criterion) {
    let content = ClipboardContent::new_text(
        "Test clipboard content for deserialization benchmark".to_string(),
        "benchmark-device".to_string(),
    );
    let serialized = serde_json::to_vec(&content).unwrap();

    c.bench_function("clipboard_content_deserialization", |b| {
        b.iter(|| {
            let deserialized: ClipboardContent = serde_json::from_slice(black_box(&serialized)).unwrap();
            black_box(deserialized)
        })
    });
}

fn bench_clipboard_content_integrity_check(c: &mut Criterion) {
    let content = ClipboardContent::new_text(
        "Test clipboard content for integrity check benchmark".to_string(),
        "benchmark-device".to_string(),
    );

    c.bench_function("clipboard_content_integrity_check", |b| {
        b.iter(|| {
            let is_valid = black_box(&content).verify_integrity();
            black_box(is_valid)
        })
    });
}

#[cfg(feature = "compression")]
fn bench_clipboard_content_compression(c: &mut Criterion) {
    let large_text = "A".repeat(10000); // 10KB of text
    let mut content = ClipboardContent::new_text(
        large_text,
        "benchmark-device".to_string(),
    );

    c.bench_function("clipboard_content_compression", |b| {
        b.iter(|| {
            let mut content_copy = black_box(content.clone());
            content_copy.compress().unwrap();
            black_box(content_copy)
        })
    });
}

#[cfg(feature = "compression")]
fn bench_clipboard_content_decompression(c: &mut Criterion) {
    let large_text = "A".repeat(10000); // 10KB of text
    let mut content = ClipboardContent::new_text(
        large_text,
        "benchmark-device".to_string(),
    );
    content.compress().unwrap();

    c.bench_function("clipboard_content_decompression", |b| {
        b.iter(|| {
            let mut content_copy = black_box(content.clone());
            content_copy.decompress().unwrap();
            black_box(content_copy)
        })
    });
}

fn bench_large_clipboard_content(c: &mut Criterion) {
    let large_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(1000); // ~57KB
    
    c.bench_function("large_clipboard_content_creation", |b| {
        b.iter(|| {
            let content = ClipboardContent::new_text(
                black_box(large_text.clone()),
                black_box("benchmark-device".to_string()),
            );
            black_box(content)
        })
    });
}

criterion_group!(
    clipboard_benches,
    bench_clipboard_content_creation,
    bench_clipboard_content_serialization,
    bench_clipboard_content_deserialization,
    bench_clipboard_content_integrity_check,
    bench_large_clipboard_content,
);

#[cfg(feature = "compression")]
criterion_group!(
    compression_benches,
    bench_clipboard_content_compression,
    bench_clipboard_content_decompression,
);

#[cfg(feature = "compression")]
criterion_main!(clipboard_benches, compression_benches);

#[cfg(not(feature = "compression"))]
criterion_main!(clipboard_benches);
