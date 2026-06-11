<script setup lang="ts">
defineOptions({ name: 'FlTable' })
</script>

<template>
  <div
    data-slot="table-wrap"
    class="w-full overflow-auto rounded-[var(--fl-radius-lg)] bg-[var(--fl-color-surface)] shadow-[var(--fl-shadow-border)]"
  >
    <table data-slot="table" class="w-full border-collapse text-sm">
      <slot />
    </table>
  </div>
</template>
