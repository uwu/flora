export type GraphicType =
  | 'waveformBars'
  | 'gridBlocks'
  | 'noiseLines'
  | 'fluidGrid'
  | 'interfaceBlueprint'

export type StackCard = {
  id: string
  title: string
  titleLines: number
  description: string
  href: string
  background: string
  foreground: string
  bodyColor: string
  graphicType: GraphicType
  seed: number
}

export const STACK_CARDS: StackCard[] = [
  {
    id: '1',
    title: 'Batteries Included',
    titleLines: 2,
    description:
      'Key-value storage, secrets management, sandboxing, and more — all built in, with more to come.',
    href: 'https://flora.uwu.network/docs/runtime',
    background: '#E54F10',
    foreground: '#FFFFC2',
    bodyColor: 'rgba(255, 255, 194, 0.7)',
    graphicType: 'waveformBars',
    seed: 12345
  },
  {
    id: '2',
    title: 'Build\nTogether',
    titleLines: 2,
    description: 'Build together with your server administrators, versioned every deployment.',
    href: 'https://flora.uwu.network/docs/deployments',
    background: '#F6EBD9',
    foreground: '#524733',
    bodyColor: 'rgba(82, 71, 51, 0.8)',
    graphicType: 'gridBlocks',
    seed: 54321
  },
  {
    id: '3',
    title: 'Observability\nPatterns',
    titleLines: 2,
    description: 'Coming soon — inspect logs and runtime behavior on how long your tasks take.',
    href: 'https://flora.uwu.network/docs/observability',
    background: '#0A90D2',
    foreground: '#AEFFFF',
    bodyColor: 'rgba(174, 255, 255, 0.8)',
    graphicType: 'noiseLines',
    seed: 11111
  },
  {
    id: '4',
    title: 'Build\nFast',
    titleLines: 2,
    description:
      "Ship faster with flora's fast V8-based runtime with built-in support for TypeScript.",
    href: 'https://flora.uwu.network/docs/runtime',
    background: '#53F399',
    foreground: '#004D00',
    bodyColor: 'rgba(0, 77, 0, 0.7)',
    graphicType: 'fluidGrid',
    seed: 22222
  },
  {
    id: '5',
    title: 'SDK\nReference',
    titleLines: 2,
    description:
      'Deep links into SDK usage, commands, and integration examples for faster shipping.',
    href: 'https://flora.uwu.network/docs/sdk',
    background: '#211F1E',
    foreground: '#F6EBD9',
    bodyColor: 'rgba(246, 235, 217, 0.7)',
    graphicType: 'interfaceBlueprint',
    seed: 33333
  }
]

export const CLUSTER_LAYOUT: {
  expanded: Record<string, { rotation: number; offsetX: number; offsetY: number }>
  collapsed: Record<string, { offsetX: number; offsetY: number; rotation: number; scale: number }>
} = {
  expanded: {
    '1': { rotation: -8, offsetX: -306, offsetY: -10 },
    '2': { rotation: 4, offsetX: -151, offsetY: 20 },
    '3': { rotation: -2, offsetX: 0, offsetY: -41 },
    '4': { rotation: 1, offsetX: 147, offsetY: 16 },
    '5': { rotation: 5, offsetX: 310, offsetY: -19 }
  },
  collapsed: {
    '1': { offsetX: 49, offsetY: 48, rotation: -4, scale: 1 },
    '2': { offsetX: 31, offsetY: 49, rotation: -2, scale: 1 },
    '3': { offsetX: 0, offsetY: 51, rotation: 0, scale: 1 },
    '4': { offsetX: -10, offsetY: 53, rotation: 2, scale: 1 },
    '5': { offsetX: -33, offsetY: 57, rotation: 3, scale: 1 }
  }
}

export const CARD_SIZES = {
  compact: {
    cardW: 228,
    cardH: 288,
    padding: 16,
    graphicW: 196,
    graphicH: 120,
    titleSize: 28,
    titleLineH: 30
  },
  expanded: {
    cardW: 360,
    cardH: 464,
    padding: 24,
    graphicW: 312,
    graphicH: 192,
    titleSize: 36,
    titleLineH: 36
  }
}

export const STACK_ANIMATION = {
  card: { type: 'spring' as const, visualDuration: 0.4, bounce: 0.15 },
  description: { type: 'spring' as const, visualDuration: 0.2, bounce: 0.1 }
}
