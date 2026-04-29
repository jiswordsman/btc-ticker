import { readFile } from 'node:fs/promises'

function fail(message) {
  throw new Error(message)
}

const appVue = await readFile(new URL('../app/app.vue', import.meta.url), 'utf8')
const indexVue = await readFile(new URL('../app/pages/index.vue', import.meta.url), 'utf8')
const chartVue = await readFile(new URL('../app/components/MiniChart.vue', import.meta.url), 'utf8')
const tauriConfigRaw = await readFile(new URL('../src-tauri/tauri.conf.json', import.meta.url), 'utf8')
const tauriConfig = JSON.parse(tauriConfigRaw)

if (appVue.includes('min-h-screen')) {
  fail('App shell should not use min-h-screen in the tray window because it can push content past the fixed popover height')
}

if (indexVue.includes('class="flex flex-col h-screen"')) {
  fail('Index page should not claim h-screen inside the tray shell; it should fill the shell with h-full/min-h-0 instead')
}

if (!indexVue.includes('min-h-0')) {
  fail('Scrollable content should include min-h-0 so the footer can stay visible inside the fixed-height flex layout')
}

const mainWindow = tauriConfig.app?.windows?.find(window => window.label === 'main')
if (!mainWindow || mainWindow.height < 520) {
  fail('Tray popover window height should be at least 520px so the bottom summary row stays visible on compact displays')
}

if (chartVue.includes('const chartHeight = 80')) {
  fail('Mini chart is too tall for the tray popover; use a more compact chart height to preserve footer visibility')
}

console.log('Layout structure avoids nested viewport-height clipping.')
