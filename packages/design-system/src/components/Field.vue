<script lang="ts">
let fieldSequence = 0
</script>

<script setup lang="ts">
import { computed } from 'vue'

defineOptions({ name: 'FlField' })

const props = withDefaults(
  defineProps<{
    id?: string
    label?: string
    description?: string
    error?: string
    required?: boolean
  }>(),
  {
    label: undefined,
    description: undefined,
    error: undefined,
    required: false
  }
)

const fallbackId = `fl-field-${++fieldSequence}`
const controlId = computed(() => props.id ?? fallbackId)
const descriptionId = computed(() => `${controlId.value}-description`)
const errorId = computed(() => `${controlId.value}-error`)
const describedBy = computed(() => {
  const ids = []
  if (props.description) ids.push(descriptionId.value)
  if (props.error) ids.push(errorId.value)
  return ids.length ? ids.join(' ') : undefined
})
</script>

<template>
  <div data-slot="field" class="grid gap-2">
    <label
      v-if="label"
      data-slot="field-label"
      :for="controlId"
      class="inline-flex items-center gap-1 text-sm font-650 leading-none text-[var(--fl-color-text)]"
    >
      {{ label }}
      <span v-if="required" aria-hidden="true" class="text-[var(--fl-color-danger)]">*</span>
    </label>
    <slot
      :id="controlId"
      :invalid="Boolean(error)"
      :describedBy="describedBy"
      :aria-describedby="describedBy"
      :aria-invalid="Boolean(error) || undefined"
    />
    <p
      v-if="description"
      :id="descriptionId"
      data-slot="field-description"
      class="m-0 text-sm leading-5 text-[var(--fl-color-text-subtle)]"
    >
      {{ description }}
    </p>
    <p
      v-if="error"
      :id="errorId"
      data-slot="field-error"
      class="m-0 text-sm font-600 leading-5 text-[var(--fl-color-danger-soft-text)]"
    >
      {{ error }}
    </p>
  </div>
</template>
