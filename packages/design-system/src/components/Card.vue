<script setup lang="ts">
import { surfaceVariants, type SurfaceVariants } from '../variants'

defineOptions({ name: 'FlCard' })

withDefaults(
  defineProps<{
    variant?: SurfaceVariants['variant']
    padding?: SurfaceVariants['padding']
  }>(),
  {
    variant: 'bordered',
    padding: 'md'
  }
)
</script>

<template>
  <section data-slot="card" :class="surfaceVariants({ variant, padding })">
    <slot />
  </section>
</template>
