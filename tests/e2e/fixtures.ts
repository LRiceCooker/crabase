import { test as base } from "@playwright/test";
import path from "path";

// Custom test fixture that injects the Tauri shim before each page load
export const test = base.extend({
  page: async ({ page }, use) => {
    await page.addInitScript({
      path: path.resolve(__dirname, "tauri-shim.js"),
    });
    await use(page);
  },
});

export { expect } from "@playwright/test";
