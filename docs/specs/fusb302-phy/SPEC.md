# FUSB302 PHY Driver

> 当前有效规范以本文为准；实现覆盖与当前状态见
> [`IMPLEMENTATION.md`](./IMPLEMENTATION.md)，主题局部演进见
> [`HISTORY.md`](./HISTORY.md)。持久决策的完整取舍见关联 ADR。

## 背景 / 问题陈述

FUSB302B 是由 host software 驱动的 Type-C 与 USB-PD BMC PHY。驱动必须让
不同的上层 PD protocol/policy implementation 安全复用相同的寄存器、FIFO 与
中断能力，而不把任意板级电源策略固定进 crate。

## 目标 / 非目标

### Goals

- 提供 FUSB302B-family 的 `no_std`、寄存器级、I2C PHY driver。
- 以相同的语义提供同步和 feature-gated asynchronous I2C API。
- 支持 Type-C electrical control、BMC packet transport、FIFO 与 interrupt
  handling，并保持上层 protocol/policy ownership。

### Non-goals

- USB-PD contract negotiation、timer-driven state machine、power policy 或
  Extended Message reassembly。
- 对 FUSB302BMPX 以外的运行时 device identification 或 board-specific API。
- GPIO interrupt ownership、async runtime integration 或 hardware-in-loop test
  harness。

## 范围（Scope）

### In scope

- FUSB302B-family I2C register access，初始硬件目标为 FUSB302BMPX。
- Software/PD reset、CC/VCONN controls、SRC/SNK/DRP toggle、status 与
  clear-on-read interrupt snapshots。
- FIFO flush、PD packet transmission/reception、hard reset，和 explicit
  GoodCRC/retry/reset hardware automation configuration。

### Out of scope

- 产品级 sink/source selection、VBUS power-path control 与 PD message policy。
- 跨 packet 的 Extended Message reassembly 和 data-object semantic parsing。

## Related ADRs

- [Keep the crate at the FUSB302B PHY boundary](../../adr/0001-phy-driver-boundary.md)
- [Select the I2C execution model at compile time](../../adr/0002-compile-time-sync-async-selection.md)
- [Keep PHY automation explicit and interrupts atomic](../../adr/0003-explicit-phy-automation-and-events.md)
- [Own a protocol-neutral PD packet transport type](../../adr/0004-own-the-pd-packet-transport-type.md)
- [Set a publishable embedded crate baseline](../../adr/0005-publishable-embedded-crate-baseline.md)
- [Keep PD 3.0 GoodCRC encoding an explicit BMPX opt-in](../../adr/0006-explicit-bmpx-pd3-goodcrc.md)
- [Bind public crate versions to immutable release units](../../adr/0007-release-unit-contract.md)
- [Authenticate crate publication with trusted OIDC](../../adr/0008-trusted-publishing.md)

## 需求（Requirements）

### MUST

- 默认 API 使用 `embedded-hal` 1.0；`async` feature 使用
  `embedded-hal-async` 1.0，且 public behavior 相同。
- `Fusb302::new` 必须使用 FUSB302BMPX 的 7-bit address `0x22`；
  `with_address` 必须支持其他 B-family address；`release` 必须返还 I2C bus。
- `PdPacket` 必须是 zero-allocation value，包含 SOP type、raw `u16` header
  和最多 28 bytes payload；发送必须拒绝 header data-object count 不匹配的
  payload。
- `read_interrupts` 必须一次读取所有 clear-on-read interrupt registers 并返回
  single snapshot；不得分散读取导致调用者丢失 event。
- 所有 PD automation 必须显式配置；初始化不得自动选择 CC role、VCONN、GoodCRC
  或 retry/reset policy。
- hardware-generated GoodCRC 必须默认使用 USB PD 2.0 的 `SPECREV=01`。`SPECREV=10`
  只能通过显式的 `PdRevision::Rev30` opt-in 写入，且仅适用于完成目标硬件验证的
  FUSB302BMPX integration；crate 不得将其表示为厂商支持的通用 B-family 行为。
- 所有 register writes 必须屏蔽 reserved bits；public API 不得提供任意 register
  address writes。

### SHOULD

- 向 callers 暴露 typed status、configuration 与 error values，而不是由 callers
  hand-encode register bits 或 FIFO tokens。
- 在 README 和 rustdoc 中提供同步和 asynchronous integration examples。

### COULD

- 未来为外部 PD stack 提供单独 adapter crate 或 optional integration feature，
  但 core crate 不以任何 stack type 作为 public dependency。

## 功能与行为规格（Functional/Behavior Spec）

### Core flows

1. Caller creates `Fusb302`, calls `init`, then explicitly applies power/CC/BMC
   configuration or starts hardware toggle mode.
2. Caller reads `InterruptSnapshot`; RX-related events are followed by an explicit
   packet read, so the driver never consumes FIFO data while reading interrupts.
3. Caller sends a valid `PdPacket`; the driver serializes required FUSB302B TX
   tokens and payload bytes, then starts transmission.
4. Caller reads an RX packet after a valid event; the driver parses its SOP,
   header, data bytes and checks FIFO structure without applying PD policy.

### Edge cases / errors

- I2C failures are returned as the caller's I2C error type.
- Packets over 28 bytes or inconsistent with header data-object count are rejected
  before FIFO writes.
- A full receive FIFO is drained as a readable packet; only malformed FIFO
  structure is reported as a receive error.
- Invalid/unknown FIFO sequences and a non-empty transmit FIFO are reported as
  typed driver errors.
- Extended Message chunks are ordinary physical packets; callers own reassembly.

## 接口契约（Interfaces & Contracts）

### 接口清单（Inventory）

| 接口（Name） | 类型（Kind） | 范围（Scope） | 变更（Change） | 契约文档（Contract Doc） | 负责人（Owner） | 使用方（Consumers） | 备注（Notes） |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `Fusb302<I2C>` | Rust API | external | New | None | crate | embedded firmware | I2C bus owner |
| `PdPacket` | Rust API | external | New | None | crate | PD protocol layer | one physical packet/chunk |
| `PhyConfig` | Rust API | external | New | None | crate | PD protocol layer | explicit hardware automation |
| `InterruptSnapshot` | Rust API | external | New | None | crate | firmware event loop | atomic read snapshot |

### 契约文档（按 Kind 拆分）

- None

## 验收标准（Acceptance Criteria）

- Given a synchronous or async I2C implementation, when equivalent driver
  methods are used, then they produce equivalent register transactions and
  typed results.
- Given a packet whose payload does not match its header data-object count,
  when transmission is attempted, then the driver returns an error without a
  FIFO write.
- Given clear-on-read interrupt bits in every interrupt register, when
  `read_interrupts` is called, then the returned snapshot contains every bit
  from that one read sequence.
- Given a valid SOP packet in the receive FIFO, when the caller reads it, then
  the returned `PdPacket` contains only the physical packet and no policy
  decision.

## 非功能性验收 / 质量门槛（Quality Gates）

### Testing

- Unit tests: register bitfields, packet validation and FIFO parsing.
- Integration tests: EH1 I2C transaction sequences, error propagation and
  configuration RMW behavior.
- Async tests: equivalent I2C transaction behavior behind the `async` feature.

### Quality checks

- `cargo fmt -- --check`.
- Clippy with warnings denied for default, `async`, `defmt` and combined features.
- Tests and doctests for the same feature combinations; default feature MSRV test
  with Rust 1.85.
- `cargo package` and `cargo publish --dry-run` release verification.

### Release Contract

- A public `fusb302` version is a release unit: the crates.io package,
  `release/<version>` tag, and GitHub Release must point to the same verified
  signed source SHA.
- Every merge-ready PR has exactly one `type:major`, `type:minor`,
  `type:patch`, or `type:none` label and exactly one `channel:stable`,
  `channel:beta`, or `channel:dev` label. A trusted `Label Gate` validates and
  snapshots the intent before merge.
- The PR's `Cargo.toml` contains the exact version. `stable` is a normal
  crates.io/GitHub release; `beta` and `dev` are crates.io/GitHub prereleases.
  `type:none` keeps the version unchanged and creates no release unit.
- Release runs automatically only after the exact `main` source passes required
  CI. It uses a durable receipt and a next-pending recovery path rather than
  relying on workflow concurrency alone.
- crates.io publication uses Trusted Publishing OIDC with no GitHub deployment
  environment. A maintainer can reconcile a missing tag or GitHub Release at an
  exact source SHA without republishing an existing crate version. Release
  failure notification includes the release intent, source SHA, target version,
  and run URL.

## Visual Evidence

PR: none

## Related PRs

- None

## 风险 / 开放问题 / 假设（Risks, Open Questions, Assumptions）

- Risk: mock tests cannot prove electrical interoperability or timing on a
  physical USB-C link; downstream firmware owns HIL validation.
- Assumption: FUSB302BMPX follows the published FUSB302B register map and its
  default address is `0x22`.

## 参考（References）

- FUSB302B datasheet.
- `CONTEXT.md`.
