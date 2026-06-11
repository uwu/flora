<script setup lang="ts">
import { inputVariants, type InputVariants } from '../variants'

defineOptions({ name: 'FlTextarea' })

const model = defineModel<string>()

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
  <textarea
    v-model="model"
    data-slot="textarea"
    :aria-invalid="invalid || undefined"
    :class="[inputVariants({ size }), 'min-h-24 resize-y py-2 leading-6']"
  />
</template>
