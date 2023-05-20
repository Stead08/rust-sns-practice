import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import {VITE_SERVER_PORT, API_SERVER_PORT} from './src/lib/env';



export default defineConfig({
	plugins: [sveltekit()],
	server: {
		port: VITE_SERVER_PORT,
		proxy: {
			'/api': {
				changeOrigin: true,
				target: `http://localhost:${API_SERVER_PORT}`,
			},
		},
		fs: {
			allow: ["static"]
		}
	},
	build: {
		outDir: 'dist/app'
	},
});
