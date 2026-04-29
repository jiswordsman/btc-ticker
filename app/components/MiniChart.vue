<template>
  <div class="w-full rounded-xl bg-white/[0.03] border border-default rounded-2xl p-2.5">
    <div class="mb-1.5 flex items-center justify-between">
      <span class="text-[11px] text-white/40 font-medium uppercase tracking-wider">价格走势</span>
      <span class="text-[11px] text-white/30 tabular-nums">近 {{ history.length }} 次更新</span>
    </div>
    <canvas
      ref="canvasRef"
      class="w-full"
      :height="chartHeight"
      @mouseenter="hovered = true"
      @mouseleave="hovered = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, nextTick } from 'vue'

const props = defineProps<{
  history: number[]
  isUp: boolean
}>()

const canvasRef = ref<HTMLCanvasElement | null>(null)
const chartHeight = 64
const hovered = ref(false)

function draw() {
  const canvas = canvasRef.value
  if (!canvas || props.history.length < 2) return

  const ctx = canvas.getContext('2d')
  if (!ctx) return

  // Set canvas actual size
  const rect = canvas.getBoundingClientRect()
  const dpr = window.devicePixelRatio || 1
  canvas.width = rect.width * dpr
  canvas.height = chartHeight * dpr
  ctx.scale(dpr, dpr)

  const width = rect.width
  const height = chartHeight
  const data = props.history
  const padding = 4

  // Find min/max
  let min = Infinity
  let max = -Infinity
  for (const v of data) {
    if (v < min) min = v
    if (v > max) max = v
  }

  // Add some padding to range
  const range = max - min || 1
  min -= range * 0.1
  max += range * 0.1
  const adjustedRange = max - min

  const stepX = (width - padding * 2) / (data.length - 1)

  // Color based on trend
  const color = props.isUp ? '#00C087' : '#FF4D4F'
  const colorFade = props.isUp ? 'rgba(0, 192, 135, 0.08)' : 'rgba(255, 77, 79, 0.08)'

  // Clear
  ctx.clearRect(0, 0, width, height)

  // Draw gradient fill
  const gradient = ctx.createLinearGradient(0, 0, 0, height)
  gradient.addColorStop(0, props.isUp ? 'rgba(0, 192, 135, 0.15)' : 'rgba(255, 77, 79, 0.15)')
  gradient.addColorStop(1, 'rgba(0, 0, 0, 0)')

  ctx.beginPath()
  ctx.moveTo(padding, height)

  for (let i = 0; i < data.length; i++) {
    const x = padding + i * stepX
    const y = height - padding - ((data[i] - min) / adjustedRange) * (height - padding * 2)
    if (i === 0) {
      ctx.lineTo(x, y)
    } else {
      // Smooth curve
      const prevX = padding + (i - 1) * stepX
      const prevY = height - padding - ((data[i - 1] - min) / adjustedRange) * (height - padding * 2)
      const cpX = (prevX + x) / 2
      ctx.bezierCurveTo(cpX, prevY, cpX, y, x, y)
    }
  }

  ctx.lineTo(padding + (data.length - 1) * stepX, height)
  ctx.closePath()
  ctx.fillStyle = gradient
  ctx.fill()

  // Draw line
  ctx.beginPath()
  for (let i = 0; i < data.length; i++) {
    const x = padding + i * stepX
    const y = height - padding - ((data[i] - min) / adjustedRange) * (height - padding * 2)
    if (i === 0) {
      ctx.moveTo(x, y)
    } else {
      const prevX = padding + (i - 1) * stepX
      const prevY = height - padding - ((data[i - 1] - min) / adjustedRange) * (height - padding * 2)
      const cpX = (prevX + x) / 2
      ctx.bezierCurveTo(cpX, prevY, cpX, y, x, y)
    }
  }

  ctx.strokeStyle = color
  ctx.lineWidth = 1.5
  ctx.stroke()

  // Draw latest price dot
  if (data.length > 0) {
    const lastX = padding + (data.length - 1) * stepX
    const lastY = height - padding - ((data[data.length - 1] - min) / adjustedRange) * (height - padding * 2)

    // Glow
    ctx.beginPath()
    ctx.arc(lastX, lastY, 4, 0, Math.PI * 2)
    ctx.fillStyle = props.isUp ? 'rgba(0, 192, 135, 0.3)' : 'rgba(255, 77, 79, 0.3)'
    ctx.fill()

    // Dot
    ctx.beginPath()
    ctx.arc(lastX, lastY, 2, 0, Math.PI * 2)
    ctx.fillStyle = color
    ctx.fill()
  }
}

watch(() => [props.history.length, props.isUp], () => {
  nextTick(draw)
}, { deep: true })

onMounted(() => {
  nextTick(draw)
  // Redraw on resize
  window.addEventListener('resize', draw)
})
</script>
