import type { InterfaceType, NormalizedResource } from '@/bindings'
import type { Reactive } from 'vue'
import { events } from '@/bindings'
import { useDocumentVisibility } from '@vueuse/core'

import { computed, reactive } from 'vue'
import { useTab } from './useTab'

export const MAX_STATISTICS_LENGTH = 20
const LOCAL_UPDATE_INTERVAL = 3

export interface StatisticData {
  'time': string
  'timestamp': number
  'System Power': number
  'System In': number
  'Battery Level': number
  'Screen Power'?: number
  'Heatpipe Power'?: number
}

interface RawPowerData {
  data: NormalizedResource
  statistics: StatisticData[]
}

export function appendStatistic(statistics: StatisticData[], sample: StatisticData) {
  if (statistics[statistics.length - 1]?.timestamp === sample.timestamp)
    return
  statistics.push(sample)
  if (statistics.length > MAX_STATISTICS_LENGTH)
    statistics.splice(0, statistics.length - MAX_STATISTICS_LENGTH)
}

const localPowerData: Reactive<RawPowerData> = reactive({
  data: {} as NormalizedResource,
  statistics: [],
})

let localUpdateCount = 0

events.powerTickEvent.listen(({ payload: { data } }) => {
  localPowerData.data = data

  localUpdateCount++
  if (localUpdateCount < LOCAL_UPDATE_INTERVAL)
    return
  localUpdateCount = 0

  const now = new Date()
  appendStatistic(localPowerData.statistics, {
    'time': now.toLocaleTimeString(),
    'timestamp': now.getTime(),
    'System Power': data.systemLoad,
    'System In': data.systemIn,
    'Battery Level': data.batteryLevel,
    'Screen Power': data.brightnessPower,
    'Heatpipe Power': data.heatpipePower,
  })
})

events.devicePowerTickEvent.listen(({ payload: { data, udid } }) => {
  const deviceData = getOrCreateDeviceData(udid)
  deviceData.data = data

  appendStatistic(deviceData.statistics, {
    'time': new Date(data.lastUpdate * 1000).toLocaleTimeString(),
    'timestamp': data.lastUpdate * 1000,
    'System Power': data.systemLoad,
    'System In': data.systemIn,
    'Battery Level': data.batteryLevel,
  })
})

export type RemotePowerData = RawPowerData & {
  name: string
  offline: boolean
  interface: Set<InterfaceType>
}

interface PowerData {
  local: RawPowerData
  remote: Record<string, RemotePowerData>
}

const power = reactive<PowerData>({
  local: localPowerData,
  remote: {},
})

function getOrCreateDeviceData(udid: string): RemotePowerData {
  if (!power.remote[udid]) {
    power.remote[udid] = {
      data: {} as NormalizedResource,
      statistics: [],
      name: '',
      offline: false,
      interface: new Set(),
    }
  }
  return power.remote[udid]
}

events.deviceEvent.listen(({ payload }) => {
  const deviceData = getOrCreateDeviceData(payload.udid)

  if (payload.action === 'Attached') {
    deviceData.interface.add(payload.interface)
    deviceData.offline = false
  }
  else if (payload.action === 'Detached') {
    deviceData.interface.delete(payload.interface)
  }
  if (deviceData.interface.size === 0) {
    deviceData.offline = true
  }
})

const vis = useDocumentVisibility()
const tab = useTab()

const currentPower = computed<RawPowerData>(() => {
  return tab.value === 'local' ? power.local : power.remote[tab.value] || {}
})

export function usePower() {
  return computed(() => ({
    ...currentPower.value.data,
    isLoading: Object.keys(currentPower.value.data).length === 0 || vis.value === 'hidden',
    isRemote: tab.value !== 'local',
    statistics: currentPower.value.statistics,
  }))
}

export function usePowerData() {
  return power
}

export function usePowerRaw() {
  return computed<
    RawPowerData & { isLocal: true } |
    RemotePowerData & { isLocal: false }
  >(() => {
    const isLocal = tab.value === 'local'
    return {
      ...isLocal ? currentPower.value : power.remote[tab.value],
      isLocal,
    } as any
  })
}
