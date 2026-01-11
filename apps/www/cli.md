---
outline: deep
---

# CLI

The Flora CLI manages deployments, logs, and KV stores against the runtime API.

## Install or run

From the repo root:

```bash
cargo run -p flora-cli -- --help
```

To install a local binary:

```bash
cargo install --path apps/cli
```

## Auth and config

- The CLI stores config via `confy` (per-user config file).
- Set `FLORA_API_URL` or pass `--api-url` to point at your runtime.
- Authenticate once with a token:

```bash
flora login <token>
```

## Commands

### Deploy

Package a script and deploy to a guild:

```bash
flora deploy --guild 123456789012345678 path/to/main.ts
```

Deploy with an explicit root (what gets packaged):

```bash
flora deploy --guild 123456789012345678 path/to/main.ts --root path/to
```

### Get

```bash
flora get --guild 123456789012345678
```

### List

```bash
flora list
```

### Health

```bash
flora health
```

### Logs

Fetch recent logs:

```bash
flora logs --guild 123456789012345678 --limit 100
```

Stream logs:

```bash
flora logs --guild 123456789012345678 --follow
```

### KV

Create a store:

```bash
flora kv create-store --guild 123456789012345678 --name settings
```

List stores:

```bash
flora kv list-stores --guild 123456789012345678
```

Delete a store:

```bash
flora kv delete-store --guild 123456789012345678 --name settings
```

Set a value:

```bash
flora kv set --guild 123456789012345678 --store settings --key prefix "!"
```

Set with expiration and metadata:

```bash
flora kv set --guild 123456789012345678 --store settings --key session "{ \"user\": \"123\" }" \
  --expiration 1735689600 --metadata "{\"source\":\"login\"}"
```

Get a value:

```bash
flora kv get --guild 123456789012345678 --store settings prefix
```

Delete a value:

```bash
flora kv delete --guild 123456789012345678 --store settings prefix
```

List keys:

```bash
flora kv list-keys --guild 123456789012345678 --store settings --prefix user: --limit 100
```

