
#[derive(Debug, PartialEq)]
pub enum SbusError {
    /// Error when the buffer is empty
    EmptyBuffer,
    /// Error when the buffer does not contain a valid SBUS packet
    InvalidFrame,
    /// Error when the buffer contains incomplete data
    IncompleteData,
    /// Error when the buffer exceeds the maximum packet size
    BufferOverflow,
}