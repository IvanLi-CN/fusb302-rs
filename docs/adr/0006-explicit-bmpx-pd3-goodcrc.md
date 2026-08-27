# Keep PD 3.0 GoodCRC encoding an explicit BMPX opt-in

The FUSB302B datasheet documents `SPECREV=0b10` as "Do Not Use" for the
hardware-generated GoodCRC header. The initial hardware target, FUSB302BMPX,
is nevertheless used by a downstream PD 3.0 PPS implementation whose existing
wire behavior requires that encoding. The driver therefore exposes
`PdRevision::Rev30`, maps it directly to `SPECREV=0b10`, and keeps
`PdRevision::Rev20` as the default.

## Consequences

Choosing PD 3.0 GoodCRC remains an explicit caller decision rather than a
device-family capability claim. The crate does not perform runtime BMPX
detection and does not promise the encoding is safe on other FUSB302B-family
parts. Downstream firmware must validate the opt-in on its actual hardware;
the driver continues to own only the register write, not PD policy or hardware
qualification.
