<script setup lang="ts">
import type { BadgeVariants } from '../variants'

defineOptions({ name: 'FlAlert' })

withDefaults(
  defineProps<{
    tone?: Exclude<BadgeVariants['tone'], null | undefined>
    title?: string
  }>(),
  {
    tone: 'info',
    title: undefined
  }
)
</script>

<template>
  <div
    data-slot="alert"
    :class="[
      'grid grid-cols-[auto_1fr] gap-3 rounded-[var(--fl-radius-lg)] p-4 text-sm shadow-[inset_0_0_0_1px_var(--fl-color-border)]',
      tone === 'neutral' && 'bg-[var(--grayA2)] text-[var(--fl-color-text-muted)]',
      tone === 'primary' &&
        'bg-[var(--fl-color-primary-soft)] text-[var(--fl-color-primary-soft-text)]',
      tone === 'success' &&
        'bg-[var(--fl-color-success-soft)] text-[var(--fl-color-success-soft-text)]',
      tone === 'warning' &&
        'bg-[var(--fl-color-warning-soft)] text-[var(--fl-color-warning-soft-text)]',
      tone === 'danger' &&
        'bg-[var(--fl-color-danger-soft)] text-[var(--fl-color-danger-soft-text)]',
      tone === 'info' && 'bg-[var(--fl-color-info-soft)] text-[var(--fl-color-info-soft-text)]'
    ]"
  >
    <slot name="icon">
      <span
        v-if="tone === 'danger'"
        class="i-lucide-alert-circle mt-0.5 size-4"
        aria-hidden="true"
      />
      <span v-else class="i-lucide-info mt-0.5 size-4" aria-hidden="true" />
    </slot>
    <div class="grid gap-1">
      <div v-if="title" class="font-700 text-[var(--fl-color-text)]">{{ title }}</div>
      <div class="leading-6"><slot /></div>
    </div>
  </div>
</template>
