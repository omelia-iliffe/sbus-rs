# sbus-rs

[![Crates.io](https://img.shields.io/crates/v/sbus-rs.svg)](https://crates.io/crates/sbus-rs)
[![Documentation](https://docs.rs/sbus-rs/badge.svg)](https://docs.rs/sbus-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A no_std compatible Rust implementation of the SBUS (Serial Bus) protocol parser, commonly used in RC (Radio Control) applications. Part of the [AeroRust](https://github.com/AeroRust) organization, dedicated to aerospace-related software in Rust.

## Features

- 🦀 Pure Rust implementation
- 🚫 `no_std` compatible for embedded systems
- ⚡ Async and blocking IO support
- 🔍 Robust error handling and validation
- 🧪 Thoroughly tested with unit tests, property-based tests, and fuzzing
- 📊 Benchmarked for performance optimization
- 🛠️ Zero-copy parsing for efficient memory usage

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
sbus-rs = "0.1.0"
```

For async support:
```toml
[dependencies]
sbus-rs = { version = "0.1.0", features = ["async"] }
```

## Usage

### Blocking Example

```rust
use sbus_rs::{SbusParser, SbusError};
use embedded_io_adapters::std::FromStd;

fn main() -> Result<(), SbusError> {
    let serial = /* your serial port */;
    let mut parser = SbusParser::new(FromStd::new(serial));
    
    // Read a single SBUS frame
    let frame = parser.read_frame()?;
    
    // Access channel values (0-2047)
    println!("Channel 1: {}", frame.channels[0]);
    
    // Check flags
    if frame.flags.failsafe {
        println!("Failsafe active!");
    }
    
    Ok(())
}
```

### Async Example

```rust
use sbus_rs::{SbusParserAsync, SbusError};
use embedded_io_adapters::tokio_1::FromTokio;

async fn read_sbus() -> Result<(), SbusError> {
    let serial = /* your async serial port */;
    let mut parser = SbusParserAsync::new(FromTokio::new(serial));
    
    // Read frames asynchronously
    let frame = parser.read_frame().await?;
    
    println!("Channels: {:?}", frame.channels);
    println!("Frame lost: {}", frame.flags.frame_lost);
    
    Ok(())
}
```

## Protocol Details

SBUS frames consist of:
- Start byte (0x0F)
- 22 bytes of channel data (16 channels, 11 bits each)
- 1 byte of flags
- End byte (0x00)

Channel values range from 0 to 2047 (11 bits).

Flag bits:
- Digital Channel 1
- Digital Channel 2
- Frame Lost
- Failsafe Active

## Performance

The library is optimized for performance with careful consideration of:
- Zero-copy parsing
- Efficient bit manipulation
- Minimal allocations
- Vectorization opportunities

Benchmarks are available and can be run with:
```bash
cargo bench
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. Make sure to:

1. Run the full test suite: `cargo test --all-features`
2. Run benchmarks: `cargo bench`
3. Run clippy: `cargo clippy --all-features`
4. Format code: `cargo fmt`

## Safety

The crate uses safe Rust and includes:
- Miri checks for undefined behavior
- Memory sanitizer tests
- Fuzzing tests
- Property-based testing

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE.txt) file for details.

## Acknowledgments

Part of the [AeroRust](https://github.com/AeroRust) organization, promoting the use of Rust in aerospace applications.

Special thanks to:
- The AeroRust community
- Contributors and maintainers
- The Rust embedded community