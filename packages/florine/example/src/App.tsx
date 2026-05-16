import { Florine } from '@flora/florine'
import '@flora/florine/style.css'

function App() {
  return (
    <Florine
      files={{
        'README.md': '# Florine example\n\nEdit files from the explorer.\n',
        'package.json': '{\n  "name": "florine-example"\n}\n',
        'src/App.tsx': "import { Florine } from '@flora/florine'\n"
      }}
    />
  )
}

export default App
