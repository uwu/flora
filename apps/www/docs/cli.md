---
outline: deep
---

# CLI

:::info
Currently the CLI is unpublished. It will be published soon as `@uwu/flora-cli`.
:::

The Flora CLI manages deployments, logs, and KV stores with the Flora Server API.

## Login

Generate a API Token at https://app.flora.uwu.network/settings and then login with:

```bash
flora login <token>
```

## Commands

### Deploy

To deploy to a guild:

```bash
flora deploy
```

- You can add `--guild [guildId]` to specify your guild (it prompts by default).
- A positional arg to specify a custom entrypoint script is also provided if needed.
- Using `--root path/to` allows you to specify a custom root (what gets packaged).

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
