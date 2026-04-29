import { ref, computed, onMounted, onUnmounted } from 'vue'

export interface TickerData {
  last: number
  open24h: number
  high24h: number
  low24h: number
  vol24h: number
  vol_ccy24h: number
  funding_rate: number
  change: number
  change_percent: number
  ts: number
}

// Price history for mini chart
const MAX_HISTORY = 60

export function useOkxPrice() {
  const price = ref(0)
  const open24h = ref(0)
  const high24h = ref(0)
  const low24h = ref(0)
  const vol24h = ref(0)
  const volCcy24h = ref(0)
  const fundingRate = ref(0)
  const change = ref(0)
  const changePercent = ref(0)
  const lastUpdate = ref(0)
  const connected = ref(false)
  const priceHistory = ref<number[]>([])
  const prevPrice = ref(0)
  const priceDirection = ref<'up' | 'down' | 'none'>('none')

  const isUp = computed(() => change.value >= 0)
  const formattedPrice = computed(() => price.value.toFixed(1))
  const formattedChange = computed(() => {
    const sign = change.value >= 0 ? '+' : ''
    return `${sign}${change.value.toFixed(1)}`
  })
  const formattedChangePercent = computed(() => {
    const sign = changePercent.value >= 0 ? '+' : ''
    return `${sign}${changePercent.value.toFixed(2)}%`
  })
  const formattedOpen = computed(() => open24h.value.toFixed(1))
  const formattedHigh = computed(() => high24h.value.toFixed(1))
  const formattedLow = computed(() => low24h.value.toFixed(1))
  const formattedVol = computed(() => {
    if (vol24h.value >= 1_000_000) {
      return `${(vol24h.value / 1_000_000).toFixed(2)}M`
    }
    if (vol24h.value >= 1_000) {
      return `${(vol24h.value / 1_000).toFixed(1)}K`
    }
    return vol24h.value.toFixed(0)
  })
  const formattedVolCcy = computed(() => {
    if (volCcy24h.value >= 1_000_000_000) {
      return `${(volCcy24h.value / 1_000_000_000).toFixed(2)}B`
    }
    if (volCcy24h.value >= 1_000_000) {
      return `${(volCcy24h.value / 1_000_000).toFixed(2)}M`
    }
    return volCcy24h.value.toFixed(0)
  })
  const formattedFundingRate = computed(() => {
    const ratePercent = fundingRate.value * 100
    const sign = ratePercent > 0 ? '+' : ''
    return `${sign}${ratePercent.toFixed(4)}%`
  })
  const formattedTime = computed(() => {
    if (!lastUpdate.value) return '--:--:--'
    const d = new Date(lastUpdate.value)
    return d.toLocaleTimeString('zh-CN', { hour12: false })
  })

  let unlisten: (() => void) | null = null

  onMounted(async () => {
    try {
      const { listen } = await import('@tauri-apps/api/event')
      unlisten = await listen<TickerData>('price-update', (event) => {
        const data = event.payload

        // Track price direction for flash animation
        if (price.value !== 0) {
          prevPrice.value = price.value
          if (data.last > price.value) {
            priceDirection.value = 'up'
          } else if (data.last < price.value) {
            priceDirection.value = 'down'
          } else {
            priceDirection.value = 'none'
          }
          // Reset direction after animation
          setTimeout(() => {
            priceDirection.value = 'none'
          }, 500)
        }

        price.value = data.last
        open24h.value = data.open24h
        high24h.value = data.high24h
        low24h.value = data.low24h
        vol24h.value = data.vol24h
        volCcy24h.value = data.vol_ccy24h
        fundingRate.value = data.funding_rate
        change.value = data.change
        changePercent.value = data.change_percent
        lastUpdate.value = data.ts
        connected.value = true

        // Add to history
        priceHistory.value.push(data.last)
        if (priceHistory.value.length > MAX_HISTORY) {
          priceHistory.value.shift()
        }
      })
    } catch {
      // Running outside Tauri (dev mode) — use mock data
      console.warn('Tauri API not available, using mock data')
      useMockData()
    }
  })

  onUnmounted(() => {
    unlisten?.()
  })

  function useMockData() {
    const basePrice = 94500
    connected.value = true

    const update = () => {
      const randomChange = (Math.random() - 0.48) * 100
      const newPrice = Math.max(90000, price.value === 0 ? basePrice : price.value + randomChange)

      prevPrice.value = price.value || basePrice
      priceDirection.value = newPrice > prevPrice.value ? 'up' : newPrice < prevPrice.value ? 'down' : 'none'
      setTimeout(() => { priceDirection.value = 'none' }, 500)

      price.value = newPrice
      open24h.value = basePrice
      high24h.value = Math.max(high24h.value, newPrice)
      low24h.value = low24h.value === 0 ? newPrice - 500 : Math.min(low24h.value, newPrice)
      vol24h.value = 125432
      volCcy24h.value = 11876543210
      fundingRate.value = 0.0001
      change.value = newPrice - basePrice
      changePercent.value = (change.value / basePrice) * 100
      lastUpdate.value = Date.now()

      priceHistory.value.push(newPrice)
      if (priceHistory.value.length > MAX_HISTORY) {
        priceHistory.value.shift()
      }
    }

    update()
    setInterval(update, 2000)
  }

  return {
    price,
    open24h,
    high24h,
    low24h,
    vol24h,
    volCcy24h,
    fundingRate,
    change,
    changePercent,
    lastUpdate,
    connected,
    priceHistory,
    prevPrice,
    priceDirection,
    isUp,
    formattedPrice,
    formattedChange,
    formattedChangePercent,
    formattedOpen,
    formattedHigh,
    formattedLow,
    formattedVol,
    formattedVolCcy,
    formattedFundingRate,
    formattedTime,
  }
}
