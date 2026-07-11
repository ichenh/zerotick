import { defineConfig } from "vite";

export default defineConfig({
  clearScreen: false,
  server: {
    // 55555：避开 Win 保留端口段，且不与常见 Vite 5173 冲突（须与 tauri.conf.json devUrl、ports.rs 同步）
    host: "127.0.0.1",
    port: 55555,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    target: "esnext",
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    outDir: "dist",
    emptyOutDir: true,
  },
});
