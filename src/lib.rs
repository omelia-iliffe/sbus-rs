#![cfg_attr(all(not(test), not(feature = "std")), no_std)]
//! # sbus-rs
//!
//! A no_std compatible library for parsing SBUS (Serial Bus) protocol, commonly used in RC (Radio Control) systems.
//! SBUS is a protocol developed by Futaba for RC receivers to communicate with flight controllers and other devices.
//!
//! ## Features
//!
//! - `blocking`: Enables blocking I/O operations (enabled by default)
//! - `async`: Enables async I/O operations
//! - `std`: Enables standard library features
//!
//! ## Example
//!
//! ```rust
//! use sbus_rs::{SbusParser, SbusPacket};
//! use embedded_io_adapters::std::FromStd;
//! use std::io::Cursor;
//!
//! let data = [0x0F, /* ... SBUS frame data ... */ 0x00];
//! let cursor = Cursor::new(data);
//! let mut parser = SbusParser::new(FromStd::new(cursor));
//!
//! match parser.read_frame() {
//!     Ok(packet) => {
//!         println!("Channel 1 value: {}", packet.channels[0]);
//!         if packet.flags.failsafe {
//!             println!("Failsafe active!");
//!         }
//!     }
//!     Err(e) => println!("Error reading frame: {:?}", e),
//! }
//! ```
//!
//! ## Protocol Details
//!
//! SBUS frames are 25 bytes long with the following structure:
//! - Start byte (0x0F)
//! - 22 bytes of channel data (16 channels, 11 bits each)
//! - 1 byte of flags
//! - End byte (0x00)

pub use error::*;
pub use packet::*;
pub use parser::*;

mod error;
mod packet;
mod parser;

#[inline(always)]
pub const fn channels_parsing(buffer: &[u8; SBUS_FRAME_LENGTH]) -> [u16; CHANNEL_COUNT] {
    [
        ((buffer[1] as u16) | ((buffer[2] as u16) << 8)) & 0x07FF,
        (((buffer[2] as u16) >> 3) | ((buffer[3] as u16) << 5)) & 0x07FF,
        (((buffer[3] as u16) >> 6) | ((buffer[4] as u16) << 2) | ((buffer[5] as u16) << 10))
            & 0x07FF,
        (((buffer[5] as u16) >> 1) | ((buffer[6] as u16) << 7)) & 0x07FF,
        (((buffer[6] as u16) >> 4) | ((buffer[7] as u16) << 4)) & 0x07FF,
        (((buffer[7] as u16) >> 7) | ((buffer[8] as u16) << 1) | ((buffer[9] as u16) << 9))
            & 0x07FF,
        (((buffer[9] as u16) >> 2) | ((buffer[10] as u16) << 6)) & 0x07FF,
        (((buffer[10] as u16) >> 5) | ((buffer[11] as u16) << 3)) & 0x07FF,
        ((buffer[12] as u16) | ((buffer[13] as u16) << 8)) & 0x07FF,
        (((buffer[13] as u16) >> 3) | ((buffer[14] as u16) << 5)) & 0x07FF,
        (((buffer[14] as u16) >> 6) | ((buffer[15] as u16) << 2) | ((buffer[16] as u16) << 10))
            & 0x07FF,
        (((buffer[16] as u16) >> 1) | ((buffer[17] as u16) << 7)) & 0x07FF,
        (((buffer[17] as u16) >> 4) | ((buffer[18] as u16) << 4)) & 0x07FF,
        (((buffer[18] as u16) >> 7) | ((buffer[19] as u16) << 1) | ((buffer[20] as u16) << 9))
            & 0x07FF,
        (((buffer[20] as u16) >> 2) | ((buffer[21] as u16) << 6)) & 0x07FF,
        (((buffer[21] as u16) >> 5) | ((buffer[22] as u16) << 3)) & 0x07FF,
    ]
}

#[inline(always)]
pub const fn pack_channels(buffer: &mut [u8; SBUS_FRAME_LENGTH], channels: &[u16; CHANNEL_COUNT]) {
    // Clear the buffer first (except header and footer)
    let mut i = 1;
    while i < SBUS_FRAME_LENGTH - 1 {
        buffer[i] = 0;
        i += 1;
    }

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
