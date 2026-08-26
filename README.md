# FUSB302 Rust PHY Driver

`fusb302` is a `no_std` Rust driver for the FUSB302B family of USB Type-C and
USB Power Delivery BMC PHY devices. FUSB302BMPX is the initial validation
target.

The crate provides register-level PHY control, Type-C electrical controls,
interrupt and FIFO access, and USB PD packet transport. It deliberately does
not implement USB PD policy, contract negotiation, or power-management decisions.

## Usage

```no_run
use fusb302::Fusb302;

fn take_bus<I2C: embedded_hal::i2c::I2c>(i2c: I2C) -> I2C {
    let phy = Fusb302::with_address(i2c, 0x22);
    phy.release()
}
```

`Fusb302::new` uses the documented default 7-bit address `0x22`. Call `init()`
before explicitly configuring the PHY. It resets and flushes the device without
selecting CC pull resistors, VCONN, BMC receive modes, GoodCRC, or reset/retry
automation.

## Features

- Default: synchronous `embedded-hal` 1.0 I2C API.
- `async`: compile-time replacement with the equivalent `embedded-hal-async`
  I2C API.
- `defmt`: derives `defmt::Format` on public status, packet, configuration, and
  error types.

The crate supports Rust 1.85 and newer. `PdPacket` contains a raw USB PD
header and up to seven 32-bit data objects (`[u8; 28]`). An Extended Message is
represented only as one physical chunk; reassembly belongs to a protocol layer.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
