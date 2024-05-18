
#[derive(Debug, PartialEq)]
pub enum SbusError {
    /// Error reading from the reader
    ReadError,
    /// Invalid header
    InvalidHeader(u8),
    /// Invalid footer
    InvalidFooter(u8),
}