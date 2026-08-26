# FUSB302 Rust PHY Driver

`fusb302` is a `no_std` Rust driver for the FUSB302B family of USB Type-C and
USB Power Delivery BMC PHY devices. FUSB302BMPX is the initial validation
target.

The crate provides register-level PHY control, Type-C electrical controls,
interrupt and FIFO access, and USB PD packet transport. It deliberately does
not implement USB PD policy, contract negotiation, or power-management
decisions.

## Status

Initial driver implementation is in progress.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
