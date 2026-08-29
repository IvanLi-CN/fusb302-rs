# FUSB302 PHY Driver

This crate provides a reusable, register-level interface to FUSB302B and
FUSB302BMPX Type-C/USB-PD BMC PHY devices. It deliberately stops before USB-PD
policy and application power-management decisions.

## Language

**PHY driver**:
The FUSB302B-family hardware-control layer: I2C register access, CC and VCONN
control, interrupts, FIFO handling, and BMC packet transmission and reception.
_Avoid_: PD stack, sink policy

**FUSB302B family**:
The register-compatible FUSB302B devices supported by this crate; FUSB302BMPX
is the initial hardware validation target, not a separate driver variant.
_Avoid_: a separately detected BMPX device

**PD policy**:
Application-owned USB-PD decision logic, including contract selection, timing,
retries, role decisions, and power management.
_Avoid_: driver state machine

**PD packet**:
A transport-level USB-PD message represented by SOP type, header, and payload;
it has no negotiation or power-selection meaning in this crate.
_Avoid_: PD contract, request policy

**GoodCRC specification revision**:
The USB-PD revision field carried by a hardware-generated GoodCRC header. It
is distinct from a protocol layer's negotiated revision or policy decision.
_Avoid_: negotiated PD revision

**Extended-message chunk**:
A physical PD packet carrying one fragment of an Extended Message. Reassembly
into a complete Extended Message belongs to the PD policy/protocol layer.
_Avoid_: extended-message reassembly

## Release Language

**Release source**:
The verified signed `main` commit from which one public crate version is built.
It is identified by its immutable commit SHA, not by a moving branch name.
_Avoid_: current main, build ref

**Release intent**:
The trusted, immutable record of a merged PR's exact Cargo version and its
validated `type:*` and `channel:*` labels. Raw PR labels are not reread as
release input after merge.
_Avoid_: current labels, release guess

**Release unit**:
One public crate version together with its crates.io record,
`release/<version>` tag, and GitHub Release, all bound to one release source.
_Avoid_: publish, tag, GitHub Release

**Release channel**:
The public class of a release unit: `stable` is a normal release, while `beta`
and `dev` are crates.io and GitHub prereleases.
_Avoid_: branch, deployment environment

**Release recovery**:
An idempotent Actions operation that completes or reconciles one incomplete
release unit at an exact release source. It never republishes a version that
already exists on crates.io.
_Avoid_: manual tag fix, republish

**Release receipt**:
The durable evidence for a release unit or its recovery, linking the intent,
release source, version, and public release surfaces.
_Avoid_: workflow log
