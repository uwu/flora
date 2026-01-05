import { buildPong } from "./utils/reply";

// Prefix command example
const ping = defineCommand({
  name: "ping",
  description: "Respond with pong",
  run: async (ctx) => {
    await ctx.reply(buildPong(ctx.args));
  },
});

// Slash command example
const hello = defineSlashCommand({
  name: "hello",
  description: "Say hello",
  options: [
    {
      name: "name",
      description: "Who to greet",
      type: "string",
      required: false,
    },
  ],
  run: async (ctx) => {
    const name = (ctx.options.name as string) || "world";
    await ctx.reply({
      content: `Hello, ${name}!`,
      ephemeral: true,
    });
  },
});

// Slash command with subcommands
const counter = defineSlashCommand({
  name: "counter",
  description: "A simple counter using KV storage",
  subcommands: [
    {
      name: "get",
      description: "Get current count",
      run: async (ctx) => {
        const store = kv.store("counters");
        const count = await store.get("main");
        await ctx.reply(`Current count: ${count || 0}`);
      },
    },
    {
      name: "increment",
      description: "Increment the counter",
      run: async (ctx) => {
        const store = kv.store("counters");
        const current = parseInt(await store.get("main") || "0", 10);
        const newCount = current + 1;
        await store.set("main", String(newCount));
        await ctx.reply(`Count is now: ${newCount}`);
      },
    },
    {
      name: "reset",
      description: "Reset the counter",
      run: async (ctx) => {
        const store = kv.store("counters");
        await store.set("main", "0");
        await ctx.reply("Counter reset to 0");
      },
    },
  ],
});

// Register the bot
createBot({
  prefix: "!",
  commands: [ping],
  slashCommands: [hello, counter],
});
