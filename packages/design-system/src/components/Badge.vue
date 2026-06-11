<script setup lang="ts">
import { badgeVariants, type BadgeVariants } from '../variants'

defineOptions({ name: 'FlBadge' })

withDefaults(
  defineProps<{
    tone?: BadgeVariants['tone']
    variant?: BadgeVariants['variant']
  }>(),
  {
    tone: 'neutral',
    variant: 'soft'
  }
)
</script>

<template>
  <span data-slot="badge" :class="badgeVariants({ tone, variant })">
    <slot />
  </span>
</template>
