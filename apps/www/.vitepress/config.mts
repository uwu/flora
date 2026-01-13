import UnoCSS from 'unocss/vite'
import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: 'flora',
  description:
    'flora lets you focus on writing discord bots for your servers, not running infrastructure.',
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    nav: [
      { text: 'Home', link: '/' },
      { text: 'SDK', link: '/sdk' },
      { text: 'Runtime', link: '/runtime' },
      { text: 'CLI', link: '/cli' },
      { text: 'Examples', link: '/examples' }
    ],

    sidebar: [
      {
        text: 'Docs',
        items: [
          { text: 'SDK', link: '/sdk' },
          { text: 'Runtime', link: '/runtime' },
          { text: 'CLI', link: '/cli' },
          { text: 'Examples', link: '/examples' }
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
