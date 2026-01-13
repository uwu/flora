const TAG_ALLOWED_ROLE = '880150867792232609'

const githubUrlRegex =
  /(?:https?:\/\/)?(?:www\.)?github\.com\/(?<owner>[A-Za-z\d-]+)\/(?<repo>[\w.-]+)(?<rest>\/[^\s]*)?/gi
const repoRefRegex =
  /\b(?<owner>[A-Za-z\d-]+)\/(?<repo>[\w.-]+)(?:(?:#|\/(?:issues|pull)\/)(?<issue>\d+)(?:#issuecomment-(?<comment>\d+))?)?\b/g

const allowed = ['981306328930713661', '886194087072510012']
const SUPPRESS_EMBEDS = 1 << 2

on('messageCreate', async (ctx) => {
  const msg = ctx.msg
  if (!msg.member?.roles?.some((r: string) => allowed.includes(r))) return

  const headers = {
    'User-Agent': 'flora-uwu.network',
    Accept: 'application/vnd.github+json'
  }
  const links: Array<{
    owner: string
    repo: string
    issue?: string
    comment?: string
  }> = []
  const seen = new Set<string>()

  const urlMatches = msg.content.matchAll(githubUrlRegex)
  for (const match of urlMatches) {
    const { owner, repo, rest } = match.groups ?? {}
    if (!owner || !repo) continue

    let issue: string | undefined
    let comment: string | undefined
    if (rest) {
      const issueMatch = rest.match(
        /\/(?:issues|pull)\/(?<issue>\d+)(?:#issuecomment-(?<comment>\d+))?/i
      )
      issue = issueMatch?.groups?.issue
      comment = issueMatch?.groups?.comment
    }

    const key = `${owner}/${repo}#${issue ?? ''}#${comment ?? ''}`
    if (seen.has(key)) continue
    seen.add(key)
    links.push({ owner, repo, issue, comment })
  }

  const repoMatches = msg.content.matchAll(repoRefRegex)
  for (const match of repoMatches) {
    const { owner, repo, issue, comment } = match.groups ?? {}
    if (!owner || !repo) continue
    if (
      match.index !== undefined &&
      /github\.com\/$/i.test(msg.content.slice(Math.max(0, match.index - 20), match.index))
    ) {
      continue
    }

    const key = `${owner}/${repo}#${issue ?? ''}#${comment ?? ''}`
    if (seen.has(key)) continue
    seen.add(key)
    links.push({ owner, repo, issue, comment })
  }

  if (links.length === 0) return
  const embeds: Array<{
    title?: string
    description?: string
    url?: string
    footer?: { text: string }
    author?: { name?: string; iconUrl?: string }
  }> = []

  for (const link of links) {
    const { owner, repo, issue, comment } = link

    let embed: {
      title?: string
      description?: string
      url?: string
      footer?: { text: string }
      author?: { name?: string; iconUrl?: string }
    } = {}

    if (issue !== undefined) {
      let user: { login: string; avatar_url: string } | undefined
      let body: string | undefined
      let url: string | undefined
      let title: string | undefined
      let state: string | undefined
      let labels: string[] = []
      let comments: number | undefined
      let isPr = false

      if (comment !== undefined) {
        const req = await fetch(
          `https://api.github.com/repos/${owner}/${repo}/issues/comments/${comment}`,
          { headers }
        )
        if (!req.ok) continue
        const json = (await req.json()) as any

        user = json.user
        body = json.body
        url = json.html_url
      }

      const req = await fetch(
        `https://api.github.com/repos/${owner}/${repo}/issues/${issue}`,
        { headers }
      )
      if (!req.ok) continue
      const json = (await req.json()) as any

      user ??= json.user
      body ??= json.body
      url ??= json.html_url
      title = json.title
      state = json.state
      labels = Array.isArray(json.labels)
        ? json.labels.map((label: any) => label?.name).filter(Boolean)
        : []
      comments = typeof json.comments === 'number' ? json.comments : undefined
      isPr = !!json.pull_request

      const titlePrefix = isPr ? '[PR] ' : ''
      const baseTitle = `${owner}/${repo}#${issue}`
      embed.title = title ? `${titlePrefix}${baseTitle}: ${title}` : `${titlePrefix}${baseTitle}`
      embed.footer = {
        text: `${owner}/${repo}#${issue}${comment !== undefined ? ` (comment ${comment})` : ''}`
      }
      embed.author = {
        name: user?.login,
        iconUrl: user?.avatar_url
      }
      const metaBits = [
        state ? `state: ${state}` : null,
        comments !== undefined ? `comments: ${comments}` : null,
        labels.length > 0 ? `labels: ${labels.slice(0, 5).join(', ')}` : null
      ].filter(Boolean)
      const metaLine = metaBits.length > 0 ? `${metaBits.join(' | ')}` : undefined
      const excerpt = body ? `${body.slice(0, 200)}${body.length > 200 ? '...' : ''}` : undefined
      embed.description = [metaLine, excerpt].filter(Boolean).join('\n\n') || undefined
      embed.url = url
    } else {
      const req = await fetch(`https://api.github.com/repos/${owner}/${repo}`, { headers })
      if (!req.ok) continue
      const json = (await req.json()) as any

      embed.title = `${owner}/${repo}`
      embed.description = json.description ?? undefined
      embed.footer = {
        text: `stars: ${json.stargazers_count ?? 0} | forks: ${
          json.forks_count ?? 0
        } | open issues: ${json.open_issues_count ?? 0}`
      }
      embed.author = {
        name: json.owner?.login,
        iconUrl: json.owner?.avatar_url
      }
      embed.url = json.html_url
    }

    embeds.push(embed)
  }

  if (embeds.length > 0) {
    const limited = embeds.slice(0, 10)
    const content = embeds.length > 10
      ? `Showing ${limited.length} of ${embeds.length} GitHub links.`
      : undefined
    await ctx.reply({ embeds: limited, content })
    await ctx.edit({ flags: SUPPRESS_EMBEDS })
  }
})

const tagCommand = slash({
  name: 'tag',
  description: 'Manage server tags',
  subcommands: [
    {
      name: 'create',
      description: 'Create a new tag',
      options: [
        {
          name: 'name',
          description: 'The tag name',
          type: 'string',
          required: true
        },
        {
          name: 'content',
          description: 'The tag content',
          type: 'string',
          required: true
        }
      ],
      async run(ctx) {
        if (!hasRole(ctx, TAG_ALLOWED_ROLE)) {
          return await ctx.reply({
            content: 'You do not have permission to create tags.',
            ephemeral: true
          })
        }

        const name = ctx.options.name as string
        const content = ctx.options.content as string

        const tagStore = kv.store('tags')
        const existing = await tagStore.get(name)

        if (existing) {
          return await ctx.reply({
            content: `A tag with the name "**${name}**" already exists.`,
            ephemeral: true
          })
        }

        await tagStore.set(name, content)
        return await ctx.reply(`Created tag "**${name}**"!`)
      }
    },
    {
      name: 'view',
      description: 'View a tag',
      options: [
        {
          name: 'name',
          description: 'The tag name',
          type: 'string',
          required: true
        }
      ],
      async run(ctx) {
        const name = ctx.options.name as string

        const tagStore = kv.store('tags')
        const content = await tagStore.get(name)

        if (!content) {
          return await ctx.reply({
            content: `Tag "**${name}**" not found.`,
            ephemeral: true
          })
        }

        return await ctx.reply(content)
      }
    },
    {
      name: 'edit',
      description: 'Edit an existing tag',
      options: [
        {
          name: 'name',
          description: 'The tag name',
          type: 'string',
          required: true
        },
        {
          name: 'content',
          description: 'The new tag content',
          type: 'string',
          required: true
        }
      ],
      async run(ctx) {
        if (!hasRole(ctx, TAG_ALLOWED_ROLE)) {
          return await ctx.reply({
            content: 'You do not have permission to edit tags.',
            ephemeral: true
          })
        }

        const name = ctx.options.name as string
        const content = ctx.options.content as string

        const tagStore = kv.store('tags')
        const existing = await tagStore.get(name)

        if (!existing) {
          return await ctx.reply({
            content: `Tag "**${name}**" not found.`,
            ephemeral: true
          })
        }

        await tagStore.set(name, content)
        return await ctx.reply(`Updated tag "**${name}**"!`)
      }
    },
    {
      name: 'delete',
      description: 'Delete a tag',
      options: [
        {
          name: 'name',
          description: 'The tag name',
          type: 'string',
          required: true
        }
      ],
      async run(ctx) {
        if (!hasRole(ctx, TAG_ALLOWED_ROLE)) {
          return await ctx.reply({
            content: 'You do not have permission to delete tags.',
            ephemeral: true
          })
        }

        const name = ctx.options.name as string

        const tagStore = kv.store('tags')
        const existing = await tagStore.get(name)

        if (!existing) {
          return await ctx.reply({
            content: `Tag "**${name}**" not found.`,
            ephemeral: true
          })
        }

        await tagStore.delete(name)
        return await ctx.reply(`Deleted tag "**${name}**"!`)
      }
    }
  ]
})

createBot({
  slashCommands: [tagCommand]
})
