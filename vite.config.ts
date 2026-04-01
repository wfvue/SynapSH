import path from "node:path";
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import tailwindcss from "@tailwindcss/vite";

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [vue(), tailwindcss()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },

  // Vite options for Electron development
  clearScreen: false,
  server: {
    port: 19000,
    strictPort: true,
    host: false,
    watch: {
      // 告诉 Vite 忽略监听 Electron 和构建产物目录，避免无限刷新
      ignored: ["**/dist-electron/**", "**/dist/**", "**/release/**"],
    },
  },
}));
