<script setup lang="ts">
defineOptions({ name: 'FlSpinner' })

withDefaults(
  defineProps<{
    label?: string
    size?: 'sm' | 'md' | 'lg'
  }>(),
  {
    label: 'Loading',
    size: 'md'
  }
)
</script>

<template>
  <span
    role="status"
    :aria-label="label"
    :class="[
      'i-lucide-loader-2 inline-block animate-spin text-[var(--fl-color-text-muted)]',
      size === 'sm' && 'size-3.5',
      size === 'md' && 'size-4',
      size === 'lg' && 'size-5'
    ]"
  />
</template>
