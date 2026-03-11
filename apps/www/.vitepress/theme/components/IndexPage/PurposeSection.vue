<template>
  <section class="purpose">
    <div class="purpose-inner">
      <div class="purpose-text">
        <div class="purpose-heading">
          <h2 class="purpose-title">Writing bots should be fun.</h2>
          <img class="purpose-title-art" src="/purpose-flowers.png" alt="" />
        </div>
        <p class="purpose-copy">
          Building Discord bots is exciting. Deploying them, chasing API changes, and maintaining
          them months later usually is not.
        </p>
        <p class="purpose-copy">
          That is why we are building flora: to make the full lifecycle, from first commit to
          long-term iteration, feel fast and exciting again.
        </p>
      </div>
      <div class="purpose-art" aria-hidden="true">
        <img src="/purpose-flowers.png" alt="" />
      </div>
    </div>
  </section>
</template>

<style scoped>
.purpose {
  --uno: "px-8 py-18";
  background: #0e0f10;
}

.purpose-inner {
  --uno: "max-w-5xl mx-auto grid gap-8 items-center";
  grid-template-columns: minmax(0, 1fr) auto;
}

.purpose-text {
  min-width: 0;
}

.purpose-art {
  --uno: "self-end";
}

.purpose-art img {
  width: clamp(220px, 24vw, 330px);
  height: auto;
  display: block;
  filter: drop-shadow(0 18px 30px rgb(0 0 0 / 0.32));
}

.purpose-heading {
  display: block;
}

.purpose-title {
  --uno: "m-0 mb-5 font-600";
  color: var(--gb-fg);
  font-size: clamp(30px, 4vw, 44px);
  line-height: 1.08;
  letter-spacing: -0.03em;
  max-width: 860px;
}

.purpose-title-art {
  display: none;
  width: min(120px, 24vw);
  height: auto;
  filter: drop-shadow(0 14px 24px rgb(0 0 0 / 0.3));
}

.purpose-copy {
  --uno: "m-0 text-base";
  color: var(--gb-fg-soft);
  max-width: 860px;
  line-height: 1.75;
}

.purpose-copy + .purpose-copy {
  margin-top: 12px;
}

@media (max-width: 900px) {
  .purpose-inner {
    grid-template-columns: 1fr;
  }

  .purpose-art {
    justify-self: end;
  }
}

@media (max-width: 600px) {
  .purpose {
    --uno: "px-5 py-14";
  }

  .purpose-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }

  .purpose-title {
    margin-bottom: 0;
  }

  .purpose-title-art {
    display: block;
    width: min(120px, 26vw);
  }

  .purpose-art {
    display: none;
  }

  .purpose-art img {
    width: min(220px, 55vw);
  }
}
</style>
