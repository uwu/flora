<script setup lang="ts">
import { computed } from 'vue'

export type DecorativeMode = 'none' | 'lines' | 'dashed' | 'lines-with-dots'
export type GridLinesVariant =
  | 'frame'
  | 'above-bottom-dots'
  | 'tabbar-dots'
  | 'navbar-lines'
  | 'navbar-dots'

const props = withDefaults(
  defineProps<{
    mode?: DecorativeMode
    variant?: GridLinesVariant
  }>(),
  {
    mode: 'lines-with-dots',
    variant: 'frame'
  }
)

const lineOffset = 'var(--grid-line-offset, 20px)'
const lineLeft = `calc(-1 * ${lineOffset})`
const lineRight = `calc(-1 * ${lineOffset})`
const fullWidthX = `calc(50% - var(--grid-max-width, 72rem) / 2 - ${lineOffset})`

const lineClass = computed(() => (props.mode === 'dashed' ? 'dashed' : 'solid'))
const showLines = computed(() => props.mode !== 'none')
const showDots = computed(() => props.mode === 'lines-with-dots')
</script>

<template>
  <template v-if="variant === 'frame' && showLines">
    <div
      aria-hidden="true"
      class="flora-grid-line-v hidden lg:block"
      :class="lineClass"
      :style="{ left: lineLeft }"
    />
    <div
      aria-hidden="true"
      class="flora-grid-line-v hidden lg:block"
      :class="lineClass"
      :style="{ right: lineRight }"
    />

    <template v-if="showDots">
      <div
        aria-hidden="true"
        class="flora-grid-dot hidden lg:flex"
        :style="{ bottom: 0, left: lineLeft, transform: 'translate(-50%, 50%)' }"
      />
      <div
        aria-hidden="true"
        class="flora-grid-dot hidden lg:flex"
        :style="{ bottom: 0, right: lineRight, transform: 'translate(50%, 50%)' }"
      />
    </template>
  </template>

  <template v-else-if="variant === 'above-bottom-dots' && showDots">
    <div
      aria-hidden="true"
      class="flora-grid-dot hidden lg:flex"
      :style="{
        bottom: 'var(--layout-gap, 2rem)',
        left: lineLeft,
        transform: 'translate(-50%, 50%)'
      }"
    />
    <div
      aria-hidden="true"
      class="flora-grid-dot hidden lg:flex"
      :style="{
        bottom: 'var(--layout-gap, 2rem)',
        right: lineRight,
        transform: 'translate(50%, 50%)'
      }"
    />
  </template>

  <template v-else-if="variant === 'tabbar-dots' && showDots">
    <div
      aria-hidden="true"
      class="flora-grid-dot hidden lg:flex"
      :style="{ top: 0, left: fullWidthX, transform: 'translate(-50%, -50%)' }"
    />
    <div
      aria-hidden="true"
      class="flora-grid-dot hidden lg:flex"
      :style="{ top: 0, right: fullWidthX, transform: 'translate(50%, -50%)' }"
    />
    <div
      aria-hidden="true"
      class="flora-grid-dot hidden lg:flex"
      :style="{ bottom: 0, left: fullWidthX, transform: 'translate(-50%, 50%)' }"
    />
    <div
      aria-hidden="true"
      class="flora-grid-dot hidden lg:flex"
      :style="{ bottom: 0, right: fullWidthX, transform: 'translate(50%, 50%)' }"
    />
  </template>

  <template v-else-if="variant === 'navbar-dots' && showDots">
    <div
      aria-hidden="true"
      class="flora-grid-dot hidden lg:flex"
      :style="{ bottom: 0, left: fullWidthX, transform: 'translate(-50%, 50%)' }"
    />
    <div
      aria-hidden="true"
      class="flora-grid-dot hidden lg:flex"
      :style="{ bottom: 0, right: fullWidthX, transform: 'translate(50%, 50%)' }"
    />
  </template>

  <template v-else-if="variant === 'navbar-lines' && showLines">
    <div
      aria-hidden="true"
      class="flora-grid-line-v hidden lg:block"
      :class="lineClass"
      :style="{ left: fullWidthX }"
    />
    <div
      aria-hidden="true"
      class="flora-grid-line-v hidden lg:block"
      :class="lineClass"
      :style="{ right: fullWidthX }"
    />
  </template>
</template>
