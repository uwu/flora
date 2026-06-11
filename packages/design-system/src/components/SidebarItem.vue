<script setup lang="ts">
import { Primitive, type PrimitiveProps } from 'reka-ui'

defineOptions({ name: 'FlSidebarItem' })

withDefaults(
  defineProps<
    PrimitiveProps & {
      active?: boolean
    }
  >(),
  {
    as: 'a',
    asChild: false,
    active: false
  }
)
</script>

<template>
  <Primitive
    data-slot="sidebar-item"
    :as="as"
    :as-child="asChild"
    :aria-current="active ? 'page' : undefined"
    :class="[
      'fl-focus-ring flex min-h-9 items-center gap-2 rounded-[var(--fl-radius-md)] px-2 text-sm font-600 text-[var(--fl-color-text-muted)] no-underline outline-none transition-[background-color,color] duration-[var(--fl-duration-fast)] ease-[var(--fl-ease-standard)] hover:bg-[var(--fl-color-bg-muted)] hover:text-[var(--fl-color-text)]',
      active && 'bg-[var(--fl-color-primary-soft)] text-[var(--fl-color-primary-soft-text)]'
    ]"
  >
    <slot />
  </Primitive>
</template>
