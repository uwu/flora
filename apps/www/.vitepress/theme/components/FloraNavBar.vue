<script setup lang="ts">
import logoCursiveSvg from '@uwu/flora-branding/logo-cursive.svg'
import logoSvg from '@uwu/flora-branding/logo.svg'
// @ts-ignore
import VPNavBarSearch from 'vitepress/dist/client/theme-default/components/VPNavBarSearch.vue'

withDefaults(
  defineProps<{
    isScreenOpen?: boolean
  }>(),
  {
    isScreenOpen: false
  }
)

defineEmits<{
  (event: 'toggle-screen'): void
}>()
</script>

<template>
  <nav class="nav" :class="{ 'screen-open': isScreenOpen }">
    <div aria-hidden="true" class="grid-line-v nav-grid-line nav-grid-line-left" />
    <div aria-hidden="true" class="grid-line-v nav-grid-line nav-grid-line-right" />
    <div aria-hidden="true" class="grid-dot nav-grid-dot nav-grid-dot-left" />
    <div aria-hidden="true" class="grid-dot nav-grid-dot nav-grid-dot-right" />

    <div class="nav-inner">
      <a href="/" class="nav-brand" aria-label="flora home">
        <img :src="logoSvg" alt="" class="nav-logo-mark" />
        <img :src="logoCursiveSvg" alt="" class="nav-logo-wordmark" />
      </a>
      <div class="nav-search">
        <VPNavBarSearch />
      </div>
      <div class="nav-links">
        <div class="nav-menu-links">
          <a href="/docs/sdk/overview">SDK</a>
          <a href="/docs/runtime">Runtime</a>
          <a href="/docs/cli">CLI</a>
          <a href="/docs/examples">Examples</a>
        </div>
        <span class="nav-separator" aria-hidden="true" />
        <div class="nav-social-links">
          <a href="https://github.com/uwu/flora" class="nav-social" aria-label="GitHub">
            <span class="nav-social-icon i-simple-icons-github" aria-hidden="true" />
          </a>
          <a href="https://discord.gg/dRGTU7n4dC" class="nav-social" aria-label="Discord">
            <span class="nav-social-icon i-simple-icons-discord" aria-hidden="true" />
          </a>
        </div>
      </div>
    </div>
  </nav>
</template>

<style scoped>
.nav {
  --nav-fallback-grid-max-width: var(--flora-grid-max-width, min(calc(100vw - 64px), 1376px));
  --nav-grid-max-width: var(--grid-max-width, var(--nav-fallback-grid-max-width));
  --nav-grid-line-offset: var(--grid-line-offset, var(--flora-grid-line-offset, 16px));
  --nav-page-max: var(--page-max, var(--flora-page-max, min(calc(100vw - 40px), 1040px)));
  --nav-edge-max: calc(var(--nav-grid-max-width) + var(--nav-grid-line-offset) * 2);
  --nav-padding-left: 32px;
  --nav-padding-right: 32px;

  position: sticky;
  top: 0;
  left: 0;
  right: 0;
  z-index: 100;
  height: var(--vp-nav-height, 56px);
  background: var(--gray1);
  border-bottom: 1px solid var(--gray5);
  pointer-events: none;
}

.grid-line-v {
  display: none;
  position: absolute;
  top: 0;
  bottom: 0;
  z-index: 10;
  width: 0;
  border-left: 1px solid var(--gray5);
  pointer-events: none;
}

.grid-dot {
  display: none;
  position: absolute;
  z-index: 20;
  width: 20px;
  height: 20px;
  align-items: center;
  justify-content: center;
  background: var(--gray1);
  border-radius: 50%;
  pointer-events: none;
}

.grid-dot::after {
  content: '';
  display: block;
  width: 2px;
  height: 2px;
  background: var(--gray9);
  border-radius: 50%;
}

.nav-grid-line-left {
  left: calc(50% - var(--nav-grid-max-width) / 2 - var(--nav-grid-line-offset));
}

.nav-grid-line-right {
  right: calc(50% - var(--nav-grid-max-width) / 2 - var(--nav-grid-line-offset));
}

.nav-grid-dot-left {
  bottom: 0;
  left: calc(50% - var(--nav-grid-max-width) / 2 - var(--nav-grid-line-offset));
  transform: translate(-50%, 50%);
}

.nav-grid-dot-right {
  right: calc(50% - var(--nav-grid-max-width) / 2 - var(--nav-grid-line-offset));
  bottom: 0;
  transform: translate(50%, 50%);
}

.nav-inner {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
  gap: 18px;
  align-items: center;
  height: var(--vp-nav-height, 56px);
  max-width: var(--nav-edge-max);
  margin: 0 auto;
  padding-left: var(--nav-padding-left);
  padding-right: var(--nav-padding-right);
  pointer-events: auto;
}

.nav-brand {
  --uno: 'flex items-center gap-2';
  justify-self: start;
}

.nav-logo-mark {
  width: 28px;
  height: 30px;
  display: block;
}

.nav-logo-wordmark {
  width: 68px;
  height: auto;
  display: block;
}

.dark .nav-logo-wordmark {
  filter: invert(1);
}

.nav-links {
  --uno: 'flex items-center';
  justify-self: end;
  gap: 18px;
}

.nav-search {
  --uno: 'flex items-center justify-center';
  justify-self: center;
  min-width: 0;
}

.nav-search :deep(.VPNavBarSearch) {
  flex: none;
  padding-left: 0;
}

.nav-search :deep(.VPNavBarSearchButton) {
  width: clamp(220px, 22vw, 264px);
  min-width: 220px;
  height: 32px;
  justify-content: flex-start;
  gap: 7px;
  border: 0;
  border-radius: 6px;
  padding: 0 7px 0 10px;
  background: var(--gray2);
  color: var(--gray11);
  box-shadow:
    0 0 0 1px var(--gray6),
    0 1px 2px -1px rgb(0 0 0 / 10%);
  font-size: 13px;
  line-height: 1;
  transition-property: background-color, box-shadow, color, scale;
  transition-duration: 150ms;
  transition-timing-function: ease;
}

.nav-search :deep(.VPNavBarSearchButton:hover) {
  background: var(--gray1);
  color: var(--gray12);
  box-shadow:
    0 0 0 1px var(--gray7),
    0 1px 2px -1px rgb(0 0 0 / 12%),
    0 2px 5px -3px rgb(0 0 0 / 12%);
}

.nav-search :deep(.VPNavBarSearchButton:active) {
  scale: 0.98;
}

.nav-search :deep(.VPNavBarSearchButton .vpi-search) {
  flex: none;
  width: 14px;
  height: 14px;
  color: var(--gray10);
}

.nav-search :deep(.VPNavBarSearchButton .text) {
  display: inline;
  flex: 1 1 auto;
  color: var(--gray10);
  font-size: 13px;
  text-align: left;
}

.nav-search :deep(.VPNavBarSearchButton .keys) {
  display: flex;
  align-items: center;
  gap: 3px;
  margin-left: auto;
  border: 0;
  padding: 0;
  background: transparent;
  color: var(--gray10);
  font-size: 10px;
  letter-spacing: 0;
}

.nav-search :deep(.VPNavBarSearchButton kbd) {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 18px;
  height: 18px;
  border: 1px solid var(--gray6);
  border-radius: 4px;
  padding: 0 4px;
  background: var(--gray1);
  font-family: inherit;
  font-weight: 500;
  line-height: 1;
  box-shadow: inset 0 -1px 0 var(--gray4);
}

.nav-menu-links {
  --uno: 'flex items-center gap-6';
}

.nav-links a {
  --uno: 'text-sm no-underline';
  color: var(--gray11);
  transition: color 150ms ease;
}

.nav-links a:hover {
  color: var(--gray12);
}

.nav-separator {
  width: 1px;
  height: 20px;
  background: var(--gray6);
}

.nav-social-links {
  --uno: 'flex items-center';
  gap: 0;
}

.nav-social {
  --uno: 'flex items-center justify-center';
  width: 36px;
  height: 36px;
  color: var(--gray11);
}

.nav-social-icon {
  display: block;
  flex: none;
  width: 18px;
  height: 18px;
}

@media (min-width: 1024px) {
  .grid-line-v {
    display: block;
  }

  .grid-dot {
    display: flex;
  }
}

@media (max-width: 1100px) {
  .nav-search :deep(.VPNavBarSearchButton) {
    width: 40px;
    min-width: 40px;
    height: 36px;
    justify-content: center;
    padding: 0;
  }

  .nav-search :deep(.VPNavBarSearchButton .text),
  .nav-search :deep(.VPNavBarSearchButton .keys) {
    display: none;
  }
}

@media (max-width: 760px) {
  .nav {
    --nav-padding-left: 20px;
    --nav-padding-right: 20px;
  }

  .nav-inner {
    height: var(--vp-nav-height, 56px);
  }

  .nav-menu-links a:nth-child(2),
  .nav-menu-links a:nth-child(3),
  .nav-menu-links a:nth-child(4) {
    display: none;
  }
}

@media (max-width: 600px) {
  .nav {
    --nav-fallback-grid-max-width: 100vw;
  }

  .nav-logo-wordmark {
    width: 62px;
  }
}
</style>
