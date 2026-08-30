use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
use fusb302::{DEFAULT_ADDRESS, Fusb302, PacketError, PdPacket, SopType};

#[cfg(not(feature = "async"))]
use embedded_hal::i2c::ErrorKind;
#[cfg(not(feature = "async"))]
use fusb302::{
    CcPin, CcPull, DataRole, Error, HostCurrent, InterruptMasks, PdRevision, PhyConfig, PowerRole,
    ReceiveError, ReceiveSopMask, RetryCount, ToggleMode, ToggleStatus, TransmitStatus,
    VbusComparator, VbusThreshold,
};
#[cfg(feature = "async")]
use fusb302::{CcPin, InterruptMasks, PhyConfig};

#[cfg(not(feature = "async"))]
const DEVICE_ID: u8 = 0x01;
const SWITCHES0: u8 = 0x02;
const SWITCHES1: u8 = 0x03;
#[cfg(not(feature = "async"))]
const MEASURE: u8 = 0x04;
const CONTROL0: u8 = 0x06;
const CONTROL1: u8 = 0x07;
#[cfg(not(feature = "async"))]
const CONTROL2: u8 = 0x08;
const CONTROL3: u8 = 0x09;
const MASK: u8 = 0x0a;
const POWER: u8 = 0x0b;
const RESET: u8 = 0x0c;
const MASKA: u8 = 0x0e;
const MASKB: u8 = 0x0f;
const STATUS1: u8 = 0x41;
#[cfg(not(feature = "async"))]
const STATUS0A: u8 = 0x3c;
#[cfg(not(feature = "async"))]
const STATUS1A: u8 = 0x3d;
#[cfg(not(feature = "async"))]
const STATUS0: u8 = 0x40;
#[cfg(not(feature = "async"))]
const INTERRUPTA: u8 = 0x3e;
const FIFO: u8 = 0x43;

fn write_read(register: u8, value: u8) -> I2cTransaction {
    I2cTransaction::write_read(DEFAULT_ADDRESS, vec![register], vec![value])
}

#[test]
fn construction_uses_default_address_and_releases_bus() {
    let bus = I2cMock::new(&[]);
    let driver = Fusb302::new(bus);
    assert_eq!(driver.address(), DEFAULT_ADDRESS);
    driver.release().done();
}

#[test]
fn packet_rejects_mismatched_data_object_count() {
    assert_eq!(
        PdPacket::new(SopType::Sop, 1 << 12, &[]),
        Err(PacketError::PayloadLengthMismatch {
            expected: 4,
            actual: 0,
        })
    );
}

#[test]
fn packet_rejects_payload_larger_than_fusb_fifo_limit() {
    assert_eq!(
        PdPacket::new(SopType::Sop, 0, &[0; 29]),
        Err(PacketError::PayloadTooLong { actual: 29 })
    );
}

#[cfg(not(feature = "async"))]
#[test]
fn device_id_is_read_as_a_typed_fusb302b_family_value() {
    let expectations = [write_read(DEVICE_ID, 0x9a)];
    let bus = I2cMock::new(&expectations);
    let mut driver = Fusb302::new(bus);

    let id = driver.device_id().unwrap();
    assert_eq!(id.bits(), 0x9a);
    assert!(id.is_fusb302b_family());
    driver.release().done();
}

#[cfg(not(feature = "async"))]
#[test]
fn init_resets_then_flushes_both_fifos() {
    let expectations = [
        I2cTransaction::write(DEFAULT_ADDRESS, vec![RESET, 0x01]),
        write_read(CONTROL0, 0x32),
        I2cTransaction::write(DEFAULT_ADDRESS, vec![CONTROL0, 0x62]),
        write_read(CONTROL1, 0x80),
        I2cTransaction::write(DEFAULT_ADDRESS, vec![CONTROL1, 0x04]),
    ];
    let bus = I2cMock::new(&expectations);
    let mut driver = Fusb302::new(bus);

    driver.init().unwrap();
    driver.release().done();
}

#[cfg(not(feature = "async"))]
#[test]
fn config_filters_command_and_reserved_bits_and_enables_requested_automation() {
    let expectations = [
        I2cTransaction::write(DEFAULT_ADDRESS, vec![POWER, 0x0f]),
        write_read(SWITCHES1, 0x0b),
        I2cTransaction::write(DEFAULT_ADDRESS, vec![SWITCHES1, 0xb7]),
        write_read(CONTROL1, 0x94),
        I2cTransaction::write(DEFAULT_ADDRESS, vec![CONTROL1, 0x13]),
        write_read(CONTROL3, 0xe0),
        I2cTransaction::write(DEFAULT_ADDRESS, vec![CONTROL3, 0x2f]),
    ];
    let bus = I2cMock::new(&expectations);
    let mut driver = Fusb302::new(bus);
    let config = PhyConfig {
        pd_revision: PdRevision::Rev20,
        power_role: PowerRole::Source,
        data_role: DataRole::Dfp,
        auto_goodcrc: true,
        retry_count: RetryCount::Three,
        auto_soft_reset: true,
        auto_hard_reset: false,
        receive_sop: ReceiveSopMask::SOP
            | ReceiveSopMask::SOP_PRIME
            | ReceiveSopMask::SOP_DOUBLE_PRIME,
    };

    driver.configure_phy(config).unwrap();
    driver.release().done();
}

#[cfg(not(feature = "async"))]
#[test]
fn config_writes_pd3_goodcrc_revision_when_explicitly_requested() {
    let expectations = [
        I2cTransaction::write(DEFAULT_ADDRESS, vec![POWER, 0x0f]),
        write_read(SWITCHES1, 0),
        I2cTransaction::write(DEFAULT_ADDRESS, vec![SWITCHES1, 0x44]),
        write_read(CONTROL1, 0),
        I2cTransaction::write(DEFAULT_ADDRESS, vec![CONTROL1, 0]),
        write_read(CONTROL3, 0),
        I2cTransaction::write(DEFAULT_ADDRESS, vec![CONTROL3, 0]),
    ];
    let bus = I2cMock::new(&expectations);
    let mut driver = Fusb302::new(bus);
    let config = PhyConfig {
        pd_revision: PdRevision::Rev30,
        auto_goodcrc: true,
        ..PhyConfig::default()
    };

    driver.configure_phy(config).unwrap();
    driver.release().done();
}

#[cfg(not(feature = "async"))]
#[test]
fn config_maps_debug_sop_masks_to_their_matching_control1_bits() {
    let expectations = [
        I2cTransaction::write(DEFAULT_ADDRESS, vec![POWER, 0x0f]),
        write_read(SWITCHES1, 0),
        I2cTransaction::write(DEFAULT_ADDRESS, vec![SWITCHES1, 0x20]),
        write_read(CONTROL1, 0),
        I2cTransaction::write(DEFAULT_ADDRESS, vec![CONTROL1, 0x60]),
        write_read(CONTROL3, 0),
        I2cTransaction::write(DEFAULT_ADDRESS, vec![CONTROL3, 0]),
    ];
    let bus = I2cMock::new(&expectations);
    let mut driver = Fusb302::new(bus);
    let config = PhyConfig {
        receive_sop: ReceiveSopMask::SOP_PRIME_DEBUG | ReceiveSopMask::SOP_DOUBLE_PRIME_DEBUG,
        ..PhyConfig::default()
    };

    driver.configure_phy(config).unwrap();
    driver.release().done();
}

#[cfg(not(feature = "async"))]
#[test]
fn cc_vconn_and_toggle_operations_are_precise_rmw_transactions() {
    let expectations = [
        write_read(SWITCHES0, 0xa8),
        I2cTransaction::write(DEFAULT_ADDRESS, vec![SWITCHES0, 0xa9]),
        write_read(SWITCHES0, 0xa9),
        I2cTransaction::write(DEFAULT_ADDRESS, vec![SWITCHES0, 0x99]),
        write_read(SWITCHES1, 0xfc),
        I2cTransaction::write(DEFAULT_ADDRESS, vec![SWITCHES1, 0xf5]),
        write_read(CONTROL2, 0xe0),
        I2cTransaction::write(DEFAULT_ADDRESS, vec![CONTROL2, 0xe3]),
        write_read(CONTROL2, 0xe3),
        I2cTransaction::write(DEFAULT_ADDRESS, vec![CONTROL2, 0xe2]),
    ];
    let bus = I2cMock::new(&expectations);
    let mut driver = Fusb302::new(bus);

    driver.set_cc_pull(CcPin::Cc1, CcPull::Down).unwrap();
    driver.set_vconn(Some(CcPin::Cc1)).unwrap();
    driver.set_tx_cc(CcPin::Cc1).unwrap();
    driver.start_toggle(ToggleMode::DualRole).unwrap();
    driver.stop_toggle().unwrap();
    driver.release().done();
}

#[cfg(not(feature = "async"))]
#[test]
fn receiver_configuration_selects_measurement_host_current_and_interrupt_masks() {
    let expectations = [
        write_read(SWITCHES0, 0xa3),
        I2cTransaction::write(DEFAULT_ADDRESS, vec![SWITCHES0, 0xab]),
        write_read(CONTROL0, 0x22),
        I2cTransaction::write(DEFAULT_ADDRESS, vec![CONTROL0, 0x26]),
        I2cTransaction::write(DEFAULT_ADDRESS, vec![MASK, 0x7d]),
        I2cTransaction::write(DEFAULT_ADDRESS, vec![MASKA, 0xe0]),
        I2cTransaction::write(DEFAULT_ADDRESS, vec![MASKB, 0x00]),
    ];
    let bus = I2cMock::new(&expectations);
    let mut driver = Fusb302::new(bus);

    driver.set_measure_cc(Some(CcPin::Cc2)).unwrap();
    driver.set_host_current(HostCurrent::Default).unwrap();
    driver
        .set_interrupt_masks(InterruptMasks::new(0x7d, 0xe0, 0x00))
        .unwrap();
    driver.release().done();
}

#[cfg(not(feature = "async"))]
#[test]
fn vbus_measurement_uses_a_typed_threshold_and_comparator_result() {
    let expectations = [
        write_read(SWITCHES0, 0xaf),
        I2cTransaction::write(DEFAULT_ADDRESS, vec![SWITCHES0, 0xa3]),
        write_read(MEASURE, 0xb1),
        I2cTransaction::write(DEFAULT_ADDRESS, vec![MEASURE, 0xd4]),
        write_read(STATUS0, 0x20),
    ];
    let bus = I2cMock::new(&expectations);
    let mut driver = Fusb302::new(bus);

    driver
        .configure_vbus_measurement(VbusThreshold::NINE_VOLTS_MIN)
        .unwrap();
    assert_eq!(
        driver.read_vbus_comparator().unwrap(),
        VbusComparator::AboveThreshold
    );
    assert_eq!(VbusThreshold::NINE_VOLTS_MIN.nominal_millivolts(), 8_820);
    assert!(VbusThreshold::from_mdac_step(64).is_none());
    driver.release().done();
}

#[cfg(not(feature = "async"))]
#[test]
fn interrupt_snapshot_is_one_contiguous_clear_on_read_transaction() {
    let expectations = [I2cTransaction::write_read(
        DEFAULT_ADDRESS,
        vec![INTERRUPTA],
        vec![0x04, 0x01, 0, 0, 0x80],
    )];
    let bus = I2cMock::new(&expectations);
    let mut driver = Fusb302::new(bus);

    let interrupts = driver.read_interrupts().unwrap();
    assert_eq!(interrupts.interrupt_a, 0x04);
    assert_eq!(interrupts.interrupt_b, 0x01);
    assert_eq!(interrupts.interrupt, 0x80);
    assert!(!interrupts.toggle_done());
    driver.release().done();
}

#[cfg(not(feature = "async"))]
#[test]
fn toggle_detection_clears_stale_interrupts_before_starting_toggle() {
    let expectations = [
        I2cTransaction::write(DEFAULT_ADDRESS, vec![POWER, 0x01]),
        I2cTransaction::write(DEFAULT_ADDRESS, vec![MASK, 0xfe]),
        I2cTransaction::write(DEFAULT_ADDRESS, vec![MASKA, 0xbf]),
        I2cTransaction::write(DEFAULT_ADDRESS, vec![MASKB, 0x01]),
        I2cTransaction::write_read(DEFAULT_ADDRESS, vec![INTERRUPTA], vec![0, 0, 0, 0, 0]),
        write_read(CONTROL2, 0xe0),
        I2cTransaction::write(DEFAULT_ADDRESS, vec![CONTROL2, 0xe7]),
    ];
    let bus = I2cMock::new(&expectations);
    let mut driver = Fusb302::new(bus);

    driver.arm_toggle_detection(ToggleMode::Source).unwrap();
    driver.release().done();
}

#[cfg(not(feature = "async"))]
#[test]
fn toggle_result_requires_togdone_before_reading_status() {
    let expectations = [
        I2cTransaction::write_read(DEFAULT_ADDRESS, vec![INTERRUPTA], vec![0, 0, 0, 0, 0]),
        I2cTransaction::write_read(DEFAULT_ADDRESS, vec![INTERRUPTA], vec![0x40, 0, 0, 0, 0]),
        write_read(STATUS1A, 0x08),
        I2cTransaction::write(DEFAULT_ADDRESS, vec![POWER, 0x0f]),
    ];
    let bus = I2cMock::new(&expectations);
    let mut driver = Fusb302::new(bus);

    assert_eq!(driver.take_toggle_result().unwrap(), None);
    assert_eq!(
        driver.take_toggle_result().unwrap(),
        Some(ToggleStatus::SourceCc1)
    );
    driver.release().done();
}

#[cfg(not(feature = "async"))]
#[test]
fn transmit_status_reports_goodcrc_acknowledgement_or_retry_failure() {
    let expectations = [
        I2cTransaction::write_read(DEFAULT_ADDRESS, vec![INTERRUPTA], vec![0x04, 0, 0, 0, 0]),
        I2cTransaction::write_read(DEFAULT_ADDRESS, vec![INTERRUPTA], vec![0x14, 0, 0, 0, 0]),
        I2cTransaction::write_read(DEFAULT_ADDRESS, vec![INTERRUPTA], vec![0, 0, 0, 0, 0]),
    ];
    let bus = I2cMock::new(&expectations);
    let mut driver = Fusb302::new(bus);

    assert_eq!(
        driver.take_transmit_status().unwrap(),
        Some(TransmitStatus::Sent)
    );
    assert_eq!(
        driver.take_transmit_status().unwrap(),
        Some(TransmitStatus::RetryFailed)
    );
    assert_eq!(driver.take_transmit_status().unwrap(), None);
    driver.release().done();
}

#[cfg(not(feature = "async"))]
#[test]
fn status_snapshot_has_no_clear_on_read_side_effect() {
    let expectations = [
        write_read(STATUS0A, 0x10),
        write_read(STATUS1A, 0x09),
        write_read(STATUS0, 0x90),
        write_read(STATUS1, 0x28),
    ];
    let bus = I2cMock::new(&expectations);
    let mut driver = Fusb302::new(bus);

    let status = driver.read_status().unwrap();
    assert_eq!(status.status0a, 0x10);
    assert_eq!(status.status1a, 0x09);
    assert_eq!(status.status0, 0x90);
    assert_eq!(status.status1, 0x28);
    driver.release().done();
}

#[cfg(not(feature = "async"))]
#[test]
fn transmit_encodes_sop_packet_and_crc_tokens() {
    let packet = PdPacket::new(SopType::Sop, 1 << 12, &[1, 2, 3, 4]).unwrap();
    let expectations = [
        write_read(STATUS1, 0x08),
        I2cTransaction::write(
            DEFAULT_ADDRESS,
            vec![
                FIFO, 0x12, 0x12, 0x12, 0x13, 0x86, 0x00, 0x10, 1, 2, 3, 4, 0xff, 0x14, 0xfe, 0xa1,
            ],
        ),
    ];
    let bus = I2cMock::new(&expectations);
    let mut driver = Fusb302::new(bus);

    driver.transmit(&packet).unwrap();
    driver.release().done();
}

#[cfg(not(feature = "async"))]
#[test]
fn transmit_encodes_sop_prime_with_its_distinct_ordered_set() {
    let packet = PdPacket::new(SopType::SopPrime, 0, &[]).unwrap();
    let expectations = [
        write_read(STATUS1, 0x08),
        I2cTransaction::write(
            DEFAULT_ADDRESS,
            vec![
                FIFO, 0x12, 0x12, 0x1b, 0x1b, 0x82, 0x00, 0x00, 0xff, 0x14, 0xfe, 0xa1,
            ],
        ),
    ];
    let bus = I2cMock::new(&expectations);
    let mut driver = Fusb302::new(bus);

    driver.transmit(&packet).unwrap();
    driver.release().done();
}

#[cfg(not(feature = "async"))]
#[test]
fn hard_reset_uses_reset_tokens_and_txon_command() {
    let expectations = [
        write_read(STATUS1, 0x08),
        I2cTransaction::write(DEFAULT_ADDRESS, vec![FIFO, 0x15, 0x15, 0x15, 0x16, 0xa1]),
    ];
    let bus = I2cMock::new(&expectations);
    let mut driver = Fusb302::new(bus);

    driver.transmit_hard_reset().unwrap();
    driver.release().done();
}

#[cfg(not(feature = "async"))]
#[test]
fn receive_drains_a_full_fifo_packet_and_discards_crc() {
    let expectations = [
        write_read(STATUS1, 0x10),
        I2cTransaction::write_read(DEFAULT_ADDRESS, vec![FIFO], vec![0xe0, 0x00, 0x10]),
        I2cTransaction::write_read(DEFAULT_ADDRESS, vec![FIFO], vec![1, 2, 3, 4, 0, 0, 0, 0]),
    ];
    let bus = I2cMock::new(&expectations);
    let mut driver = Fusb302::new(bus);

    let packet = driver.receive().unwrap().unwrap();
    assert_eq!(packet.sop(), SopType::Sop);
    assert_eq!(packet.header(), 0x1000);
    assert_eq!(packet.payload(), [1, 2, 3, 4]);
    driver.release().done();
}

#[cfg(not(feature = "async"))]
#[test]
fn receive_classifies_sop_prime_from_its_defined_high_bits() {
    let expectations = [
        write_read(STATUS1, 0),
        I2cTransaction::write_read(DEFAULT_ADDRESS, vec![FIFO], vec![0xcf, 0, 0]),
        I2cTransaction::write_read(DEFAULT_ADDRESS, vec![FIFO], vec![0, 0, 0, 0]),
    ];
    let bus = I2cMock::new(&expectations);
    let mut driver = Fusb302::new(bus);

    assert_eq!(driver.receive().unwrap().unwrap().sop(), SopType::SopPrime);
    driver.release().done();
}

#[cfg(not(feature = "async"))]
#[test]
fn receive_rejects_malformed_sop_token() {
    let expectations = [
        write_read(STATUS1, 0),
        I2cTransaction::write_read(DEFAULT_ADDRESS, vec![FIFO], vec![0x40, 0, 0]),
    ];
    let bus = I2cMock::new(&expectations);
    let mut driver = Fusb302::new(bus);

    assert_eq!(
        driver.receive(),
        Err(Error::Receive(ReceiveError::InvalidSopToken(0x40)))
    );
    driver.release().done();
}

#[cfg(not(feature = "async"))]
#[test]
fn transmit_reports_busy_fifo_before_packet_io() {
    let packet = PdPacket::new(SopType::Sop, 0, &[]).unwrap();
    let bus = I2cMock::new(&[write_read(STATUS1, 0)]);
    let mut driver = Fusb302::new(bus);
    assert_eq!(driver.transmit(&packet), Err(Error::TransmitFifoBusy));
    driver.release().done();
}

#[cfg(not(feature = "async"))]
#[test]
fn i2c_failures_are_not_hidden() {
    let expectations = [
        I2cTransaction::write_read(DEFAULT_ADDRESS, vec![DEVICE_ID], vec![0])
            .with_error(ErrorKind::Other),
    ];
    let bus = I2cMock::new(&expectations);
    let mut driver = Fusb302::new(bus);

    assert_eq!(driver.device_id(), Err(Error::I2c(ErrorKind::Other)));
    driver.release().done();
}

#[cfg(feature = "async")]
#[test]
fn async_driver_has_the_same_transaction_semantics() {
    futures_executor::block_on(async {
        let expectations = [
            I2cTransaction::write(DEFAULT_ADDRESS, vec![RESET, 0x01]),
            write_read(CONTROL0, 0x32),
            I2cTransaction::write(DEFAULT_ADDRESS, vec![CONTROL0, 0x62]),
            write_read(CONTROL1, 0x80),
            I2cTransaction::write(DEFAULT_ADDRESS, vec![CONTROL1, 0x04]),
            I2cTransaction::write(DEFAULT_ADDRESS, vec![POWER, 0x0f]),
            write_read(SWITCHES1, 0),
            I2cTransaction::write(DEFAULT_ADDRESS, vec![SWITCHES1, 0x20]),
            write_read(CONTROL1, 0),
            I2cTransaction::write(DEFAULT_ADDRESS, vec![CONTROL1, 0]),
            write_read(CONTROL3, 0),
            I2cTransaction::write(DEFAULT_ADDRESS, vec![CONTROL3, 0]),
            write_read(STATUS1, 0x08),
            I2cTransaction::write(
                DEFAULT_ADDRESS,
                vec![
                    FIFO, 0x12, 0x12, 0x12, 0x13, 0x82, 0, 0, 0xff, 0x14, 0xfe, 0xa1,
                ],
            ),
            write_read(STATUS1, 0),
            I2cTransaction::write_read(DEFAULT_ADDRESS, vec![FIFO], vec![0xe0, 0, 0]),
            I2cTransaction::write_read(DEFAULT_ADDRESS, vec![FIFO], vec![0, 0, 0, 0]),
            write_read(SWITCHES0, 0xa3),
            I2cTransaction::write(DEFAULT_ADDRESS, vec![SWITCHES0, 0xa7]),
            write_read(CONTROL0, 0x22),
            I2cTransaction::write(DEFAULT_ADDRESS, vec![CONTROL0, 0x26]),
            I2cTransaction::write(DEFAULT_ADDRESS, vec![MASK, 0x7d]),
            I2cTransaction::write(DEFAULT_ADDRESS, vec![MASKA, 0xe0]),
            I2cTransaction::write(DEFAULT_ADDRESS, vec![MASKB, 0x00]),
        ];
        let bus = I2cMock::new(&expectations);
        let mut driver = Fusb302::new(bus);

        driver.init().await.unwrap();
        driver.configure_phy(PhyConfig::default()).await.unwrap();
        driver
            .transmit(&PdPacket::new(SopType::Sop, 0, &[]).unwrap())
            .await
            .unwrap();
        assert_eq!(driver.receive().await.unwrap().unwrap().sop(), SopType::Sop);
        driver.set_measure_cc(Some(CcPin::Cc1)).await.unwrap();
        driver.set_host_current_default().await.unwrap();
        driver
            .set_interrupt_masks(InterruptMasks::new(0x7d, 0xe0, 0x00))
            .await
            .unwrap();
        driver.release().done();
    });
}
