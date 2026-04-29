<template>
  <div class="ticker-page flex h-full min-h-0 flex-col">
    <!-- Drag handle / header -->
    <div class="drag-region flex items-center justify-between px-4 pt-2.5 pb-2">
      <div class="flex items-center gap-2">
        <span class="text-lg">₿</span>
        <span class="text-sm font-semibold text-white/90">BTC Ticker</span>
      </div>
      <div class="flex items-center gap-2 no-drag">
        <!-- Live indicator -->
        <div class="flex items-center gap-1.5">
          <span
            class="w-1.5 h-1.5 rounded-full live-dot"
            :class="connected ? 'text-emerald-400 bg-emerald-400' : 'text-red-400 bg-red-400'"
          ></span>
          <span class="text-[10px] text-white/45 font-medium uppercase tracking-widest">
            {{ connected ? 'Live' : 'Offline' }}
          </span>
        </div>
      </div>
    </div>

    <!-- Separator -->
    <div class="mx-4 h-px bg-white/8"></div>

    <!-- Main content -->
    <div class="min-h-0 flex-1 overflow-y-auto px-4 py-2.5 space-y-3">
      <!-- Price display -->
      <PriceDisplay
        :formatted-price="formattedPrice"
        :change-percent="formattedChangePercent"
        :change-amount="formattedChange"
        :is-up="isUp"
        :direction="priceDirection"
      />

      <!-- Price range bar -->
      <div class="px-1">
        <div class="mb-1 flex justify-between text-[10px] text-white/42">
          <span>24h 最低</span>
          <span>24h 最高</span>
        </div>
        <div class="relative h-1.5 overflow-hidden rounded-full bg-white/8">
          <div
            class="absolute inset-y-0 left-0 rounded-full"
            :class="isUp ? 'bg-gradient-to-r from-emerald-500/40 to-emerald-400' : 'bg-gradient-to-r from-red-500/40 to-red-400'"
            :style="{ width: priceRangePercent + '%' }"
          />
          <!-- Current price indicator -->
          <div
            class="absolute top-1/2 -translate-y-1/2 w-2.5 h-2.5 rounded-full border-2 border-white/80 shadow-lg"
            :class="isUp ? 'bg-emerald-400' : 'bg-red-400'"
            :style="{ left: `calc(${priceRangePercent}% - 5px)` }"
          />
        </div>
        <div class="mt-1 flex justify-between text-[11px] tabular-nums">
          <span class="text-red-400/70">{{ formattedLow }}</span>
          <span class="text-emerald-400/70">{{ formattedHigh }}</span>
        </div>
      </div>

      <!-- Mini chart -->
      <!-- <MiniChart :history="priceHistory" :is-up="isUp" /> -->

      <!-- Stats grid -->
      <PriceStats
        :open="formattedOpen"
        :funding-rate="formattedFundingRate"
        :funding-positive="fundingRate >= 0"
        :vol="formattedVol"
        :vol-ccy="formattedVolCcy"
      />
    </div>

    <!-- Footer -->
    <div class="shrink-0 border-t border-white/8 px-4 py-2">
      <div class="flex items-center justify-between">
        <span class="text-[10px] text-white/28 tabular-nums">
          OKX · BTC-USDT-SWAP
        </span>
        <span class="text-[10px] text-white/28 tabular-nums">
          {{ formattedTime }}
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useOkxPrice } from '~/composables/useOkxPrice'

const {
  price,
  high24h,
  low24h,
  connected,
  priceHistory,
  formattedOpen,
  fundingRate,
  priceDirection,
  isUp,
  formattedPrice,
  formattedChange,
  formattedChangePercent,
  formattedFundingRate,
  formattedHigh,
  formattedLow,
  formattedVol,
  formattedVolCcy,
  formattedTime,
} = useOkxPrice()

// Calculate where current price sits in the 24h range
const priceRangePercent = computed(() => {
  const range = high24h.value - low24h.value
  if (range <= 0) return 50
  return Math.min(100, Math.max(0, ((price.value - low24h.value) / range) * 100))
})
</script>

<style scoped>
.ticker-page {
  color: rgba(255, 255, 255, 0.92);
}
</style>
