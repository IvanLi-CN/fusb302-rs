# FUSB302 PHY Driver 实现状态

> 当前有效规范以 [`SPEC.md`](./SPEC.md) 为准；此文只记录实现覆盖与 rollout
> 事实。

## Current Status

- Implementation: 初始实现进行中
- Lifecycle: active
- Catalog note: FUSB302BMPX is the initial validation target.

## Coverage / rollout summary

- 计划覆盖同步和 async I2C、Type-C PHY controls、BMC packet FIFO transport、
  interrupt snapshots 和 publishable crate tooling。

## Remaining Gaps

- 实现与 mock-based verification 尚未完成。
- Physical hardware validation remains owned by downstream firmware.

## Related Changes

- Initial implementation PR.

## References

- `./SPEC.md`
- `./HISTORY.md`
