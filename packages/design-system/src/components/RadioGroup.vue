<script setup lang="ts">
import { RadioGroupIndicator, RadioGroupItem, RadioGroupRoot, type AcceptableValue } from 'reka-ui'
import type { RadioOption } from '../types'

defineOptions({ name: 'FlRadioGroup' })

const model = defineModel<AcceptableValue>()

withDefaults(
  defineProps<{
    options: RadioOption[]
    orientation?: 'horizontal' | 'vertical'
    disabled?: boolean
  }>(),
  {
    orientation: 'vertical',
    disabled: false
  }
)
</script>

<template>
  <RadioGroupRoot
    v-model="model"
    data-slot="radio-group"
    :orientation="orientation"
    :disabled="disabled"
    :class="['flex gap-2', orientation === 'vertical' ? 'flex-col' : 'flex-row flex-wrap']"
  >
    <label
      v-for="option in options"
      :key="String(option.value)"
      class="inline-flex min-h-10 items-start gap-2 rounded-[var(--fl-radius-md)] text-sm text-[var(--fl-color-text)]"
    >
      <RadioGroupItem
        data-slot="radio-item"
        class="fl-focus-ring fl-hit-area mt-0.5 inline-flex size-5 shrink-0 items-center justify-center rounded-full border border-[var(--fl-color-border)] bg-[var(--fl-color-surface)] outline-none transition-[border-color,box-shadow] duration-[var(--fl-duration-base)] ease-[var(--fl-ease-standard)] data-[state=checked]:border-[var(--fl-color-primary)] disabled:cursor-not-allowed disabled:opacity-55"
        :value="option.value"
        :disabled="option.disabled"
      >
        <RadioGroupIndicator class="flex items-center justify-center">
          <span class="size-2.5 rounded-full bg-[var(--fl-color-primary)]" />
        </RadioGroupIndicator>
      </RadioGroupItem>
      <span class="grid gap-1 leading-5">
        <span class="font-550">{{ option.label }}</span>
        <span v-if="option.description" class="text-sm text-[var(--fl-color-text-subtle)]">
          {{ option.description }}
        </span>
      </span>
    </label>
  </RadioGroupRoot>
</template>
