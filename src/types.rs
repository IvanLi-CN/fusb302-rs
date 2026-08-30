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

/// Values written to the FUSB302B interrupt-mask registers.
///
/// A set bit masks its corresponding hardware interrupt source. The FUSB302B
/// defines the meaning of each bit in its register reference; this type keeps
/// the three mask registers configured as one atomic PHY setting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InterruptMasks {
    primary: u8,
    extended_a: u8,
    extended_b: u8,
}

impl InterruptMasks {
    /// Construct the values for `MASK`, `MASKA`, and `MASKB`, respectively.
    pub const fn new(primary: u8, extended_a: u8, extended_b: u8) -> Self {
        Self {
            primary,
            extended_a,
            extended_b,
        }
    }

    /// Mask every interrupt except the autonomous Type-C toggle completion signal.
    ///
    /// This is the documented mask set for polling [`Fusb302::take_toggle_result`].
    /// It keeps unrelated interrupt sources from asserting the shared interrupt line
    /// while preserving the FUSB302B `TOGDONE` latch.
    ///
    /// [`Fusb302::take_toggle_result`]: crate::Fusb302::take_toggle_result
    pub const fn toggle_detection() -> Self {
        Self::new(0xfe, 0xbf, 0x01)
    }

    pub(crate) const fn primary(self) -> u8 {
        self.primary
    }

    pub(crate) const fn extended_a(self) -> u8 {
        self.extended_a
    }

    pub(crate) const fn extended_b(self) -> u8 {
        self.extended_b
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
    /// USB PD revision encoded in hardware-generated GoodCRC headers.
    ///
    /// The default, [`PdRevision::Rev20`], uses the supported FUSB302B
    /// encoding. [`PdRevision::Rev30`] is an explicit FUSB302BMPX
    /// compatibility opt-in and requires target-hardware validation.
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
            pd_revision: PdRevision::Rev20,
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

impl InterruptSnapshot {
    /// Return whether an autonomous Type-C toggle operation completed.
    ///
    /// The FUSB302B clears this indication when its interrupt registers are read.
    /// Prefer [`crate::Fusb302::take_toggle_result`] when using the toggle engine.
    pub const fn toggle_done(self) -> bool {
        self.interrupt_a & (1 << 6) != 0
    }

    /// Return the completed PD-transmit outcome, if the hardware has one.
    ///
    /// A retry failure takes precedence because no GoodCRC was received for
    /// the packet. The FUSB302B clears this indication when its interrupt
    /// registers are read. Prefer [`crate::Fusb302::take_transmit_status`]
    /// when polling a transmitted packet.
    pub const fn transmit_status(self) -> Option<TransmitStatus> {
        if self.interrupt_a & (1 << 4) != 0 {
            Some(TransmitStatus::RetryFailed)
        } else if self.interrupt_a & (1 << 2) != 0 {
            Some(TransmitStatus::Sent)
        } else {
            None
        }
    }
}

/// Completion outcome of one PD packet sent by the FUSB302B.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TransmitStatus {
    /// The packet was transmitted and acknowledged before the retry limit.
    Sent,
    /// The packet exhausted the configured automatic retries without GoodCRC.
    RetryFailed,
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

/// Threshold configured into the FUSB302B VBUS measurement DAC.
///
/// The FUSB302B VBUS comparator increments in nominal 420 mV steps. The
/// threshold is a bounded typed value so downstream code never has to address
/// the MEASURE register or manufacture MDAC bit patterns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct VbusThreshold(u8);

impl VbusThreshold {
    /// A nominal 4.62 V threshold suitable for confirming vSafe5V.
    pub const VSAFE5: Self = Self(10);
    /// A nominal 8.82 V lower bound for a fixed 9 V contract.
    pub const NINE_VOLTS_MIN: Self = Self(20);
    /// A nominal 12.18 V upper guard for a fixed 9 V contract.
    pub const NINE_VOLTS_MAX: Self = Self(28);

    /// Construct a threshold from a validated FUSB302B DAC step (0 through
    /// 63). Returns `None` instead of allowing a value outside the hardware
    /// range.
    pub const fn from_mdac_step(step: u8) -> Option<Self> {
        if step <= 0x3f { Some(Self(step)) } else { None }
    }

    /// Return the FUSB302B DAC step selected by this threshold.
    pub const fn mdac_step(self) -> u8 {
        self.0
    }

    /// Return the nominal comparator threshold in millivolts.
    pub const fn nominal_millivolts(self) -> u16 {
        (self.0 as u16 + 1) * 420
    }
}

/// Result returned by the FUSB302B VBUS comparator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum VbusComparator {
    /// VBUS is at or below the configured threshold.
    AtOrBelowThreshold,
    /// VBUS is above the configured threshold.
    AboveThreshold,
}

/// Result of the FUSB302B autonomous Type-C toggle engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ToggleStatus {
    /// Toggle logic is still running.
    Running,
    /// A source termination was detected on CC1.
    SourceCc1,
    /// A source termination was detected on CC2.
    SourceCc2,
    /// A sink termination was detected on CC1.
    SinkCc1,
    /// A sink termination was detected on CC2.
    SinkCc2,
    /// An audio accessory was detected.
    AudioAccessory,
    /// The hardware returned an undefined TOGSS encoding.
    Unknown(u8),
}

impl ToggleStatus {
    /// Decode the three-bit TOGSS field from STATUS1A.
    pub const fn from_status1a(status1a: u8) -> Self {
        match (status1a >> 3) & 0x07 {
            0b000 => Self::Running,
            0b001 => Self::SourceCc1,
            0b010 => Self::SourceCc2,
            0b101 => Self::SinkCc1,
            0b110 => Self::SinkCc2,
            0b111 => Self::AudioAccessory,
            value => Self::Unknown(value),
        }
    }

    /// Return the settled CC pin, if the toggle engine found one.
    pub const fn settled_cc(self) -> Option<crate::CcPin> {
        match self {
            Self::SourceCc1 | Self::SinkCc1 => Some(crate::CcPin::Cc1),
            Self::SourceCc2 | Self::SinkCc2 => Some(crate::CcPin::Cc2),
            Self::Running | Self::AudioAccessory | Self::Unknown(_) => None,
        }
    }
}
