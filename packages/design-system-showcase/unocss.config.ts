import { presetFloraShadcn } from '@flora-internal/design-system/unocss'
import { defineConfig, presetIcons, presetWind4, transformerDirectives } from 'unocss'

export default defineConfig({
  transformers: [transformerDirectives()],
  presets: [presetWind4(), presetIcons(), presetFloraShadcn()],
  content: {
    pipeline: {
      include: [
        /\.(vue|svelte|[jt]sx|mdx?|astro|elm|php|phtml|html)($|\?)/,
        'src/**/*.{js,ts}'
      ]
    }
  }
})
