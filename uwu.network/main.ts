const TAG_ALLOWED_ROLE = "880150867792232609";

const regex =
  /(?:github\.com\/)?(?<owner>[A-Za-z\d-]+)\/(?<repo>[\w.-]+)(?:(?:#|\/(?:issues|pull)\/)(?<issue>\d+)(?:#issuecomment-(?<comment>\d+))?)?/g;

type PluginManifest = {
  name: string;
  description: string;
  author: string;
  url: string;
};

const shelterPluginRegex = /\[\[(.+?)\]\]/g;
const allowed = ["981306328930713661", "886194087072510012"];
const SUPPRESS_EMBEDS = 1 << 2;

on("messageCreate", async (ctx) => {
  const msg = ctx.msg;
  if (!msg.member?.roles?.some((r) => allowed.includes(r))) return;

  let suppress = false;
  const matches = msg.content.matchAll(regex);
  for (const match of matches) {
    const { owner, repo, issue, comment } = match.groups ?? {};
    if (!owner || !repo) continue;
    if (owner === "com") continue; // i'd love to fix this in the regex but nop

    let embed: {
      title?: string;
      description?: string;
      url?: string;
      footer?: { text: string };
      author?: { name?: string; iconUrl?: string };
    } = {};

    if (issue !== undefined) {
      let user: { login: string; avatar_url: string } | undefined;
      let body: string | undefined;
      let url: string | undefined;

      if (comment !== undefined) {
        const req = await fetch(
          `https://api.github.com/repos/${owner}/${repo}/issues/comments/${comment}`,
        );
        if (!req.ok) continue;
        const json = await req.json();

        user = json.user;
        body = json.body;
        url = json.html_url;
      }

      const req = await fetch(
        `https://api.github.com/repos/${owner}/${repo}/issues/${issue}`,
      );
      if (!req.ok) continue;
      const json = await req.json();

      user ??= json.user;
      body ??= json.body;
      url ??= json.html_url;

      embed.title = json.title;
      embed.footer = {
        text: `${owner}/${repo}#${issue}${
          comment !== undefined ? ` (comment ${comment})` : ""
        }`,
      };
      embed.author = {
        name: user?.login,
        iconUrl: user?.avatar_url,
      };
      if (body) {
        embed.description = `${body.slice(0, 77)}${body.length > 77 ? "..." : ""}`;
      }
      embed.url = url;
    } else {
      const req = await fetch(`https://api.github.com/repos/${owner}/${repo}`);
      if (!req.ok) continue;
      const json = await req.json();

      embed.title = `${owner}/${repo}`;
      embed.description = json.description ?? undefined;
      embed.url = json.html_url;
    }

    await ctx.reply({ embeds: [embed] });
    suppress = true;
  }

  if (suppress) {
    await ctx.edit({ flags: SUPPRESS_EMBEDS });
  }
});

const tagCommand = defineSlashCommand({
  name: "tag",
  description: "Manage server tags",
  subcommands: [
    {
      name: "create",
      description: "Create a new tag",
      options: [
        {
          name: "name",
          description: "The tag name",
          type: "string",
          required: true,
        },
        {
          name: "content",
          description: "The tag content",
          type: "string",
          required: true,
        },
      ],
      async run(ctx) {
        if (!hasRole(ctx, TAG_ALLOWED_ROLE)) {
          return await ctx.reply({
            content: "You do not have permission to create tags.",
            ephemeral: true,
          });
        }

        const name = ctx.options.getString("name");
        const content = ctx.options.getString("content");

        const tagStore = kv.store("tags");
        const existing = await tagStore.get(name);

        if (existing) {
          return await ctx.reply({
            content: `A tag with the name "**${name}**" already exists.`,
            ephemeral: true,
          });
        }

        await tagStore.set(name, content);
        return await ctx.reply(`Created tag "**${name}**"!`);
      },
    },
    {
      name: "view",
      description: "View a tag",
      options: [
        {
          name: "name",
          description: "The tag name",
          type: "string",
          required: true,
        },
      ],
      async run(ctx) {
        const name = ctx.options.getString("name");

        const tagStore = kv.store("tags");
        const content = await tagStore.get(name);

        if (!content) {
          return await ctx.reply({
            content: `Tag "**${name}**" not found.`,
            ephemeral: true,
          });
        }

        return await ctx.reply(content);
      },
    },
    {
      name: "edit",
      description: "Edit an existing tag",
      options: [
        {
          name: "name",
          description: "The tag name",
          type: "string",
          required: true,
        },
        {
          name: "content",
          description: "The new tag content",
          type: "string",
          required: true,
        },
      ],
      async run(ctx) {
        if (!hasRole(ctx, TAG_ALLOWED_ROLE)) {
          return await ctx.reply({
            content: "You do not have permission to edit tags.",
            ephemeral: true,
          });
        }

        const name = ctx.options.getString("name");
        const content = ctx.options.getString("content");

        const tagStore = kv.store("tags");
        const existing = await tagStore.get(name);

        if (!existing) {
          return await ctx.reply({
            content: `Tag "**${name}**" not found.`,
            ephemeral: true,
          });
        }

        await tagStore.set(name, content);
        return await ctx.reply(`Updated tag "**${name}**"!`);
      },
    },
    {
      name: "delete",
      description: "Delete a tag",
      options: [
        {
          name: "name",
          description: "The tag name",
          type: "string",
          required: true,
        },
      ],
      async run(ctx) {
        if (!hasRole(ctx, TAG_ALLOWED_ROLE)) {
          return await ctx.reply({
            content: "You do not have permission to delete tags.",
            ephemeral: true,
          });
        }

        const name = ctx.options.getString("name");

        const tagStore = kv.store("tags");
        const existing = await tagStore.get(name);

        if (!existing) {
          return await ctx.reply({
            content: `Tag "**${name}**" not found.`,
            ephemeral: true,
          });
        }

        await tagStore.delete(name);
        return await ctx.reply(`Deleted tag "**${name}**"!`);
      },
    },
  ],
});

createBot({
  slashCommands: [tagCommand],
});
