# Set a publishable embedded crate baseline

The package is named `fusb302`, targets Rust 1.85 with edition 2024, and uses
MIT OR Apache-2.0 licensing. It defaults to the FUSB302BMPX 7-bit I2C address
`0x22` while allowing a custom B-family address.

## Consequences

Continuous integration checks formatting, warnings-as-errors clippy, tests,
documentation tests, and the default, async, and defmt feature combinations.
The original manual-publish conclusion is replaced by
[ADR 0007](./0007-release-unit-contract.md) and
[ADR 0008](./0008-trusted-publishing.md); the package, MSRV, license, and I2C
address baseline above remain in effect.
