<script setup lang="ts">
import type { BulletLegendItemInterface } from '@unovis/ts'
import { VisBulletLegend } from '@unovis/vue'

const props = withDefaults(defineProps<{ items: BulletLegendItemInterface[] }>(), {
  items: () => [],
})

const emits = defineEmits<{
  'legendItemClick': [d: BulletLegendItemInterface, i: number]
  'update:items': [payload: BulletLegendItemInterface[]]
}>()

function onLegendItemClick(d: BulletLegendItemInterface, i: number) {
  emits('legendItemClick', d, i)
  const isBulletActive = !props.items[i].inactive
  const isFilterApplied = props.items.some(i => i.inactive)
  if (isFilterApplied && isBulletActive) {
    // reset filter
    emits('update:items', props.items.map(item => ({ ...item, inactive: false })))
  }
  else {
    // apply selection, set other item as inactive
    emits('update:items', props.items.map(item => item.name === d.name ? ({ ...d, inactive: false }) : { ...item, inactive: true }))
  }
}
</script>

<template>
  <div class="w-max">
    <VisBulletLegend
      :items="items"
      :on-legend-item-click="onLegendItemClick"
    />
  </div>
</template>
