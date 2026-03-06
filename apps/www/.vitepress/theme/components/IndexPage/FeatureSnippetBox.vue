<script setup lang="ts">
import type { IndexFeature } from './types'

const props = defineProps<{
  feature: IndexFeature
}>()
</script>

<template>
  <div class="feature-codebox">
    <div class="terminal-body">
      <template v-if="props.feature.id === 'cli'">
        <pre>
<code><span class="t-prompt">λ</span> <span class="t-cmd">pnpm flora deploy --guild 847291053618249801</span>
<span class="t-info">ℹ</span> <span class="t-text">Uploading project...</span>
<span class="t-ok">✔</span> <span class="t-text">Upload complete (3 files, 1kb)</span>
<span class="t-info">ℹ</span> <span class="t-text">Building... (a3f1c82d-91e7-4b3a-b052-7e8d4a19f603)</span>
  <span class="t-dim">↳ Extracting project...</span>
  <span class="t-dim">↳ Extracted 3 files (2.0kb)</span>
  <span class="t-dim">↳ Validating package.json...</span>
  <span class="t-dim">↳ Bundling src/main.ts</span>
<span class="t-ok">✔</span> <span class="t-text">Deployed guild 847291053618249801</span></code></pre>
      </template>
      <template v-else>
        <pre><code v-html="props.feature.snippetHtml"></code></pre>
      </template>
    </div>
  </div>
</template>

<style scoped>
.feature-codebox {
  --uno: "absolute rounded-2.5 border overflow-hidden";
  border-color: var(--gb-border);
  top: 34px;
  right: 34px;
  z-index: 6;
  width: min(760px, 74vw);
  background: color-mix(in srgb, var(--background) 82%, #282828);
  box-shadow: 0 10px 28px oklch(0% 0 0 / 0.35), 0 2px 10px oklch(0% 0 0 / 0.22);
}

.terminal-body {
  --uno: "p-5 px-6 overflow-x-auto";
  min-height: 300px;
}

.terminal-body pre {
  --uno: "m-0 font-mono";
  color: var(--gb-fg-soft);
  font-size: 14px;
  line-height: 1.7;
  text-align: left;
}

.terminal-body code {
  font-family: inherit;
  white-space: pre;
}

:deep(.ts-key) {
  color: #fb4934;
}

:deep(.ts-var) {
  color: #ebdbb2;
}

:deep(.ts-fn) {
  color: #83a598;
}

:deep(.ts-prop) {
  color: #b8bb26;
}

:deep(.ts-str) {
  color: #b8bb26;
}

:deep(.ts-punc) {
  color: #d5c4a1;
}

:deep(.ts-op) {
  color: #fe8019;
}

:deep(.ts-cmt) {
  color: #928374;
}

.t-dim {
  color: color-mix(in srgb, var(--muted-foreground) 70%, var(--background));
}

.t-prompt {
  color: var(--gb-muted);
}

.t-cmd {
  color: var(--gb-fg);
}

.t-text {
  color: var(--gb-fg-soft);
}

.t-info {
  color: color-mix(in srgb, var(--foreground) 70%, var(--accent));
}

.t-ok {
  color: var(--accent);
}

@media (max-width: 900px) {
  .feature-codebox {
    top: 62px;
    right: 16px;
    left: 16px;
    width: auto;
  }

  .terminal-body {
    --uno: "p-4 px-5 overflow-x-auto";
    min-height: 240px;
  }
}

@media (max-width: 600px) {
  .terminal-body pre {
    font-size: 11px;
  }

  .feature-codebox {
    top: 56px;
    right: 10px;
    left: 10px;
  }
}
</style>
