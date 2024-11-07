use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use sbus_rs::{
    pack_channels, SbusError, SbusPacket, CHANNEL_MAX, SBUS_FOOTER, SBUS_FRAME_LENGTH, SBUS_HEADER,
};

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
        value in 0u16..=CHANNEL_MAX
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
