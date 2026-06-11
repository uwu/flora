import type { Meta, StoryObj } from '@storybook/vue3'
import { Badge, Kbd } from '../src'

const meta = {
  title: 'Foundations/Tokens',
  parameters: {
    layout: 'fullscreen'
  }
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

const swatches = [
  ['Background', 'var(--fl-color-bg)'],
  ['Subtle', 'var(--fl-color-bg-subtle)'],
  ['Surface', 'var(--fl-color-surface)'],
  ['Primary', 'var(--fl-color-primary)'],
  ['Danger', 'var(--fl-color-danger)'],
  ['Border', 'var(--fl-color-border)']
]

export const Tokens: Story = {
  render: () => ({
    components: { Badge, Kbd },
    setup() {
      return { swatches }
    },
    template: `
      <section class="grid gap-8">
        <div class="grid gap-2">
          <Badge tone="primary">Vue 3 + UnoCSS</Badge>
          <h1 class="m-0 max-w-160 text-3xl font-750 leading-tight text-[var(--fl-color-text)]">
            Flora design tokens are semantic CSS variables backed by Radix color scales.
          </h1>
          <p class="m-0 max-w-150 text-sm leading-6 text-[var(--fl-color-text-muted)]">
            Components use restrained surfaces, stable radii, visible focus rings, and tabular numbers where layout can shift.
          </p>
        </div>

        <div class="grid grid-cols-2 gap-3 md:grid-cols-3">
          <div
            v-for="[label, value] in swatches"
            :key="label"
            class="rounded-[var(--fl-radius-lg)] bg-[var(--fl-color-surface)] p-3 shadow-[var(--fl-shadow-border)]"
          >
            <div class="h-14 rounded-[var(--fl-radius-md)] shadow-[inset_0_0_0_1px_var(--fl-color-border)]" :style="{ background: value }" />
            <div class="mt-3 text-sm font-650">{{ label }}</div>
            <code class="text-xs text-[var(--fl-color-text-subtle)]">{{ value }}</code>
          </div>
        </div>

        <div class="flex flex-wrap items-center gap-2">
          <Kbd>Cmd</Kbd>
          <Kbd>K</Kbd>
          <span class="text-sm text-[var(--fl-color-text-muted)]">Keyboard tokens use tabular alignment and compact radii.</span>
        </div>
      </section>
    `
  })
}
