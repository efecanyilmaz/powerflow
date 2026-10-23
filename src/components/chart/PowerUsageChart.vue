<script setup lang="ts">
import { MAX_STATISTICS_LENGTH } from '@/composables/usePower'
import { useI18n } from 'vue-i18n'
import CustomChartTooltip from './CustomChartTooltip.vue'

const { t } = useI18n()
const power = usePower()
const preference = usePreference()
const isRemote = computed(() => power.value.isRemote)
const isCharging = computed(() => power.value.isCharging)
const statistics = computed(() => power.value.statistics)
const colors = ['#2563eb', '#60a5fa', '#818cf8', '#0891b2']
const formatPower = (value: number | Date) => `${value}w`

const categories = computed(() => {
  const base = ['System Power'] as (keyof StatisticData)[]
  if (!isRemote.value) {
    base.push('Screen Power', 'Heatpipe Power')
  }
  if (isCharging.value) {
    base.push('System In')
  }
  return base
})

const localeMap = computed(() => ({
  'System Power': t('flow.system_total'),
  'Screen Power': t('flow.screen_power'),
  'Heatpipe Power': t('flow.heatpipe_power'),
  'System In': t('flow.system_in'),
}))

const localedData = computed(() => {
  return statistics.value.map((item) => {
    return Object.fromEntries(Object.entries(item).map(([key, value]) => [localeMap.value[key] || key, value]))
  })
})

const localedCategories = computed(() => categories.value.map(item => localeMap.value[item] || item))
</script>

<template>
  <Card class="w-full space-y-8 relative">
    <CardHeader class="pb-0">
      <CardTitle>
        {{ $t('power_usage') }}
      </CardTitle>
    </CardHeader>
    <CardContent>
      <Skeleton v-if="power.isLoading" class="w-full h-[240px]" />
      <LineChart
        v-else
        class="w-full h-[240px] font-bold"
        index="time"
        data-key="timestamp"
        :window-size="MAX_STATISTICS_LENGTH"
        :animation="preference.chartAnimation"
        :animations-enabled="preference.animationsEnabled"
        :y-formatter="formatPower"
        :data="localedData"
        :categories="localedCategories"
        :custom-tooltip="CustomChartTooltip"
        :colors="colors"
      />
    </CardContent>
  </Card>
</template>
