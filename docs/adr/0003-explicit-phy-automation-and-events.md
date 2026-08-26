# Keep PHY automation explicit and interrupts atomic

The driver exposes FUSB302B automatic GoodCRC, retry, and reset controls but
does not enable them by default or combine them into PD negotiation behavior.
It supports source, sink, and DRP Type-C PHY controls without selecting a
product role. Clear-on-read interrupt registers are read together into one
`InterruptStatus` snapshot so an event is not lost through fragmented reads.

## Consequences

The public register layer uses typed registers and bitfields rather than
arbitrary register writes. The caller owns its event loop, FIFO drain policy,
and all PD protocol decisions.
