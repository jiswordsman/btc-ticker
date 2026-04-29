// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
    compatibilityDate: '2025-05-15',
    srcDir: 'app',

    // Development server configuration
    devServer: {
        port: 3008,  // Changed from default 3000 to avoid port conflicts
    },

    app: {
        baseURL: '',
        buildAssetsDir: '_nuxt/',
    },

    modules: ['@nuxt/ui'],

    css: ['~/assets/css/main.css'],

    // Tauri requires SPA mode
    ssr: false,

    devtools: {enabled: true},
    experimental: {
        payloadExtraction: "client"
    },

    // Tauri-specific Vite configuration
    vite: {
        clearScreen: false,
        envPrefix: ['VITE_', 'TAURI_'],
        server: {
            strictPort: true,
        },
    },

    // Avoid watching Tauri files
    ignore: ['**/src-tauri/**'],
})
