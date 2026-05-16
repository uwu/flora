# Florine

Florine is a lightweight VS Code-inspired React editor shell. It combines a
Monaco editor, Dockview workbench, and [`@pierre/trees`](https://trees.software/)
file explorer without bundling a browser shell or terminal runtime.

## Usage

Import the component and stylesheet from the package:

```tsx
import { Florine } from '@flora/florine'
import '@flora/florine/style.css'

export function App() {
  return (
    <Florine
      files={{
        'README.md': '# Hello Florine\n',
        'src/index.ts': "console.log('hello')\n"
      }}
      style={{ height: 600 }}
    />
  )
}
```

## Development

- Install dependencies:

```bash
vp install
```

- Run the unit tests:

```bash
vp test
```

- Build the library:

```bash
vp pack
```

- Run the example:

```bash
vp run dev:example
```
