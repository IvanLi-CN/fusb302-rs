//! FUSB302B driver implementation.

#[cfg(not(feature = "async"))]
use embedded_hal::i2c::I2c;
#[cfg(feature = "async")]
use embedded_hal_async::i2c::I2c;

use crate::{
    DEFAULT_ADDRESS, DeviceId, InterruptSnapshot, MAX_PD_PAYLOAD_BYTES, PdPacket, PhyConfig,
    ReceiveSopMask, SopType, Status,
    error::{Error, ReceiveError},
    registers::{
        CONTROL0_TX_FLUSH, CONTROL1_ENSOP1, CONTROL1_ENSOP1DB, CONTROL1_ENSOP2, CONTROL1_ENSOP2DB,
        CONTROL1_RX_FLUSH, CONTROL2_MODE_DFP, CONTROL2_MODE_DRP, CONTROL2_MODE_UFP,
        CONTROL2_TOGGLE, CONTROL3_AUTO_HARD_RESET, CONTROL3_AUTO_RETRY, CONTROL3_AUTO_SOFT_RESET,
        CONTROL3_RETRY_COUNT_MASK, CcPin, CcPull, DataRole, POWER_ALL, PdRevision, PowerRole,
        RESET_PD, RESET_SW, Register, STATUS1_RX_EMPTY, SWITCHES0_PDWN1, SWITCHES0_PDWN2,
        SWITCHES0_PU_EN1, SWITCHES0_PU_EN2, SWITCHES0_VCONN_CC1, SWITCHES0_VCONN_CC2,
        SWITCHES1_AUTO_CRC, SWITCHES1_DATA_ROLE, SWITCHES1_POWER_ROLE, SWITCHES1_SPEC_REV_MASK,
        SWITCHES1_TXCC1, SWITCHES1_TXCC2, ToggleMode,
    },
};

const FIFO_TX_ON: u8 = 0xa1;
const FIFO_SYNC1: u8 = 0x12;
const FIFO_SYNC2: u8 = 0x13;
const FIFO_SYNC3: u8 = 0x1b;
const FIFO_RESET1: u8 = 0x15;
const FIFO_RESET2: u8 = 0x16;
const FIFO_PACKSYM: u8 = 0x80;
const FIFO_JAM_CRC: u8 = 0xff;
const FIFO_EOP: u8 = 0x14;
const FIFO_TX_OFF: u8 = 0xfe;
const FIFO_SOP_MASK: u8 = 0xe0;
const FIFO_SOP_PREFIX: u8 = 0xe0;
const FIFO_TX_MAX: usize = 40;

/// FUSB302B-family PHY driver that owns its I2C bus.
pub struct Fusb302<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C> Fusb302<I2C> {
    /// Construct a driver at [`DEFAULT_ADDRESS`].
    pub fn new(i2c: I2C) -> Self {
        Self::with_address(i2c, DEFAULT_ADDRESS)
    }

    /// Construct a driver with an explicitly selected 7-bit I2C address.
    pub fn with_address(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }

    /// Return the configured 7-bit I2C address.
    pub const fn address(&self) -> u8 {
        self.address
    }

    /// Release ownership of the I2C bus.
    pub fn release(self) -> I2C {
        self.i2c
    }
}

#[maybe_async_cfg::maybe(
    sync(cfg(not(feature = "async")), self = "Fusb302",),
    async(feature = "async", keep_self)
)]
impl<I2C> Fusb302<I2C>
where
    I2C: I2c,
{
    /// Read the FUSB302B device ID.
    pub async fn device_id(&mut self) -> Result<DeviceId, Error<I2C::Error>> {
        Ok(DeviceId::new(self.read_register(Register::DeviceId).await?))
    }

    /// Reset the whole device.
    pub async fn software_reset(&mut self) -> Result<(), Error<I2C::Error>> {
        self.write_register(Register::Reset, RESET_SW).await
    }

    /// Reset the USB PD protocol engine without changing the I2C address.
    pub async fn pd_reset(&mut self) -> Result<(), Error<I2C::Error>> {
        self.write_register(Register::Reset, RESET_PD).await
    }

    /// Reset the device and flush both FIFOs without enabling CC, BMC, or automation.
    pub async fn init(&mut self) -> Result<(), Error<I2C::Error>> {
        self.software_reset().await?;
        self.flush_fifos().await
    }

    /// Apply the explicit PD PHY configuration. This enables the BMC power domain only.
    pub async fn configure_phy(&mut self, config: PhyConfig) -> Result<(), Error<I2C::Error>> {
        self.write_register(Register::Power, POWER_ALL).await?;

        self.update_register(Register::Switches1, |value| {
            let mut next = value
                & !(SWITCHES1_POWER_ROLE
                    | SWITCHES1_SPEC_REV_MASK
                    | SWITCHES1_DATA_ROLE
                    | SWITCHES1_AUTO_CRC);
            if matches!(config.power_role, PowerRole::Source) {
                next |= SWITCHES1_POWER_ROLE;
            }
            next |= match config.pd_revision {
                PdRevision::Rev20 => 0b01 << 5,
                PdRevision::Rev30 => 0b10 << 5,
            };
            if matches!(config.data_role, DataRole::Dfp) {
                next |= SWITCHES1_DATA_ROLE;
            }
            if config.auto_goodcrc {
                next |= SWITCHES1_AUTO_CRC;
            }
            next
        })
        .await?;

        self.update_register(Register::Control1, |value| {
            let mask = CONTROL1_ENSOP1 | CONTROL1_ENSOP2 | CONTROL1_ENSOP1DB | CONTROL1_ENSOP2DB;
            (value & !mask) | receive_sop_bits(config.receive_sop)
        })
        .await?;

        self.update_register(Register::Control3, |value| {
            let mask = CONTROL3_AUTO_HARD_RESET
                | CONTROL3_AUTO_SOFT_RESET
                | CONTROL3_RETRY_COUNT_MASK
                | CONTROL3_AUTO_RETRY;
            let mut next = value & !mask;
            if config.auto_hard_reset {
                next |= CONTROL3_AUTO_HARD_RESET;
            }
            if config.auto_soft_reset {
                next |= CONTROL3_AUTO_SOFT_RESET;
            }
            next |= config.retry_count.control3_bits();
            if config.retry_count.automatic() {
                next |= CONTROL3_AUTO_RETRY;
            }
            next
        })
        .await
    }

    /// Apply one CC pull resistor while preserving the other CC pin and unrelated bits.
    pub async fn set_cc_pull(&mut self, pin: CcPin, pull: CcPull) -> Result<(), Error<I2C::Error>> {
        self.update_register(Register::Switches0, |value| {
            let (up, down) = match pin {
                CcPin::Cc1 => (SWITCHES0_PU_EN1, SWITCHES0_PDWN1),
                CcPin::Cc2 => (SWITCHES0_PU_EN2, SWITCHES0_PDWN2),
            };
            match pull {
                CcPull::Open => value & !(up | down),
                CcPull::Down => (value & !up) | down,
                CcPull::Up => (value & !down) | up,
            }
        })
        .await
    }

    /// Select the CC pin used to supply VCONN, or disable VCONN.
    pub async fn set_vconn(&mut self, pin: Option<CcPin>) -> Result<(), Error<I2C::Error>> {
        self.update_register(Register::Switches0, |value| {
            let mut next = value & !(SWITCHES0_VCONN_CC1 | SWITCHES0_VCONN_CC2);
            next |= match pin {
                Some(CcPin::Cc1) => SWITCHES0_VCONN_CC1,
                Some(CcPin::Cc2) => SWITCHES0_VCONN_CC2,
                None => 0,
            };
            next
        })
        .await
    }

    /// Select the CC pin used for PD BMC transmission.
    pub async fn set_tx_cc(&mut self, pin: CcPin) -> Result<(), Error<I2C::Error>> {
        self.update_register(Register::Switches1, |value| {
            let mut next = value & !(SWITCHES1_TXCC1 | SWITCHES1_TXCC2);
            next |= match pin {
                CcPin::Cc1 => SWITCHES1_TXCC1,
                CcPin::Cc2 => SWITCHES1_TXCC2,
            };
            next
        })
        .await
    }

    /// Start FUSB302B autonomous Type-C toggling with an explicit role mode.
    pub async fn start_toggle(&mut self, mode: ToggleMode) -> Result<(), Error<I2C::Error>> {
        let mode_bits = match mode {
            ToggleMode::Source => CONTROL2_MODE_DFP,
            ToggleMode::Sink => CONTROL2_MODE_UFP,
            ToggleMode::DualRole => CONTROL2_MODE_DRP,
        };
        self.update_register(Register::Control2, |value| {
            (value & !0b111) | mode_bits | CONTROL2_TOGGLE
        })
        .await
    }

    /// Stop autonomous Type-C toggling while retaining the currently selected mode bits.
    pub async fn stop_toggle(&mut self) -> Result<(), Error<I2C::Error>> {
        self.update_register(Register::Control2, |value| value & !CONTROL2_TOGGLE)
            .await
    }

    /// Flush both hardware FIFOs.
    pub async fn flush_fifos(&mut self) -> Result<(), Error<I2C::Error>> {
        self.update_register(Register::Control0, |value| value | CONTROL0_TX_FLUSH)
            .await?;
        self.update_register(Register::Control1, |value| value | CONTROL1_RX_FLUSH)
            .await
    }

    /// Read a non-destructive four-register status snapshot.
    pub async fn read_status(&mut self) -> Result<Status, Error<I2C::Error>> {
        let status0a = self.read_register(Register::Status0A).await?;
        let status1a = self.read_register(Register::Status1A).await?;
        let status0 = self.read_register(Register::Status0).await?;
        let status1 = self.read_register(Register::Status1).await?;
        Ok(Status {
            status0,
            status1,
            status0a,
            status1a,
        })
    }

    /// Read all clear-on-read interrupt registers in one contiguous I2C read.
    pub async fn read_interrupts(&mut self) -> Result<InterruptSnapshot, Error<I2C::Error>> {
        let mut registers = [0; 7];
        self.i2c
            .write_read(
                self.address,
                &[Register::InterruptA.address()],
                &mut registers,
            )
            .await
            .map_err(Error::I2c)?;
        Ok(InterruptSnapshot {
            interrupt_a: registers[0],
            interrupt_b: registers[1],
            interrupt: registers[6],
        })
    }

    /// Send one physical USB PD packet through the token FIFO.
    pub async fn transmit(&mut self, packet: &PdPacket) -> Result<(), Error<I2C::Error>> {
        let payload = packet.payload();
        let packet_len = payload.len() + 2;
        let mut frame = [0; FIFO_TX_MAX];
        let mut cursor = 0;

        for token in tx_sop_tokens(packet.sop()) {
            frame[cursor] = token;
            cursor += 1;
        }
        frame[cursor] = FIFO_PACKSYM | packet_len as u8;
        cursor += 1;
        let header = packet.header().to_le_bytes();
        frame[cursor..cursor + header.len()].copy_from_slice(&header);
        cursor += header.len();
        frame[cursor..cursor + payload.len()].copy_from_slice(payload);
        cursor += payload.len();
        for token in [FIFO_JAM_CRC, FIFO_EOP, FIFO_TX_OFF, FIFO_TX_ON] {
            frame[cursor] = token;
            cursor += 1;
        }

        self.write_fifo(&frame[..cursor]).await
    }

    /// Send an explicit USB PD hard-reset ordered set.
    pub async fn transmit_hard_reset(&mut self) -> Result<(), Error<I2C::Error>> {
        self.write_fifo(&[
            FIFO_RESET1,
            FIFO_RESET1,
            FIFO_RESET1,
            FIFO_RESET2,
            FIFO_TX_ON,
        ])
        .await
    }

    /// Read one complete received physical packet, if the receive FIFO is non-empty.
    pub async fn receive(&mut self) -> Result<Option<PdPacket>, Error<I2C::Error>> {
        if self.read_register(Register::Status1).await? & STATUS1_RX_EMPTY != 0 {
            return Ok(None);
        }

        let mut prefix = [0; 3];
        self.read_fifo(&mut prefix).await?;
        let sop = rx_sop_type(prefix[0]).ok_or(ReceiveError::InvalidSopToken(prefix[0]))?;
        let header = u16::from_le_bytes([prefix[1], prefix[2]]);
        let payload_len = PdPacket::payload_len_from_header(header);
        let mut tail = [0; MAX_PD_PAYLOAD_BYTES + 4];
        self.read_fifo(&mut tail[..payload_len + 4]).await?;
        let mut payload = [0; MAX_PD_PAYLOAD_BYTES];
        payload[..payload_len].copy_from_slice(&tail[..payload_len]);
        Ok(Some(PdPacket::from_fifo(sop, header, payload)))
    }

    async fn read_register(&mut self, register: Register) -> Result<u8, Error<I2C::Error>> {
        let mut value = [0];
        self.i2c
            .write_read(self.address, &[register.address()], &mut value)
            .await
            .map_err(Error::I2c)?;
        Ok(value[0])
    }

    async fn write_register(
        &mut self,
        register: Register,
        value: u8,
    ) -> Result<(), Error<I2C::Error>> {
        self.i2c
            .write(self.address, &[register.address(), value])
            .await
            .map_err(Error::I2c)
    }

    async fn update_register(
        &mut self,
        register: Register,
        update: impl FnOnce(u8) -> u8,
    ) -> Result<(), Error<I2C::Error>> {
        let value = self.read_register(register).await?;
        self.write_register(register, update(value)).await
    }

    async fn write_fifo(&mut self, tokens: &[u8]) -> Result<(), Error<I2C::Error>> {
        let mut write = [0; FIFO_TX_MAX + 1];
        write[0] = Register::Fifo.address();
        write[1..tokens.len() + 1].copy_from_slice(tokens);
        self.i2c
            .write(self.address, &write[..tokens.len() + 1])
            .await
            .map_err(Error::I2c)
    }

    async fn read_fifo(&mut self, buffer: &mut [u8]) -> Result<(), Error<I2C::Error>> {
        self.i2c
            .write_read(self.address, &[Register::Fifo.address()], buffer)
            .await
            .map_err(Error::I2c)
    }
}

fn receive_sop_bits(mask: ReceiveSopMask) -> u8 {
    let mut bits = 0;
    if mask.bits() & ReceiveSopMask::SOP_PRIME.bits() != 0 {
        bits |= CONTROL1_ENSOP1;
    }
    if mask.bits() & ReceiveSopMask::SOP_DOUBLE_PRIME.bits() != 0 {
        bits |= CONTROL1_ENSOP2;
    }
    if mask.bits() & ReceiveSopMask::SOP_PRIME_DEBUG.bits() != 0 {
        bits |= CONTROL1_ENSOP2DB;
    }
    bits
}

fn tx_sop_tokens(sop: SopType) -> [u8; 4] {
    match sop {
        SopType::Sop => [FIFO_SYNC1, FIFO_SYNC1, FIFO_SYNC1, FIFO_SYNC2],
        SopType::SopPrime => [FIFO_SYNC1, FIFO_SYNC1, FIFO_SYNC3, FIFO_SYNC3],
        SopType::SopDoublePrime => [FIFO_SYNC1, FIFO_SYNC3, FIFO_SYNC1, FIFO_SYNC3],
        SopType::SopPrimeDebug => [FIFO_SYNC1, FIFO_SYNC1, FIFO_SYNC3, FIFO_SYNC2],
        SopType::SopDoublePrimeDebug => [FIFO_SYNC1, FIFO_SYNC3, FIFO_SYNC1, FIFO_SYNC2],
    }
}

fn rx_sop_type(token: u8) -> Option<SopType> {
    match token & FIFO_SOP_MASK {
        FIFO_SOP_PREFIX => Some(SopType::Sop),
        0xc0 => Some(SopType::SopPrime),
        0xa0 => Some(SopType::SopDoublePrime),
        0x80 => Some(SopType::SopPrimeDebug),
        0x60 => Some(SopType::SopDoublePrimeDebug),
        _ => None,
    }
}
