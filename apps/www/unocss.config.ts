import { defineConfig, presetIcons, presetWebFonts, presetWind4 } from 'unocss'

export default defineConfig({
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
