<template>
  <section class="hero">
    <p class="hero-description">
      <span class="hero-description-brand">flora</span> is a fast runtime that lets you write
      Discord bots for your servers with a rich TypeScript SDK, without worrying about running
      infrastructure.
    </p>

    <div class="hero-actions">
      <a
        href="https://app.flora.uwu.network"
        target="_blank"
        rel="noopener noreferrer"
        class="btn-common btn-primary"
      >
        <span class="btn-icon i-lucide-layout-dashboard" aria-hidden="true" />
        Dashboard
      </a>
      <a href="/docs/sdk" class="btn-common btn-secondary">
        <span class="btn-icon i-lucide-book-open" aria-hidden="true" />
        Documentation
      </a>
      <a href="https://github.com/uwu/flora" class="btn-common btn-secondary">
        <span class="btn-icon i-lucide-github" aria-hidden="true" />
        View on GitHub
      </a>
    </div>
  </section>
</template>

<style scoped>
.hero {
  --uno: 'flex flex-col items-start text-left';
  width: 100%;
  padding: 56px 16px 40px;
}

.hero-description {
  --uno: 'm-0 mb-8';
  color: var(--gray11);
  max-width: 690px;
  font-size: clamp(17px, 2vw, 22px);
  line-height: 1.45;
  letter-spacing: -0.01em;
}

.hero-description-brand {
  color: var(--gray12);
  font-weight: 600;
}

.hero-actions {
  --uno: 'flex gap-3 items-center flex-wrap';
}

.btn-common {
  --uno: 'inline-flex items-center justify-center gap-1.5 whitespace-nowrap px-2 py-2 text-sm font-500 no-underline select-none';
  min-height: 34px;
  line-height: 1.25;
  border: 1px solid transparent;
  border-radius: 4px;
  transition:
    background-color 150ms ease,
    color 150ms ease,
    border-color 150ms ease;
}

.btn-icon {
  width: 16px;
  height: 16px;
  flex: none;
}

.btn-primary {
  --uno: 'bg-primary-9 text-white border-primary-9';
}

.btn-primary:hover {
  --uno: 'bg-primary-10 border-primary-10';
}

.btn-secondary {
  background: var(--gray1);
  color: var(--gray12);
  border-color: var(--gray6);
}

.btn-secondary:hover {
  background: var(--gray3);
  border-color: var(--gray8);
}

@media (max-width: 600px) {
  .hero {
    padding: 40px 20px 32px;
  }

  .hero-actions {
    --uno: 'flex-col';
    width: min(100%, 360px);
  }

  .hero-actions a {
    width: 100%;
    justify-content: center;
  }
}
</style>
