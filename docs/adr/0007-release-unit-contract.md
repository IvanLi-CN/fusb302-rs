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

Release starts only after the exact protected-main source has passed CI. Each
run resolves its triggering merge SHA, and a durable receipt and release-failure
sidecar preserve the release intent through failures. A maintainer may manually
publish a selected merged source through the same contract with an exact SHA and
one version value: `major`/`minor`/`patch` must match the PR intent, while a
SemVer value must match `Cargo.toml`. It still requires signed intent, green CI,
and never repeats a crate publication for an existing version.
