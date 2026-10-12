import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react-swc'

// https://vitejs.dev/config/
export default defineConfig({
    plugins: [react()],
    server: {
        proxy: {
            '^/api/.*': {
                target: 'http://localhost:1337',
                changeOrigin: true,
                // /api/ai/ws is a WebSocket; without this the dev server
                // proxies plain HTTP only and the socket never connects.
                ws: true
            }
        }
    }
})
