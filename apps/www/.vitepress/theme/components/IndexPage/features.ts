import type { IndexFeature } from './types'

export const features: IndexFeature[] = [
  {
    id: 'sdk',
    title: 'Typed SDK',
    desc: 'Slash commands, prefix commands, embeds, and Discord helpers with TypeScript types from the start.',
    href: '/docs/sdk',
    linkLabel: 'Read SDK docs'
  },
  {
    id: 'cli',
    title: 'Guild deploys',
    desc: 'Bundle and deploy a bot to a guild with one CLI command. No containers, hosts, or release scripts to babysit.',
    href: '/docs/cli',
    linkLabel: 'Use the CLI'
  },
  {
    id: 'runtime',
    title: 'Runtime batteries',
    desc: 'KV storage, secrets, sandboxing, and runtime primitives are built in so bot code can stay focused.',
    href: '/docs/runtime',
    linkLabel: 'Explore runtime'
  }
]
