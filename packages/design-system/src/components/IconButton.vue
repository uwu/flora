<script setup lang="ts">
import Button from './Button.vue'
import type { ButtonVariants } from '../variants'

defineOptions({ name: 'FlIconButton' })

withDefaults(
  defineProps<{
    label: string
    variant?: ButtonVariants['variant']
    size?: Extract<ButtonVariants['size'], 'icon' | 'icon-sm' | 'icon-lg'>
    loading?: boolean
    disabled?: boolean
  }>(),
  {
    variant: 'ghost',
    size: 'icon',
    loading: false,
    disabled: false
  }
)
</script>

<template>
  <Button
    :variant="variant"
    :size="size"
    :loading="loading"
    :disabled="disabled"
    :aria-label="label"
  >
    <slot />
  </Button>
</template>
