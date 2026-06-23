import { defineConfig, presetIcons, presetWind4, transformerDirectives } from 'unocss'
import { presetRadixColors } from 'unocss-preset-radix-colors'

export default defineConfig({
  transformers: [transformerDirectives()],
  presets: [
    presetWind4(),
    presetIcons(),
    presetRadixColors({
      prefix: '',
      lightSelector: '.light',
      darkSelector: '.dark',
      colors: ['gray', 'iris', 'red', 'yellow', 'orange', 'pink', 'blue', 'green'],
      aliases: {
        neutral: 'gray',
        primary: 'iris',
        info: 'gray',
        tip: 'iris',
        warning: 'yellow',
        danger: 'red'
      }
    })
  ]
})
