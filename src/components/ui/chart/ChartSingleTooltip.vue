<script setup lang="ts">
import type { BulletLegendItemInterface } from '@unovis/ts'
import { omit } from '@unovis/ts'
import { VisTooltip } from '@unovis/vue'
import { type Component, createApp } from 'vue'
import { ChartTooltip } from '.'

const props = withDefaults(defineProps<{
  selector: string
  index: string
  items?: BulletLegendItemInterface[]
  valueFormatter?: (tick: number, i?: number, ticks?: number[]) => string
  customTooltip?: Component
}>(), {
  valueFormatter: (tick: number) => `${tick}`,
})

// Use weakmap to store reference to each datapoint for Tooltip
let wm = new WeakMap<object, string>()
watch(() => [props.items, props.index, props.valueFormatter, props.customTooltip], () => {
  wm = new WeakMap()
})
function template(d: any, i: number, elements: (HTMLElement | SVGElement)[]) {
  const key = props.index in d ? d : d.data
  if (wm.has(key))
    return wm.get(key)

  let tooltipData
  if (props.index in d) {
    tooltipData = Object.entries(omit(d, [props.index])).map(([key, value]) => {
      const legendReference = props.items?.find(i => i.name === key)
      return { ...legendReference, value: props.valueFormatter(value) }
    })
  }
  else {
    const data = d.data
    const style = getComputedStyle(elements[i])
    tooltipData = [{ name: data.name, value: props.valueFormatter(data[props.index]), color: style.fill }]
  }

  const componentDiv = document.createElement('div')
  const TooltipComponent = props.customTooltip ?? ChartTooltip
  const app = createApp(TooltipComponent, { title: d[props.index], data: tooltipData })
  try {
    app.mount(componentDiv)
    const html = componentDiv.innerHTML
    wm.set(key, html)
    return html
  }
  finally {
    app.unmount()
  }
}
</script>

<template>
  <VisTooltip
    :horizontal-shift="20"
    :vertical-shift="20"
    :triggers="{
      [selector]: template,
    }"
  />
</template>
