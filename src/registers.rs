//! FUSB302B register names and strongly typed electrical controls.

/// FUSB302B register address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Register {
    /// Device identity and revision.
    DeviceId = 0x01,
    /// CC, VCONN, measurement, and pull control.
    Switches0 = 0x02,
    /// PD header and transmit CC control.
    Switches1 = 0x03,
    /// Comparator measurement configuration.
    Measure = 0x04,
    /// Transmit and host-current control.
    Control0 = 0x06,
    /// Receive SOP masks and FIFO control.
    Control1 = 0x07,
    /// Type-C toggle control.
    Control2 = 0x08,
    /// PD automatic retry and reset control.
    Control3 = 0x09,
    /// Primary interrupt mask.
    Mask = 0x0a,
    /// Power control.
    Power = 0x0b,
    /// Software and PD reset.
    Reset = 0x0c,
    /// Extended interrupt mask A.
    MaskA = 0x0e,
    /// Extended interrupt mask B.
    MaskB = 0x0f,
    /// Extended status A.
    Status0A = 0x3c,
    /// Extended status B.
    Status1A = 0x3d,
    /// Clear-on-read interrupt A.
    InterruptA = 0x3e,
    /// Clear-on-read interrupt B.
    InterruptB = 0x3f,
    /// Primary status 0.
    Status0 = 0x40,
    /// Primary status 1.
    Status1 = 0x41,
    /// Clear-on-read primary interrupt.
    Interrupt = 0x42,
    /// Receive and transmit FIFO.
    Fifo = 0x43,
}

impl Register {
    /// Return the register's I2C address.
    pub const fn address(self) -> u8 {
        self as u8
    }
}

/// CC pin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CcPin {
    /// CC1.
    Cc1,
    /// CC2.
    Cc2,
}

/// Termination applied to one CC pin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CcPull {
    /// Leave the pin electrically open.
    Open,
    /// Apply the FUSB302B pull-down (Rd).
    Down,
    /// Apply the FUSB302B pull-up (Rp).
    Up,
}

/// USB PD power role encoded in a transmitted PD header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PowerRole {
    /// Sink/UFP power role.
    Sink,
    /// Source/DFP power role.
    Source,
}

/// USB PD data role encoded in a transmitted PD header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DataRole {
    /// UFP data role.
    Ufp,
    /// DFP data role.
    Dfp,
}

/// USB PD revision encoded in a transmitted PD header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PdRevision {
    /// USB PD revision 2.0.
    Rev20,
    /// USB PD revision 3.0.
    Rev30,
}

/// Type-C role detection mode used by FUSB302B autonomous toggling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ToggleMode {
    /// Toggle while looking for a sink (DFP/source behavior).
    Source,
    /// Toggle while looking for a source (UFP/sink behavior).
    Sink,
    /// Toggle between source and sink behavior.
    DualRole,
}

pub(crate) const SWITCHES0_PU_EN2: u8 = 1 << 7;
pub(crate) const SWITCHES0_PU_EN1: u8 = 1 << 6;
pub(crate) const SWITCHES0_VCONN_CC2: u8 = 1 << 5;
pub(crate) const SWITCHES0_VCONN_CC1: u8 = 1 << 4;
pub(crate) const SWITCHES0_PDWN2: u8 = 1 << 1;
pub(crate) const SWITCHES0_PDWN1: u8 = 1;
pub(crate) const SWITCHES1_POWER_ROLE: u8 = 1 << 7;
pub(crate) const SWITCHES1_SPEC_REV_MASK: u8 = 0b11 << 5;
pub(crate) const SWITCHES1_DATA_ROLE: u8 = 1 << 4;
pub(crate) const SWITCHES1_AUTO_CRC: u8 = 1 << 2;
pub(crate) const SWITCHES1_TXCC2: u8 = 1 << 1;
pub(crate) const SWITCHES1_TXCC1: u8 = 1;
pub(crate) const CONTROL0_TX_FLUSH: u8 = 1 << 6;
pub(crate) const CONTROL1_ENSOP2DB: u8 = 1 << 6;
pub(crate) const CONTROL1_ENSOP1DB: u8 = 1 << 5;
pub(crate) const CONTROL1_RX_FLUSH: u8 = 1 << 2;
pub(crate) const CONTROL1_ENSOP2: u8 = 1 << 1;
pub(crate) const CONTROL1_ENSOP1: u8 = 1;
pub(crate) const CONTROL2_MODE_DFP: u8 = 0b110;
pub(crate) const CONTROL2_MODE_UFP: u8 = 0b100;
pub(crate) const CONTROL2_MODE_DRP: u8 = 0b010;
pub(crate) const CONTROL2_TOGGLE: u8 = 1;
pub(crate) const CONTROL3_AUTO_HARD_RESET: u8 = 1 << 4;
pub(crate) const CONTROL3_AUTO_SOFT_RESET: u8 = 1 << 3;
pub(crate) const CONTROL3_RETRY_COUNT_MASK: u8 = 0b110;
pub(crate) const CONTROL3_AUTO_RETRY: u8 = 1;
pub(crate) const POWER_ALL: u8 = 0x0f;
pub(crate) const RESET_PD: u8 = 1 << 1;
pub(crate) const RESET_SW: u8 = 1;
pub(crate) const STATUS1_RX_EMPTY: u8 = 1 << 5;
