#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
use heapless::Deque;
#[cfg(feature = "std")]
use std::collections::VecDeque;

use crate::error::SbusError;
#[cfg(feature = "embedded-io")]
use embedded_io::Read;

mod error;
#[cfg(test)]
mod tests;

#[cfg(not(feature = "std"))]
type SbusDeque = Deque<u8, MAX_PACKET_SIZE>;
#[cfg(feature = "std")]
type SbusDeque = VecDeque<u8>;


// Important bytes for correctness checks
const FLAG_MASK: u8 = 0b11110000;
const HEAD_BYTE: u8 = 0b00001111;
const FOOT_BYTE: u8 = 0b00000000;

// Number of bytes in SBUS message
const PACKET_SIZE: usize = 25;
const MAX_PACKET_SIZE: usize = 50;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub struct SBusPacket {
    pub channels: [u16; 16],
    pub d1: bool,
    pub d2: bool,
    pub failsafe: bool,
    pub frame_lost: bool,
}

#[derive(Debug, Default)]
pub struct SBusPacketParser {
    pub buffer: SbusDeque,
}

impl SBusPacketParser {
    pub fn new() -> SBusPacketParser {
            #[cfg(feature = "std")]
            {
                SBusPacketParser {
                    buffer: SbusDeque::with_capacity(MAX_PACKET_SIZE),
                }
            }
        #[cfg(not(feature = "std"))]
         {
            SBusPacketParser {
                buffer: SbusDeque::new(),
            }
        }
    }

    /// Clears the buffer of the parser.
    #[inline(always)]
    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
    }

    /// Push single `u8` byte into buffer.
    #[inline(always)]
    pub fn push_byte(&mut self, byte: u8) {
        self.buffer.push_back(byte);
    }

    /// Push array of `u8` bytes into buffer.
    pub fn push_bytes(&mut self, bytes: &[u8]) -> Result<(), SbusError> {
        if self.buffer.len() + bytes.len() > self.buffer.capacity() {
            dbg!(self.buffer.len(), bytes.len(), self.buffer.len() + bytes.len(), self.buffer.capacity());
            return Err(SbusError::BufferOverflow);
        }
        for &byte in bytes {
            self.buffer
                .push_back(byte);
        }
        Ok(())
    }
    /// Exhaustively reads the bytes from uart device implementing
    /// the `embedded_io::serial::Read<u8>` trait.
    #[cfg(feature = "embedded-io")]
    pub fn read_serial<U: Read>(&mut self, uart: &mut U) {
        while let Ok(byte) = uart.read(&mut []) {
            self.push_byte(byte as u8);
        }
    }

    /// Equivalent to consecutively calling `read_serial()` and `try_parse()`.
    #[cfg(feature = "embedded-io")]
    pub fn read_serial_try_parse<U: Read>(&mut self, uart: &mut U) -> Option<SBusPacket> {
        self.read_serial(uart);
        self.try_parse().ok()
    }

    /// Attempts to parse a valid SBUS packet from the buffer
    pub fn try_parse(&mut self) -> Result<SBusPacket, SbusError> {
        // Ensure the buffer is not empty
        if self.buffer.is_empty() {
            return Err(SbusError::EmptyBuffer);
        }

        // Align the buffer to start with the HEAD_BYTE
        while self.buffer.front() != Some(&HEAD_BYTE) && self.buffer.len() > PACKET_SIZE {
            self.buffer.pop_front().ok_or(SbusError::EmptyBuffer)?;
        }

        // Ensure the buffer has enough data to form a complete packet
        if self.buffer.len() < PACKET_SIZE {
            return Err(SbusError::IncompleteData);
        }

        // Check if entire frame is valid
        if !self.valid_frame() {
            return Err(SbusError::InvalidFrame);
        }

        // Extract the relevant data from the buffer
        let mut data = [0u16; 24];
        for i in 0..24 {
            data[i] = self.buffer.pop_front().ok_or(SbusError::IncompleteData)? as u16;
        }

        // Decode channels using bit manipulation
        let mut channels = [0u16; 16];
        channels[0] = (data[1] | (data[2] << 8)) & 0x07FF;
        // Repeat similar calculations for other channels...

        let flag_byte = data[23] as u8;

        Ok(SBusPacket {
            channels,
            d1: is_flag_set(flag_byte, 0),
            d2: is_flag_set(flag_byte, 1),
            frame_lost: is_flag_set(flag_byte, 2),
            failsafe: is_flag_set(flag_byte, 3),
        })
    }

    /// Returns `true` if the first part of the buffer contains a valid SBUS frame
    fn valid_frame(&self) -> bool {
        // Ensure there are enough bytes to form a valid frame
        if self.buffer.len() < PACKET_SIZE {
            return false;
        }

        // Retrieve head, flag, and foot using iterator and indexing
        let head = *self.buffer.front().unwrap_or(&0); // Safe because of the length check above
        let foot = *self.buffer.iter().nth(PACKET_SIZE - 1).unwrap_or(&0); // Safe because of the length check above
        let flag = *self.buffer.iter().nth(PACKET_SIZE - 2).unwrap_or(&0); // Safe because of the length check above

        head == HEAD_BYTE && foot == FOOT_BYTE && (flag & FLAG_MASK) == 0
    }
}

#[inline(always)]
fn is_flag_set(flag_byte: u8, shift_by: u8) -> bool {
    (flag_byte >> shift_by) & 1 == 1
}
