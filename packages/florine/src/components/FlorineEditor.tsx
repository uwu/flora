import { Editor as MonacoEditor, type OnChange } from '@monaco-editor/react'

import { languageForPath } from '../utils/path.ts'

export type FlorineEditorProps = {
  path: string
  content: string
  onChange: (path: string, content: string) => void
}

export function FlorineEditor(props: FlorineEditorProps) {
  const handleChange: OnChange = (value) => props.onChange(props.path, value ?? '')

  return (
    <MonacoEditor
      path={props.path}
      value={props.content}
      theme='vs-dark'
      language={languageForPath(props.path)}
      options={{
        automaticLayout: true,
        minimap: { enabled: false },
        padding: { top: 12 },
        scrollBeyondLastLine: false
      }}
      onChange={handleChange}
    />
  )
}
