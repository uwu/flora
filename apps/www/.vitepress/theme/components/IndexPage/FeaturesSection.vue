<script setup lang="ts">
import { AnimatePresence, motion } from 'motion-v'
import { Stepper, useAutoPlay } from 'pasito/vue'
import { onMounted, ref } from 'vue'
import 'pasito/styles.css'

import { features } from './features'
import FeatureSnippetBox from './FeatureSnippetBox.vue'

const active = ref(0)
const stepCount = ref(features.length)

const { toggle, filling, fillDuration } = useAutoPlay({
  count: stepCount,
  active,
  onStepChange: (i) => {
    active.value = i
  },
  stepDuration: 5000,
  loop: true
})

onMounted(() => {
  toggle()
})
</script>

<template>
  <section class="features">
    <div class="features-inner">
      <h2 class="features-heading">Built for speed, built for guilds.</h2>

      <div class="features-carousel">
        <div class="feature-slide active">
          <AnimatePresence mode="sync" :initial="false">
            <motion.img
              :key="features[active].bg"
              :src="features[active].bg"
              alt=""
              class="feature-slide-bg"
              :initial="{ opacity: 0 }"
              :animate="{ opacity: 1 }"
              :exit="{ opacity: 0 }"
              :transition="{ duration: 0.5, ease: 'easeInOut' }"
            />
          </AnimatePresence>
          <div class="feature-slide-overlay" />
          <div class="feature-slide-content">
            <h3>{{ features[active].title }}</h3>
            <p>{{ features[active].desc }}</p>
          </div>
          <FeatureSnippetBox :feature="features[active]" />
        </div>

        <div class="stepper-wrap">
          <Stepper
            :count="features.length"
            :active="active"
            :filling="filling"
            :fill-duration="fillDuration"
            class="flora-stepper"
            @step-click="active = $event"
          />
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.features {
  --uno: "bg-[#0e0f10]";
  background: #0e0f10;
  padding: 120px 32px;
}

.features-inner {
  --uno: "max-w-5xl mx-auto";
}

.features-heading {
  --uno: "font-600 m-0 mb-14";
  color: var(--gb-fg);
  font-size: 36px;
  letter-spacing: -0.03em;
}

.features-carousel {
  --uno: "relative rounded-4 overflow-hidden border";
  border-color: var(--gb-border);
  aspect-ratio: 16 / 9;
}

.feature-slide {
  position: absolute;
  inset: 0;
  opacity: 0;
  transition: opacity 600ms ease;
  pointer-events: none;
}

.feature-slide.active {
  opacity: 1;
  pointer-events: auto;
}

.feature-slide-bg {
  --uno: "absolute inset-0 w-full h-full object-cover";
}

.feature-slide-overlay {
  position: absolute;
  inset: 0;
  background: linear-gradient(
    to top,
    color-mix(in srgb, var(--background) 88%, black) 0%,
    color-mix(in srgb, var(--background) 35%, transparent) 60%,
    transparent 100%
  );
}

.feature-slide-content {
  --uno: "absolute bottom-0 left-0 p-10";
}

.feature-slide-content h3 {
  --uno: "font-600 m-0 mb-2";
  color: var(--gb-fg);
  font-size: 24px;
  letter-spacing: -0.02em;
}

.feature-slide-content p {
  --uno: "m-0";
  color: var(--gb-fg-soft);
  font-size: 15px;
  line-height: 1.6;
  max-width: 480px;
}

.stepper-wrap {
  position: absolute;
  right: 18px;
  bottom: 18px;
  z-index: 10;
}

.flora-stepper {
  --pill-bg: rgba(255, 255, 255, 0.3);
  --pill-active-bg: rgba(255, 255, 255, 0.9);
  --pill-container-bg: rgba(0, 0, 0, 0.2);
  --pill-container-border: rgba(255, 255, 255, 0.12);
  width: auto;
  height: 32px;
  padding: 0 10px;
  justify-content: center;
  align-items: center;
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
}

@media (max-width: 900px) {
  .features-heading {
    font-size: 28px;
  }

  .features-carousel {
    aspect-ratio: auto;
    min-height: 760px;
  }

  .stepper-wrap {
    left: auto;
    right: 12px;
    bottom: 12px;
    top: auto;
  }

  .feature-slide-content {
    --uno: "left-0 right-0 p-8";
    bottom: 14px;
  }

  .feature-slide-content p {
    max-width: 100%;
  }
}

@media (max-width: 600px) {
  .features {
    padding: 80px 20px;
  }

  .features-carousel {
    min-height: 680px;
  }

  .feature-slide-content {
    --uno: "left-0 right-0 p-6";
    bottom: 8px;
  }

  .stepper-wrap {
    right: 8px;
    bottom: 10px;
    top: auto;
  }
}
</style>
