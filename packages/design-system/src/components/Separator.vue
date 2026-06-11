<script setup lang="ts">
import { Separator as RekaSeparator } from 'reka-ui'

defineOptions({ name: 'FlSeparator' })

withDefaults(
  defineProps<{
    orientation?: 'horizontal' | 'vertical'
  }>(),
  {
    orientation: 'horizontal'
  }
)
</script>

<template>
  <RekaSeparator
    data-slot="separator"
    :orientation="orientation"
    :class="[
      'shrink-0 bg-[var(--fl-color-border)]',
      orientation === 'horizontal' && 'h-px w-full',
      orientation === 'vertical' && 'h-full w-px'
    ]"
  />
</template>
