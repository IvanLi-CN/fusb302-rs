//! Driver errors.

/// A malformed or unsupported packet construction error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PacketError {
    /// The USB PD header data-object count does not match the supplied payload length.
    PayloadLengthMismatch {
        /// Payload length implied by the header.
        expected: usize,
        /// Payload length supplied by the caller.
        actual: usize,
    },
    /// The supplied payload exceeds the FUSB302B physical packet limit.
    PayloadTooLong {
        /// Payload length supplied by the caller.
        actual: usize,
    },
}

/// A malformed frame read from the FUSB302B receive FIFO.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ReceiveError {
    /// The receive FIFO overflowed before the host could consume its frame.
    FifoOverflow,
    /// The FUSB302B reported a failed CRC check for the received frame.
    CrcCheckFailed,
    /// The FIFO did not start with a received SOP token.
    InvalidSopToken(u8),
}

/// Error returned by an [`crate::Fusb302`] operation.
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<E> {
    /// The underlying I2C bus failed.
    I2c(E),
    /// A packet supplied for transmission was invalid.
    Packet(PacketError),
    /// A received FIFO frame was malformed.
    Receive(ReceiveError),
    /// The transmit FIFO cannot accept another physical packet.
    TransmitFifoFull,
}

impl<E> From<PacketError> for Error<E> {
    fn from(value: PacketError) -> Self {
        Self::Packet(value)
    }
}

impl<E> From<ReceiveError> for Error<E> {
    fn from(value: ReceiveError) -> Self {
        Self::Receive(value)
    }
}
