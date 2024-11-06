#[cfg(feature = "blocking")]
pub mod blocking;
pub mod nonblocking;

const SBUS_HEADER: u8 = 0x0F;
const SBUS_FOOTER: u8 = 0x00;
const SBUS_FRAME_LENGTH: usize = 25;
