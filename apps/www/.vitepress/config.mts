import UnoCSS from 'unocss/vite'
import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: 'flora',
  description:
    'flora lets you focus on writing discord bots for your servers, not running infrastructure.',
  themeConfig: {
    logo: '/logo.svg',
    // https://vitepress.dev/reference/default-theme-config
    nav: [
      { text: 'Home', link: '/' },
      { text: 'SDK', link: '/docs/sdk' },
      { text: 'Runtime', link: '/docs/runtime' },
      { text: 'CLI', link: '/docs/cli' },
      { text: 'Examples', link: '/docs/examples' },
      { text: 'Limitations', link: '/docs/limitations' }
    ],

    sidebar: [
      {
        text: 'Docs',
        items: [
          { text: 'SDK', link: '/docs/sdk' },
          { text: 'Runtime', link: '/docs/runtime' },
          { text: 'CLI', link: '/docs/cli' },
          { text: 'Examples', link: '/docs/examples' },
          { text: 'Limitations', link: '/docs/limitations' }
        ]
      }
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/uwu/flora' }
    ]
  },
  vite: {
    // @ts-expect-error: some weird types error again wow
    plugins: [UnoCSS({ config: '../unocss.config.ts' })]
  },
  cleanUrls: true
})
