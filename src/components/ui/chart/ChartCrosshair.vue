<script setup lang="ts">
import type { BulletLegendItemInterface } from '@unovis/ts'
import { VisCrosshair, VisTooltip } from '@unovis/vue'
import { type Component, createApp } from 'vue'
import { ChartTooltip } from '.'

const props = withDefaults(defineProps<{
  colors: string[]
  index: string
  items: BulletLegendItemInterface[]
  customTooltip?: Component
}>(), {
  colors: () => [],
})

// Use weakmap to store reference to each datapoint for Tooltip
let wm = new WeakMap<object, string>()
watch(() => [props.items, props.index, props.customTooltip], () => {
  wm = new WeakMap()
})
function template(d: any) {
  if (wm.has(d)) {
    return wm.get(d)
  }
  else {
    const componentDiv = document.createElement('div')
    const omittedData = props.items.map(item => ({ ...item, value: d[item.name] }))
    const TooltipComponent = props.customTooltip ?? ChartTooltip
    const app = createApp(TooltipComponent, { title: String(d[props.index]), data: omittedData })
    try {
      app.mount(componentDiv)
      const html = componentDiv.innerHTML
      wm.set(d, html)
      return html
    }
    finally {
      app.unmount()
    }
  }
}

function color(_d: unknown, i: number) {
  return props.colors[i] ?? 'transparent'
}
</script>

<template>
  <VisTooltip :horizontal-shift="20" :vertical-shift="20" />
  <VisCrosshair :template="template" :color="color" />
</template>
