<script setup lang="ts">
defineOptions({ name: 'FlEmptyState' })

withDefaults(
  defineProps<{
    title: string
    description?: string
  }>(),
  {
    description: undefined
  }
)
</script>

<template>
  <div
    data-slot="empty-state"
    class="flex min-h-40 flex-col items-center justify-center rounded-[var(--fl-radius-lg)] bg-[var(--fl-color-bg-subtle)] px-6 py-8 text-center shadow-[inset_0_0_0_1px_var(--fl-color-border)]"
  >
    <slot name="icon">
      <span
        class="i-lucide-search mb-3 size-5 text-[var(--fl-color-text-subtle)]"
        aria-hidden="true"
      />
    </slot>
    <h3 class="m-0 text-base font-700 leading-6 text-[var(--fl-color-text)]">{{ title }}</h3>
    <p
      v-if="description"
      class="m-0 mt-1 max-w-80 text-sm leading-6 text-[var(--fl-color-text-muted)]"
    >
      {{ description }}
    </p>
    <div v-if="$slots.actions" class="mt-4 flex flex-wrap items-center justify-center gap-2">
      <slot name="actions" />
    </div>
  </div>
</template>
