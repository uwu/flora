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

  // shelter plugins
  /**
  const spMatches = msg.content.matchAll(shelterPluginRegex);
  for (const [, query] of spMatches) {
    console.log("query", query);
    const req = await fetch(
      `https://shindex.uwu.network/search?q=${encodeURIComponent(query)}`,
    );

    if (!req.ok) {
      console.log({ request: req, err: req.statusText });
      continue;
    }
    const results = (await req.json()) as PluginManifest[];

    console.log({ results });
    await ctx.reply({
      content: results
        .map(
          (r) => `**[${r.name}](${r.url})** | ${r.description} (${r.author})`,
        )
        .join("\n"),
    });
  }
  **/

  if (suppress) {
    await ctx.edit({ flags: SUPPRESS_EMBEDS });
  }
});
