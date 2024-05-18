use crate::{
    channels_parsing,
    error::SbusError,
    packet::SbusPacket,
    parser::{SBUS_FOOTER, SBUS_FRAME_LENGTH, SBUS_HEADER},
};
use embedded_io::Read;

pub struct SbusParser<R>
where
    R: Read,
{
    reader: R,
}

impl<'a, R> SbusParser<R>
where
    R: Read,
{
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub fn read_frame(&mut self) -> Result<SbusPacket, SbusError> {
        let mut buffer = [0u8; SBUS_FRAME_LENGTH];
        self.reader
            .read_exact(&mut buffer)
            .map_err(|_| SbusError::ReadError)?;

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
    use super::*;
    use embedded_io_adapters::std::FromStd;
    use std::io::Cursor;

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

    #[test]
    fn test_valid_sbus_frame() {
        // Simulate a valid SBUS frame
        let data = [
            0x0F, // Header
            0x00, 0x00, // Channel 1 (bits 0-10)
            0x00, 0x00, // Channel 2 (bits 0-10)
            // Remaining channels omitted for brevity, but should be similar
            // Ensure to simulate all 16 channels and the flags byte
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Flags
            0x00, // Footer
        ];
        let cursor = Cursor::new(data);
        let mut parser = SbusParser::new(FromStd::new(cursor));

        let result = parser.read_frame();
        assert!(result.is_ok());

        let packet = result.unwrap();
        assert_eq!(packet.channels[0], 0);
        assert_eq!(packet.channels[15], 0);
        assert!(!packet.d1);
        assert!(!packet.d2);
        assert!(!packet.frame_lost);
        assert!(!packet.failsafe);
    }

    #[test]
    fn test_invalid_header() {
        // Simulate a frame with an invalid header
        let mut data = TEST_PACKET;
        data[0] = 0x00; // Invalid header

        let cursor = Cursor::new(data);
        let mut parser = SbusParser::new(FromStd::new(cursor));

        let result = parser.read_frame();
        assert!(matches!(result, Err(SbusError::InvalidHeader)));
    }

    #[test]
    fn test_invalid_footer() {
        let mut data = TEST_PACKET;
        data[24] = 0xFF; // Invalid footer

        let cursor = Cursor::new(data);
        let mut parser = SbusParser::new(FromStd::new(cursor));

        let result = parser.read_frame();
        assert!(matches!(result, Err(SbusError::InvalidHeader)));
    }

    #[test]
    fn test_flag_bytes() {
        let mut data = TEST_PACKET;
        data[23] = 0b00001111; // All flags set

        let cursor = Cursor::new(data);
        let mut parser = SbusParser::new(FromStd::new(cursor));

        let result = parser.read_frame();
        assert!(result.is_ok());
        let packet = result.unwrap();
        assert!(packet.d1);
        assert!(packet.d2);
        assert!(packet.frame_lost);
        assert!(packet.failsafe);
    }

    #[test]
    fn test_partial_frame() {
        let data = &TEST_PACKET[..20]; // Cut off the last few bytes

        let cursor = Cursor::new(data);
        let mut parser = SbusParser::new(FromStd::new(cursor));

        let result = parser.read_frame();
        assert!(matches!(result, Err(SbusError::ReadError)));
    }

    #[test]
    fn test_channel_decoding() {
        let mut data = [0u8; 25];
        data[0] = 0x0F; // Header
                        // Channel 1 set to 0
        data[1] = 0;
        data[2] = 0;
        // Channel 2 set to 2047, needs to correctly span bytes 2, 3, and 4
        data[2] |= (2047 << 3) as u8; // Start from bit 3 of byte 2
        data[3] = ((2047 >> 5) & 0xFF) as u8; // Next full byte
        data[4] = ((2047 >> 5) & 0x07) as u8; // Last few bits that fit into byte 4
        data[24] = 0x00; // Footer

        let cursor = Cursor::new(data);
        let mut parser = SbusParser::new(FromStd::new(cursor));

        let result = parser.read_frame();
        assert!(result.is_ok());
        let packet = result.unwrap();
        assert_eq!(packet.channels[0], 0); // Channel 1 should be 0
        assert_eq!(packet.channels[1], 2047); // Channel 2 should be 2047
    }
}
