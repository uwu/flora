import type { DetailedHTMLProps, HTMLAttributes } from 'react'

type MonacoEditorProps = DetailedHTMLProps<HTMLAttributes<HTMLElement>, HTMLElement> & {
  theme?: string
  fontSize?: string | number
  minimap?: string
  automaticLayout?: string | boolean
  padding?: string
}

declare module 'react' {
  namespace JSX {
    interface IntrinsicElements {
      'monaco-editor': MonacoEditorProps
    }
  }
}
