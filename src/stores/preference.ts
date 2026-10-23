import type { ChartAnimation, StatusBarItem, Theme } from '@/bindings'
import { defineStore } from 'pinia'
import { ref } from 'vue'

export const usePreference = defineStore('preference', () => {
  const theme = ref<Theme>('system')
  // Use the system default until a saved application preference is loaded.
  const animationsEnabled = ref(!window.matchMedia('(prefers-reduced-motion: reduce)').matches)
  const chartAnimation = ref<ChartAnimation>('redraw')
  const updateInterval = ref(1500)
  const language = ref('en')
  const statusBarItem = ref<StatusBarItem>('system')
  const statusBarShowCharging = ref(true)

  return {
    theme,
    animationsEnabled,
    chartAnimation,
    updateInterval,
    language,
    statusBarItem,
    statusBarShowCharging,
  }
}, {
  tauri: {
    saveOnChange: true,
    saveStrategy: 'debounce',
    saveInterval: 1000,
  },
})

export function usePreferenceAsync() {
  const preference = usePreference()
  const isLoading = ref(true)
  preference.$tauri.start().then(() => isLoading.value = false)
  return { preference, isLoading }
}
