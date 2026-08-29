# Bind public crate versions to immutable release units

Every public `fusb302` version is a release unit: its crates.io record,
`release/<version>` tag, and GitHub Release must identify the same verified
signed release source. A trusted `Label Gate` snapshots exactly one
`type:major`, `type:minor`, `type:patch`, or `type:none` label and exactly one
`channel:stable`, `channel:beta`, or `channel:dev` label before merge; the
release workflow consumes that immutable intent rather than mutable PR labels.

## Consequences

The feature PR owns the exact `Cargo.toml` version. `stable` creates a normal
crates.io package and GitHub Release; `beta` and `dev` create Cargo SemVer
prereleases on crates.io and GitHub. `type:none` requires no version change and
creates no release unit.

Release starts only after the exact protected-main source has passed CI. A
draft or recovery record may exist while work is in progress, but terminal
success requires all public surfaces to agree. A durable receipt, a
next-pending path, and a release-failure sidecar preserve the release intent
through burst merges and failures. If an already-published version is missing a
GitHub surface, a maintainer may backfill it at the exact source SHA without
republishing the crates.io version.
