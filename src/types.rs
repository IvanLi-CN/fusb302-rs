//! Public transport, configuration, and status types.

use core::ops::{BitOr, BitOrAssign};

use crate::{
    MAX_PD_PAYLOAD_BYTES,
    error::PacketError,
    registers::{DataRole, PdRevision, PowerRole},
};

/// Start-of-packet sequence associated with a USB PD packet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SopType {
    /// SOP.
    Sop,
    /// SOP'.
    SopPrime,
    /// SOP''.
    SopDoublePrime,
    /// SOP' Debug.
    SopPrimeDebug,
    /// SOP'' Debug.
    SopDoublePrimeDebug,
}

/// A bounded, allocation-free USB PD physical packet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PdPacket {
    sop: SopType,
    header: u16,
    payload: [u8; MAX_PD_PAYLOAD_BYTES],
    payload_len: u8,
}

impl PdPacket {
    /// Construct a packet after checking that the PD header object count matches `payload`.
    pub fn new(sop: SopType, header: u16, payload: &[u8]) -> Result<Self, PacketError> {
        if payload.len() > MAX_PD_PAYLOAD_BYTES {
            return Err(PacketError::PayloadTooLong {
                actual: payload.len(),
            });
        }

        let expected = Self::payload_len_from_header(header);
        if payload.len() != expected {
            return Err(PacketError::PayloadLengthMismatch {
                expected,
                actual: payload.len(),
            });
        }

        let mut storage = [0; MAX_PD_PAYLOAD_BYTES];
        storage[..payload.len()].copy_from_slice(payload);
        Ok(Self {
            sop,
            header,
            payload: storage,
            payload_len: payload.len() as u8,
        })
    }

    pub(crate) const fn from_fifo(sop: SopType, header: u16, payload: [u8; 28]) -> Self {
        Self {
            sop,
            header,
            payload,
            payload_len: Self::payload_len_from_header(header) as u8,
        }
    }

    /// Return the SOP type.
    pub const fn sop(self) -> SopType {
        self.sop
    }

    /// Return the raw USB PD header in host-endian representation.
    pub const fn header(self) -> u16 {
        self.header
    }

    /// Return the packet payload, without the 2-byte header or CRC.
    pub fn payload(&self) -> &[u8] {
        &self.payload[..self.payload_len as usize]
    }

    /// Return the payload length implied by the header's data-object count.
    pub const fn payload_len_from_header(header: u16) -> usize {
        ((header >> 12) as usize & 0x07) * 4
    }
}

/// Bit mask selecting received USB PD packet variants.
///
/// Standard SOP packets are always accepted by the FUSB302B receiver. The remaining variants
/// control the four explicit enable bits in `CONTROL1`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReceiveSopMask(u8);

impl ReceiveSopMask {
    /// Do not enable additional cable-marked SOP variants.
    pub const NONE: Self = Self(0);
    /// Represent standard SOP packets, which are always accepted by the hardware.
    pub const SOP: Self = Self(1 << 0);
    /// Accept SOP' packets.
    pub const SOP_PRIME: Self = Self(1 << 1);
    /// Accept SOP'' packets.
    pub const SOP_DOUBLE_PRIME: Self = Self(1 << 2);
    /// Accept SOP' Debug packets.
    pub const SOP_PRIME_DEBUG: Self = Self(1 << 3);
    /// Accept SOP'' Debug packets.
    pub const SOP_DOUBLE_PRIME_DEBUG: Self = Self(1 << 4);

    pub(crate) const fn bits(self) -> u8 {
        self.0
    }
}

impl BitOr for ReceiveSopMask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for ReceiveSopMask {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Number of hardware retransmissions after the original transmission.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RetryCount {
    /// Do not automatically retry.
    #[default]
    Disabled,
    /// Retry once.
    One,
    /// Retry twice.
    Two,
    /// Retry three times.
    Three,
}

impl RetryCount {
    pub(crate) const fn control3_bits(self) -> u8 {
        match self {
            Self::Disabled => 0,
            Self::One => 0b010,
            Self::Two => 0b100,
            Self::Three => 0b110,
        }
    }

    pub(crate) const fn automatic(self) -> bool {
        !matches!(self, Self::Disabled)
    }
}

/// Explicit FUSB302B PD PHY automation configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PhyConfig {
    /// USB PD revision encoded in transmitted PD headers.
    pub pd_revision: PdRevision,
    /// Power role encoded in transmitted PD headers.
    pub power_role: PowerRole,
    /// Data role encoded in transmitted PD headers.
    pub data_role: DataRole,
    /// Enable hardware GoodCRC generation.
    pub auto_goodcrc: bool,
    /// Hardware retry policy. Disabled by default.
    pub retry_count: RetryCount,
    /// Allow hardware automatic soft reset handling.
    pub auto_soft_reset: bool,
    /// Allow hardware automatic hard reset handling.
    pub auto_hard_reset: bool,
    /// Receive SOP variants accepted by the PHY.
    pub receive_sop: ReceiveSopMask,
}

impl Default for PhyConfig {
    fn default() -> Self {
        Self {
            pd_revision: PdRevision::Rev30,
            power_role: PowerRole::Sink,
            data_role: DataRole::Ufp,
            auto_goodcrc: false,
            retry_count: RetryCount::Disabled,
            auto_soft_reset: false,
            auto_hard_reset: false,
            receive_sop: ReceiveSopMask::NONE,
        }
    }
}

/// Device identification value returned by the FUSB302B.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DeviceId(u8);

impl DeviceId {
    pub(crate) const fn new(bits: u8) -> Self {
        Self(bits)
    }

    /// Return the raw device ID register value.
    pub const fn bits(self) -> u8 {
        self.0
    }

    /// Return whether the ID uses the documented FUSB302B-family `0x9x` encoding.
    pub const fn is_fusb302b_family(self) -> bool {
        self.0 & 0xf0 == 0x90
    }
}

/// Snapshot of all FUSB302B clear-on-read interrupt registers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InterruptSnapshot {
    /// Raw `INTERRUPT` (0x42) bits.
    pub interrupt: u8,
    /// Raw `INTERRUPTA` (0x3e) bits.
    pub interrupt_a: u8,
    /// Raw `INTERRUPTB` (0x3f) bits.
    pub interrupt_b: u8,
}

/// Non-destructive snapshot of FUSB302B status registers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Status {
    /// Raw `STATUS0` (0x40) bits.
    pub status0: u8,
    /// Raw `STATUS1` (0x41) bits.
    pub status1: u8,
    /// Raw `STATUS0A` (0x3c) bits.
    pub status0a: u8,
    /// Raw `STATUS1A` (0x3d) bits.
    pub status1a: u8,
}
