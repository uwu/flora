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
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  background-color: #a4b793;
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
}

.brand-name {
  position: absolute;
  left: 163px;
  top: 68px;
  color: #e1ecd9;
  font-size: 52px;
  line-height: 58px;
}

.project-tagline {
  position: absolute;
  right: 40px;
  top: 68px;
  color: #e1ecd9;
  font-size: 34px;
  line-height: 52px;
}

.description {
  max-width: 980px;
  margin-top: 32px;
  padding: 0 54px;
  color: #e1ecd9;
  font-size: 20px;
  line-height: 32px;
  letter-spacing: -0.005em;
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
}

@media (max-width: 900px) {
  .header {
    height: 140px;
  }

  .logo {
    width: 96px;
    height: 96px;
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
