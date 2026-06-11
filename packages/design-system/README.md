# @flora-internal/design-system

Flora's internal Vue 3 design system. It uses TypeScript, Reka UI primitives, VueUse, UnoCSS, and Radix color variables.

## Usage

```ts
import { Button, DialogContent, DialogRoot } from '@flora-internal/design-system'
import '@flora-internal/design-system/style.css'
```

The package ships compiled CSS. Consumers do not need to scan this package with UnoCSS.

## Development

```bash
pnpm --filter @flora-internal/design-system build
pnpm --filter @flora-internal/design-system test
pnpm --filter @flora-internal/design-system storybook
```

## Scope

- Foundations: semantic CSS variables, light/dark Radix color mapping, focus rings, reduced motion, typography helpers, elevation, and hit-area utilities.
- Components: buttons, form controls, feedback, tables, app shell/sidebar, and Reka-backed overlays/navigation primitives.
- Tests: Vitest, Vue Test Utils, Happy DOM, and `vitest-axe`.
- Docs: Storybook with a11y addon.

Terrazzo is not in the active v1 build path. Tokens are authored as semantic CSS variables in `src/style.css`.
