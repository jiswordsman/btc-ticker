import { readFile } from 'node:fs/promises'

function fail(message) {
  throw new Error(message)
}

const tauriConfigRaw = await readFile(new URL('../src-tauri/tauri.conf.json', import.meta.url), 'utf8')
const tauriConfig = JSON.parse(tauriConfigRaw)
const packageJsonRaw = await readFile(new URL('../package.json', import.meta.url), 'utf8')
const packageJson = JSON.parse(packageJsonRaw)
const nuxtConfig = await readFile(new URL('../nuxt.config.ts', import.meta.url), 'utf8')

if (tauriConfig.build.beforeDevCommand !== 'npm run dev:tauri') {
  fail(`Expected beforeDevCommand to use the dedicated dev:tauri preview server, got "${tauriConfig.build.beforeDevCommand}"`)
}

if (packageJson.scripts?.['dev:tauri'] !== 'npm run build && PORT=3000 node .output/server/index.mjs') {
  fail('Expected package.json to define dev:tauri as a build + direct Nitro server command on port 3000')
}

if (nuxtConfig.includes('@vite/client') || nuxtConfig.includes('process.env.TAURI_DEV')) {
  fail('Tauri dev should not rely on a TAURI_DEV HMR workaround once it uses preview mode')
}

console.log('Tauri dev config uses a stable preview server instead of Nuxt/Vite HMR.')
