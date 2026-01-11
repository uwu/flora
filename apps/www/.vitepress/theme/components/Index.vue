<template>
  <div class="page-root">
    <div class="header">
      <div class="logo" />

      <div class="brand-name">flora</div>

      <div class="project-tagline">a uwu.network project</div>
    </div>

    <div class="description">
      flora is a fast runtime that lets you write discord bots for your servers,
      with a rich TypeScript SDK, without worrying about running infrastructure.
      <br /><br />
      You deploy scripts on a per-guild basis through a single bot, handled
      through the V8 JavaScript engine using V8 Isolates.
      <br /><br />
      This is still taking shape, and will have a proper release soon. To stay
      up to date on flora's progress, check out its repository:
      https://github.com/uwu/flora
    </div>

    <div class="background-gif" />
  </div>
</template>

<style>
.page-root {
  background-color: #a4b793 !important;
  box-sizing: border-box;
  contain: content;
  font-synthesis: none;
  -moz-osx-font-smoothing: grayscale;
  -webkit-font-smoothing: antialiased;
  position: relative;
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
  background-image: url("https://workers.paper.design/file-assets/01KEFKN1D3898F8CDSF2YNKHVF/01KEMP9M9JKS19GX80F5T4Z3EV.png");
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
  font-family: "Rethink Sans", system-ui, sans-serif;
  font-size: 60px;
  line-height: 64px;
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
  font-family: "Rethink Sans", system-ui, sans-serif;
  font-size: 40px;
  line-height: 64px;
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
  font-family: "Rethink Sans", system-ui, sans-serif;
  font-size: 40px;
  line-height: 60px;
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
  background-image: url("https://workers.paper.design/file-assets/01KEFKN1D3898F8CDSF2YNKHVF/01KEMPNZZ5H2YXCDATPA499VXH.gif");
  background-position: center;
  background-size: cover;
  box-sizing: border-box;
}
</style>
