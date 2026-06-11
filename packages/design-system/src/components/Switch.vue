<script setup lang="ts">
import { SwitchRoot, SwitchThumb } from 'reka-ui'

defineOptions({ name: 'FlSwitch' })

const model = defineModel<boolean>({ default: false })

withDefaults(
  defineProps<{
    label?: string
    disabled?: boolean
  }>(),
  {
    label: undefined,
    disabled: false
  }
)
</script>

<template>
  <label
    data-slot="switch-field"
    class="inline-flex min-h-10 items-center gap-2 text-sm text-[var(--fl-color-text)]"
  >
    <SwitchRoot
      v-model="model"
      data-slot="switch"
      :disabled="disabled"
      class="fl-focus-ring relative inline-flex h-6 w-10 shrink-0 cursor-pointer items-center rounded-[var(--fl-radius-pill)] bg-[var(--gray6)] outline-none transition-[background-color,box-shadow] duration-[var(--fl-duration-base)] ease-[var(--fl-ease-standard)] data-[state=checked]:bg-[var(--fl-color-primary)] disabled:cursor-not-allowed disabled:opacity-55"
    >
      <SwitchThumb
        class="block size-5 translate-x-0.5 rounded-full bg-white shadow-[0_1px_2px_rgb(0_0_0_/_0.16)] transition-transform duration-[var(--fl-duration-base)] ease-[var(--fl-ease-standard)] data-[state=checked]:translate-x-[18px]"
      />
    </SwitchRoot>
    <span v-if="label || $slots.default" class="leading-5">
      <slot>{{ label }}</slot>
    </span>
  </label>
</template>
