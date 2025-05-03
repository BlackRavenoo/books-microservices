import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
    plugins: [svelte()],
    server: {
        proxy: {
            '/api/v1': {
                target: 'http://127.0.0.1:4001',
                changeOrigin: true,
            },
            '/oauth': {
                target: 'http://127.0.0.1:5001',
                changeOrigin: true
            }
        }
    }
});