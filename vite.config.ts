import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri dev server: fixed port 1420 (matches tauri.conf.json devUrl)
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: "0.0.0.0",
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
