import { readFile } from 'node:fs/promises'

function fail(message) {
  throw new Error(message)
}

const tauriConfigRaw = await readFile(new URL('../src-tauri/tauri.conf.json', import.meta.url), 'utf8')
const tauriConfig = JSON.parse(tauriConfigRaw)

if (tauriConfig.build.frontendDist !== '../.output/public') {
  fail(`Expected src-tauri/tauri.conf.json build.frontendDist to be "../.output/public", got "${tauriConfig.build.frontendDist}"`)
}

const indexHtml = await readFile(new URL('../.output/public/index.html', import.meta.url), 'utf8')

if (indexHtml.includes('href="/_nuxt/') || indexHtml.includes('src="/_nuxt/')) {
  fail('Expected generated static HTML to use relative asset URLs for Tauri, but found absolute /_nuxt/ asset paths')
}

if (indexHtml.includes('baseURL:"/"') || indexHtml.includes('buildAssetsDir:"/_nuxt/"')) {
  fail('Expected generated static HTML to expose relative Nuxt runtime paths for Tauri, but found absolute baseURL/buildAssetsDir values')
}

console.log('Tauri static build paths look correct.')
