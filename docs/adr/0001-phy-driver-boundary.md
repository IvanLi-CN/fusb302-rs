# Keep the crate at the FUSB302B PHY boundary

The crate owns FUSB302B-family register access, Type-C electrical controls,
interrupts, FIFO tokens, and BMC PD packet transport. USB-PD policy, including
contract selection, timing, retries, role decisions, and Extended Message
reassembly, remains in the caller so the driver is reusable by different stacks.

## Consequences

The driver can offer explicit `AUTO_CRC` configuration but must not choose that
behavior during initialization. A PD packet API represents a single physical
packet or Extended Message chunk, not a policy-level message exchange.
