/// <reference types="vitest/config" />
import path from "node:path";
import tailwindcss from "@tailwindcss/vite";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(({ command }) => ({
  clearScreen: false,
  envPrefix: ["VITE_", "TAURI_ENV_"],
  plugins: [
    tanstackRouter({
      target: "react",
      routesDirectory: "./src/app/routes",
      generatedRouteTree: "./src/routeTree.gen.ts",
      routeFileIgnorePattern: "\\.test\\.(tsx|ts)$",
    }),
    react(),
    tailwindcss(),
  ],
  resolve: {
    alias: {
      "@": path.resolve(import.meta.dirname, "src"),
    },
  },
  server: {
    port: 5173,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**", "**/bridge/**"],
    },
  },
  build:
    command === "serve"
      ? undefined
      : process.env.TAURI_ENV_PLATFORM
        ? {
            target:
              process.env.TAURI_ENV_PLATFORM === "windows"
                ? "chrome105"
                : "safari13",
            minify: !process.env.TAURI_ENV_DEBUG,
            sourcemap: Boolean(process.env.TAURI_ENV_DEBUG),
          }
        : {
            // hidden: maps for tooling; not linked from the public bundle.
            sourcemap: "hidden",
          },
  test: {
    include: ["src/**/*.test.{ts,tsx}", "scripts/**/*.test.ts"],
    environment: "jsdom",
    setupFiles: ["src/testing/setup.ts"],
    globals: false,
    css: true,
    exclude: ["**/node_modules/**", "**/e2e/**", "**/bridge/**"],
    coverage: {
      include: ["src/**"],
      exclude: ["src/routeTree.gen.ts", "**/*.d.ts"],
    },
  },
}));
