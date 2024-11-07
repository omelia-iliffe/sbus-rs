// benches/sbus_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use embedded_io_adapters::std::FromStd;
use sbus_rs::{SbusPacket, SbusParser, SBUS_FOOTER, SBUS_FRAME_LENGTH, SBUS_HEADER};
use std::io::Cursor;

fn create_test_frame(channels: &[u16; 16], flags: u8) -> [u8; SBUS_FRAME_LENGTH] {
    let mut buffer = [0u8; SBUS_FRAME_LENGTH];
    buffer[0] = SBUS_HEADER;
    buffer[SBUS_FRAME_LENGTH - 1] = SBUS_FOOTER;

    // Pack channels using the same logic from tests
    pack_channels(&mut buffer, channels);
    buffer[23] = flags;

    buffer
}

fn create_streaming_buffer(frame_count: usize) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(SBUS_FRAME_LENGTH * frame_count);
    let channels = [1000u16; 16]; // Mid-range value for all channels
    let frame = create_test_frame(&channels, 0);

    for _ in 0..frame_count {
        buffer.extend_from_slice(&frame);
    }
    buffer
}

fn bench_frame_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("frame_parsing");

    // Test different channel value scenarios
    let scenarios = vec![
        ("all_min", [0u16; 16]),
        ("all_max", [2047u16; 16]),
        ("all_mid", [1024u16; 16]),
        (
            "alternating",
            <[u16; 16]>::try_from([0u16, 2047u16].repeat(8)).unwrap(),
        ),
        (
            "ascending",
            (0..16)
                .map(|i| i as u16 * 128)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        ),
    ];

    for (name, channels) in scenarios {
        let frame = create_test_frame(&channels, 0);
        group.bench_with_input(BenchmarkId::new("parse_frame", name), &frame, |b, frame| {
            b.iter(|| black_box(SbusPacket::from_array(black_box(frame))).unwrap())
        });
    }

    group.finish();
}

fn bench_streaming_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming_parser");

    // Test different frame sequence lengths
    for frames in [1, 10, 100] {
        let buffer = create_streaming_buffer(frames);

        group.bench_with_input(
            BenchmarkId::new("parse_stream", frames),
            &buffer,
            |b, data| {
                b.iter(|| {
                    let cursor = Cursor::new(data);
                    let mut parser = SbusParser::new(FromStd::new(cursor));
                    for _ in 0..frames {
                        black_box(parser.read_frame()).unwrap();
                    }
                })
            },
        );
    }

    group.finish();
}

fn bench_frame_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("frame_validation");

    // Valid frame
    let valid_frame = create_test_frame(&[1000u16; 16], 0);

    // Invalid frames
    let mut invalid_header = valid_frame;
    invalid_header[0] = 0x00;

    let mut invalid_footer = valid_frame;
    invalid_footer[SBUS_FRAME_LENGTH - 1] = 0xFF;

    group.bench_function("validate_valid_frame", |b| {
        b.iter(|| black_box(SbusPacket::from_array(black_box(&valid_frame))).unwrap())
    });

    group.bench_function("validate_invalid_header", |b| {
        b.iter(|| {
            let _ = black_box(SbusPacket::from_array(black_box(&invalid_header)));
        })
    });

    group.bench_function("validate_invalid_footer", |b| {
        b.iter(|| {
            let _ = black_box(SbusPacket::from_array(black_box(&invalid_footer)));
        })
    });

    group.finish();
}

#[cfg(feature = "async")]
fn bench_async_parser(c: &mut Criterion) {
    use embedded_io_adapters::tokio_1::FromTokio;
    use sbus_rs::SbusParserAsync;
    use tokio::runtime::Runtime;

    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("async_parser");

    for frames in [1, 10, 100] {
        let buffer = create_streaming_buffer(frames);

        group.bench_with_input(
            BenchmarkId::new("parse_async_stream", frames),
            &buffer,
            |b, data| {
                b.iter(|| {
                    rt.block_on(async {
                        let cursor = Cursor::new(data);
                        let mut parser = SbusParserAsync::new(FromTokio::new(cursor));
                        for _ in 0..frames {
                            black_box(parser.read_frame().await).unwrap();
                        }
                    })
                })
            },
        );
    }

    group.finish();
}

// Pack channels function from tests
fn pack_channels(buffer: &mut [u8; SBUS_FRAME_LENGTH], channels: &[u16; 16]) {
    buffer
        .iter_mut()
        .take(SBUS_FRAME_LENGTH - 1)
        .skip(1)
        .for_each(|x| *x = 0);

    // Pack channels using the exact inverse of the parsing logic
    let ch = channels;

    // Channel 1 - Bytes 1-2
    buffer[1] = (ch[0] & 0xFF) as u8;
    buffer[2] = ((ch[0] >> 8) & 0x07) as u8;

    // Channel 2 - Bytes 2-3
    buffer[2] |= ((ch[1] & 0x1F) << 3) as u8;
    buffer[3] = ((ch[1] >> 5) & 0x3F) as u8;

    // Channel 3 - Bytes 3-5
    buffer[3] |= ((ch[2] & 0x03) << 6) as u8;
    buffer[4] = ((ch[2] >> 2) & 0xFF) as u8;
    buffer[5] = ((ch[2] >> 10) & 0x01) as u8;

    // Channel 4 - Bytes 5-6
    buffer[5] |= ((ch[3] & 0x7F) << 1) as u8;
    buffer[6] = ((ch[3] >> 7) & 0x0F) as u8;

    // Channel 5 - Bytes 6-7
    buffer[6] |= ((ch[4] & 0x0F) << 4) as u8;
    buffer[7] = ((ch[4] >> 4) & 0x7F) as u8;

    // Channel 6 - Bytes 7-9
    buffer[7] |= ((ch[5] & 0x01) << 7) as u8;
    buffer[8] = ((ch[5] >> 1) & 0xFF) as u8;
    buffer[9] = ((ch[5] >> 9) & 0x03) as u8;

    // Channel 7 - Bytes 9-10
    buffer[9] |= ((ch[6] & 0x3F) << 2) as u8;
    buffer[10] = ((ch[6] >> 6) & 0x1F) as u8;

    // Channel 8 - Bytes 10-11
    buffer[10] |= ((ch[7] & 0x07) << 5) as u8;
    buffer[11] = ((ch[7] >> 3) & 0xFF) as u8;

    // Channel 9 - Bytes 12-13
    buffer[12] = (ch[8] & 0xFF) as u8;
    buffer[13] = ((ch[8] >> 8) & 0x07) as u8;

    // Channel 10 - Bytes 13-14
    buffer[13] |= ((ch[9] & 0x1F) << 3) as u8;
    buffer[14] = ((ch[9] >> 5) & 0x3F) as u8;

    // Channel 11 - Bytes 14-16
    buffer[14] |= ((ch[10] & 0x03) << 6) as u8;
    buffer[15] = ((ch[10] >> 2) & 0xFF) as u8;
    buffer[16] = ((ch[10] >> 10) & 0x01) as u8;

    // Channel 12 - Bytes 16-17
    buffer[16] |= ((ch[11] & 0x7F) << 1) as u8;
    buffer[17] = ((ch[11] >> 7) & 0x0F) as u8;

    // Channel 13 - Bytes 17-18
    buffer[17] |= ((ch[12] & 0x0F) << 4) as u8;
    buffer[18] = ((ch[12] >> 4) & 0x7F) as u8;

    // Channel 14 - Bytes 18-20
    buffer[18] |= ((ch[13] & 0x01) << 7) as u8;
    buffer[19] = ((ch[13] >> 1) & 0xFF) as u8;
    buffer[20] = ((ch[13] >> 9) & 0x03) as u8;

    // Channel 15 - Bytes 20-21
    buffer[20] |= ((ch[14] & 0x3F) << 2) as u8;
    buffer[21] = ((ch[14] >> 6) & 0x1F) as u8;

    // Channel 16 - Bytes 21-22
    buffer[21] |= ((ch[15] & 0x07) << 5) as u8;
    buffer[22] = ((ch[15] >> 3) & 0xFF) as u8;
}

#[cfg(not(feature = "async"))]
criterion_group!(
    benches,
    bench_frame_parsing,
    bench_streaming_parser,
    bench_frame_validation
);

#[cfg(feature = "async")]
criterion_group!(
    benches,
    bench_frame_parsing,
    bench_streaming_parser,
    bench_frame_validation,
    bench_async_parser
);

criterion_main!(benches);
