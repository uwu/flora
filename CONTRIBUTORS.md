# Contributors Guide

## Release: npm

### Prereqs (one-time)

- npm Trusted Publishing configured for workflow `.github/workflows/npm-publish.yml`
- Run from `trunk` for clean provenance.

### Publish (SDK / CLI / API client)

1. SDK only: `pnpm --filter @uwu/flora-sdk run typecheck`; if SDK API changed, run `pnpm --filter @uwu/flora-sdk run build` and commit `sdk/global-types.d.ts`.
2. In GitHub Actions, run workflow `npm-publish` (`workflow_dispatch`).
3. Inputs:
   - `package`: `sdk` | `cli` | `api-client` | `all`
   - `tag`: `latest` | `canary` | `next`
   - `version` (optional): set exact version; if empty, uses `package.json`.
4. Verify:
   - `npm view <pkg> dist-tags`
   - `npm view <pkg> versions --json | tail`

### Canary versioning

- If using `canary`, set `version` to `0.0.0-canary.<run_number>.<sha7>` (manual).

### Retry / rollback

- npm versions are immutable. If a run fails, rerun with a new version.
