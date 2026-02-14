# Contributors Guide

## Release: npm canary

### Prereqs (one-time)

- npm Trusted Publishing configured for:
  - `@uwu/flora-sdk` -> workflow `.github/workflows/npm-publish-sdk-canary.yml`
  - `@uwu/flora-cli` -> workflow `.github/workflows/npm-publish-cli-canary.yml`
- Run from `trunk` for clean provenance/release history.

### Cut SDK canary (`@uwu/flora-sdk`)

1. Ensure types are current:
   - `pnpm --filter @uwu/flora-sdk run typecheck`
   - if SDK API changed, regenerate types: `pnpm --filter @uwu/flora-sdk run build` and commit `sdk/global-types.d.ts`
2. In GitHub Actions, run workflow `npm-publish-sdk-canary` (manual `workflow_dispatch`).
3. Workflow publishes `0.0.0-canary.<run_number>.<sha7>` with npm provenance under tag `canary`.
4. Verify:
   - `npm view @uwu/flora-sdk dist-tags`
   - `npm view @uwu/flora-sdk versions --json | tail`

### Cut CLI canary (`@uwu/flora-cli`)

1. In GitHub Actions, run workflow `npm-publish-cli-canary` (manual `workflow_dispatch`).
2. Workflow:
   - builds Rust CLI for 4 targets
   - creates release `flora-cli-v0.0.0-canary.<run_number>.<sha7>`
   - uploads binaries + `checksums.txt`
   - creates build provenance attestation
   - publishes npm package with provenance under tag `canary`
3. Verify:
   - `npm view @uwu/flora-cli dist-tags`
   - `npm view @uwu/flora-cli versions --json | tail`
   - check GitHub release assets/checksums for matching version

### Retry / rollback

- npm versions are immutable. If a run fails, re-run workflow to get a new canary version.
- Do not delete/reuse tags. New run number = new version.

### Stable releases

- Not implemented yet in workflows. Current automation is canary-only.
