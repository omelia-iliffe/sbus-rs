use embedded_io_async::Read;

use crate::channels_parsing;
use crate::error::SbusError;
use crate::packet::SbusPacket;
use crate::parser::{SBUS_FOOTER, SBUS_FRAME_LENGTH, SBUS_HEADER};

pub struct SbusParserAsync<R>
    where
        R: Read,
{
    reader: R,
}

impl<'a, R> SbusParserAsync<R>
    where
        R: Read,
{
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub async fn read_frame(&mut self) -> Result<SbusPacket, SbusError> {
        let mut buffer = [0u8; SBUS_FRAME_LENGTH];
        self.reader.read_exact(&mut buffer).await.map_err(|_| SbusError::ReadError)?;

        // Check header and footer
        if buffer[0] != SBUS_HEADER || buffer[SBUS_FRAME_LENGTH - 1] != SBUS_FOOTER {
            return Err(SbusError::InvalidHeader);
        }

        // Parse channels and flags
        let channels = channels_parsing(&buffer);

        let flag_byte = buffer[23];
        
        let packet = SbusPacket {
            channels,
            d1: (flag_byte & (1 << 0)) != 0,
            d2: (flag_byte & (1 << 1)) != 0,
            frame_lost: (flag_byte & (1 << 2)) != 0,
            failsafe: (flag_byte & (1 << 3)) != 0,
        };

        Ok(packet)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use embedded_io_adapters::tokio_1::FromTokio;
    use crate::parser::nonblocking::SbusParserAsync;
    use super::*;

    const TEST_PACKET: [u8; 25] = [
        0x0F, // HEAD_BYTE
        (1024 & 0x07FF) as u8,
        (((1024 & 0x07FF) >> 8) | ((1024 & 0x07FF) << 3)) as u8,
        (((1024 & 0x07FF) >> 5) | ((1024 & 0x07FF) << 6)) as u8,
        ((1024 & 0x07FF) >> 2) as u8,
        (((1024 & 0x07FF) >> 10) | ((1024 & 0x07FF) << 1)) as u8,
        (((1024 & 0x07FF) >> 7) | ((1024 & 0x07FF) << 4)) as u8,
        ((1024 & 0x07FF) >> 4) as u8,
        ((1024 & 0x07FF) << 2) as u8,
        (((1024 & 0x07FF) >> 8) | ((1024 & 0x07FF) << 5)) as u8,
        ((1024 & 0x07FF) >> 1) as u8,
        (((1024 & 0x07FF) >> 9) | ((1024 & 0x07FF) << 6)) as u8,
        ((1024 & 0x07FF) >> 3) as u8,
        (((1024 & 0x07FF) >> 10) | ((1024 & 0x07FF) << 1)) as u8,
        (((1024 & 0x07FF) >> 7) | ((1024 & 0x07FF) << 4)) as u8,
        ((1024 & 0x07FF) >> 4) as u8,
        ((1024 & 0x07FF) << 2) as u8,
        (((1024 & 0x07FF) >> 8) | ((1024 & 0x07FF) << 5)) as u8,
        ((1024 & 0x07FF) >> 1) as u8,
        (((1024 & 0x07FF) >> 9) | ((1024 & 0x07FF) << 6)) as u8,
        ((1024 & 0x07FF) >> 3) as u8,
        (((1024 & 0x07FF) >> 10) | ((1024 & 0x07FF) << 1)) as u8,
        (((1024 & 0x07FF) >> 7) | ((1024 & 0x07FF) << 4)) as u8,
        0x00, // FLAGS_BYTE, no flags set
        0x00, // FOOT_BYTE
    ];

    #[tokio::test]
    async fn test_valid_sbus_frame_async() {
        // Simulate a valid SBUS frame
        let data = [
            0x0F,  // Header
            0x00, 0x00, // Channel 1 (bits 0-10)
            0x00, 0x00, // Channel 2 (bits 0-10)
            // Ensure to simulate all 16 channels and the flags byte
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00,  // Flags
            0x00   // Footer
        ];
        let cursor = Cursor::new(data);
        let mut parser = SbusParserAsync::new(FromTokio::new(cursor));

        let result = parser.read_frame().await;
        assert!(result.is_ok());

        let packet = result.unwrap();
        assert_eq!(packet.channels[0], 0);
        assert_eq!(packet.channels[15], 0);
        assert!(!packet.d1);
        assert!(!packet.d2);
        assert!(!packet.frame_lost);
        assert!(!packet.failsafe);
    }

    #[tokio::test]
    async fn test_invalid_header_async() {
        // Simulate a frame with an invalid header
        let mut data = TEST_PACKET;
        data[0] = 0x00; // Invalid header
        
        
        let cursor = Cursor::new(data);
        let mut parser = SbusParserAsync::new(FromTokio::new(cursor));

        let result = parser.read_frame().await;
        assert!(matches!(result, Err(SbusError::InvalidHeader)));
    }

    // Additional async tests for invalid footer, buffer overflow, parse errors, etc., can be added here.
}
