<script setup lang="ts">
import { AvatarFallback, AvatarImage, AvatarRoot } from 'reka-ui'

defineOptions({ name: 'FlAvatar' })

withDefaults(
  defineProps<{
    src?: string
    alt?: string
    fallback?: string
    size?: 'sm' | 'md' | 'lg'
  }>(),
  {
    alt: '',
    fallback: '',
    size: 'md'
  }
)
</script>

<template>
  <AvatarRoot
    data-slot="avatar"
    :class="[
      'inline-flex shrink-0 items-center justify-center overflow-hidden rounded-full bg-[var(--fl-color-bg-muted)] text-[var(--fl-color-text-muted)] shadow-[inset_0_0_0_1px_var(--fl-color-border)]',
      size === 'sm' && 'size-7 text-xs',
      size === 'md' && 'size-9 text-sm',
      size === 'lg' && 'size-11 text-base'
    ]"
  >
    <AvatarImage
      v-if="src"
      class="fl-image-outline h-full w-full object-cover"
      :src="src"
      :alt="alt"
    />
    <AvatarFallback class="font-650 uppercase leading-none" :delay-ms="120">
      <slot>{{ fallback }}</slot>
    </AvatarFallback>
  </AvatarRoot>
</template>
