<script lang="ts" setup>
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/utils'
import { useDocumentVisibility } from '@vueuse/core'
import { ref, watch } from 'vue'

interface ShimmerProps {
  as?: keyof HTMLElementTagNameMap
  class?: HTMLAttributes['class']
  duration?: number
  spread?: number
  delay?: number
  repeatDelay?: number
}

const props = defineProps<ShimmerProps>()
const element = ref<HTMLElement>()
const preference = usePreference()
const visibility = useDocumentVisibility()

watch([
  element,
  () => preference.animationsEnabled,
  visibility,
  () => props.duration,
  () => props.delay,
  () => props.repeatDelay,
], ([element, enabled, visibility], _, onCleanup) => {
  if (!element || !enabled || visibility !== 'visible')
    return
  const duration = Math.max(1, props.duration ?? 1200)
  const period = duration + Math.max(0, props.repeatDelay ?? 0)
  const animation = element.animate([
    { transform: 'translateX(-100%)', offset: 0 },
    { transform: 'translateX(100%)', offset: duration / period },
    { transform: 'translateX(100%)', offset: 1 },
  ], { duration: period, delay: props.delay ?? 200, iterations: Infinity })
  onCleanup(() => animation.cancel())
})
</script>

<template>
  <div
    :class="cn(
      'relative inline-block overflow-hidden',
      '[--base-color:#a1a1aa] [--base-gradient-color:#000]',
      '[--bg:linear-gradient(90deg,#0000_calc(50%-var(--spread)),var(--base-gradient-color),#0000_calc(50%+var(--spread)))]',
      'dark:[--base-color:#71717a] dark:[--base-gradient-color:#ffffff] dark:[--bg:linear-gradient(90deg,#0000_calc(50%-var(--spread)),var(--base-gradient-color),#0000_calc(50%+var(--spread)))]',
      props.class,
    )"
    :style="{
      '--spread': `${props.spread ?? 20}px`,
      'background-color': 'var(--base-color)',
    }"
  >
    <span
      ref="element"
      aria-hidden="true"
      class="pointer-events-none absolute inset-0"
      :style="{ backgroundImage: 'var(--bg)', transform: 'translateX(-100%)' }"
    />
    <slot />
  </div>
</template>
