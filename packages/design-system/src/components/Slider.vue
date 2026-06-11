<script setup lang="ts">
import { SliderRange, SliderRoot, SliderThumb, SliderTrack } from 'reka-ui'

defineOptions({ name: 'FlSlider' })

const model = defineModel<number[]>({ default: [0] })

withDefaults(
  defineProps<{
    min?: number
    max?: number
    step?: number
    disabled?: boolean
    label?: string
  }>(),
  {
    min: 0,
    max: 100,
    step: 1,
    disabled: false,
    label: 'Value'
  }
)
</script>

<template>
  <SliderRoot
    v-model="model"
    data-slot="slider"
    class="relative flex h-10 w-full touch-none select-none items-center"
    :min="min"
    :max="max"
    :step="step"
    :disabled="disabled"
    :aria-label="label"
  >
    <SliderTrack
      class="relative h-2 w-full grow overflow-hidden rounded-[var(--fl-radius-pill)] bg-[var(--fl-color-bg-muted)]"
    >
      <SliderRange
        class="absolute h-full rounded-[var(--fl-radius-pill)] bg-[var(--fl-color-primary)]"
      />
    </SliderTrack>
    <SliderThumb
      v-for="(_, index) in model"
      :key="index"
      class="fl-focus-ring block size-5 rounded-full bg-[var(--fl-color-surface)] shadow-[var(--fl-shadow-border-hover)] outline-none transition-[scale,box-shadow] duration-[var(--fl-duration-base)] ease-[var(--fl-ease-standard)] hover:scale-105 active:scale-[0.96] disabled:pointer-events-none disabled:opacity-50"
      :aria-label="label"
    />
  </SliderRoot>
</template>
