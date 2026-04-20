import { test, expect } from "@playwright/test";

const CONNECTION_STRING = "postgresql://test:test@localhost:5433/crabase_test";

async function connectToDb(page) {
  await page.goto("/");
  const input = page.locator(
    'input[placeholder="postgresql://user:password@host:port/dbname"]'
  );
  await input.fill(CONNECTION_STRING);
  await page.locator('button:has-text("Next")').click();
  await expect(page.locator("text=Connection details")).toBeVisible();
  await page.locator('button:has-text("Connect")').click();
  await expect(page.locator("text=users")).toBeVisible({ timeout: 15000 });
}

test.describe("Theme switching", () => {
  test.beforeEach(async ({ page }) => {
    await connectToDb(page);
  });

  test("Open settings via command palette", async ({ page }) => {
    await page.keyboard.press("Meta+Shift+P");
    const input = page.locator('input[placeholder="Type a command..."]');
    await input.fill("Settings");
    await page.keyboard.press("Enter");

    // Verify settings view opens
    await expect(page.locator("text=Theme")).toBeVisible({ timeout: 5000 });
  });

  test("Toggle to dark theme, verify dark class on html", async ({ page }) => {
    // Open settings
    await page.keyboard.press("Meta+Shift+P");
    const input = page.locator('input[placeholder="Type a command..."]');
    await input.fill("Settings");
    await page.keyboard.press("Enter");
    await expect(page.locator("text=Theme")).toBeVisible({ timeout: 5000 });

    // Click Dark option
    await page.locator('button:has-text("Dark")').click();

    // Verify html element has "dark" class
    const htmlElement = page.locator("html");
    await expect(htmlElement).toHaveClass(/dark/);
  });

  test("Toggle back to light theme, verify dark class removed", async ({
    page,
  }) => {
    // Open settings and set dark
    await page.keyboard.press("Meta+Shift+P");
    const input = page.locator('input[placeholder="Type a command..."]');
    await input.fill("Settings");
    await page.keyboard.press("Enter");
    await expect(page.locator("text=Theme")).toBeVisible({ timeout: 5000 });

    await page.locator('button:has-text("Dark")').click();
    await expect(page.locator("html")).toHaveClass(/dark/);

    // Switch to light
    await page.locator('button:has-text("Light")').click();

    // Verify "dark" class is removed
    await expect(page.locator("html")).not.toHaveClass(/dark/);
  });
});
