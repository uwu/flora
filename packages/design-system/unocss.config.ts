import {
  defineConfig,
  presetIcons,
  presetWind4,
  transformerDirectives,
  transformerVariantGroup
} from 'unocss'
import { presetRadixColors } from 'unocss-preset-radix-colors'

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
    presetRadixColors({
      prefix: '',
      lightSelector: '.light',
      darkSelector: '.dark',
      colors: ['gray', 'iris', 'red', 'yellow', 'orange', 'pink', 'blue', 'green'],
      aliases: {
        neutral: 'gray',
        primary: 'iris',
        info: 'blue',
        tip: 'green',
        warning: 'yellow',
        danger: 'red'
      }
    })
  ],
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
