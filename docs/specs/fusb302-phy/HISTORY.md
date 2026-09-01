# FUSB302 PHY Driver 主题历史

> 这里保留主题局部生命周期和兼容性背景；ADR 的完整取舍仍位于
> [`docs/adr/`](../../adr/)。

## Lifecycle / Compatibility

- The topic is active and owns the FUSB302B-family PHY driver contract.
- The crate supports FUSB302BMPX as the initial validation target while keeping
  address selection available for documented B-family variants.
- PD 3.0 GoodCRC encoding is retained as an explicit BMPX opt-in so a downstream
  PPS protocol layer can preserve its existing wire behavior after HIL validation.
- Release failure handoff uses the immutable Oidrune `notify.yml` release
  reference `e48822f99c6402a753ed86557ea029754cbab20b`; the caller owns the
  complete notification summary and no longer forwards a Telegram secret or
  moving shared-workflow reference.

## Replacements / Background

- The initial topic separates hardware PHY responsibilities from USB-PD policy
  so multiple protocol implementations can use the same crate.

## References

- `./SPEC.md`
- `./IMPLEMENTATION.md`
