import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import reactScan from '@react-scan/vite-plugin-react-scan';

export default defineConfig({
  base: '/react/demo/',
  plugins: [
    react(),
    reactScan({
      // Subpath deploy (`base: '/react/demo/'`): the plugin's production script URL is
      // `/assets/auto.global.js` and 404s under a base path. We call `scan()` from
      // `react-scan/all-environments` in main.tsx instead; keep babel display names only.
      enable: false,
      autoDisplayNames: true
    })
  ]
});
