use embedded_io_async::Read;

use crate::{
    error::SbusError,
    packet::SbusPacket,
    parser::{Parser, SBUS_FRAME_LENGTH},
};

pub struct Async {}
impl super::Mode for Async {}
impl super::Sealed for Async {}

impl<R, M> Parser<R, M>
where
    M: super::Mode,
{
    pub fn new<R1: Read>(reader: R1) -> Parser<R1, Async> {
        Parser {
            reader,
            _mode: Default::default(),
        }
    }
}

impl<R: Read> Parser<R, Async> {
    pub async fn read_frame(&mut self) -> Result<SbusPacket, SbusError> {
        let mut buffer = [0u8; SBUS_FRAME_LENGTH];
        self.reader
            .read_exact(&mut buffer)
            .await
            .map_err(|_| SbusError::ReadError)?;

        SbusPacket::from_array(&buffer)
    }
}

pub struct SbusParserAsync<R>
where
    R: Read,
{
    reader: R,
    circular_buffer: [u8; 256],
    write_pos: usize,
    read_pos: usize,
}

impl<R> SbusParserAsync<R>
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
    pub async fn read_next_valid_frame(&mut self) -> Result<SbusPacket, SbusError> {
        loop {
            // Read data into the circular buffer
            match self
                .reader
                .read(&mut self.circular_buffer[self.write_pos..self.write_pos + 1])
                .await
            {
                Ok(_) => {
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
                    // Copy 25 bytes to the packet buffer
                    let mut packet = [0u8; SBUS_FRAME_LENGTH];
                    for i in 0..SBUS_FRAME_LENGTH {
                        packet[i] =
                            self.circular_buffer[(self.read_pos + i) % self.circular_buffer.len()];
                    }

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
                        self.read_pos =
                            (self.read_pos + SBUS_FRAME_LENGTH) % self.circular_buffer.len();

                        return Ok(sbus_packet);
                    } else {
                        // Move read position forward by one byte if the end byte is incorrect
                        self.read_pos = (self.read_pos + 1) % self.circular_buffer.len();
                    }
                } else {
                    // Move read position forward by one byte if the start byte is incorrect
                    self.read_pos = (self.read_pos + 1) % self.circular_buffer.len();
                }
            }
        }
    }

    /// Read a single SBUS frame from the reader
    ///
    /// This function reads data from the reader and parses it into an SBUS packet.
    /// It expects the first byte to be the SBUS header and will return an error if the frame is invalid.
    pub async fn read_single_frame(&mut self) -> Result<SbusPacket, SbusError> {
        // Read 25 bytes into the packet buffer
        let mut packet = [0u8; SBUS_FRAME_LENGTH];
        self.reader
            .read_exact(&mut packet)
            .await
            .map_err(|_| SbusError::ReadError)?;

        SbusPacket::from_array(&buffer)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use crate::parser::asynch::SbusParserAsync;
    use embedded_io_adapters::tokio_1::FromTokio;

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
            0x0F, // Header
            0x00, 0x00, // Channel 1 (bits 0-10)
            0x00, 0x00, // Channel 2 (bits 0-10)
            // Ensure to simulate all 16 channels and the flags byte
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Flags
            0x00, // Footer
        ];
        let cursor = Cursor::new(data);
        let mut parser = SbusParserAsync::new(FromTokio::new(cursor));

        let packet = parser.read_next_valid_frame().await.expect("Should be a valid frame");

        assert_eq!(packet.channels[0], 0);
        assert_eq!(packet.channels[15], 0);
        assert!(!packet.flags.d1);
        assert!(!packet.flags.d2);
        assert!(!packet.flags.frame_lost);
        assert!(!packet.flags.failsafe);
    }

    #[tokio::test]
    async fn test_invalid_footer_async() {
        // Simulate a frame with an invalid header
        let mut data = TEST_PACKET;
        data[24] = 0x50; // Invalid footer

        let cursor = Cursor::new(data);
        let mut parser = SbusParserAsync::new(FromTokio::new(cursor));

        let result = parser.read_single_frame().await;
        assert!(matches!(result, Err(SbusError::InvalidFooter)));
    }

    #[tokio::test]
    async fn test_invalid_header_async() {
        // Simulate a frame with an invalid header
        let mut data = TEST_PACKET;
        data[0] = 0x00; // Invalid header

        let cursor = Cursor::new(data);
        let mut parser = SbusParserAsync::new(FromTokio::new(cursor));

        let result = parser.read_single_frame().await;
        assert!(matches!(result, Err(SbusError::InvalidHeader(0x00))));
    }
}
