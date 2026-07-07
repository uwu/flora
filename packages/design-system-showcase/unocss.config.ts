import { presetFlora } from '@flora-internal/design-system/unocss'
import {
  defineConfig,
  presetIcons,
  presetWebFonts,
  presetWind4,
  transformerDirectives
} from 'unocss'

export default defineConfig({
  transformers: [transformerDirectives()],
  presets: [
    presetWind4(),
    presetIcons({
      extraProperties: {
        display: 'inline-block',
        'vertical-align': '-0.125em'
      }
    }),
    presetWebFonts({
      provider: 'google',
      fonts: {
        sans: 'Geist:400,500,600,700',
        mono: 'Geist Mono:400,500,600,700'
      }
    }),
    presetFlora()
  ],
  content: {
    pipeline: {
      include: [
        /\.(vue|svelte|[jt]sx|mdx?|astro|elm|php|phtml|html)($|\?)/,
        'src/**/*.{js,ts}',
        '../design-system/src/**/*.{js,ts,vue}'
      ]
    }
  },
  safelist: [
    'i-lucide-book-open',
    'i-lucide-check-circle-2',
    'i-lucide-grid-2x2',
    'i-lucide-moon',
    'i-lucide-sparkles',
    'i-lucide-sun'
  ]
})
