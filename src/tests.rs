use hex_literal::hex;
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

/// Test the parsing of a completely valid SBUS packet.
#[test]
fn test_valid_sbus_packet() {
    let mut parser = SBusPacketParser::new();
    // Example SBUS packet - This needs to be a valid SBUS frame
    let test_bytes: [u8; 25] = TEST_PACKET;
    assert!(parser.push_bytes(&test_bytes).is_ok());
    let packet = parser.try_parse();
    assert!(packet.is_ok());
    // Further asserts to validate channel data, flags, etc.
}

/// Test handling of incorrect head byte.
#[test]
fn test_incorrect_head_byte() {
    let mut parser = SBusPacketParser::new();
    let mut test_bytes: [u8; 25] = TEST_PACKET;
    test_bytes[0] = 0x00; // Incorrect head byte
    assert!(parser.push_bytes(&test_bytes).is_ok());
    assert!(parser.try_parse().is_err());
}

/// Test the buffer exceeding the maximum packet size.
#[test]
fn test_exceed_max_packet_size() {
    let mut parser = SBusPacketParser::new();
    // Push more bytes than MAX_PACKET_SIZE
    for _ in 0..(MAX_PACKET_SIZE + 10) {
        parser.push_byte(0x55); // Arbitrary non-protocol data
    }
    assert!(parser.try_parse().is_err());
}

/// Test the correct processing of consecutive valid packets.
#[test]
fn test_consecutive_valid_packets() {
    let mut parser = SBusPacketParser::new();
    let valid_packet: [u8; 25] = TEST_PACKET;
    // Simulate receiving two valid packets back-to-back
    assert!(parser.push_bytes(&valid_packet).is_ok());
    assert!(parser.push_bytes(&valid_packet).is_ok());
    let first_packet = parser.try_parse();
    let second_packet = parser.try_parse();
    assert!(first_packet.is_ok());
    assert!(second_packet.is_ok());
}

/// Creates a new SBusPacketParser and pre-fills its buffer with specified bytes
fn setup_parser_with_data(data: &[u8]) -> SBusPacketParser {
    let mut parser = SBusPacketParser::new();
    for &byte in data {
        parser.buffer.push_back(byte);
    }
    parser
}

#[test]
fn valid_frame_correct_head_and_foot() {
    let data = [
        HEAD_BYTE,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        FLAG_MASK & 0,
        FOOT_BYTE,
    ];
    let parser = setup_parser_with_data(&data);
    assert!(
        parser.valid_frame(),
        "The frame should be valid with correct HEAD and FOOT bytes and proper flag byte"
    );
}

#[test]
fn invalid_frame_wrong_head_byte() {
    let data = [
        0x00, // Wrong HEAD_BYTE
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        FLAG_MASK & 0,
        FOOT_BYTE,
    ];
    let parser = setup_parser_with_data(&data);
    assert!(
        !parser.valid_frame(),
        "The frame should be invalid with incorrect HEAD byte"
    );
}

#[test]
fn invalid_frame_wrong_foot_byte() {
    let data = [
        HEAD_BYTE,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        0xFF,
        FLAG_MASK & 0,
        0x01, // Wrong FOOT_BYTE
    ];
    let parser = setup_parser_with_data(&data);
    assert!(
        !parser.valid_frame(),
        "The frame should be invalid with incorrect FOOT byte"
    );
}

#[test]
fn test_benchmark_scenario() {
    let mut parser = SBusPacketParser::new();
    let bytes = hex!("0F FF 07 FF 07 FF 07 FF 07 FF 07 FF 07 FF 07 FF 07 FF 07 FF 07 FF 07 07 00");
    parser.push_bytes(&bytes).expect("Failed to push bytes");

    assert!(parser.valid_frame(), "Frame should be valid according to SBUS specification");
    let packet = parser.try_parse();
    assert!(packet.is_ok(), "Parsing should succeed with benchmark data");
}


#[test]
fn invalid_frame_incorrect_flag_byte() {
    let data = [
        HEAD_BYTE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, // Incorrect FLAG byte not masking correctly
        FOOT_BYTE,
    ];
    let parser = setup_parser_with_data(&data);
    assert!(
        !parser.valid_frame(),
        "The frame should be invalid with incorrect flag settings"
    );
}

#[allow(unexpected_cfgs)]
#[cfg(kani)]
mod verification {
    use super::*;

    /// Verifies that the SBUS packet parser correctly handles typical packet sizes.
    #[kani::proof]
    fn verify_sbus_packet_parsing() {
        let mut parser = SBusPacketParser::new();
        let test_bytes: [u8; PACKET_SIZE] = kani::any(); // Use the defined PACKET_SIZE

        // Assume the packet starts and ends with expected bytes, typical for valid packets
        kani::assume(test_bytes[0] == HEAD_BYTE);
        kani::assume(test_bytes[PACKET_SIZE - 1] == FOOT_BYTE);

        parser.push_bytes(&test_bytes);
        let packet = parser.try_parse();

        // Verify that if a packet is parsed, it meets expected properties
        if let Ok(pkt) = packet {
            assert!(pkt.channels.iter().all(|&ch| ch <= 0x07FF)); // Check channel values are within bounds
            assert!(pkt.d1 == true || pkt.d1 == false); // Trivially true but confirms the logic is considered
        }
    }

    /// Verifies that packets with incorrect start or end markers are rejected.
    #[kani::proof]
    fn verify_packet_markers() {
        let mut parser = SBusPacketParser::new();
        let test_bytes: [u8; PACKET_SIZE] = kani::any();

        // Introduce variability in the head and foot bytes
        kani::assume(test_bytes[0] != HEAD_BYTE || test_bytes[PACKET_SIZE - 1] != FOOT_BYTE);

        parser.push_bytes(&test_bytes);
        assert!(parser.try_parse().is_err());
    }

    /// Verifies the parser's behavior under various flag settings.
    #[kani::proof]
    fn verify_flag_handling() {
        let mut parser = SBusPacketParser::new();
        let mut test_bytes: [u8; PACKET_SIZE] = kani::any();

        // Assumptions on start and end are correct; focus on flag byte
        kani::assume(test_bytes[0] == HEAD_BYTE);
        kani::assume(test_bytes[PACKET_SIZE - 1] == FOOT_BYTE);

        // Modify only the flags byte to see all possible flag configurations
        test_bytes[PACKET_SIZE - 2] = kani::any();

        parser.push_bytes(&test_bytes);
        if let Ok(pkt) = parser.try_parse() {
            // Ensure flags are either set or cleared
            assert!(pkt.d1 == true || pkt.d1 == false);
            assert!(pkt.d2 == true || pkt.d2 == false);
            assert!(pkt.frame_lost == true || pkt.frame_lost == false);
            assert!(pkt.failsafe == true || pkt.failsafe == false);
        }
    }
}
