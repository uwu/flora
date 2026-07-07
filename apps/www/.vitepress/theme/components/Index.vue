<script setup lang="ts">
import { GridLines } from '@flora-internal/design-system'
import FeaturesSection from './IndexPage/FeaturesSection.vue'
import HeroSection from './IndexPage/HeroSection.vue'
import NavBar from './IndexPage/NavBar.vue'
</script>

<template>
  <div class="page-root bg-gray-1 text-gray-12">
    <NavBar />
    <div class="page-frame">
      <GridLines mode="lines" />
      <GridLines mode="lines-with-dots" variant="tabbar-dots" />

      <main>
        <HeroSection />
        <FeaturesSection />
      </main>
      <footer class="page-footer">
        <div class="page-footer-inner">
          <p class="page-footer-copy">
            <a href="https://uwu.network">a uwu.network project</a>
            <span class="page-footer-sep">|</span>
            <a href="https://uwu.network/~tasky">made by taskylizard</a>
          </p>
        </div>
      </footer>
    </div>
  </div>
</template>

<style>
.page-root {
  --grid-max-width: var(--flora-grid-max-width, min(calc(100vw - 64px), 1376px));
  --grid-line-offset: var(--flora-grid-line-offset, 16px);
  --grid-line-color: var(--gray5);
  --grid-dot-color: var(--gray9);
  --grid-dot-fill: var(--gray1);
  --page-max: var(--flora-page-max, min(calc(100vw - 40px), 1040px));
  color-scheme: light dark;
  min-height: 100vh;
  overflow-x: hidden;
}

.page-root::selection {
  background: var(--irisA5);
  color: var(--gray12);
}

.page-root a {
  color: inherit;
  text-decoration: none;
}

.page-frame {
  position: relative;
  width: 100%;
  max-width: var(--grid-max-width);
  margin: 0 auto;
  overflow-y: clip;
}

.page-footer {
  --uno: 'px-6 py-8';
  background: var(--gray1);
  border-top: 1px solid var(--gray5);
}

.page-footer-inner {
  --uno: 'mx-auto flex items-center justify-between gap-4';
  max-width: var(--page-max);
}

.page-footer-copy {
  --uno: 'm-0 text-sm';
  color: var(--gray11);
}

.page-footer a {
  color: inherit;
  text-decoration: none;
}

.page-footer a:hover {
  color: var(--gray12);
}

.page-footer-copy a:hover {
  text-decoration: underline;
  text-underline-offset: 2px;
}

.page-footer-sep {
  --uno: 'mx-2';
  color: var(--gray8);
}

@media (max-width: 600px) {
  .page-root {
    --grid-max-width: 100vw;
  }

  .page-footer {
    --uno: 'px-5 py-6';
  }

  .page-footer-inner {
    --uno: 'flex-col items-start gap-3';
  }

  .page-footer-copy {
    --uno: 'text-xs';
    line-height: 1.6;
  }
}
</style>
