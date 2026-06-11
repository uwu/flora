<script setup lang="ts">
import { Label } from 'reka-ui'

defineOptions({ name: 'FlLabel' })
</script>

<template>
  <Label
    data-slot="label"
    class="inline-flex items-center gap-1 text-sm font-650 leading-none text-[var(--fl-color-text)]"
  >
    <slot />
  </Label>
</template>
