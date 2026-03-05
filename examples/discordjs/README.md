# discord.js Flora example

This example uses the real `discord.js` `Client` (no `@uwu/flora-sdk` helpers).

## Commands

- `!ping`
- `!echo <text>`

## Setup

1. Set secret `DISCORD_TOKEN` for your guild deployment.
2. Deploy with entry `src/main.ts`.

Example:

```bash
flora deployments deploy <guild-id> src/main.ts --root examples/discordjs
```
