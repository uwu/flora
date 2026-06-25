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

export interface PresetFloraShadcnOptions {
  colors?: RadixHue[]
  aliases?: Partial<Record<string, RadixHue>>
  primary?: RadixHue
  neutral?: RadixHue
  lightSelector?: string
  darkSelector?: string
  prefix?: string
  p3?: boolean
}

const steps = Array.from({ length: 12 }, (_, index) => index + 1)
const lightForegrounds = new Set<RadixHue>(['sky', 'mint', 'lime', 'yellow', 'amber'])

function isRadixHue(value: string): value is RadixHue {
  return radixHues.includes(value as RadixHue)
}

function getForeground(hue: RadixHue) {
  return lightForegrounds.has(hue) ? 'black' : 'white'
}

function varName(prefix: string, name: string) {
  return `--${prefix}${name}`
}

function getRadixScale(hue: RadixHue, theme: 'light' | 'dark', alpha = false, p3 = false) {
  const exportName = `${hue}${theme === 'dark' ? 'Dark' : ''}${p3 ? 'P3' : ''}${alpha ? 'A' : ''}`
  return (radix as Record<string, Record<string, string> | undefined>)[exportName] ?? {}
}

function getOverlayScale(color: 'black' | 'white', p3 = false) {
  return (radix as Record<string, Record<string, string> | undefined>)[`${color}${p3 ? 'P3' : ''}A`] ?? {}
}

function radixThemeColor(prefix: string, hue: RadixHue) {
  return {
    DEFAULT: `var(${varName(prefix, `${hue}9`)})`,
    foreground: `var(${varName(prefix, `fg-${getForeground(hue)}`)})`,
    fg: `var(${varName(prefix, `fg-${getForeground(hue)}`)})`,
    ...Object.fromEntries(steps.map((step) => [step, `var(${varName(prefix, `${hue}${step}`)})`])),
    ...Object.fromEntries(steps.map((step) => [`${step}a`, `var(${varName(prefix, `${hue}A${step}`)})`])),
    ...Object.fromEntries(steps.map((step) => [`${step}A`, `var(${varName(prefix, `${hue}A${step}`)})`]))
  }
}

function makeScaleVars(prefix: string, hue: RadixHue, theme: 'light' | 'dark', p3 = false) {
  const solid = getRadixScale(hue, theme, false, p3)
  const alpha = getRadixScale(hue, theme, true, p3)

  return [
    ...steps.map((step) => `${varName(prefix, `${hue}${step}`)}: ${solid[`${hue}${step}`]};`),
    ...steps.map((step) => `${varName(prefix, `${hue}A${step}`)}: ${alpha[`${hue}A${step}`]};`)
  ].join('')
}

function makeOverlayVars(prefix: string, p3 = false) {
  return (['black', 'white'] as const)
    .flatMap((color) => {
      const scale = getOverlayScale(color, p3)
      return steps.map((step) => `${varName(prefix, `${color}A${step}`)}: ${scale[`${color}A${step}`]};`)
    })
    .join('')
}

function makeShadcnVars(prefix: string, primary: RadixHue, neutral: RadixHue) {
  return `
:root, .light {
  --background: var(${varName(prefix, `${neutral}1`)});
  --foreground: var(${varName(prefix, `${neutral}12`)});
  --card: var(${varName(prefix, `${neutral}1`)});
  --card-foreground: var(${varName(prefix, `${neutral}12`)});
  --popover: var(${varName(prefix, `${neutral}1`)});
  --popover-foreground: var(${varName(prefix, `${neutral}12`)});
  --primary: var(${varName(prefix, `${primary}9`)});
  --primary-foreground: var(${varName(prefix, `fg-${getForeground(primary)}`)});
  --secondary: var(${varName(prefix, `${neutral}3`)});
  --secondary-foreground: var(${varName(prefix, `${neutral}12`)});
  --muted: var(${varName(prefix, `${neutral}3`)});
  --muted-foreground: var(${varName(prefix, `${neutral}11`)});
  --accent: var(${varName(prefix, `${neutral}3`)});
  --accent-foreground: var(${varName(prefix, `${neutral}12`)});
  --destructive: var(${varName(prefix, 'red9')});
  --destructive-foreground: var(${varName(prefix, 'fg-white')});
  --border: var(${varName(prefix, `${neutral}6`)});
  --input: var(${varName(prefix, `${neutral}7`)});
  --ring: var(${varName(prefix, `${primary}8`)});
  --chart-1: var(${varName(prefix, 'orange9')});
  --chart-2: var(${varName(prefix, 'teal9')});
  --chart-3: var(${varName(prefix, 'blue9')});
  --chart-4: var(${varName(prefix, 'yellow9')});
  --chart-5: var(${varName(prefix, 'pink9')});
  --radius: 0.625rem;
  --sidebar: var(${varName(prefix, `${neutral}2`)});
  --sidebar-foreground: var(${varName(prefix, `${neutral}12`)});
  --sidebar-primary: var(${varName(prefix, `${primary}9`)});
  --sidebar-primary-foreground: var(${varName(prefix, `fg-${getForeground(primary)}`)});
  --sidebar-accent: var(${varName(prefix, `${neutral}3`)});
  --sidebar-accent-foreground: var(${varName(prefix, `${neutral}12`)});
  --sidebar-border: var(${varName(prefix, `${neutral}6`)});
  --sidebar-ring: var(${varName(prefix, `${primary}8`)});
}
.dark {
  --background: var(${varName(prefix, `${neutral}1`)});
  --foreground: var(${varName(prefix, `${neutral}12`)});
  --card: var(${varName(prefix, `${neutral}2`)});
  --card-foreground: var(${varName(prefix, `${neutral}12`)});
  --popover: var(${varName(prefix, `${neutral}2`)});
  --popover-foreground: var(${varName(prefix, `${neutral}12`)});
  --primary: var(${varName(prefix, `${primary}9`)});
  --primary-foreground: var(${varName(prefix, `fg-${getForeground(primary)}`)});
  --secondary: var(${varName(prefix, `${neutral}4`)});
  --secondary-foreground: var(${varName(prefix, `${neutral}12`)});
  --muted: var(${varName(prefix, `${neutral}4`)});
  --muted-foreground: var(${varName(prefix, `${neutral}11`)});
  --accent: var(${varName(prefix, `${neutral}4`)});
  --accent-foreground: var(${varName(prefix, `${neutral}12`)});
  --destructive: var(${varName(prefix, 'red9')});
  --destructive-foreground: var(${varName(prefix, 'fg-white')});
  --border: var(${varName(prefix, `${neutral}6`)});
  --input: var(${varName(prefix, `${neutral}7`)});
  --ring: var(${varName(prefix, `${primary}8`)});
  --chart-1: var(${varName(prefix, 'blue9')});
  --chart-2: var(${varName(prefix, 'green9')});
  --chart-3: var(${varName(prefix, 'orange9')});
  --chart-4: var(${varName(prefix, 'purple9')});
  --chart-5: var(${varName(prefix, 'pink9')});
  --sidebar: var(${varName(prefix, `${neutral}2`)});
  --sidebar-foreground: var(${varName(prefix, `${neutral}12`)});
  --sidebar-primary: var(${varName(prefix, `${primary}9`)});
  --sidebar-primary-foreground: var(${varName(prefix, `fg-${getForeground(primary)}`)});
  --sidebar-accent: var(${varName(prefix, `${neutral}4`)});
  --sidebar-accent-foreground: var(${varName(prefix, `${neutral}12`)});
  --sidebar-border: var(${varName(prefix, `${neutral}6`)});
  --sidebar-ring: var(${varName(prefix, `${primary}8`)});
}
`
}

export function presetFloraShadcn(options: PresetFloraShadcnOptions = {}): Preset {
  const primary = options.primary ?? 'iris'
  const neutral = options.neutral ?? 'gray'
  const aliases = {
    neutral,
    primary,
    info: 'blue',
    tip: 'green',
    warning: 'yellow',
    danger: 'red',
    ...options.aliases
  } satisfies Record<string, RadixHue>
  const colors = [...new Set([neutral, primary, 'red', 'orange', 'yellow', 'pink', 'blue', 'green', 'teal', 'purple', ...(options.colors ?? [])])].filter(isRadixHue)
  const prefix = options.prefix ?? ''
  const lightSelector = options.lightSelector ?? '.light'
  const darkSelector = options.darkSelector ?? '.dark'
  const p3 = options.p3 ?? true

  return {
    name: 'flora-shadcn',
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
        ...Object.fromEntries(colors.map((hue) => [hue, radixThemeColor(prefix, hue)])),
        ...Object.fromEntries(Object.entries(aliases).map(([alias, hue]) => [alias, radixThemeColor(prefix, hue)])),
        black: { DEFAULT: '#000', ...Object.fromEntries(steps.map((step) => [`${step}a`, `var(${varName(prefix, `blackA${step}`)})`])) },
        white: { DEFAULT: '#fff', ...Object.fromEntries(steps.map((step) => [`${step}a`, `var(${varName(prefix, `whiteA${step}`)})`])) }
      },
      borderRadius: {
        lg: 'var(--radius)',
        md: 'calc(var(--radius) - 2px)',
        sm: 'calc(var(--radius) - 4px)',
        xl: 'calc(var(--radius) + 4px)'
      }
    },
    preflights: [
      {
        layer: 'base',
        getCSS: () => {
          const globalVars = `:root { ${varName(prefix, 'fg-white')}: #fff; ${varName(prefix, 'fg-black')}: #000; ${makeOverlayVars(prefix)} }`
          const lightVars = `${lightSelector}, :root:not(${darkSelector}) { ${colors.map((hue) => makeScaleVars(prefix, hue, 'light')).join('')} }`
          const darkVars = `${darkSelector} { ${colors.map((hue) => makeScaleVars(prefix, hue, 'dark')).join('')} }`
          const p3Vars = p3
            ? `@supports (color: color(display-p3 1 1 1)) { @media (color-gamut: p3) { :root { ${makeOverlayVars(prefix, true)} } ${lightSelector}, :root:not(${darkSelector}) { ${colors.map((hue) => makeScaleVars(prefix, hue, 'light', true)).join('')} } ${darkSelector} { ${colors.map((hue) => makeScaleVars(prefix, hue, 'dark', true)).join('')} } } }`
            : ''
          return `${globalVars}${lightVars}${darkVars}${p3Vars}${makeShadcnVars(prefix, primary, neutral)}`
        }
      }
    ]
  }
}
