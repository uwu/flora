<script setup lang="ts">
import { inputVariants, type InputVariants } from '../variants'

defineOptions({ name: 'FlInput' })

const model = defineModel<string | number>()

withDefaults(
  defineProps<{
    size?: InputVariants['size']
    invalid?: boolean
  }>(),
  {
    size: 'md',
    invalid: false
  }
)
</script>

<template>
  <input
    v-model="model"
    data-slot="input"
    :aria-invalid="invalid || undefined"
    :class="inputVariants({ size })"
  />
</template>
