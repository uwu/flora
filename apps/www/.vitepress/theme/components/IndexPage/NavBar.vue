<script setup>
import logoSvg from '@uwu/flora-branding/logo.svg'
</script>

<template>
  <nav class="nav">
    <div class="nav-inner">
      <div class="nav-left">
        <img :src="logoSvg" alt="flora" class="nav-logo" />
        <span class="nav-brand">flora</span>
      </div>
      <div class="nav-links">
        <a href="/docs/sdk">SDK</a>
        <a href="/docs/runtime">Runtime</a>
        <a href="/docs/cli">CLI</a>
        <a href="/docs/examples">Examples</a>
        <a href="https://github.com/uwu/flora" class="nav-gh">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path
              d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"
            />
          </svg>
        </a>
      </div>
    </div>
  </nav>
</template>

<style scoped>
.nav {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  z-index: 100;
  transition:
    opacity 220ms ease,
    transform 220ms ease;
}

.nav-inner {
  --uno: 'max-w-5xl mx-auto px-8 h-14 flex items-center justify-between';
}

.nav-left {
  --uno: 'flex items-center gap-2.5';
}

.nav-logo {
  --uno: 'size-8';
}

.nav-brand {
  --uno: 'text-lg font-600 tracking-tight';
  color: var(--gb-fg);
}

.nav-links {
  --uno: 'flex items-center gap-6';
}

.nav-links a {
  --uno: 'text-sm no-underline';
  color: var(--gb-muted);
  transition: color 150ms ease;
}

.nav-links a:hover {
  color: var(--gb-fg);
}

.nav-gh {
  --uno: 'flex items-center';
}

@media (max-width: 760px) {
  .nav-inner {
    --uno: 'px-4 h-13';
  }

  .nav-links a:not(.nav-gh) {
    display: none;
  }
}

@media (max-width: 600px) {
  .nav-left {
    --uno: 'gap-2';
  }

  .nav-logo {
    --uno: 'size-7';
  }

  .nav-brand {
    --uno: 'text-base';
  }
}
</style>
