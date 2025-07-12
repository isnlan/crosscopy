//! Network performance benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use crosscopy::network::{Message, MessageType};

fn bench_message_creation(c: &mut Criterion) {
    let payload = b"Test message payload for benchmarking".to_vec();
    let device_id = "benchmark-device".to_string();

    c.bench_function("message_creation", |b| {
        b.iter(|| {
            let message = Message::new(
                black_box(MessageType::ClipboardSync),
                black_box(payload.clone()),
                black_box(device_id.clone()),
            );
            black_box(message)
        })
    });
}

fn bench_message_serialization(c: &mut Criterion) {
    let payload = b"Test message payload for serialization benchmark".to_vec();
    let message = Message::new(
        MessageType::ClipboardSync,
        payload,
        "benchmark-device".to_string(),
    );

    c.bench_function("message_serialization", |b| {
        b.iter(|| {
            let serialized = serde_json::to_vec(black_box(&message)).unwrap();
            black_box(serialized)
        })
    });
}

fn bench_message_deserialization(c: &mut Criterion) {
    let payload = b"Test message payload for deserialization benchmark".to_vec();
    let message = Message::new(
        MessageType::ClipboardSync,
        payload,
        "benchmark-device".to_string(),
    );
    let serialized = serde_json::to_vec(&message).unwrap();

    c.bench_function("message_deserialization", |b| {
        b.iter(|| {
            let deserialized: Message = serde_json::from_slice(black_box(&serialized)).unwrap();
            black_box(deserialized)
        })
    });
}

fn bench_message_verification(c: &mut Criterion) {
    let payload = b"Test message payload for verification benchmark".to_vec();
    let message = Message::new(
        MessageType::ClipboardSync,
        payload,
        "benchmark-device".to_string(),
    );

    c.bench_function("message_verification", |b| {
        b.iter(|| {
            let is_valid = black_box(&message).verify();
            black_box(is_valid)
        })
    });
}

fn bench_large_message_handling(c: &mut Criterion) {
    let large_payload = vec![0u8; 1024 * 1024]; // 1MB payload
    let message = Message::new(
        MessageType::ClipboardSync,
        large_payload,
        "benchmark-device".to_string(),
    );

    c.bench_function("large_message_serialization", |b| {
        b.iter(|| {
            let serialized = serde_json::to_vec(black_box(&message)).unwrap();
            black_box(serialized)
        })
    });
}

fn bench_message_types(c: &mut Criterion) {
    let payload = b"Test payload".to_vec();
    let device_id = "benchmark-device".to_string();

    let message_types = [
        MessageType::Handshake,
        MessageType::Heartbeat,
        MessageType::ClipboardSync,
        MessageType::DeviceInfo,
        MessageType::Ack,
        MessageType::Error,
    ];

    for msg_type in &message_types {
        c.bench_function(&format!("message_creation_{:?}", msg_type), |b| {
            b.iter(|| {
                let message = Message::new(
                    black_box(*msg_type),
                    black_box(payload.clone()),
                    black_box(device_id.clone()),
                );
                black_box(message)
            })
        });
    }
}

fn bench_checksum_calculation(c: &mut Criterion) {
    let data_sizes = [100, 1000, 10000, 100000]; // Different data sizes

    for size in &data_sizes {
        let data = vec![0u8; *size];
        
        c.bench_function(&format!("checksum_calculation_{}bytes", size), |b| {
            b.iter(|| {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(black_box(&data));
                let checksum = format!("{:x}", hasher.finalize());
                black_box(checksum)
            })
        });
    }
}

fn bench_protocol_overhead(c: &mut Criterion) {
    let payload_sizes = [10, 100, 1000, 10000]; // Different payload sizes
    let device_id = "benchmark-device".to_string();

    for size in &payload_sizes {
        let payload = vec![0u8; *size];
        
        c.bench_function(&format!("protocol_overhead_{}bytes", size), |b| {
            b.iter(|| {
                let message = Message::new(
                    black_box(MessageType::ClipboardSync),
                    black_box(payload.clone()),
                    black_box(device_id.clone()),
                );
                let serialized = serde_json::to_vec(&message).unwrap();
                black_box(serialized)
            })
        });
    }
}

criterion_group!(
    network_benches,
    bench_message_creation,
    bench_message_serialization,
    bench_message_deserialization,
    bench_message_verification,
    bench_large_message_handling,
    bench_message_types,
    bench_checksum_calculation,
    bench_protocol_overhead,
);

criterion_main!(network_benches);
