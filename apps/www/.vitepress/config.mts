import { fileURLToPath } from 'node:url'
import UnoCSS from 'unocss/vite'
import { defineConfig } from 'vitepress'

const floraNavBarPath = fileURLToPath(
  new URL('./theme/components/FloraNavBar.vue', import.meta.url)
)

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: 'flora',
  description:
    'flora lets you focus on writing discord bots for your servers, not running infrastructure.',
  head: [['link', { rel: 'icon', type: 'image/svg+xml', href: '/logo.svg' }]],
  themeConfig: {
    logo: '/logo.svg',
    // https://vitepress.dev/reference/default-theme-config
    nav: [
      { text: 'SDK', link: '/docs/sdk/overview' },
      { text: 'Runtime', link: '/docs/runtime' },
      { text: 'CLI', link: '/docs/cli' },
      { text: 'Examples', link: '/docs/examples' }
    ],

    sidebar: [
      {
        text: 'Get Started',
        items: [
          { text: 'Introduction', link: '/docs/get-started/introduction' },
          { text: 'Quickstart', link: '/docs/get-started/quickstart' }
        ]
      },
      {
        text: 'SDK',
        items: [
          { text: 'Overview', link: '/docs/sdk/overview' },
          { text: 'Commands', link: '/docs/sdk/commands' },
          { text: 'Events', link: '/docs/sdk/events' },
          { text: 'Components', link: '/docs/sdk/components' },
          { text: 'Embeds', link: '/docs/sdk/embeds' },
          { text: 'KV Storage', link: '/docs/sdk/kv-storage' },
          { text: 'Cron Jobs', link: '/docs/sdk/cron-jobs' },
          { text: 'Utilities', link: '/docs/sdk/utilities' }
        ]
      },
      {
        text: 'Reference',
        items: [
          { text: 'Runtime', link: '/docs/runtime' },
          { text: 'CLI', link: '/docs/cli' },
          { text: 'Examples', link: '/docs/examples' },
          { text: 'Limitations', link: '/docs/limitations' }
        ]
      }
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/uwu/flora' },
      { icon: 'discord', link: 'https://discord.gg/dRGTU7n4dC' }
    ]
  },
  vite: {
    // @ts-expect-error: some weird types error again wow
    plugins: [UnoCSS({ config: '../unocss.config.ts' })],
    resolve: {
      alias: [
        {
          find: /^\.\/VPNavBar\.vue$/,
          replacement: floraNavBarPath
        },
        {
          find: /^.*[/\\]vitepress[/\\]dist[/\\]client[/\\]theme-default[/\\]components[/\\]VPNavBar\.vue$/,
          replacement: floraNavBarPath
        }
      ]
    }
  },
  cleanUrls: true
})
