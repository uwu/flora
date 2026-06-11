<script setup lang="ts">
import { useMediaQuery, useStorage } from '@vueuse/core'
import { computed } from 'vue'
import IconButton from './IconButton.vue'

defineOptions({ name: 'FlSidebar' })

const props = withDefaults(
  defineProps<{
    title?: string
    storageKey?: string
    defaultOpen?: boolean
    collapsible?: boolean
  }>(),
  {
    title: undefined,
    storageKey: 'flora-sidebar-open',
    defaultOpen: true,
    collapsible: true
  }
)

const emit = defineEmits<{
  toggle: [open: boolean]
}>()

const openModel = defineModel<boolean>('open')
const storedOpen = useStorage(props.storageKey, props.defaultOpen)
const isMobile = useMediaQuery('(max-width: 767px)')

const open = computed({
  get: () => openModel.value ?? storedOpen.value,
  set: (value) => {
    storedOpen.value = value
    openModel.value = value
    emit('toggle', value)
  }
})

function toggle() {
  open.value = !open.value
}
</script>

<template>
  <aside
    data-slot="sidebar"
    :data-state="open ? 'expanded' : 'collapsed'"
    :data-mobile="isMobile || undefined"
    :class="[
      'sticky top-0 hidden h-dvh shrink-0 border-r border-[var(--fl-color-border)] bg-[var(--fl-color-surface)] text-[var(--fl-color-text)] md:flex md:flex-col',
      open ? 'w-64' : 'w-14'
    ]"
  >
    <header class="flex h-14 items-center gap-2 border-b border-[var(--fl-color-border)] px-3">
      <slot name="brand" :open="open">
        <div v-if="open && title" class="truncate text-sm font-750">{{ title }}</div>
      </slot>
      <IconButton
        v-if="collapsible"
        class="ml-auto"
        label="Toggle sidebar"
        size="icon-sm"
        variant="ghost"
        @click="toggle"
      >
        <span class="i-lucide-panel-left size-4" aria-hidden="true" />
      </IconButton>
    </header>
    <nav class="min-h-0 flex-1 overflow-auto p-2">
      <slot :open="open" :toggle="toggle" />
    </nav>
    <footer v-if="$slots.footer" class="border-t border-[var(--fl-color-border)] p-2">
      <slot name="footer" :open="open" />
    </footer>
  </aside>
</template>
