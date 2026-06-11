<script setup lang="ts">
import { Primitive, type PrimitiveProps } from 'reka-ui'
import { computed } from 'vue'
import { buttonVariants, type ButtonVariants } from '../variants'

defineOptions({ name: 'FlButton' })

const props = withDefaults(
  defineProps<
    PrimitiveProps & {
      variant?: ButtonVariants['variant']
      size?: ButtonVariants['size']
      loading?: boolean
      static?: boolean
      disabled?: boolean
    }
  >(),
  {
    as: 'button',
    asChild: false,
    variant: 'primary',
    size: 'md',
    loading: false,
    static: false,
    disabled: false
  }
)

const isDisabled = computed(() => props.disabled || props.loading)
</script>

<template>
  <Primitive
    data-slot="button"
    :as="as"
    :as-child="asChild"
    :class="buttonVariants({ variant, size, loading, static: props.static })"
    :disabled="isDisabled || undefined"
    :aria-disabled="isDisabled || undefined"
    :aria-busy="loading || undefined"
  >
    <span v-if="loading" class="i-lucide-loader-2 size-4 animate-spin" aria-hidden="true" />
    <slot />
  </Primitive>
</template>
