import { defineConfig } from "vite";
import { pathToFileURL } from "node:url";

const COMPLETE_BUNDLED_LOCALES = ["zh-TW", "ja", "ko", "de"];

function completeLocalePacksOnly() {
  let localeModulePromise;
  return {
    name: "zerotick-complete-locale-packs-only",
    enforce: "pre",
    async transform(_source, id) {
      const cleanId = id.split("?", 1)[0];
      if (!cleanId.replaceAll("\\", "/").endsWith("/src/locales/packs.js")) return null;
      localeModulePromise ??= import(`${pathToFileURL(cleanId).href}?core-bundle`);
      const { packs, terminology } = await localeModulePromise;
      const pick = (source) => Object.fromEntries(
        COMPLETE_BUNDLED_LOCALES.map((code) => [code, source[code] ?? {}]),
      );
      return {
        code: `export const packs = ${JSON.stringify(pick(packs))};\nexport const terminology = ${JSON.stringify(pick(terminology))};`,
        map: null,
      };
    },
  };
}

export default defineConfig({
  plugins: [completeLocalePacksOnly()],
  clearScreen: false,
  server: {
    // 避开 Win 保留端口段，且不与常见 Vite 5173 冲突（须与 tauri.conf.json devUrl 同步）
    host: "127.0.0.1",
    port: 55555,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    target: "esnext",
    minify: !process.env.TAURI_DEBUG ? "oxc" : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    outDir: "dist",
    emptyOutDir: true,
  },
});
