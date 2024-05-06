use criterion::{black_box, Criterion};
use hex_literal::hex;

pub fn bench_parser(c: &mut Criterion) {
    let mut parser = sbus_rs::SBusPacketParser::new();
    // Example SBUS packet
    let bytes =
        hex!("00 0F E0 03 1F 58 C0 07 16 B0 80 05 2C 60 01 0B F8 C0 07 00 00 00 00 00 03 00");

    c.bench_function("parser", |b| {
        b.iter(|| {
            parser.push_bytes(&bytes).expect("Failed to push bytes");
            let msg = parser.try_parse().unwrap();

            black_box(msg);
        })
    });
}

pub fn bench_min_values(c: &mut Criterion) {
    let mut parser = sbus_rs::SBusPacketParser::new();
    // All channels set to minimum (assuming 0 as min)
    let bytes = hex!("0F 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00");

    c.bench_function("parser_min_values", |b| b.iter(||{
        parser.push_bytes(&bytes).expect("Failed to push bytes");
        let msg = parser.try_parse().unwrap();
        black_box(msg);
    }));
}

pub fn bench_max_values(c: &mut Criterion) {
    let mut parser = sbus_rs::SBusPacketParser::new();
    // Construct a packet where all channels are at maximum value (2047)
    // Channels: 11111111111 (each) requires careful bit manipulation across bytes
    let bytes = hex!("0F FF 07 FF 07 FF 07 FF 07 FF 07 FF 07 FF 07 FF 07 FF 07 FF 07 FF 07 07 00");

    c.bench_function("parser_max_values", |b| b.iter(||{
        parser.push_bytes(&bytes).expect("Failed to push bytes");
        let msg = parser.try_parse().expect("Failed to parse SBUS packet");
        black_box(msg);
    }));
}

pub fn bench_corrupted_data(c: &mut Criterion) {
    let mut parser = sbus_rs::SBusPacketParser::new();
    // Introduce bytes that do not conform to any known packet format
    let bytes = hex!("FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF");

    c.bench_function("parser_corrupted_data", |b| b.iter(||{
        parser.push_bytes(&bytes).expect("Failed to push bytes");
        let msg = parser.try_parse();
        let _ = black_box(msg);
    }));
}

pub fn bench_consecutive_packets(c: &mut Criterion) {
    let mut parser = sbus_rs::SBusPacketParser::new();
    // Simulate receiving two valid packets back-to-back
    let bytes = hex!("\
    0F E0 03 1F 58 C0 07 16 B0 80 05 2C 60 01 0B F8 C0 07 00 00 00 00 00 03 00\
    0F E0 03 1F 58 C0 07 16 B0 80 05 2C 60 01 0B F8 C0 07 00 00 00 00 00 03 00");

    dbg!(bytes.len());
    c.bench_function("parser_consecutive_packets", |b| b.iter(||{
        parser.clear_buffer();
        parser.push_bytes(&bytes).expect("Failed to push bytes");
        let msg1 = parser.try_parse().unwrap();
        let msg2 = parser.try_parse().unwrap();
        black_box((msg1, msg2));
    }));
}
