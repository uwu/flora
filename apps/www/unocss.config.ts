import {
  defineConfig,
  presetIcons,
  presetWebFonts,
  presetWind4,
  transformerDirectives
} from 'unocss'

export default defineConfig({
  transformers: [transformerDirectives()],
  theme: {
    colors: {
      olive: {
        50: 'oklch(98.8% 0.003 106.5)',
        100: 'oklch(96.6% 0.005 106.5)',
        200: 'oklch(93% 0.007 106.5)',
        300: 'oklch(88% 0.011 106.6)',
        400: 'oklch(73.7% 0.021 106.9)',
        500: 'oklch(58% 0.031 107.3)',
        600: 'oklch(46.6% 0.025 107.3)',
        700: 'oklch(39.4% 0.023 107.4)',
        800: 'oklch(28.6% 0.016 107.4)',
        900: 'oklch(22.8% 0.013 107.4)',
        950: 'oklch(15.3% 0.006 107.1)'
      }
    }
  },
  presets: [
    presetWind4(),
    presetIcons(),
    presetWebFonts({
      fonts: {
        rethink: 'Rethink Sans'
      }
    })
  ]
})
