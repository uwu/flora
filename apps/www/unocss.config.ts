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
    presetIcons(),
    presetWebFonts({
      fonts: {
        rethink: 'Rethink Sans'
      }
    })
  ]
})
