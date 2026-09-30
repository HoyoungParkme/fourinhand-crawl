import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// 개발 서버 주소는 backend/tauri.conf.json의 devUrl과 같아야 한다
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 5173, strictPort: true },
  build: { target: 'es2022' },
})
