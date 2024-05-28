
#[derive(Debug, PartialEq)]
pub enum SbusError {
    /// Error reading from the reader
    ReadError,
    /// Invalid header
    InvalidHeader,
    /// Invalid footer
    InvalidFooter,
}