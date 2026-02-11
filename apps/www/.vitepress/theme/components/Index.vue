<template>
  <div class="page-root font-rethink">
    <div class="page-canvas">
      <header class="header">
        <div class="logo" />
        <div class="brand-name">flora</div>
        <div class="project-tagline">a uwu.network project</div>
      </header>

      <section class="description">
        flora is a fast runtime that lets you write discord bots for your servers, with a rich
        TypeScript SDK, without worrying about running infrastructure.
        <br /><br />
        You deploy scripts on a per-guild basis through a single bot, handled through the V8
        JavaScript engine using V8 Isolates.
        <br /><br />
        This is still taking shape, and will have a proper release soon. To stay up to date on
        flora's progress, check out its repository:
        <a class="underline" href="https://github.com/uwu/flora">
          https://github.com/uwu/flora
        </a>
        <br /><br />
        Start here:
        <a class="underline" href="/sdk">SDK docs</a>
        ·
        <a class="underline" href="/runtime">Runtime docs</a>
        ·
        <a class="underline" href="/cli">CLI docs</a>
        ·
        <a class="underline" href="/examples">Examples</a>
      </section>
    </div>

    <footer class="footer-bg" />
  </div>
</template>

<style>
.page-root {
  --page-bg: oklch(0.84 0.052 148);
  --page-text: oklch(0.24 0.025 154);
  --page-link: oklch(0.3 0.085 151);
  --page-link-hover: oklch(0.24 0.098 151);
  --page-accent-soft: oklch(0.96 0.028 148 / 0.54);
  --footer-opacity: 0.8;
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  background-color: var(--page-bg);
  color: var(--page-text);
}

html.dark .page-root {
  --page-bg: oklch(0.24 0.03 152);
  --page-text: oklch(0.92 0.018 150);
  --page-link: oklch(0.82 0.12 151);
  --page-link-hover: oklch(0.89 0.1 151);
  --page-accent-soft: oklch(0.31 0.04 151 / 0.5);
  --footer-opacity: 0.6;
}

@media (prefers-color-scheme: dark) {
  html:not(.light) .page-root {
    --page-bg: oklch(0.24 0.03 152);
    --page-text: oklch(0.92 0.018 150);
    --page-link: oklch(0.82 0.12 151);
    --page-link-hover: oklch(0.89 0.1 151);
    --page-accent-soft: oklch(0.31 0.04 151 / 0.5);
    --footer-opacity: 0.6;
  }
}

.page-canvas {
  max-width: 1440px;
  margin: 0 auto;
  width: 100%;
}

.header {
  position: relative;
  height: 202px;
}

.logo {
  position: absolute;
  top: 0;
  left: 0;
  width: 200px;
  height: 200px;
  background-image: url("/flora-logo.png");
  background-size: cover;
  background-position: center;
  border-radius: 20px;
}

.brand-name {
  position: absolute;
  left: 163px;
  top: 68px;
  color: var(--page-text);
  font-size: 52px;
  line-height: 58px;
}

.project-tagline {
  position: absolute;
  right: 40px;
  top: 68px;
  color: var(--page-text);
  font-size: 34px;
  line-height: 52px;
}

.description {
  max-width: 980px;
  margin-top: 32px;
  padding: 0 54px;
  color: var(--page-text);
  font-size: 20px;
  line-height: 32px;
  letter-spacing: -0.005em;
}

.description .underline {
  color: var(--page-link);
  text-decoration-color: var(--page-link);
  text-decoration-thickness: 1.5px;
  text-underline-offset: 0.14em;
  border-radius: 3px;
  background: linear-gradient(var(--page-accent-soft), var(--page-accent-soft)) 0 100% / 100% 0
    no-repeat;
  transition: color 160ms ease, background-size 160ms ease, text-decoration-color 160ms ease;
}

.description .underline:hover {
  color: var(--page-link-hover);
  text-decoration-color: var(--page-link-hover);
  background-size: 100% 100%;
}

.description .underline:focus-visible {
  outline: 2px solid var(--page-link);
  outline-offset: 3px;
}

.footer-bg {
  margin-top: auto;
  width: 100%;
  height: 420px;
  background-image: url("/flora-footer.gif");
  background-size: cover;
  background-position: bottom center;
  background-repeat: no-repeat;
  pointer-events: none;
  opacity: var(--footer-opacity);
}

@media (max-width: 900px) {
  .header {
    height: 140px;
  }

  .logo {
    width: 96px;
    height: 96px;
    border-radius: 12px;
  }

  .brand-name {
    left: 96px;
    top: 32px;
    font-size: 36px;
  }

  .project-tagline {
    left: 96px;
    top: 74px;
    right: auto;
    font-size: 18px;
  }

  .description {
    padding: 0 20px;
    font-size: 16px;
    line-height: 26px;
  }

  .footer-bg {
    height: 360px;
  }
}

@media (max-width: 520px) {
  .description {
    padding: 0 16px;
    font-size: 15px;
    line-height: 24px;
  }

  .footer-bg {
    height: 320px;
  }
}
</style>
