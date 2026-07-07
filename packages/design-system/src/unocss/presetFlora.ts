import * as radix from '@radix-ui/colors'
import type { Preset } from 'unocss'

export const radixHues = [
  'gray',
  'mauve',
  'slate',
  'sage',
  'olive',
  'sand',
  'tomato',
  'red',
  'ruby',
  'crimson',
  'pink',
  'plum',
  'purple',
  'violet',
  'iris',
  'indigo',
  'blue',
  'cyan',
  'teal',
  'jade',
  'green',
  'grass',
  'bronze',
  'gold',
  'brown',
  'orange',
  'amber',
  'yellow',
  'lime',
  'mint',
  'sky'
] as const

export type RadixHue = (typeof radixHues)[number]

type RadixTheme = 'light' | 'dark'

export interface PresetFloraOptions {
  colors?: RadixHue[]
  aliases?: Partial<Record<string, RadixHue>>
  neutral?: RadixHue
  primary?: RadixHue
  info?: RadixHue
  tip?: RadixHue
  warning?: RadixHue
  danger?: RadixHue
  prefix?: string
  lightSelector?: string
  darkSelector?: string
  p3?: boolean
}

const steps = Array.from({ length: 12 }, (_, index) => index + 1)
const lightForegrounds = new Set<RadixHue>(['sky', 'mint', 'lime', 'yellow', 'amber'])

function isRadixHue(value: string): value is RadixHue {
  return radixHues.includes(value as RadixHue)
}

function foregroundFor(hue: RadixHue) {
  return lightForegrounds.has(hue) ? 'black' : 'white'
}

function variableName(prefix: string, name: string) {
  return `--${prefix}${name}`
}

function cssVariable(prefix: string, name: string) {
  return `var(${variableName(prefix, name)})`
}

function radixScale(hue: RadixHue, theme: RadixTheme, alpha = false, p3 = false) {
  const exportName = `${hue}${theme === 'dark' ? 'Dark' : ''}${p3 ? 'P3' : ''}${alpha ? 'A' : ''}`
  return (radix as Record<string, Record<string, string> | undefined>)[exportName] ?? {}
}

function overlayScale(color: 'black' | 'white', p3 = false) {
  return (
    (radix as Record<string, Record<string, string> | undefined>)[`${color}${p3 ? 'P3' : ''}A`] ??
    {}
  )
}

function radixColor(prefix: string, hue: RadixHue) {
  return {
    DEFAULT: cssVariable(prefix, `${hue}9`),
    foreground: cssVariable(prefix, `fg-${foregroundFor(hue)}`),
    fg: cssVariable(prefix, `fg-${foregroundFor(hue)}`),
    ...Object.fromEntries(steps.map((step) => [step, cssVariable(prefix, `${hue}${step}`)])),
    ...Object.fromEntries(steps.map((step) => [`${step}a`, cssVariable(prefix, `${hue}A${step}`)])),
    ...Object.fromEntries(steps.map((step) => [`${step}A`, cssVariable(prefix, `${hue}A${step}`)]))
  }
}

function scaleVars(prefix: string, hue: RadixHue, theme: RadixTheme, p3 = false) {
  const solid = radixScale(hue, theme, false, p3)
  const alpha = radixScale(hue, theme, true, p3)

  return [
    ...steps.map((step) => `${variableName(prefix, `${hue}${step}`)}: ${solid[`${hue}${step}`]};`),
    ...steps.map((step) => `${variableName(prefix, `${hue}A${step}`)}: ${alpha[`${hue}A${step}`]};`)
  ].join('')
}

function overlayVars(prefix: string, p3 = false) {
  return (['black', 'white'] as const)
    .flatMap((color) => {
      const scale = overlayScale(color, p3)
      return steps.map(
        (step) => `${variableName(prefix, `${color}A${step}`)}: ${scale[`${color}A${step}`]};`
      )
    })
    .join('')
}

function shadcnThemeVars(
  prefix: string,
  tokens: Required<
    Pick<PresetFloraOptions, 'neutral' | 'primary' | 'info' | 'tip' | 'warning' | 'danger'>
  >
) {
  const { neutral, primary, info, tip, warning, danger } = tokens

  return `
:root, .light {
  --radius: 0.625rem;
  --background: ${cssVariable(prefix, `${neutral}1`)};
  --foreground: ${cssVariable(prefix, `${neutral}12`)};
  --card: ${cssVariable(prefix, `${neutral}1`)};
  --card-foreground: ${cssVariable(prefix, `${neutral}12`)};
  --popover: ${cssVariable(prefix, `${neutral}1`)};
  --popover-foreground: ${cssVariable(prefix, `${neutral}12`)};
  --primary: ${cssVariable(prefix, `${primary}9`)};
  --primary-foreground: ${cssVariable(prefix, `fg-${foregroundFor(primary)}`)};
  --secondary: ${cssVariable(prefix, `${neutral}3`)};
  --secondary-foreground: ${cssVariable(prefix, `${neutral}12`)};
  --muted: ${cssVariable(prefix, `${neutral}3`)};
  --muted-foreground: ${cssVariable(prefix, `${neutral}11`)};
  --accent: ${cssVariable(prefix, `${primary}3`)};
  --accent-foreground: ${cssVariable(prefix, `${primary}12`)};
  --destructive: ${cssVariable(prefix, `${danger}9`)};
  --destructive-foreground: ${cssVariable(prefix, `fg-${foregroundFor(danger)}`)};
  --border: ${cssVariable(prefix, `${neutral}6`)};
  --input: ${cssVariable(prefix, `${neutral}7`)};
  --ring: ${cssVariable(prefix, `${primary}8`)};
  --grid-line-color: ${cssVariable(prefix, `${neutral}6`)};
  --grid-dot-color: ${cssVariable(prefix, `${neutral}9`)};
  --grid-dot-fill: var(--background);
  --chart-1: ${cssVariable(prefix, `${primary}9`)};
  --chart-2: ${cssVariable(prefix, `${info}9`)};
  --chart-3: ${cssVariable(prefix, `${tip}9`)};
  --chart-4: ${cssVariable(prefix, `${warning}9`)};
  --chart-5: ${cssVariable(prefix, `${danger}9`)};
  --sidebar: ${cssVariable(prefix, `${neutral}2`)};
  --sidebar-foreground: ${cssVariable(prefix, `${neutral}12`)};
  --sidebar-primary: ${cssVariable(prefix, `${primary}9`)};
  --sidebar-primary-foreground: ${cssVariable(prefix, `fg-${foregroundFor(primary)}`)};
  --sidebar-accent: ${cssVariable(prefix, `${primary}3`)};
  --sidebar-accent-foreground: ${cssVariable(prefix, `${primary}12`)};
  --sidebar-border: ${cssVariable(prefix, `${neutral}6`)};
  --sidebar-ring: ${cssVariable(prefix, `${primary}8`)};
}

.dark {
  --background: ${cssVariable(prefix, `${neutral}1`)};
  --foreground: ${cssVariable(prefix, `${neutral}12`)};
  --card: ${cssVariable(prefix, `${neutral}2`)};
  --card-foreground: ${cssVariable(prefix, `${neutral}12`)};
  --popover: ${cssVariable(prefix, `${neutral}2`)};
  --popover-foreground: ${cssVariable(prefix, `${neutral}12`)};
  --primary: ${cssVariable(prefix, `${primary}9`)};
  --primary-foreground: ${cssVariable(prefix, `fg-${foregroundFor(primary)}`)};
  --secondary: ${cssVariable(prefix, `${neutral}4`)};
  --secondary-foreground: ${cssVariable(prefix, `${neutral}12`)};
  --muted: ${cssVariable(prefix, `${neutral}4`)};
  --muted-foreground: ${cssVariable(prefix, `${neutral}11`)};
  --accent: ${cssVariable(prefix, `${primary}4`)};
  --accent-foreground: ${cssVariable(prefix, `${primary}12`)};
  --destructive: ${cssVariable(prefix, `${danger}9`)};
  --destructive-foreground: ${cssVariable(prefix, `fg-${foregroundFor(danger)}`)};
  --border: ${cssVariable(prefix, `${neutral}6`)};
  --input: ${cssVariable(prefix, `${neutral}7`)};
  --ring: ${cssVariable(prefix, `${primary}8`)};
  --grid-line-color: ${cssVariable(prefix, `${neutral}6`)};
  --grid-dot-color: ${cssVariable(prefix, `${neutral}9`)};
  --grid-dot-fill: var(--background);
  --chart-1: ${cssVariable(prefix, `${primary}9`)};
  --chart-2: ${cssVariable(prefix, `${info}9`)};
  --chart-3: ${cssVariable(prefix, `${tip}9`)};
  --chart-4: ${cssVariable(prefix, `${warning}9`)};
  --chart-5: ${cssVariable(prefix, `${danger}9`)};
  --sidebar: ${cssVariable(prefix, `${neutral}2`)};
  --sidebar-foreground: ${cssVariable(prefix, `${neutral}12`)};
  --sidebar-primary: ${cssVariable(prefix, `${primary}9`)};
  --sidebar-primary-foreground: ${cssVariable(prefix, `fg-${foregroundFor(primary)}`)};
  --sidebar-accent: ${cssVariable(prefix, `${primary}4`)};
  --sidebar-accent-foreground: ${cssVariable(prefix, `${primary}12`)};
  --sidebar-border: ${cssVariable(prefix, `${neutral}6`)};
  --sidebar-ring: ${cssVariable(prefix, `${primary}8`)};
}
`
}

export function presetFlora(options: PresetFloraOptions = {}): Preset {
  const tokens = {
    neutral: options.neutral ?? 'gray',
    primary: options.primary ?? 'iris',
    info: options.info ?? 'blue',
    tip: options.tip ?? 'green',
    warning: options.warning ?? 'yellow',
    danger: options.danger ?? 'red'
  } satisfies Required<
    Pick<PresetFloraOptions, 'neutral' | 'primary' | 'info' | 'tip' | 'warning' | 'danger'>
  >
  const aliases = { ...tokens, ...options.aliases } satisfies Record<string, RadixHue>
  const colors = [
    ...new Set([
      ...Object.values(tokens),
      'orange' as const,
      'pink' as const,
      'purple' as const,
      'teal' as const,
      ...(options.colors ?? [])
    ])
  ].filter(isRadixHue)
  const prefix = options.prefix ?? ''
  const lightSelector = options.lightSelector ?? '.light'
  const darkSelector = options.darkSelector ?? '.dark'
  const p3 = options.p3 ?? false

  return {
    name: 'preset-flora',
    theme: {
      colors: {
        border: 'var(--border)',
        input: 'var(--input)',
        ring: 'var(--ring)',
        background: 'var(--background)',
        foreground: 'var(--foreground)',
        primary: { DEFAULT: 'var(--primary)', foreground: 'var(--primary-foreground)' },
        secondary: { DEFAULT: 'var(--secondary)', foreground: 'var(--secondary-foreground)' },
        destructive: { DEFAULT: 'var(--destructive)', foreground: 'var(--destructive-foreground)' },
        muted: { DEFAULT: 'var(--muted)', foreground: 'var(--muted-foreground)' },
        accent: { DEFAULT: 'var(--accent)', foreground: 'var(--accent-foreground)' },
        popover: { DEFAULT: 'var(--popover)', foreground: 'var(--popover-foreground)' },
        card: { DEFAULT: 'var(--card)', foreground: 'var(--card-foreground)' },
        sidebar: {
          DEFAULT: 'var(--sidebar)',
          foreground: 'var(--sidebar-foreground)',
          primary: 'var(--sidebar-primary)',
          'primary-foreground': 'var(--sidebar-primary-foreground)',
          accent: 'var(--sidebar-accent)',
          'accent-foreground': 'var(--sidebar-accent-foreground)',
          border: 'var(--sidebar-border)',
          ring: 'var(--sidebar-ring)'
        },
        chart: Object.fromEntries(steps.slice(0, 5).map((step) => [step, `var(--chart-${step})`])),
        ...Object.fromEntries(colors.map((hue) => [hue, radixColor(prefix, hue)])),
        ...Object.fromEntries(
          Object.entries(aliases).map(([alias, hue]) => [alias, radixColor(prefix, hue)])
        ),
        black: {
          DEFAULT: '#000',
          ...Object.fromEntries(
            steps.map((step) => [`${step}a`, cssVariable(prefix, `blackA${step}`)])
          )
        },
        white: {
          DEFAULT: '#fff',
          ...Object.fromEntries(
            steps.map((step) => [`${step}a`, cssVariable(prefix, `whiteA${step}`)])
          )
        }
      },
      borderRadius: {
        sm: 'calc(var(--radius) - 4px)',
        md: 'calc(var(--radius) - 2px)',
        lg: 'var(--radius)',
        xl: 'calc(var(--radius) + 4px)'
      }
    },
    rules: [
      ['animate-in', { animation: 'flora-enter 150ms ease-out' }],
      ['animate-out', { animation: 'flora-exit 150ms ease-in forwards' }],
      ['fade-in-0', { '--flora-enter-opacity': '0' }],
      ['fade-out-0', { '--flora-exit-opacity': '0' }],
      ['zoom-in-95', { '--flora-enter-scale': '.95' }],
      ['zoom-out-95', { '--flora-exit-scale': '.95' }],
      ['slide-in-from-top-2', { '--flora-enter-translate-y': '-0.5rem' }],
      ['slide-in-from-bottom-2', { '--flora-enter-translate-y': '0.5rem' }],
      ['slide-in-from-left-2', { '--flora-enter-translate-x': '-0.5rem' }],
      ['slide-in-from-right-2', { '--flora-enter-translate-x': '0.5rem' }],
      ['animate-accordion-down', { animation: 'flora-accordion-down 0.2s ease-out' }],
      ['animate-accordion-up', { animation: 'flora-accordion-up 0.2s ease-out' }],
      ['animate-collapsible-down', { animation: 'flora-collapsible-down 0.2s ease-out' }],
      ['animate-collapsible-up', { animation: 'flora-collapsible-up 0.2s ease-out' }]
    ],
    preflights: [
      {
        layer: 'base',
        getCSS: () => {
          const baseVars = `:root { ${variableName(prefix, 'fg-white')}: #fff; ${variableName(prefix, 'fg-black')}: #000; ${overlayVars(prefix)} }`
          const lightVars = `${lightSelector}, :root:not(${darkSelector}) { ${colors.map((hue) => scaleVars(prefix, hue, 'light')).join('')} }`
          const darkVars = `${darkSelector} { ${colors.map((hue) => scaleVars(prefix, hue, 'dark')).join('')} }`
          const p3Vars = p3
            ? `@supports (color: color(display-p3 1 1 1)) { @media (color-gamut: p3) { :root { ${overlayVars(prefix, true)} } ${lightSelector}, :root:not(${darkSelector}) { ${colors.map((hue) => scaleVars(prefix, hue, 'light', true)).join('')} } ${darkSelector} { ${colors.map((hue) => scaleVars(prefix, hue, 'dark', true)).join('')} } } }`
            : ''
          const keyframes = `
@keyframes flora-enter { from { opacity: var(--flora-enter-opacity, 1); transform: translate3d(var(--flora-enter-translate-x, 0), var(--flora-enter-translate-y, 0), 0) scale3d(var(--flora-enter-scale, 1), var(--flora-enter-scale, 1), var(--flora-enter-scale, 1)) rotate(var(--flora-enter-rotate, 0)); } }
@keyframes flora-exit { to { opacity: var(--flora-exit-opacity, 1); transform: translate3d(var(--flora-exit-translate-x, 0), var(--flora-exit-translate-y, 0), 0) scale3d(var(--flora-exit-scale, 1), var(--flora-exit-scale, 1), var(--flora-exit-scale, 1)) rotate(var(--flora-exit-rotate, 0)); } }
@keyframes flora-accordion-down { from { height: 0; } to { height: var(--reka-accordion-content-height); } }
@keyframes flora-accordion-up { from { height: var(--reka-accordion-content-height); } to { height: 0; } }
@keyframes flora-collapsible-down { from { height: 0; } to { height: var(--reka-collapsible-content-height); } }
@keyframes flora-collapsible-up { from { height: var(--reka-collapsible-content-height); } to { height: 0; } }

*, ::before, ::after, ::backdrop { border-color: var(--border); outline-color: color-mix(in srgb, var(--ring) 50%, transparent); }
.flora-grid-line-v { position: absolute; top: 0; bottom: 0; z-index: 10; width: 1px; pointer-events: none; }
.flora-grid-line-v.solid { background: var(--grid-line-color); }
.flora-grid-line-v.dashed { background-image: linear-gradient(to bottom, var(--grid-line-color) 0 50%, transparent 50% 100%); background-size: 1px 8px; }
.flora-grid-dot { position: absolute; z-index: 20; width: 9px; height: 9px; align-items: center; justify-content: center; border-radius: 9999px; background: var(--grid-dot-fill); pointer-events: none; }
.flora-grid-dot::after { content: ''; width: 3px; height: 3px; border-radius: 9999px; background: var(--grid-dot-color); }
`

          return `${baseVars}${lightVars}${darkVars}${p3Vars}${shadcnThemeVars(prefix, tokens)}${keyframes}`
        }
      }
    ]
  }
}
