import path from "path"
import react from "@vitejs/plugin-react"
import { defineConfig } from "vite"

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  clearScreen: false,
  server: {
    host: host || false,
    port: 5173,
    strictPort: true,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  optimizeDeps: {
    include: [
      "@chakra-ui/react",
      "@emotion/react",
      "react",
      "react-dom",
      "react-dom/client",
      "react-virtuoso",
      "react-icons/lu",
      "lucide-react",
      "next-themes",
      "zustand",
      "@tauri-apps/api/event",
      "@tauri-apps/api/core",
    ],
  },
})
