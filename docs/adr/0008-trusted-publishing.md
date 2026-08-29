# Authenticate crate publication with trusted OIDC

The release workflow will use crates.io Trusted Publishing and a GitHub
`release` environment restricted to `main`, with no required human approval.
The publishing job receives a short-lived OIDC credential rather than a
long-lived registry token; this is the least-privilege path compatible with
automatic label-driven releases.

## Consequences

Only the release job receives `id-token: write`; the GitHub-side release work
receives the minimum explicit `contents: write` and read scopes it needs. Once
one OIDC publication has completed and the release contract check has passed,
the legacy `CARGO_REGISTRY_TOKEN` is revoked with no fallback publishing path.
Recovery of an already-published crate version does not request a crates.io
credential, because it only reconciles GitHub release surfaces.
