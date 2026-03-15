<script setup lang="ts">
import FeaturesSection from './IndexPage/FeaturesSection.vue'
import HeroSection from './IndexPage/HeroSection.vue'
import NavBar from './IndexPage/NavBar.vue'
import PurposeSection from './IndexPage/PurposeSection.vue'
</script>

<template>
  <div class="page-root font-rethink">
    <div class="top-hero">
      <img src="/bg.png" alt="" class="hero-bg-image" />
      <div class="hero-vignette" />
      <NavBar />
      <HeroSection />
    </div>
    <PurposeSection />
    <FeaturesSection />
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
</template>

<style>
.page-root {
  --background: #1d2021;
  --foreground: #ebdbb2;
  --muted-foreground: #a89984;
  --accent: #fabd2f;
  --border: #504945;
  --primary: #fabd2f;
  --primary-foreground: #282828;
  --accent-foreground: #282828;
  --gb-bg: var(--background);
  --gb-bg-soft: color-mix(in srgb, var(--background) 86%, white);
  --gb-fg: var(--foreground);
  --gb-fg-soft: color-mix(in srgb, var(--foreground) 82%, var(--background));
  --gb-muted: var(--muted-foreground);
  --gb-border: var(--border);
  --gb-accent: var(--primary);
  --gb-accent-strong: color-mix(in srgb, var(--primary) 88%, black);
  background-color: var(--gb-bg);
  color: var(--gb-fg);
  min-height: 100vh;
  overflow-x: hidden;
}

.page-root a {
  color: inherit;
  text-decoration: none;
}

.top-hero {
  position: relative;
  isolation: isolate;
  overflow: clip;
}

.hero-bg-image {
  position: absolute;
  inset: 0;
  z-index: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  object-position: center 28%;
  pointer-events: none;
  mix-blend-mode: difference;
  opacity: 0;
  animation: hero-bg-in 1.5s ease-out forwards;
  mask-image: linear-gradient(to bottom, #000 78%, transparent 100%);
}

.hero-vignette {
  position: absolute;
  inset: 0;
  z-index: 1;
  pointer-events: none;
  background:
    radial-gradient(130% 80% at 50% 10%, rgb(0 0 0 / 0%) 38%, rgb(0 0 0 / 52%) 100%),
    linear-gradient(to bottom, rgb(29 32 33 / 24%) 0%, rgb(29 32 33 / 70%) 74%, var(--gb-bg) 100%);
}

@keyframes hero-bg-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 0.6;
  }
}

.page-footer {
  --uno: 'px-6 py-7';
  background: #0e0f10;
}

.page-footer-inner {
  --uno: 'max-w-5xl mx-auto flex items-center justify-between gap-4';
}

.page-footer-copy {
  --uno: 'm-0 text-sm';
  color: var(--gb-fg-soft);
}

.page-footer a {
  color: inherit;
  text-decoration: none;
}

.page-footer a:hover {
  color: var(--gb-fg);
}

.page-footer-copy a:hover {
  text-decoration: underline;
  text-underline-offset: 2px;
}

.page-footer-sep {
  --uno: 'mx-2';
  color: var(--gb-muted);
}

@media (max-width: 600px) {
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
