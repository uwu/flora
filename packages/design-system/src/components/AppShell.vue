<script setup lang="ts">
defineOptions({ name: 'FlAppShell' })
</script>

<template>
  <div
    data-slot="app-shell"
    class="fl-root grid min-h-dvh grid-cols-[auto_minmax(0,1fr)] bg-[var(--fl-color-bg)] text-[var(--fl-color-text)]"
  >
    <slot name="sidebar" />
    <main data-slot="app-shell-main" class="min-w-0 overflow-auto">
      <slot />
    </main>
  </div>
</template>
