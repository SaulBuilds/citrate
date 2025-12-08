import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// Dev mode detection - set CITRATE_DEV_MODE=true to enable mock code
// @ts-expect-error process is a nodejs global
const isDevMode = process.env.CITRATE_DEV_MODE === 'true' || process.env.NODE_ENV === 'development';

// https://vite.dev/config/
export default defineConfig(async () => ({
  // Use relative paths for production build so Tauri and file:// preview work
  base: './',
  plugins: [
    react({
      babel: {
        plugins: ['styled-jsx/babel'],
      },
    }),
  ],

  // Define global constants for dev mode gating
  define: {
    '__DEV_MODE__': JSON.stringify(isDevMode),
    '__CITRATE_VERSION__': JSON.stringify(process.env.npm_package_version || '0.1.0'),
    '__BUILD_TIME__': JSON.stringify(new Date().toISOString()),
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 3456,
    strictPort: true,
    host: host || '127.0.0.1',
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },

  // Production build optimizations
  build: {
    // Strip dev-only code in production
    minify: !isDevMode,
    // Generate source maps only in dev mode
    sourcemap: isDevMode,
    // Rollup options for dead code elimination
    rollupOptions: {
      output: {
        // Ensure clean chunk names
        chunkFileNames: 'assets/[name]-[hash].js',
        entryFileNames: 'assets/[name]-[hash].js',
        assetFileNames: 'assets/[name]-[hash].[ext]',
      },
    },
  },

  // Enable esbuild for faster builds and better tree-shaking
  esbuild: {
    // Drop console.log in production builds
    drop: isDevMode ? [] : ['console', 'debugger'],
    // Keep pure calls (allows tree-shaking of unused code)
    pure: isDevMode ? [] : ['console.log', 'console.debug'],
  },
}));
