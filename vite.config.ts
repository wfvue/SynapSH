import path from "node:path";
import { defineConfig } from "vite-plus";
import vue from "@vitejs/plugin-vue";
import tailwindcss from "@tailwindcss/vite";
import electron from "vite-plugin-electron";
import renderer from "vite-plugin-electron-renderer";
import AutoImport from "unplugin-auto-import/vite";
import Components from "unplugin-vue-components/vite";

// https://vite.dev/config/
export default defineConfig(() => ({
  plugins: [
    vue(),
    tailwindcss(),
    // 自动导入组件
    Components({
      dts: true,
      dirs: ["src/components"],
    }),
    // 自动导入 API (ref, onMounted 等)
    AutoImport({
      imports: ["vue", "@vueuse/core"],
      dts: true,
    }),
    // Electron 后端托管
    electron([
      {
        // 主进程入口
        entry: "electron/main.ts",
        onstart(options) {
          options.startup();
        },
        vite: {
          build: {
            outDir: "dist-electron",
            minify: process.env.NODE_ENV === "production",
            lib: {
              entry: "electron/main.ts",
              formats: ["es"],
              fileName: () => "[name].mjs",
            },
            rolldownOptions: {
              external: ["better-sqlite3", "ssh2"],
            },
          },
        },
      },
      {
        entry: "electron/preload.ts",
        onstart(options) {
          options.reload();
        },
        vite: {
          build: {
            outDir: "dist-electron",
            lib: {
              entry: "electron/preload.ts",
              formats: ["es"],
              fileName: () => "[name].mjs",
            },
          },
        },
      },
    ]),
    // 让 Electron 支持 Renderer 进程调用 Node
    renderer(),
  ],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  server: {
    port: 19000,
    strictPort: true,
  },
  // Vite+ Staged Config: used for commit hooks
  staged: {
    "*.{ts,vue}": "vp check --fix",
  },
}));
