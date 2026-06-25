import { defineConfig, presetIcons, presetWind4, transformerDirectives } from 'unocss'
import { presetFloraShadcn } from '@flora-internal/design-system/unocss'

export default defineConfig({
  transformers: [transformerDirectives()],
  presets: [presetWind4(), presetIcons(), presetFloraShadcn()]
})
