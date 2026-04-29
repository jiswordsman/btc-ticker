import { readFile } from 'node:fs/promises'

function fail(message) {
  throw new Error(message)
}

const readme = await readFile(new URL('../readme.md', import.meta.url), 'utf8')
const iconScript = await readFile(new URL('../gen_icon.swift', import.meta.url), 'utf8')
const traySource = await readFile(new URL('../src-tauri/src/tray.rs', import.meta.url), 'utf8')

if (!readme.includes('pnpm tauri build')) {
  fail('README should document how to package the app with pnpm tauri build')
}

if (!readme.includes('可安装程序') && !readme.includes('打包')) {
  fail('README should contain a dedicated packaging/install section')
}

if (!iconScript.includes('₿')) {
  fail('Icon generation script should use the Bitcoin symbol as the app icon source')
}

if (!traySource.includes('tray-up.png') || !traySource.includes('tray-down.png')) {
  fail('Tray source should expose green/red tray icon variants for up/down movement')
}

console.log('Branding and packaging docs are wired for the Bitcoin icon and installer flow.')
