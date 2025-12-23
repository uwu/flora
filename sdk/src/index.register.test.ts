import { describe, expect, it, mock } from "bun:test";
import { createBot, defineSlashCommand, InteractionContext } from "./index";

describe("createBot slash registration", () => {
  it("registers slash commands when guild id is present", () => {
    const onHandlers: Record<string, (ctx: any) => any> = {};
    // @ts-expect-error
    globalThis.on = (event: string, handler: (ctx: any) => any) => {
      onHandlers[event] = handler;
    };

    const register = mock(() => Promise.resolve());
    // @ts-expect-error
    globalThis.registerSlashCommands = register;
    // @ts-expect-error
    globalThis.__floraGuildId = "123";

    createBot({
      slashCommands: [
        defineSlashCommand({
          name: "ping",
          options: [
            { name: "text", description: "say something", required: true },
          ],
          run: () => {},
        }),
      ],
    });

    expect(register).toHaveBeenCalledWith([
      {
        name: "ping",
        description: undefined,
        options: [
          {
            name: "text",
            description: "say something",
            required: true,
            type: undefined,
          },
        ],
      },
    ]);
  });
});
