# discord.js Flora example

This example uses the real `discord.js` `Client` (no `@uwu/flora-sdk` helpers).

## Commands

- `!ping`
- `!echo <text>`

## Setup

1. Use `secrets.get('__FLORA_THIRDPARTY_DISCORD_TOKEN__')` in your script.
2. Deploy with entry `src/main.ts`.

`__FLORA_THIRDPARTY_DISCORD_TOKEN__` is a Flora runtime magic marker and should
not be replaced with a raw token in source code.

Example:

```bash
flora deployments deploy <guild-id> src/main.ts --root examples/discordjs
```
