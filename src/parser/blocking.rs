use crate::{error::SbusError, packet::SbusPacket, parser::SBUS_FRAME_LENGTH, Parser};
use embedded_io::Read;

pub struct Blocking {}
impl super::Mode for Blocking {}
impl super::Sealed for Blocking {}

impl<R, M> Parser<R, M>
where
    M: super::Mode,
{
    pub fn new_blocking<R1: Read>(reader: R1) -> Parser<R1, Blocking> {
        Parser {
            reader,
            _mode: Default::default(),
        }
    }
}

impl<R: Read> Parser<R, Blocking> {
    pub fn read_frame(&mut self) -> Result<SbusPacket, SbusError> {
        let mut buffer = [0u8; SBUS_FRAME_LENGTH];
        self.reader
            .read_exact(&mut buffer)
            .map_err(|_| SbusError::ReadError)?;

        SbusPacket::from_array(&buffer)
    }
}

pub struct SbusParser<R>
where
    R: Read,
{
    reader: R,
    circular_buffer: [u8; 256], // Adjust the size as needed
    write_pos: usize,
    read_pos: usize,
}

impl<R> SbusParser<R>
where
    R: Read,
{
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            circular_buffer: [0u8; 256],
            write_pos: 0,
            read_pos: 0,
        }
    }

    /// Read the next valid SBUS frame from the reader
    ///
    /// This function reads data from the reader and parses it into an SBUS packet.
    /// It will return an error if the reader encounters an error and will otherwise loop until a valid SBUS packet is found.
    pub fn read_next_valid_frame(&mut self) -> Result<SbusPacket, SbusError> {
        loop {
            // Read data into the circular buffer
            let mut single_byte = [0u8; 1];
            match self.reader.read_exact(&mut single_byte) {
                Ok(_) => {
                    self.circular_buffer[self.write_pos] = single_byte[0];
                    self.write_pos = (self.write_pos + 1) % self.circular_buffer.len();
                }
                Err(_) => {
                    return Err(SbusError::ReadError);
                }
            }

            // Check if we have at least 25 bytes to process
            while available_bytes(self.write_pos, self.read_pos, self.circular_buffer.len())
                >= SBUS_FRAME_LENGTH
            {
                // Look for the start of an SBUS packet (0x0F)
                if self.circular_buffer[self.read_pos] == SBUS_HEADER {
                    if let Some(value) = self.create_sbus_packet() {
                        return value;
                    }
                } else {
                    // Move read position forward by one byte if the start byte is incorrect
                    self.read_pos = (self.read_pos + 1) % self.circular_buffer.len();
                }
            }
        }
    }

    fn create_sbus_packet(&mut self) -> Option<Result<SbusPacket, SbusError>> {
        // Copy 25 bytes to the packet buffer
        let mut packet = [0u8; SBUS_FRAME_LENGTH];
        packet.iter_mut().enumerate().for_each(|(i, byte)| {
            *byte = self.circular_buffer[(self.read_pos + i) % self.circular_buffer.len()];
        });

        let end_byte = packet[SBUS_FRAME_LENGTH - 1];

        // Verify the end byte
        if end_byte == SBUS_FOOTER {
            // Parse the SBUS packet
            let channels = channels_parsing(&packet);

            let flag_byte = packet[23];

            let sbus_packet = SbusPacket {
                channels,
                d1: (flag_byte & (1 << 0)) != 0,
                d2: (flag_byte & (1 << 1)) != 0,
                frame_lost: (flag_byte & (1 << 2)) != 0,
                failsafe: (flag_byte & (1 << 3)) != 0,
            };

            // Move read position forward by 25 bytes
            self.read_pos = (self.read_pos + SBUS_FRAME_LENGTH) % self.circular_buffer.len();

            return Some(Ok(sbus_packet));
        } else {
            // Move read position forward by one byte if the end byte is incorrect
            self.read_pos = (self.read_pos + 1) % self.circular_buffer.len();
        }
        None
    }

    pub fn read_frame(&mut self) -> Result<SbusPacket, SbusError> {
        let mut buffer = [0u8; SBUS_FRAME_LENGTH];
        self.reader
            .read_exact(&mut buffer)
            .map_err(|_| SbusError::ReadError)?;

        SbusPacket::from_array(&buffer)
    }

    /// Read a single SBUS frame from the reader
    ///
    /// This function reads data from the reader and parses it into an SBUS packet.
    /// It expects the first byte to be the SBUS header and will return an error if the frame is invalid.
    pub fn read_single_frame(&mut self) -> Result<SbusPacket, SbusError> {
        // Read 25 bytes into the packet buffer
        let mut packet = [0u8; SBUS_FRAME_LENGTH];
        self.reader
            .read_exact(&mut packet)
            .map_err(|_| SbusError::ReadError)?;

        // Check header and footer
        if packet[0] != SBUS_HEADER || packet[SBUS_FRAME_LENGTH - 1] != SBUS_FOOTER {
            return Err(SbusError::InvalidHeader);
        }

        // Parse the SBUS packet
        let channels = channels_parsing(&packet);

        let flag_byte = packet[23];

        let sbus_packet = SbusPacket {
            channels,
            d1: (flag_byte & (1 << 0)) != 0,
            d2: (flag_byte & (1 << 1)) != 0,
            frame_lost: (flag_byte & (1 << 2)) != 0,
            failsafe: (flag_byte & (1 << 3)) != 0,
        };

        Ok(sbus_packet)
    }
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

        let packet = parser.read_next_valid_frame().expect("Should be a valid frame");

        assert_eq!(packet.channels[0], 0);
        assert_eq!(packet.channels[15], 0);
        assert!(!packet.flags.d1);
        assert!(!packet.flags.d2);
        assert!(!packet.flags.frame_lost);
        assert!(!packet.flags.failsafe);
    }

    #[test]
    fn test_invalid_header() {
        // Simulate a frame with an invalid header
        let mut data = TEST_PACKET;
        data[0] = 0x00; // Invalid header

        let cursor = Cursor::new(data);
        let mut parser = SbusParser::new(FromStd::new(cursor));

        let result = parser.read_single_frame();
        assert!(matches!(result, Err(SbusError::InvalidHeader(0x00))));
    }

    #[test]
    fn test_invalid_footer() {
        let mut data = TEST_PACKET;
        data[24] = 0xFF; // Invalid footer

        let cursor = Cursor::new(data);
        let mut parser = SbusParser::new(FromStd::new(cursor));

        let result = parser.read_single_frame();
        assert!(matches!(result, Err(SbusError::InvalidFooter(0xFF))));
    }

    #[test]
    fn test_flag_bytes() {
        let mut data = TEST_PACKET;
        data[23] = 0b00001111; // All flags set

        let cursor = Cursor::new(data);
        let mut parser = SbusParser::new(FromStd::new(cursor));

        let result = parser.read_next_valid_frame();
        assert!(result.is_ok());
        let packet = result.unwrap();
        assert!(packet.flags.d1);
        assert!(packet.flags.d2);
        assert!(packet.flags.frame_lost);
        assert!(packet.flags.failsafe);
    }

    #[test]
    fn test_partial_frame() {
        let data = &TEST_PACKET[..20]; // Cut off the last few bytes

        let cursor = Cursor::new(data);
        let mut parser = SbusParser::new(FromStd::new(cursor));

        let result = parser.read_single_frame();
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

        let result = parser.read_single_frame();
        assert!(result.is_ok());
        let packet = result.unwrap();
        assert_eq!(packet.channels[0], 0); // Channel 1 should be 0
        assert_eq!(packet.channels[1], 2047); // Channel 2 should be 2047
    }
}
