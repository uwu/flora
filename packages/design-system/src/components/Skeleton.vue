<script setup lang="ts">
defineOptions({ name: 'FlSkeleton' })

withDefaults(
  defineProps<{
    shape?: 'line' | 'rect' | 'circle'
  }>(),
  {
    shape: 'line'
  }
)
</script>

<template>
  <span
    aria-hidden="true"
    data-slot="skeleton"
    :class="[
      'block animate-pulse bg-[var(--fl-color-bg-muted)]',
      shape === 'line' && 'h-4 w-full rounded-[var(--fl-radius-sm)]',
      shape === 'rect' && 'min-h-20 w-full rounded-[var(--fl-radius-lg)]',
      shape === 'circle' && 'size-10 rounded-full'
    ]"
  />
</template>
