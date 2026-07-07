import { defineConfig, presetIcons, presetWind4, transformerDirectives } from 'unocss'
import { presetFlora } from '@flora-internal/design-system/unocss'

export default defineConfig({
  transformers: [transformerDirectives()],
  presets: [presetWind4(), presetIcons(), presetFlora()],
  content: {
    pipeline: {
      include: [
        /\.(vue|svelte|[jt]sx|mdx?|astro|elm|php|phtml|html)($|\?)/,
        '.vitepress/**/*.{js,ts}',
        '../../packages/design-system/src/**/*.{js,ts,vue}'
      ]
    }
  }
})
