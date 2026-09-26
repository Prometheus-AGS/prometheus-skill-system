// TJ-ARCH-MOB-001 compliant
import { defineConfig } from 'vite';
export default defineConfig({ server: { port: 1420, strictPort: true, proxy: { '/api': 'http://127.0.0.1:3000' } }, clearScreen: false });
