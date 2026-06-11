<script setup lang="ts">
import {
  SelectContent,
  SelectIcon,
  SelectItem,
  SelectItemIndicator,
  SelectItemText,
  SelectPortal,
  SelectRoot,
  SelectTrigger,
  SelectValue,
  SelectViewport,
  type AcceptableValue
} from 'reka-ui'
import type { SelectOption } from '../types'

defineOptions({ name: 'FlSelect' })

const model = defineModel<AcceptableValue>()

withDefaults(
  defineProps<{
    options: SelectOption[]
    placeholder?: string
    disabled?: boolean
  }>(),
  {
    placeholder: 'Select an option',
    disabled: false
  }
)
</script>

<template>
  <SelectRoot v-model="model" :disabled="disabled">
    <SelectTrigger
      data-slot="select-trigger"
      class="fl-focus-ring inline-flex h-9 min-w-44 items-center justify-between gap-2 rounded-[var(--fl-radius-md)] border border-[var(--fl-color-border)] bg-[var(--fl-color-surface)] px-3 text-sm text-[var(--fl-color-text)] outline-none transition-[background-color,border-color,box-shadow] duration-[var(--fl-duration-base)] ease-[var(--fl-ease-standard)] hover:border-[var(--fl-color-border-strong)] disabled:cursor-not-allowed disabled:opacity-55"
    >
      <SelectValue class="truncate" :placeholder="placeholder" />
      <SelectIcon
        class="i-lucide-chevron-down size-4 shrink-0 text-[var(--fl-color-text-subtle)]"
      />
    </SelectTrigger>
    <SelectPortal>
      <SelectContent
        data-slot="select-content"
        class="fl-popover-enter z-[var(--fl-z-dropdown)] max-h-80 min-w-44 overflow-hidden rounded-[var(--fl-radius-lg)] bg-[var(--fl-color-surface-overlay)] p-1 text-[var(--fl-color-text)] shadow-[var(--fl-shadow-overlay)]"
        position="popper"
        :side-offset="6"
      >
        <SelectViewport>
          <SelectItem
            v-for="option in options"
            :key="String(option.value)"
            data-slot="select-item"
            class="relative flex min-h-8 cursor-default select-none items-center gap-2 rounded-[var(--fl-radius-md)] py-1.5 pl-8 pr-3 text-sm outline-none transition-[background-color,color] duration-[var(--fl-duration-fast)] ease-[var(--fl-ease-standard)] data-[highlighted]:bg-[var(--fl-color-bg-muted)] data-[highlighted]:text-[var(--fl-color-text)] data-[disabled]:pointer-events-none data-[disabled]:opacity-50"
            :value="option.value"
            :disabled="option.disabled"
          >
            <span class="absolute left-2 inline-flex size-4 items-center justify-center">
              <SelectItemIndicator>
                <span class="i-lucide-check size-4" aria-hidden="true" />
              </SelectItemIndicator>
            </span>
            <SelectItemText>{{ option.label }}</SelectItemText>
          </SelectItem>
        </SelectViewport>
      </SelectContent>
    </SelectPortal>
  </SelectRoot>
</template>
