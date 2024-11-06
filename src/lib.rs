#![cfg_attr(not(test), no_std)]

#[cfg(not(any(feature = "blocking", feature = "non-blocking")))]
compile_error!("At least one of the blocking or non-blocking features must be enabled");

#[cfg(feature = "blocking")]
pub use parser::blocking;
#[cfg(feature = "non-blocking")]
pub use parser::nonblocking;

pub mod error;

pub mod packet;
pub mod parser;

#[inline(always)]
pub(crate) const fn channels_parsing(buffer: &[u8; 25]) -> [u16; 16] {
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
pub fn available_bytes(write_pos: usize, read_pos: usize, buffer_len: usize) -> usize {
    if write_pos >= read_pos {
        write_pos - read_pos
    } else {
        buffer_len - (read_pos - write_pos)
    }
}
