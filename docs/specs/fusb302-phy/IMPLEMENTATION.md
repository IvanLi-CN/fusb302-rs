# FUSB302 PHY Driver 实现状态

> 当前有效规范以 [`SPEC.md`](./SPEC.md) 为准；此文只记录实现覆盖与 rollout
> 事实。

## Current Status

- Implementation: complete for the initial crate scope
- Lifecycle: active
- Catalog note: FUSB302BMPX is the initial validation target.

## Coverage / rollout summary

- `Fusb302<I2C>` owns the bus and supports sync and compile-time async I2C.
- Typed Type-C controls cover CC pulls, VCONN, TX CC selection, and SRC/SNK/DRP
  toggling.
- `PhyConfig` requires explicit GoodCRC, retry, reset automation, and cable SOP
  receive configuration. GoodCRC defaults to PD 2.0; PD 3.0 encoding is an
  explicit FUSB302BMPX compatibility opt-in.
- `PdPacket` provides bounded FIFO transport with SOP type, a raw header, and
  up to 28 payload bytes.
- Mock transactions cover register RMW, full and non-full FIFO frames, interrupt
  snapshots, and I2C errors across the supported feature combinations.
- Release automation validates pull-request labels, snapshots immutable release
  intent, publishes through crates.io OIDC, and records the crate/tag/GitHub
  Release contract. Manual publication reuses the same checks for a selected
  merged source and exact SHA, with one version value that is either a semantic
  bump selector or an exact SemVer value.
- Release failure notification preserves the failed `Release` `workflow_run`
  filter and artifact context extraction while using the immutable Oidrune
  reusable workflow reference `notify.yml@e48822f99c6402a753ed86557ea029754cbab20b`.
  The caller supplies the complete project, status/result, failure title, source
  SHA, version, and run URL summary with job-level OIDC permission and no legacy
  Telegram secret.

## Remaining Gaps

- Physical hardware validation remains owned by downstream firmware.
- Product-level USB PD policy, timers, and Extended Message reassembly remain
  outside this crate.

## Related Changes

- Initial implementation PR introduces the driver, verification, and release
  automation.
- The release workflows are deliberately separate from the Rust API and do not
  add USB-PD policy or extended-message reassembly to the crate.

## References

- `./SPEC.md`
- `./HISTORY.md`
