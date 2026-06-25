import {
  defineConfig,
  presetIcons,
  presetWind4,
  transformerDirectives,
  transformerVariantGroup,
  presetTypography
} from 'unocss'
import { presetFloraShadcn } from './src/unocss'

export default defineConfig({
  transformers: [transformerDirectives(), transformerVariantGroup()],
  presets: [
    presetWind4(),
    presetIcons({
      extraProperties: {
        display: 'inline-block',
        'vertical-align': '-0.125em'
      }
    }),
    presetTypography(),
    presetFloraShadcn()
  ],
  content: {
    pipeline: {
      include: [
        /\.(vue|svelte|[jt]sx|mdx?|astro|elm|php|phtml|html)($|\?)/,
        'src/**/*.{js,ts}'
      ]
    }
  },
  safelist: [
    'i-lucide-alert-circle',
    'i-lucide-arrow-left',
    'i-lucide-arrow-right',
    'i-lucide-check',
    'i-lucide-chevron-down',
    'i-lucide-chevron-right',
    'i-lucide-circle',
    'i-lucide-info',
    'i-lucide-loader-2',
    'i-lucide-minus',
    'i-lucide-panel-left',
    'i-lucide-search',
    'i-lucide-x'
  ]
})
