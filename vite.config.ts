import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    // WebView2 on Windows 10 1809+, WKWebView on macOS 12+, WebKitGTK on
    // Ubuntu 22.04+. Both baselines support ES2022.
    target: ["chrome105", "safari15"],
    sourcemap: false,
    reportCompressedSize: true,
  },
  test: {
    // Tests arrive with the palette/search tasks; keep the skeleton green.
    passWithNoTests: true,
    environment: "node",
  },
});
