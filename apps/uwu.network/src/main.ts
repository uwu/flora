import tag from './tag'

/// Only on the official @flora bot
const emojis = {
  logo: '<:gh_github:1543528080424046663>',
  star: '<:gh_star:1543528122719273083>',
  fork: '<:gh_fork:1543528074526720062>',
  issue: '<:gh_issue_open:1543528089626091561>',
  issueClosed: '<:gh_issue_closed:1543528083649331250>',
  skipped: '<:gh_issue_notplanned:1543528086451261470>',
  pullRequest: '<:gh_pr_open:1543528120047378482>',
  pullRequestClosed: '<:gh_pr_closed:1543528110916509736>',
  draft: '<:gh_pr_draft:1543528114578006036>',
  merged: '<:gh_pr_merged:1543528117245706331>',
  comment: '<:gh_comment:1543528071422939258>',
  label: '<:gh_label:1543528093858271302>'
}

const COLORS = {
  repo: 0x2f81f7,
  commit: 0xdbab09,
  open: 0x3fb950,
  closed: 0xa371f7,
  notPlanned: 0x59636e,
  merged: 0xa371f7,
  closedPr: 0xf85149,
  draft: 0x6e7681
}

// what comes after 6
const MAX_LINKS = 6

/// TODO: This is dumb as fuck AND I HATE MYSELF FUCKKKKKKKKKKKKKKKKKKK
const addParts = <T>(parent: T, ...items: ComponentLike[]): T => {
  const add = (parent as { addComponents?: unknown }).addComponents as
    | ((...items: ComponentLike[]) => unknown)
    | undefined
  if (!add) throw new TypeError('Component does not support addComponents LOL!')
  add.call(parent, ...items)
  return parent
}

const githubUrlRegex =
  /(?:https?:\/\/)?(?:www\.)?github\.com\/(?<owner>[A-Za-z\d-]+)\/(?<repo>[\w.-]+)(?<rest>\/[^\s]*)?/gi
const repoRefRegex =
  /\b(?<owner>[A-Za-z\d-]+)\/(?<repo>[\w.-]+)(?:(?:#|\/(?:issues|pull)\/)(?<issue>\d+)(?:#issuecomment-(?<comment>\d+))?)?\b/g

const commitPathRegex = /\/commits?\/([0-9a-f]{7,40})/i
const blobPathRegex = /\/(?:blob|raw)\/([^/\s#]+)\/([^#\s]+)?(?:\?[^#\s]*)?(?:#L(\d+)(?:-L(\d+))?)?/i

const allowed = ['981306328930713661', '886194087072510012']

type IssueState = {
  emoji: string
  label: string
  color: number
}

const issueState = (
  isPr: boolean,
  state: string | undefined,
  merged: boolean,
  draft: boolean,
  reason: string | undefined
): IssueState => {
  if (!isPr) {
    if (state === 'open') return { emoji: emojis.issue, label: 'Open', color: COLORS.open }
    if (reason === 'not_planned')
      return { emoji: emojis.skipped, label: 'Closed (not planned)', color: COLORS.notPlanned }
    return { emoji: emojis.issueClosed, label: 'Closed', color: COLORS.closed }
  }
  if (merged) return { emoji: emojis.merged, label: 'Merged', color: COLORS.merged }
  if (draft) return { emoji: emojis.draft, label: 'Draft', color: COLORS.draft }
  if (state === 'open') return { emoji: emojis.pullRequest, label: 'Open', color: COLORS.open }
  return { emoji: emojis.pullRequestClosed, label: 'Closed', color: COLORS.closedPr }
}

const formatCount = (n: number): string => {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1).replace(/\.0$/, '')}m`
  if (n >= 1000) return `${(n / 1000).toFixed(1).replace(/\.0$/, '')}k`
  return `${n}`
}

const unixTimestamp = (iso: string | undefined): string => {
  const seconds = iso ? Math.floor(Date.parse(iso) / 1000) : Number.NaN
  return Number.isNaN(seconds) ? 'unknown' : `<t:${seconds}:R>`
}

const excerptOf = (body: string | undefined): string | undefined => {
  if (!body) return undefined
  const flat = body.replace(/\r/g, '').trim()
  if (!flat) return undefined
  return `${flat.slice(0, 300)}${flat.length > 300 ? '…' : ''}`
}

const CODE_LANGS: Record<string, string> = {
  ts: 'typescript',
  tsx: 'tsx',
  mjs: 'javascript',
  cjs: 'javascript',
  js: 'javascript',
  jsx: 'jsx',
  py: 'python',
  rs: 'rust',
  go: 'go',
  rb: 'ruby',
  java: 'java',
  kt: 'kotlin',
  swift: 'swift',
  c: 'c',
  h: 'c',
  cpp: 'cpp',
  cc: 'cpp',
  hpp: 'cpp',
  cs: 'csharp',
  php: 'php',
  sh: 'bash',
  bash: 'bash',
  zsh: 'bash',
  lua: 'lua',
  zig: 'zig',
  ex: 'elixir',
  exs: 'elixir',
  hs: 'haskell',
  dart: 'dart',
  vue: 'vue',
  svelte: 'svelte',
  html: 'html',
  css: 'css',
  scss: 'scss',
  json: 'json',
  yml: 'yaml',
  yaml: 'yaml',
  toml: 'toml',
  sql: 'sql',
  md: 'markdown'
}

const codeLangOf = (path: string): string => {
  const ext = path.slice(path.lastIndexOf('.') + 1).toLowerCase()
  return CODE_LANGS[ext] ?? ''
}

on('messageCreate', async (ctx) => {
  const msg = ctx.msg
  if (!msg.member?.roles?.some((r: string) => allowed.includes(r))) return
  if (/(?:^|\s)-ignore(?:\s|$)/i.test(msg.content)) return

  const parseContent = msg.content.replace(
    /<(?:https?:\/\/)?(?:www\.)?github\.com\/[^>\s]+>/gi,
    ' '
  )

  const headers = {
    'User-Agent': 'flora-uwu.network',
    Accept: 'application/vnd.github+json'
  }
  type ParsedLink = {
    kind: 'issue' | 'commit' | 'blob' | 'repo'
    owner: string
    repo: string
    issue?: string
    comment?: string
    commit?: string
    ref?: string
    path?: string
    start?: number
    end?: number
  }
  const links: ParsedLink[] = []
  const seen = new Set<string>()

  const urlMatches = parseContent.matchAll(githubUrlRegex)
  for (const match of urlMatches) {
    const { owner, repo, rest } = match.groups ?? {}
    if (!owner || !repo) continue

    const issueMatch = rest?.match(/(?:issues?|pulls?)\/(?<issue>\d+)(?:#issuecomment-(?<comment>\d+))?/i)
    const commitMatch = rest?.match(commitPathRegex)
    const blobMatch = rest?.match(blobPathRegex)
    const link: ParsedLink = {
      kind: issueMatch ? 'issue' : commitMatch ? 'commit' : blobMatch ? 'blob' : 'repo',
      owner,
      repo,
      issue: issueMatch?.[1],
      comment: issueMatch?.[2],
      commit: commitMatch?.[1],
      ref: blobMatch?.[1],
      path: blobMatch?.[2],
      start: blobMatch?.[3] !== undefined ? parseInt(blobMatch[3], 10) : undefined,
      end: blobMatch?.[4] !== undefined ? parseInt(blobMatch[4], 10) : undefined
    }
    const key = [
      link.kind,
      owner,
      repo,
      link.issue ?? '',
      link.comment ?? '',
      link.commit ?? '',
      link.ref ?? '',
      link.path ?? '',
      link.start ?? '',
      link.end ?? ''
    ].join('#')
    if (seen.has(key)) continue
    seen.add(key)
    links.push(link)
  }

  const repoMatches = parseContent.matchAll(repoRefRegex)
  for (const match of repoMatches) {
    const { owner, repo, issue, comment } = match.groups ?? {}
    if (!owner || !repo) continue
    if (
      match.index !== undefined &&
      /github\.com\/$/i.test(parseContent.slice(Math.max(0, match.index - 20), match.index))
    ) {
      continue
    }

    const key = [
      issue !== undefined ? 'issue' : 'repo',
      owner,
      repo,
      issue ?? '',
      comment ?? ''
    ].join('#')
    if (seen.has(key)) continue
    seen.add(key)
    links.push({ kind: issue !== undefined ? 'issue' : 'repo', owner, repo, issue, comment })
  }

  if (links.length === 0) return

  const components: ComponentJson[] = []

  let textBudget = 4000 // Discord caps components-v2 text per message
  const addText = (card: { addComponents?: unknown }, text: string): boolean => {
    if (text.length > textBudget) return false
    textBudget -= text.length
    addParts(card as never, textDisplay(text))
    return true
  }

  for (const link of links.slice(0, MAX_LINKS)) {
    const { owner, repo } = link

    if (link.kind === 'commit') {
      const req = await fetch(
        `https://api.github.com/repos/${owner}/${repo}/commits/${link.commit ?? ''}`,
        { headers }
      )
      if (!req.ok) continue

      const json = (await req.json()) as any
      const sha: string = json.sha ?? link.commit ?? ''
      const short = sha.slice(0, 7)
      const message = typeof json.commit?.message === 'string' ? json.commit.message : ''
      const detail = excerptOf(message)
      const login: string | undefined = json.author?.login
      const date: string | undefined = json.commit?.author?.date
      const stats = json.stats as { additions?: number; deletions?: number } | undefined

      const card = container().setAccentColor(COLORS.commit)
      if (
        !addText(
          card,
          `### ${emojis.logo} Commit [\`${short}\`](https://github.com/${owner}/${repo}/commit/${sha})`
        )
      ) {
        continue
      }
      if (detail) addText(card, detail)

      const bits = [
        login ? `by **${login}**` : null,
        unixTimestamp(date),
        stats ? `+${stats.additions ?? 0} -${stats.deletions ?? 0}` : null
      ].filter(Boolean)
      const meta = `-# ${owner}/${repo}${bits.length ? ` · ${bits.join(' · ')}` : ''}`

      addText(card, meta)

      components.push(card.toJSON())
      continue
    }

    if (link.kind === 'blob' && link.ref && link.path && link.start !== undefined) {
      const rawUrl = `https://raw.githubusercontent.com/${owner}/${repo}/${encodeURIComponent(
        link.ref
      )}/${link.path
        .split('/')
        .map(encodeURIComponent)
        .join('/')}`
      const raw = await fetch(rawUrl, { headers })
      if (!raw.ok) continue

      const source = await raw.text()
      const allLines = source.split('\n')
      const from = Math.max(1, link.start)
      const to = Math.max(from, Math.min(allLines.length, link.end ?? from))
      let code = allLines.slice(from - 1, to).join('\n')
      const lang = codeLangOf(link.path)

      const card = container().setAccentColor(COLORS.repo)
      if (
        !addText(
          card,
          `### ${emojis.logo} [${link.path} @ ${link.ref}](https://github.com/${owner}/${repo}/blob/${encodeURIComponent(link.ref)}/${link.path
            .split('/')
            .map(encodeURIComponent)
            .join('/')})`
        )) {
      continue
      }

      const maxCode = textBudget - 9 - lang.length
      if (code && maxCode > 20) {
        if (code.length > maxCode) code = `${code.slice(0, maxCode)}…`
        addText(card, `\`\`\`${lang}\n${code}\n\`\`\``)
      }

      addText(
        card,
        `-# ${owner}/${repo} · ${link.ref} · lines ${from === to ? `${from}` : `${from}-${to}`}`
      )
      components.push(card.toJSON())
      continue
    }

    const { issue, comment } = link

    if (issue !== undefined) {
      let user: { login: string; avatar_url: string } | undefined
      let body: string | undefined
      let url: string | undefined
      let title: string | undefined
      let state: string | undefined
      let stateReason: string | undefined
      let merged = false
      let draft = false
      let labels: string[] = []
      let comments: number | undefined
      let createdAt: string | undefined
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

      const req = await fetch(`https://api.github.com/repos/${owner}/${repo}/issues/${issue}`, {
        headers
      })
      if (!req.ok) continue
      const json = (await req.json()) as any

      user ??= json.user
      body ??= json.body
      url ??= json.html_url
      title = json.title
      state = json.state
      stateReason = json.state_reason
      labels = Array.isArray(json.labels)
        ? json.labels.map((label: any) => label?.name).filter(Boolean)
        : []
      comments = typeof json.comments === 'number' ? json.comments : undefined
      createdAt = json.created_at
      isPr = !!json.pull_request
      merged = !!json.pull_request?.merged_at
      draft = !!json.pull_request?.draft

      const status = issueState(isPr, state, merged, draft, stateReason)
      const ref = `${owner}/${repo}#${issue}`
      const excerpt = excerptOf(body)

      const card = container().setAccentColor(status.color)
      if (
        !addText(
          card,
          `### ${status.emoji} ${title ? `[${title}](${url ?? ''})` : `[${ref}](${url ?? ''})`}`
        )
      ) {
        continue
      }

      if (excerpt) addText(card, excerpt)

      const metaBits = [
        comments !== undefined ? `${emojis.comment} ${formatCount(comments)}` : null,
        labels.length > 0 ? `${emojis.label} ${labels.slice(0, 5).join(', ')}` : null
      ].filter(Boolean)
      const metaText =
        `-# ${ref}${comment !== undefined ? ` (comment ${comment})` : ''} · by **${user?.login ?? 'unknown'}** · created ${unixTimestamp(createdAt)}` +
        (metaBits.length > 0 ? `   ${metaBits.join('   ')}` : '')

      addText(card, metaText)

      components.push(card.toJSON())
    } else {
      const req = await fetch(`https://api.github.com/repos/${owner}/${repo}`, { headers })
      if (!req.ok) continue
      const json = (await req.json()) as any

      const description = typeof json.description === 'string' ? json.description : ''
      const fullName = json.full_name ?? `${owner}/${repo}`

      const card = container().setAccentColor(COLORS.repo)
      if (!addText(card, `### ${emojis.logo} **[${fullName}](${json.html_url})**`)) continue
      if (description) addText(card, description)
      addText(
        card,
        `-# ${[
          `${emojis.star} ${formatCount(json.stargazers_count ?? 0)}`,
          `${emojis.fork} ${formatCount(json.forks_count ?? 0)}`,
          `${emojis.issue} ${formatCount(json.open_issues_count ?? 0)}`,
          json.language,
          json.license?.spdx_id !== 'NOASSERTION' ? json.license?.spdx_id : null,
          `active ${unixTimestamp(json.pushed_at)}`
        ]
          .filter(Boolean)
          .join(' · ')}`
      )

      components.push(card.toJSON())
    }
  }

  if (components.length === 0) return

  if (links.length > MAX_LINKS) {
    components.push(
      textDisplay(`-# Showing ${MAX_LINKS} of ${links.length} GitHub links.`).toJSON()
    )
  }

  await ctx.reply({
    components,
    flags: MessageFlags.IS_COMPONENTS_V2 | MessageFlags.SUPPRESS_EMBEDS
  })
  await ctx.edit({ flags: MessageFlags.SUPPRESS_EMBEDS })
})

createBot({
  slashCommands: [tag]
})
