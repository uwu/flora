import { createBundledHighlighter } from '@shikijs/core'
import {
  createJavaScriptRegexEngine,
  defaultJavaScriptRegexConstructor
} from '@shikijs/engine-javascript'

const bundledLanguages = {
  typescript: () => import('@shikijs/langs/typescript'),
  tsx: () => import('@shikijs/langs/tsx'),
  javascript: () => import('@shikijs/langs/javascript'),
  jsx: () => import('@shikijs/langs/jsx'),
  json: () => import('@shikijs/langs/json')
} as const

const bundledLanguagesInfo = [
  { id: 'typescript', name: 'TypeScript', import: bundledLanguages.typescript },
  { id: 'tsx', name: 'TSX', import: bundledLanguages.tsx },
  { id: 'javascript', name: 'JavaScript', import: bundledLanguages.javascript },
  { id: 'jsx', name: 'JSX', import: bundledLanguages.jsx },
  { id: 'json', name: 'JSON', import: bundledLanguages.json }
] as const

const bundledLanguagesAlias = {
  ts: 'typescript',
  tsx: 'tsx',
  js: 'javascript',
  jsx: 'jsx'
} as const

const bundledLanguagesBase = bundledLanguages
const bundledThemes = {} as const
const bundledThemesInfo: Array<unknown> = []

const createHighlighter = createBundledHighlighter({
  langs: bundledLanguages,
  themes: bundledThemes,
  engine: () => createJavaScriptRegexEngine()
})

export {
  bundledLanguages,
  bundledLanguagesInfo,
  bundledLanguagesAlias,
  bundledLanguagesBase,
  bundledThemes,
  bundledThemesInfo,
  createHighlighter,
  createJavaScriptRegexEngine,
  defaultJavaScriptRegexConstructor
}

export * from '@shikijs/core'
