# @flora-internal/design-system

Flora's Vue design system package. It provides shadcn-vue components, the `presetFlora` UnoCSS preset, and layout helpers used by Flora frontends.

## Install in a workspace app

```json
{
  "dependencies": {
    "@flora-internal/design-system": "workspace:*"
  }
}
```

Import the package stylesheet once in the app entry:

```ts
import '@unocss/reset/tailwind.css'
import '@flora-internal/design-system/style.css'
import 'virtual:uno.css'
```

Configure UnoCSS with `presetFlora`:

```ts
import { presetFlora } from '@flora-internal/design-system/unocss'
import {
  defineConfig,
  presetIcons,
  presetWebFonts,
  presetWind4,
  transformerDirectives
} from 'unocss'

export default defineConfig({
  transformers: [transformerDirectives()],
  presets: [presetWind4(), presetIcons(), presetWebFonts(), presetFlora()],
  content: {
    pipeline: {
      include: [
        /\.(vue|svelte|[jt]sx|mdx?|astro|elm|php|phtml|html)($|\?)/,
        'src/**/*.{js,ts}',
        '../../packages/design-system/src/**/*.{js,ts,vue}'
      ]
    }
  }
})
```

## Tokens

The default token aliases are:

- `neutral`: gray
- `primary`: iris
- `info`: blue
- `tip`: green
- `warning`: yellow
- `danger`: red

Use semantic utilities like `bg-background`, `text-foreground`, `border-border`, `bg-card`, `text-muted-foreground`, and direct Radix scale utilities like `bg-primary-9`, `text-info-11`, or `border-neutral-6`.

## Components

Components are generated shadcn-vue components and keep the usual `class` override API.

```vue
<script setup lang="ts">
import { Button, Card, CardContent, CardHeader, CardTitle } from '@flora-internal/design-system'
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle>Example</CardTitle>
    </CardHeader>
    <CardContent>
      <Button>Save</Button>
    </CardContent>
  </Card>
</template>
```

## Grid lines

`GridLines` renders decorative frame lines and dots. Put it inside a `relative` container.

```vue
<script setup lang="ts">
import { GridLines } from '@flora-internal/design-system'
</script>

<template>
  <section class="relative mx-auto max-w-6xl [--grid-line-offset:20px]">
    <GridLines mode="lines-with-dots" />
    <slot />
  </section>
</template>
```

Available modes: `none`, `lines`, `dashed`, `lines-with-dots`.

Available variants: `frame`, `above-bottom-dots`, `tabbar-dots`, `navbar-lines`.
