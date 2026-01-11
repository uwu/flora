<template>
  <div class="page-root font-rethink">
    <div class="page-canvas">
      <div class="header">
        <div class="logo" />

        <div class="brand-name">flora</div>

        <div class="project-tagline">a uwu.network project</div>
      </div>

      <div class="description">
        flora is a fast runtime that lets you write discord bots for your
        servers, with a rich TypeScript SDK, without worrying about running
        infrastructure.
        <br /><br />
        You deploy scripts on a per-guild basis through a single bot, handled
        through the V8 JavaScript engine using V8 Isolates.
        <br /><br />
      This is still taking shape, and will have a proper release soon. To stay
      up to date on flora's progress, check out its repository:
      <a class="underline" href="https://github.com/uwu/flora">
        https://github.com/uwu/flora
      </a>
      <br /><br />
      Start here:
      <a class="underline" href="/sdk">
        SDK docs
      </a>
      ·
      <a class="underline" href="/runtime">
        Runtime docs
      </a>
      ·
      <a class="underline" href="/cli">
        CLI docs
      </a>
      ·
      <a class="underline" href="/examples">
        Examples
      </a>
    </div>

      <div class="background-gif" />
    </div>
  </div>
</template>

<style>
.page-root {
  background-color: #a4b793 !important;
  box-sizing: border-box;
  min-height: 100vh;
  width: 100%;
  contain: content;
  font-synthesis: none;
  -moz-osx-font-smoothing: grayscale;
  -webkit-font-smoothing: antialiased;
  position: relative;
}

.page-canvas {
  position: relative;
  width: 100%;
  max-width: 1440px;
  height: 1024px;
  margin: 0 auto;
  transform: scale(var(--canvas-scale, 1));
  transform-origin: top center;
}

.header {
  box-sizing: border-box;
  contain: content;
  position: absolute;
  top: 0;
  left: 0;
  width: 1440px;
  height: 202px;
}

.logo {
  position: absolute;
  top: 0;
  left: 0;
  width: 200px;
  height: 200px;
  background-image: url("/flora-logo.png");
  background-position: center;
  background-size: cover;
  box-sizing: border-box;
}

.brand-name {
  position: absolute;
  top: 0;
  left: 0;
  translate: 163px 68px;
  color: #e1ecd9;
  font-size: 52px;
  line-height: 58px;
  letter-spacing: -0.01em;
  width: fit-content;
  height: fit-content;
  box-sizing: border-box;
}

.project-tagline {
  position: absolute;
  top: 0;
  left: 0;
  translate: 982px 68px;
  color: #e1ecd9;
  font-size: 34px;
  line-height: 52px;
  letter-spacing: -0.01em;
  width: fit-content;
  height: fit-content;
  box-sizing: border-box;
}

.description {
  position: absolute;
  top: 0;
  left: 0;
  translate: 54px 200px;
  width: 1323px;
  height: 586px;
  color: #e1ecd9;
  font-size: 34px;
  line-height: 52px;
  letter-spacing: -0.01em;
  font-variation-settings: "wght" 410;
  font-weight: 410;
  text-wrap: pretty;
  box-sizing: border-box;
}

.background-gif {
  position: absolute;
  top: 0;
  left: 0;
  translate: 0 382px;
  width: 1440px;
  height: 642px;
  background-image: url("/flora-footer.gif");
  background-position: center;
  background-size: cover;
  box-sizing: border-box;
}

@media (max-width: 1440px) {
  .header {
    width: 100%;
  }

  .project-tagline {
    left: auto;
    right: 40px;
    translate: 0 68px;
  }

  .description {
    width: calc(100% - 108px);
  }

  .background-gif {
    width: 100%;
  }
}

@media (min-width: 1200px) {
  .page-root {
    --canvas-scale: 0.92;
  }

  .background-gif {
    left: 50%;
    width: calc(100vw / var(--canvas-scale, 1));
    translate: -50% 470px;
  }
}

@media (max-width: 900px) {
  .page-canvas {
    height: 900px;
  }

  .header {
    width: 100%;
    height: 140px;
  }

  .logo {
    width: 96px;
    height: 96px;
  }

  .brand-name {
    translate: 96px 32px;
    font-size: 36px;
    line-height: 40px;
  }

  .project-tagline {
    translate: 96px 74px;
    font-size: 18px;
    line-height: 24px;
    left: 0;
    right: auto;
    max-width: calc(100% - 120px);
  }

  .description {
    translate: 20px 150px;
    width: calc(100% - 40px);
    height: auto;
    font-size: 18px;
    line-height: 28px;
  }

  .background-gif {
    translate: 0 520px;
    width: 100%;
    height: 360px;
    background-position: bottom center;
  }
}

@media (max-width: 520px) {
  .page-canvas {
    height: 820px;
  }

  .brand-name {
    translate: 88px 28px;
    font-size: 32px;
  }

  .project-tagline {
    translate: 88px 68px;
    font-size: 16px;
    max-width: calc(100% - 104px);
  }

  .description {
    translate: 16px 140px;
    width: calc(100% - 32px);
    font-size: 16px;
    line-height: 26px;
  }

  .background-gif {
    translate: 0 500px;
    height: 320px;
  }
}
</style>
