<script setup lang="ts">
defineProps<{
  section: 'deployments' | 'cli' | 'files' | 'checks'
}>()
</script>

<template>
  <svg
    v-if="section === 'deployments'"
    fill="none"
    viewBox="0 0 297 635"
    class="w-full h-auto showcase-svg"
  >
    <use href="#showcase-deployments" x="0" y="-50" />
  </svg>

  <svg
    v-else-if="section === 'cli'"
    fill="none"
    viewBox="0 0 297 120"
    class="w-full h-auto showcase-svg"
  >
    <use href="#showcase-cli" x="0" y="10" />
  </svg>

  <svg
    v-else-if="section === 'files'"
    fill="none"
    viewBox="0 0 301 635"
    class="w-full h-auto showcase-svg"
  >
    <use href="#showcase-files" x="0" y="-50" />
  </svg>

  <svg
    v-else
    fill="none"
    viewBox="0 0 301 157"
    class="w-full h-auto showcase-svg"
  >
    <use href="#showcase-checks" x="0" y="-13" />
  </svg>
</template>
