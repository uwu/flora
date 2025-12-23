import { buildPong } from "./utils/reply";

const ping = defineCommand({
  name: "ping",
  description: "Respond with pong",
  run: async (ctx) => {
    await ctx.reply(buildPong(ctx.args));
  },
});

createBot({
  prefix: "!",
  commands: [ping],
});
