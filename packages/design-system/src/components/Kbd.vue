<script setup lang="ts">
defineOptions({ name: 'FlKbd' })
</script>

<template>
  <kbd
    data-slot="kbd"
    class="fl-tabular-nums inline-flex min-w-5 items-center justify-center rounded-[var(--fl-radius-sm)] bg-[var(--fl-color-bg-muted)] px-1.5 py-0.5 font-mono text-[11px] font-650 leading-none text-[var(--fl-color-text-muted)] shadow-[inset_0_0_0_1px_var(--fl-color-border)]"
  >
    <slot />
  </kbd>
</template>
