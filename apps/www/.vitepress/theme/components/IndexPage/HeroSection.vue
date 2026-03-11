<template>
  <section class="hero">
    <p class="hero-description">
      flora is a fast runtime that lets you write Discord bots for your servers with a rich
      TypeScript SDK, without worrying about running infrastructure.
    </p>

    <div class="hero-actions">
      <a
        href="https://app.flora.uwu.network"
        target="_blank"
        rel="noopener noreferrer"
        class="btn-common btn-primary ring-offset-background focus-visible:ring-ring inline-flex items-center justify-center gap-1 whitespace-nowrap font-medium transition-colors focus-visible:outline-hidden focus-visible:ring-1 disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 select-none border border-transparent rounded-md px-3 py-1.5 text-lg [&_svg]:size-5 [&_svg]:stroke-[1.5px] font-sans w-fit rail-tick-start"
      >
        Dashboard
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <path d="M3 3h7v7H3zM14 3h7v5h-7zM14 10h7v11h-7zM3 14h7v7H3z" />
        </svg>
      </a>
      <a
        href="/docs/sdk"
        class="btn-common btn-secondary ring-offset-background focus-visible:ring-ring inline-flex items-center justify-center gap-1 whitespace-nowrap font-medium transition-colors focus-visible:outline-hidden focus-visible:ring-1 disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 select-none border rounded-md px-3 py-1.5 text-lg [&_svg]:size-5 [&_svg]:stroke-[1.5px] font-sans w-fit rail-tick-start"
      >
        Documentation
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          aria-hidden="true"
        >
          <path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H20v20H6.5a2.5 2.5 0 0 1 0-5H20" />
        </svg>
      </a>
      <a
        href="https://github.com/uwu/flora"
        class="btn-common btn-secondary ring-offset-background focus-visible:ring-ring inline-flex items-center justify-center gap-1 whitespace-nowrap font-medium transition-colors focus-visible:outline-hidden focus-visible:ring-1 disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 select-none border rounded-md px-3 py-1.5 text-lg [&_svg]:size-5 [&_svg]:stroke-[1.5px] font-sans w-fit rail-tick-start"
      >
        View on GitHub
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <path d="M7 17L17 7M17 7H7M17 7v10" />
        </svg>
      </a>
    </div>
  </section>
</template>

<style scoped>
.hero {
  --uno: "max-w-5xl mx-auto flex flex-col items-start text-left";
  position: relative;
  z-index: 2;
  padding: 140px 32px 80px;
}

.hero-description {
  --uno: "mb-8";
  color: var(--gb-fg-soft);
  max-width: 640px;
  font-size: 22px;
  line-height: 1.55;
  letter-spacing: -0.01em;
  margin-top: 0;
}

.hero-actions {
  --uno: "flex gap-3 items-center";
}

.btn-common {
  --uno: "no-underline";
  transform: translateY(0);
}

.btn-primary {
  color: var(--primary-foreground);
  background-color: #d3dbd2;
  transition: background-color 150ms ease, color 150ms ease, transform 100ms ease;
}

.btn-primary:hover {
  background-color: color-mix(in srgb, #d3dbd2 90%, white);
  color: var(--primary-foreground);
  transform: translateY(-1px);
}

.btn-secondary {
  background: color-mix(in srgb, var(--background) 62%, transparent);
  color: var(--gb-muted);
  border: 1px solid var(--gb-border);
  transition:
    background-color 150ms ease,
    color 150ms ease,
    border-color 150ms ease,
    transform 100ms ease;
}

.btn-secondary:hover {
  background: color-mix(in srgb, var(--background) 52%, #3c3836);
  color: var(--gb-fg);
  border-color: var(--gb-fg-soft);
  transform: translateY(-1px);
}

@media (max-width: 900px) {
  .hero-description {
    font-size: 18px;
  }
}

@media (max-width: 600px) {
  .hero {
    padding: 110px 20px 60px;
  }

  .hero-description {
    font-size: 16px;
  }

  .hero-actions {
    --uno: "flex-col";
    width: min(100%, 360px);
  }

  .hero-actions a {
    width: 100%;
    justify-content: center;
  }
}
</style>
