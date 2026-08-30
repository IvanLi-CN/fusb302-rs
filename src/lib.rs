#![no_std]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

//! Register-level driver for the FUSB302B family USB Type-C and USB PD BMC PHY.
//!
//! This crate owns only physical-layer configuration, status, FIFO, and packet transport. It
//! deliberately does not implement USB Power Delivery policy or contract negotiation.

mod driver;
mod error;
pub mod registers;
mod types;

pub use driver::Fusb302;
pub use error::{Error, PacketError, ReceiveError};
pub use registers::{CcPin, CcPull, DataRole, HostCurrent, PdRevision, PowerRole, ToggleMode};
pub use types::{
    DeviceId, InterruptMasks, InterruptSnapshot, PdPacket, PhyConfig, ReceiveSopMask, RetryCount,
    SopType, Status, VbusComparator, VbusThreshold,
};

/// Default 7-bit I2C address of FUSB302B-family devices.
pub const DEFAULT_ADDRESS: u8 = 0x22;

/// Largest USB PD packet payload the FUSB302B FIFO transport accepts.
pub const MAX_PD_PAYLOAD_BYTES: usize = 28;
