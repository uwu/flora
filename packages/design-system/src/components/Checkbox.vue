<script setup lang="ts">
import { CheckboxIndicator, CheckboxRoot } from 'reka-ui'

defineOptions({ name: 'FlCheckbox' })

const model = defineModel<boolean | 'indeterminate'>({ default: false })

withDefaults(
  defineProps<{
    label?: string
    disabled?: boolean
    invalid?: boolean
  }>(),
  {
    label: undefined,
    disabled: false,
    invalid: false
  }
)
</script>

<template>
  <label
    data-slot="checkbox-field"
    class="inline-flex min-h-10 items-center gap-2 text-sm text-[var(--fl-color-text)]"
  >
    <CheckboxRoot
      v-model="model"
      data-slot="checkbox"
      :disabled="disabled"
      :aria-invalid="invalid || undefined"
      class="fl-focus-ring fl-hit-area inline-flex size-5 shrink-0 items-center justify-center rounded-[var(--fl-radius-sm)] border border-[var(--fl-color-border)] bg-[var(--fl-color-surface)] text-white outline-none transition-[background-color,border-color,box-shadow] duration-[var(--fl-duration-base)] ease-[var(--fl-ease-standard)] data-[state=checked]:border-[var(--fl-color-primary)] data-[state=checked]:bg-[var(--fl-color-primary)] data-[state=indeterminate]:border-[var(--fl-color-primary)] data-[state=indeterminate]:bg-[var(--fl-color-primary)] disabled:cursor-not-allowed disabled:opacity-55 aria-invalid:border-[var(--fl-color-danger)]"
    >
      <CheckboxIndicator class="flex items-center justify-center">
        <span
          :class="[model === 'indeterminate' ? 'i-lucide-minus' : 'i-lucide-check', 'size-3.5']"
          aria-hidden="true"
        />
      </CheckboxIndicator>
    </CheckboxRoot>
    <span v-if="label || $slots.default" class="leading-5">
      <slot>{{ label }}</slot>
    </span>
  </label>
</template>
