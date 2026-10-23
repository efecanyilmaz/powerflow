<script setup lang="ts" generic="T extends Record<string, any>">
import type { BaseChartProps } from '@/components/ui/chart/interface'
import { ChartCrosshair, ChartLegend, defaultColors } from '@/components/ui/chart'
import { cn } from '@/lib/utils'
import { Axis, type BulletLegendItemInterface, CurveType, Line, type XYContainer } from '@unovis/ts'
import { VisAxis, VisLine, VisXYContainer } from '@unovis/vue'
import { useDocumentVisibility, useMounted } from '@vueuse/core'
import { type Component, computed, nextTick, onBeforeUnmount, ref, shallowRef, watch } from 'vue'

const props = withDefaults(defineProps<BaseChartProps<T> & {
  /**
   * Render custom tooltip component.
   */
  customTooltip?: Component
  /**
   * Type of curve
   */
  curveType?: CurveType
  animation?: 'redraw' | 'scroll'
  animationsEnabled?: boolean
  windowSize?: number
  dataKey?: Extract<keyof T, string>
}>(), {
  curveType: CurveType.MonotoneX,
  filterOpacity: 0.2,
  margin: () => ({ top: 0, bottom: 0, left: 0, right: 0 }),
  showXAxis: true,
  showYAxis: true,
  showTooltip: true,
  showLegend: true,
  showGridLine: true,
  animation: 'redraw',
  animationsEnabled: true,
  windowSize: 20,
})

const emits = defineEmits<{
  legendItemClick: [d: BulletLegendItemInterface, i: number]
}>()

type KeyOfT = Extract<keyof T, string>
type Data = typeof props.data[number]

const index = computed(() => props.index as KeyOfT)
const colors = computed(() => props.colors?.length ? props.colors : defaultColors(props.categories.length))

const legendItems = ref<BulletLegendItemInterface[]>([])
watch([() => props.categories, colors], () => {
  legendItems.value = props.categories.map((category, i) => ({
    name: category,
    color: colors.value[i],
    inactive: legendItems.value.find(item => item.name === category)?.inactive ?? false,
  }))
}, { immediate: true })

const chart = ref<{ component: XYContainer<T> }>()
const visibility = useDocumentVisibility()
const canAnimate = computed(() => props.animationsEnabled && visibility.value === 'visible')
const renderedData = shallowRef<T[]>([])
const xOffset = ref(0)
const scrolling = ref(false)
const duration = computed(() => props.animation === 'scroll' || !canAnimate.value ? 0 : 600)
const xDomain = computed<[number, number] | undefined>(() => props.animation === 'scroll' ? [0, props.windowSize - 1] : undefined)
let previousData: T[] = []
let frame = 0
let animations: Animation[] = []

function cancelScroll() {
  cancelAnimationFrame(frame)
  frame = 0
  for (const animation of animations) {
    animation.onfinish = null
    animation.cancel()
  }
  animations = []
  scrolling.value = false
}

watch([
  () => props.data.slice(),
  () => props.animation,
  () => props.categories,
  canAnimate,
], async ([data, mode, categories, enabled], previous, onCleanup) => {
  let active = true
  onCleanup(() => active = false)
  cancelScroll()
  const next = mode === 'scroll' ? data.slice(-props.windowSize) : data
  const key = props.dataKey ?? props.index
  const last = previousData[previousData.length - 1]
  const sameSeries = previous?.[1] === mode && previous?.[2] === categories
  const appended = next.length === previousData.length + 1
  const shifted = next.length === props.windowSize && previousData.length === props.windowSize
    && next[0]?.[key] === previousData[1]?.[key]
  const slide = mode === 'scroll' && enabled && sameSeries && last
    && next[next.length - 2]?.[key] === last[key] && (appended || shifted)

  // Retain only one outgoing point, outside the fixed plot clip, for this transition.
  renderedData.value = slide && shifted ? [previousData[0], ...next] : next
  xOffset.value = mode === 'scroll' ? props.windowSize - renderedData.value.length : 0
  previousData = next
  if (!slide)
    return

  scrolling.value = true
  await nextTick()
  if (!active)
    return
  // The Vue flush queues Unovis's render first; translate before that frame is painted.
  frame = requestAnimationFrame(() => {
    frame = 0
    const container = chart.value?.component
    if (!container) {
      scrolling.value = false
      return
    }
    const step = container.width / (props.windowSize - 1)
    for (const line of container.components) {
      for (const element of line.element.querySelectorAll(`.${Line.selectors.line}`)) {
        const animation = element.animate([
          { transform: `translateX(${step}px)` },
          { transform: 'translateX(0)' },
        ], { duration: 600, easing: 'linear' })
        animations.push(animation)
        animation.onfinish = () => {
          animation.onfinish = null
          animation.cancel()
          animations = animations.filter(item => item !== animation)
          if (!animations.length) {
            renderedData.value = previousData
            xOffset.value = props.windowSize - previousData.length
            scrolling.value = false
          }
        }
      }
    }
  })
}, { immediate: true })

onBeforeUnmount(() => {
  cancelScroll()
  previousData = []
  renderedData.value = []
})

const isMounted = useMounted()

function handleLegendItemClick(d: BulletLegendItemInterface, i: number) {
  emits('legendItemClick', d, i)
}
</script>

<template>
  <div :class="cn('w-full h-[400px] flex flex-col items-end', $attrs.class ?? '')">
    <ChartLegend
      v-if="showLegend"
      v-model:items="legendItems"
      class="absolute top-5 right-4"
      @legend-item-click="handleLegendItemClick"
    />

    <VisXYContainer
      ref="chart"
      :data="renderedData"
      :duration="duration"
      :x-domain="xDomain"
      :style="{ height: isMounted ? '100%' : 'auto', pointerEvents: scrolling ? 'none' : undefined }"
      width="100%"
    >
      <ChartCrosshair
        v-if="showTooltip"
        :colors="colors"
        :items="legendItems"
        :index="index"
        :custom-tooltip="customTooltip"
      />

      <template v-for="(category, i) in categories" :key="category">
        <VisLine
          :x="(_d: Data, i: number) => i + xOffset"
          :duration="duration"
          :y="(d: Data) => d[category]"
          :curve-type="curveType"
          :color="colors[i]"
          :attributes="{
            [Line.selectors.line]: {
              opacity: legendItems.find(item => item.name === category)?.inactive ? filterOpacity : 1,
            },
          }"
        />
      </template>

      <VisAxis
        v-if="showXAxis"
        type="x"
        :tick-format="xFormatter ?? ((v: number) => renderedData[v - xOffset]?.[index] ?? '')"
        :grid-line="false"
        :tick-line="false"
        tick-text-color="hsl(var(--vis-text-color))"
      />
      <VisAxis
        v-if="showYAxis"
        type="y"
        :tick-line="false"
        :tick-format="yFormatter"
        :domain-line="false"
        :grid-line="showGridLine"
        :attributes="{
          [Axis.selectors.grid]: {
            class: 'text-muted',
          },
        }"
        tick-text-color="hsl(var(--vis-text-color))"
      />

      <slot />
    </VisXYContainer>
  </div>
</template>
