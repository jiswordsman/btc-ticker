<template>
  <div class="flex flex-col items-center gap-0.5">
    <!-- Price direction arrow + percentage badge -->
    <div class="flex items-center gap-1.5">
      <span
        class="text-xs font-medium tracking-wider uppercase opacity-60"
      >
        BTC-USDT 永续
      </span>
      <span
        class="inline-flex items-center rounded-full px-1.5 py-0.5 text-[11px] font-bold"
        :class="isUp
          ? 'bg-emerald-500/15 text-emerald-400'
          : 'bg-red-500/15 text-red-400'"
      >
        <span class="mr-0.5">{{ isUp ? '▲' : '▼' }}</span>
        {{ changePercent }}
      </span>
    </div>

    <!-- Main price display -->
    <div class="relative">
      <span
        :key="animKey"
        class="text-[2.7rem] font-bold tabular-nums tracking-tight price-animate leading-none"
        :class="[
          isUp ? 'text-emerald-400' : 'text-red-400',
          direction === 'up' ? 'flash-up' : '',
          direction === 'down' ? 'flash-down' : '',
        ]"
      >
        {{ formattedPrice }}
      </span>
    </div>

    <!-- Change amount -->
    <div
      class="text-sm font-medium tabular-nums"
      :class="isUp ? 'text-emerald-400' : 'text-red-400'"
    >
      {{ changeAmount }} USDT
    </div>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  formattedPrice: string
  changePercent: string
  changeAmount: string
  isUp: boolean
  direction: 'up' | 'down' | 'none'
}>()

// Animation key to trigger re-render animation
const animKey = ref(0)
watch(() => props.formattedPrice, () => {
  animKey.value++
})
</script>
