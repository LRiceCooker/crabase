import { defineConfig } from "@playwright/test";
import path from "path";

export default defineConfig({
  testDir: ".",
  testMatch: "**/*.spec.ts",
  timeout: 30_000,
  retries: 0,
  workers: 1,
  use: {
    baseURL: "http://localhost:8080",
    browserName: "chromium",
    headless: true,
  },
  globalSetup: path.resolve(__dirname, "global-setup.ts"),
  globalTeardown: path.resolve(__dirname, "global-teardown.ts"),
  projects: [
    {
      name: "e2e",
      use: {
        // Inject the Tauri shim before WASM loads
        contextOptions: {
          initScripts: [path.resolve(__dirname, "tauri-shim.js")],
        },
      },
    },
  ],
});
