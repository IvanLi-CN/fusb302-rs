# Own a protocol-neutral PD packet transport type

The crate defines its own fixed-capacity `PdPacket` with SOP type, raw 16-bit
header, and up to 28 payload bytes. It has no dependency on a USB-PD protocol
stack; packet parsing, Extended Message reassembly, and negotiation remain
outside the driver.

## Consequences

`fusb302` can integrate with multiple existing or future PD stacks without
making their type system or release cadence part of this crate's public API.
