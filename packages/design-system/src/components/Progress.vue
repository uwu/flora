<script setup lang="ts">
import { ProgressIndicator, ProgressRoot } from 'reka-ui'
import { computed } from 'vue'

defineOptions({ name: 'FlProgress' })

const props = withDefaults(
  defineProps<{
    modelValue?: number
    max?: number
    label?: string
  }>(),
  {
    modelValue: 0,
    max: 100,
    label: 'Progress'
  }
)

const clampedValue = computed(() => Math.min(Math.max(props.modelValue, 0), props.max))
const width = computed(() => `${(clampedValue.value / props.max) * 100}%`)
</script>

<template>
  <ProgressRoot
    data-slot="progress"
    class="relative h-2 w-full overflow-hidden rounded-[var(--fl-radius-pill)] bg-[var(--fl-color-bg-muted)]"
    :model-value="clampedValue"
    :max="max"
    :aria-label="label"
  >
    <ProgressIndicator
      class="h-full rounded-[var(--fl-radius-pill)] bg-[var(--fl-color-primary)] transition-[width] duration-[var(--fl-duration-slow)] ease-[var(--fl-ease-standard)]"
      :style="{ width }"
    />
  </ProgressRoot>
</template>
