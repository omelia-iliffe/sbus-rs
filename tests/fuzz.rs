use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use sbus_rs::{SbusError, SbusPacket, SBUS_FOOTER, SBUS_FRAME_LENGTH, SBUS_HEADER};

#[derive(Debug, Arbitrary)]
struct FuzzedSbusFrame {
    header: u8,
    payload: [u8; SBUS_FRAME_LENGTH - 2], // -2 for header and footer
    footer: u8,
}

impl FuzzedSbusFrame {
    fn to_buffer(&self) -> [u8; SBUS_FRAME_LENGTH] {
        let mut buffer = [0u8; SBUS_FRAME_LENGTH];
        buffer[0] = self.header;
        buffer[1..SBUS_FRAME_LENGTH - 1].copy_from_slice(&self.payload);
        buffer[SBUS_FRAME_LENGTH - 1] = self.footer;
        buffer
    }
}

// Basic fuzzing target
fuzz_target!(|frame: FuzzedSbusFrame| {
    let buffer = frame.to_buffer();
    let _ = SbusPacket::from_array(&buffer);
});

use proptest::prelude::*;

proptest! {
    // Test that valid frames are always parsed correctly
        #[test]
        #[ignore]
    fn test_valid_frame_parsing(
        channels in prop::array::uniform16(0..=2047u16),
        flags in 0u8..=0x0F
    ) {
        let mut buffer = [0u8; SBUS_FRAME_LENGTH];
        buffer[0] = SBUS_HEADER;
        buffer[SBUS_FRAME_LENGTH-1] = SBUS_FOOTER;

        // Pack channels into the buffer
        pack_channels(&mut buffer, &channels);
        buffer[23] = flags;

        let result = SbusPacket::from_array(&buffer);
        prop_assert!(result.is_ok());

        if let Ok(packet) = result {
            // Verify all channels were parsed correctly
            for (i, (&expected, &actual)) in channels.iter().zip(packet.channels.iter()).enumerate() {
                prop_assert_eq!(expected, actual, "Channel {} mismatch", i);
            }
            // Verify flags
            prop_assert_eq!(packet.flags.d1, flags & 0x01 != 0);
            prop_assert_eq!(packet.flags.d2, flags & 0x02 != 0);
            prop_assert_eq!(packet.flags.frame_lost, flags & 0x04 != 0);
            prop_assert_eq!(packet.flags.failsafe, flags & 0x08 != 0);
        }
    }

    // Test that frames with invalid headers are rejected
        #[test]
        #[ignore]
    fn test_invalid_header_rejection(
        header in (0u8..=0xFF).prop_filter("non-sbus headers", |h| *h != SBUS_HEADER),
        payload in prop::collection::vec(any::<u8>(), SBUS_FRAME_LENGTH-2),
        footer in 0u8..=0xFF
    ) {
        let mut buffer = [0u8; SBUS_FRAME_LENGTH];
        buffer[0] = header;
        buffer[1..SBUS_FRAME_LENGTH-1].copy_from_slice(&payload);
        buffer[SBUS_FRAME_LENGTH-1] = footer;

        let result = SbusPacket::from_array(&buffer);
        prop_assert!(matches!(result, Err(SbusError::InvalidHeader(_))));
    }

    // Test that frames with invalid footers are rejected
        #[test]
        #[ignore]
    fn test_invalid_footer_rejection(
        payload in prop::collection::vec(any::<u8>(), SBUS_FRAME_LENGTH-2),
        footer in (0u8..=0xFF).prop_filter("non-sbus footers", |f| *f != SBUS_FOOTER)
    ) {
        let mut buffer = [0u8; SBUS_FRAME_LENGTH];
        buffer[0] = SBUS_HEADER;
        buffer[1..SBUS_FRAME_LENGTH-1].copy_from_slice(&payload);
        buffer[SBUS_FRAME_LENGTH-1] = footer;

        let result = SbusPacket::from_array(&buffer);
        prop_assert!(matches!(result, Err(SbusError::InvalidFooter(_))));
    }


    // Test channel value boundaries
    #[test]
    #[ignore]
    fn test_channel_value_boundaries(
        channel_idx in 0usize..16,
        value in 0u16..=2047u16
    ) {
        let mut buffer = [0u8; SBUS_FRAME_LENGTH];
        buffer[0] = SBUS_HEADER;
        buffer[SBUS_FRAME_LENGTH-1] = SBUS_FOOTER;

        let mut channels = [1000u16; 16]; // Default mid-range value
        channels[channel_idx] = value;

        pack_channels(&mut buffer, &channels);

        let result = SbusPacket::from_array(&buffer);
        prop_assert!(result.is_ok());
        if let Ok(packet) = result {
            prop_assert_eq!(packet.channels[channel_idx], value);
        }
    }
}

fn pack_channels(buffer: &mut [u8; SBUS_FRAME_LENGTH], channels: &[u16; 16]) {
    // Clear the buffer first (except header and footer)
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
