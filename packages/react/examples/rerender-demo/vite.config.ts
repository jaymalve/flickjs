import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import reactScan from '@react-scan/vite-plugin-react-scan';

export default defineConfig({
  plugins: [
    react(),
    reactScan({
      enable: process.env.NODE_ENV === 'development',
      autoDisplayNames: true
    })
  ]
});
