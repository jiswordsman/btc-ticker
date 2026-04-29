<template>
  <div class="grid grid-cols-2 gap-2.5">
    <!-- Open Price -->
    <div class="stat-card">
      <div class="mb-1 flex items-center gap-1.5">
        <span class="h-1.5 w-1.5 rounded-full bg-amber-300"></span>
        <span class="stat-label">开盘价</span>
      </div>
      <div class="stat-value text-white/92">{{ open }}</div>
    </div>

    <!-- Funding Rate -->
    <div class="stat-card">
      <div class="mb-1 flex items-center gap-1.5">
        <span
          class="h-1.5 w-1.5 rounded-full"
          :class="fundingPositive ? 'bg-emerald-400' : 'bg-red-400'"
        ></span>
        <span class="stat-label">资金费率</span>
      </div>
      <div
        class="stat-value"
        :class="fundingPositive ? 'text-emerald-300' : 'text-red-300'"
      >
        {{ fundingRate }}
      </div>
    </div>

    <!-- 24h Volume -->
    <div class="stat-card">
      <div class="mb-1 flex items-center gap-1.5">
        <span class="h-1.5 w-1.5 rounded-full bg-sky-400"></span>
        <span class="stat-label">24h 量 (张)</span>
      </div>
      <div class="stat-value text-white/92">{{ vol }}</div>
    </div>

    <!-- 24h Volume (USDT) -->
    <div class="stat-card">
      <div class="mb-1 flex items-center gap-1.5">
        <span class="h-1.5 w-1.5 rounded-full bg-violet-400"></span>
        <span class="stat-label">24h 额 (USDT)</span>
      </div>
      <div class="stat-value text-white/92">{{ volCcy }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  open: string
  fundingRate: string
  fundingPositive: boolean
  vol: string
  volCcy: string
}>()
</script>

<style scoped>
.stat-card {
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 18px;
  background: rgba(255, 255, 255, 0.035);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.03);
  padding: 0.7rem;
}

.stat-label {
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: rgba(255, 255, 255, 0.42);
}

.stat-value {
  font-size: 1.6rem;
  line-height: 1;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}
</style>
