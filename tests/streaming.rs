use embedded_io_adapters::std::FromStd;
use sbus_rs::*;
use std::io::Cursor;

#[test]
fn test_multiple_frame_reading() {
    // Create a buffer with multiple consecutive valid frames
    let mut frames = Vec::new();
    for _ in 0..10 {
        frames.extend_from_slice(&create_valid_frame());
    }

    let cursor = Cursor::new(frames);
    let mut parser = SbusParser::new(FromStd::new(cursor));

    // Try reading all frames
    for _ in 0..10 {
        let result = parser.read_frame();
        assert!(result.is_ok());
    }
}

#[test]
fn test_partial_frame_reading() {
    // Create a buffer with partial frames
    let mut frames = create_valid_frame().to_vec();
    frames.truncate(frames.len() - 5); // Remove last 5 bytes

    let cursor = Cursor::new(frames);
    let mut parser = SbusParser::new(FromStd::new(cursor));

    let result = parser.read_frame();
    assert!(matches!(result, Err(SbusError::ReadError)));
}

fn create_valid_frame() -> [u8; SBUS_FRAME_LENGTH] {
    let mut frame = [0u8; SBUS_FRAME_LENGTH];
    frame[0] = SBUS_HEADER;
    frame[SBUS_FRAME_LENGTH - 1] = SBUS_FOOTER;
    frame
}
