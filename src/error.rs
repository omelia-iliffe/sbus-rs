/// Error types for SBUS operations
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub enum SbusError {
    /// Error reading from the reader
    ReadError,
    /// Invalid header
    InvalidHeader(u8),
    /// Invalid footer
    InvalidFooter(u8),
}
