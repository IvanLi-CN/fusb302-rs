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

When built with the `async` feature, the same API awaits `embedded-hal-async`
I2C operations:

```ignore
async fn configure<I2C>(i2c: I2C) -> Result<(), fusb302::Error<I2C::Error>>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    let mut phy = fusb302::Fusb302::new(i2c);
    phy.init().await?;
    phy.configure_phy(fusb302::PhyConfig::default()).await?;
    Ok(())
}
```

## Features

- Default: synchronous `embedded-hal` 1.0 I2C API.
- `async`: compile-time replacement with the equivalent `embedded-hal-async`
  I2C API.
- `defmt`: derives `defmt::Format` on public status, packet, configuration, and
  error types.

The crate supports Rust 1.85 and newer. `PdPacket` contains a raw USB PD
header and up to seven 32-bit data objects (`[u8; 28]`). Extended Message
reassembly belongs to a protocol layer. Hardware-generated GoodCRC defaults to
the FUSB302B-supported USB PD 2.0 revision. `PdRevision::Rev30` is an explicit
FUSB302BMPX compatibility opt-in: its `SPECREV=0b10` encoding is marked "Do Not
Use" by the FUSB302B datasheet, so downstream firmware must validate it on its
target hardware before enabling it.

## Release workflow

Release intent is declared on every pull request with exactly one `type:*`
label (`major`, `minor`, `patch`, or `none`) and one `channel:*` label (`stable`,
`beta`, or `dev`). The trusted `Label Gate` snapshots that intent without
checking out pull request code. After a successful `Rust CI` run on `main`, the
`Release` workflow creates the `release/<version>` tag and draft GitHub Release,
then publishes new versions to crates.io through Trusted Publishing (OIDC).

The first infrastructure merge is a narrow bootstrap: if no trusted
`Label Gate` run exists yet, `Release` may reconstruct only a `type:none`
intent from the merged PR and immutable `Cargo.toml` contents. Publishable
intents always require the signed Label Gate artifact.

`type:none` is an exact no-release change. A failed release never republishes a
crate: maintainers use the `Release` workflow's `recover` dispatch with the
version, exact source SHA, and typed confirmation to complete only missing
GitHub surfaces. The separate `Notify release failure` workflow reports the
release unit and recovery details through the shared Telegram notifier; it
requires the repository `SHOUTRRR_URL` secret.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
