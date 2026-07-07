import {
  defineConfig,
  presetIcons,
  presetWebFonts,
  presetWind4,
  transformerDirectives
} from 'unocss'
import { presetFlora } from './src/unocss'

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
      include: [/\.(vue|svelte|[jt]sx|mdx?|astro|elm|php|phtml|html)($|\?)/, 'src/**/*.{js,ts}']
    }
  }
})
