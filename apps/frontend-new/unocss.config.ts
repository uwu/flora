import {
  defineConfig,
  presetIcons,
  presetWebFonts,
  presetWind4,
  transformerDirectives
} from 'unocss'
import { presetFlora } from '@flora-internal/design-system/unocss'

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
    presetFlora({
      neutral: 'gray',
      primary: 'iris',
      info: 'blue',
      tip: 'green',
      warning: 'yellow',
      danger: 'red'
    })
  ],
  shortcuts: {
    'flora-page': 'min-h-svh bg-background text-foreground font-sans antialiased',
    'flora-shell': 'mx-auto w-full max-w-6xl px-6 py-10 sm:px-8 lg:px-10',
    'flora-card': 'rounded-xl border bg-card text-card-foreground shadow-sm',
    'flora-muted': 'text-muted-foreground',
    'flora-focus': 'focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-ring/50',
    'flora-icon': 'size-4 shrink-0'
  },
  content: {
    pipeline: {
      include: [
        /\.(vue|svelte|[jt]sx|mdx?|astro|elm|php|phtml|html)($|\?)/,
        'src/**/*.{js,ts}',
        '../../packages/design-system/src/**/*.{js,ts,vue}'
      ]
    }
  },
  safelist: []
})
