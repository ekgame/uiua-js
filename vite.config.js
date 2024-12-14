// vite.config.lib
import { resolve } from 'path'
import { defineConfig } from 'vite'

export default defineConfig({
    build: {
        target: 'esnext',
        lib: {
            formats: ['es'],
            entry: resolve(__dirname, 'lib/main.ts'),
            name: 'uiuajs',
            fileName: 'uiua'
        },
        rollupOptions: {
            // make sure to externalize deps that shouldn't be bundled
            // into your library
            external: [],
        }
    }
})
