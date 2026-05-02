import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tsconfigPaths from "vite-tsconfig-paths";

export default defineConfig({
    plugins: [
        svelte(),
        tsconfigPaths(),
    ],
    resolve: {
        conditions: ["browser"],
    },
    test: {
        include: ["src/**/*.test.ts"],
        environment: "jsdom",
    },
});
